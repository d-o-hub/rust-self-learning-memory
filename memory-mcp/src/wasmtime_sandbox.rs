//! Wasmtime-based WASM sandbox for secure code execution
//!
//! This module provides a production-grade WASM sandbox using wasmtime,
//! replacing the rquickjs implementation that had GC stability issues.
//!
//! ## Architecture
//!
//! - **Engine**: Shared wasmtime engine with fuel-based timeout enforcement
//! - **Store**: Per-execution isolated store with WASI context
//! - **Pooling**: Semaphore-based concurrency control
//! - **Metrics**: Execution statistics and health monitoring
//!
//! ## Phase 2A: Basic POC (Complete)
//!
//! - Execute pre-compiled WASM modules
//! - Concurrent execution without GC crashes
//! - Basic metrics and health monitoring
//!
//! ## Phase 2B: Enhanced WASM Execution (In Progress)
//!
//! - **WASI Support**: Stdout/stderr capture via WASI context ✅
//! - **Fuel-Based Timeouts**: Deterministic execution time limits ✅
//! - **JavaScript Support**: Future - Javy integration for JS→WASM compilation

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info, warn};
use wasmtime::*;
use wasmtime_wasi::preview1::{add_to_linker_sync, WasiP1Ctx};

use crate::types::{ErrorType, ExecutionContext, ExecutionResult};

/// Wasmtime sandbox configuration
#[derive(Debug, Clone)]
pub struct WasmtimeConfig {
    /// Maximum execution time
    pub max_execution_time: Duration,
    /// Maximum memory in bytes (default: 128MB)
    pub max_memory_bytes: usize,
    /// Maximum number of concurrent executions
    pub max_pool_size: usize,
    /// Enable console output capture
    pub allow_console: bool,
}

impl Default for WasmtimeConfig {
    fn default() -> Self {
        Self {
            max_execution_time: Duration::from_secs(5),
            max_memory_bytes: 128 * 1024 * 1024, // 128MB
            max_pool_size: 20,
            allow_console: true,
        }
    }
}

impl WasmtimeConfig {
    pub fn restrictive() -> Self {
        Self {
            max_execution_time: Duration::from_secs(2),
            max_memory_bytes: 64 * 1024 * 1024, // 64MB
            max_pool_size: 10,
            allow_console: false,
        }
    }
}

/// Wasmtime execution metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WasmtimeMetrics {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub timeout_count: u64,
    pub security_violations: u64,
    pub avg_execution_time_ms: f64,
    pub peak_memory_bytes: usize,
}

/// Captured WASI output for a single execution using wasmtime_wasi memory pipes
struct CapturedOutput {
    stdout: wasmtime_wasi::pipe::MemoryOutputPipe,
    stderr: wasmtime_wasi::pipe::MemoryOutputPipe,
}

impl CapturedOutput {
    fn new() -> Self {
        Self {
            stdout: wasmtime_wasi::pipe::MemoryOutputPipe::new(1024 * 64),
            stderr: wasmtime_wasi::pipe::MemoryOutputPipe::new(1024 * 64),
        }
    }

    fn into_strings(self) -> (String, String) {
        let stdout = String::from_utf8_lossy(&self.stdout.try_into_inner().unwrap_or_default())
            .into_owned();
        let stderr = String::from_utf8_lossy(&self.stderr.try_into_inner().unwrap_or_default())
            .into_owned();
        (stdout, stderr)
    }
}

/// Wasmtime-based WASM sandbox
pub struct WasmtimeSandbox {
    config: WasmtimeConfig,
    engine: Engine,
    pool_semaphore: Arc<Semaphore>,
    metrics: Arc<RwLock<WasmtimeMetrics>>,
}

// Manual Debug implementation since Engine doesn't implement Debug
impl std::fmt::Debug for WasmtimeSandbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmtimeSandbox")
            .field("config", &self.config)
            .field("engine", &"<Engine>")
            .field("pool_semaphore", &self.pool_semaphore)
            .field("metrics", &self.metrics)
            .finish()
    }
}

impl WasmtimeSandbox {
    /// Create a new wasmtime sandbox
    pub fn new(config: WasmtimeConfig) -> Result<Self> {
        info!("Initializing wasmtime sandbox with config: {:?}", config);

        // Configure wasmtime engine with fuel support for timeout enforcement
        let mut engine_config = Config::new();
        engine_config.consume_fuel(true);

        let engine = Engine::new(&engine_config).context("Failed to create wasmtime engine")?;

        debug!("Wasmtime engine created successfully with fuel support");

        let pool_size = config.max_pool_size;

        Ok(Self {
            config,
            engine,
            pool_semaphore: Arc::new(Semaphore::new(pool_size)),
            metrics: Arc::new(RwLock::new(WasmtimeMetrics::default())),
        })
    }

    /// Execute WASM module with the given context
    ///
    /// Phase 2A: Accepts pre-compiled WASM bytecode ✅
    /// Phase 2B.1: WASI stdout/stderr capture ✅
    /// Phase 2B.2: Will add JavaScript→WASM compilation via Javy
    pub async fn execute(
        &self,
        wasm_bytecode: &[u8],
        _context: &ExecutionContext,
    ) -> Result<ExecutionResult> {
        // Acquire permit from pool
        let _permit = self
            .pool_semaphore
            .acquire()
            .await
            .context("Failed to acquire execution permit")?;

        let start = Instant::now();
        debug!("Starting WASM execution");

        // Execute in blocking task to avoid blocking async runtime
        let engine = self.engine.clone();
        let wasm_bytecode = wasm_bytecode.to_vec();
        let config = self.config.clone();

        let result = tokio::task::spawn_blocking(move || {
            Self::execute_sync(&engine, &wasm_bytecode, &config)
        })
        .await
        .context("Failed to spawn blocking task")??;

        let elapsed = start.elapsed();

        // Update metrics
        self.update_metrics(&result, elapsed).await;

        debug!("WASM execution completed in {:?}", elapsed);

        Ok(result)
    }

    /// Synchronous execution (runs in blocking task)
    fn execute_sync(
        engine: &Engine,
        wasm_bytecode: &[u8],
        config: &WasmtimeConfig,
    ) -> Result<ExecutionResult> {
        // Load WASM module
        let module =
            Module::from_binary(engine, wasm_bytecode).context("Failed to load WASM module")?;

        // Set up output capture buffers
        let captured_output = CapturedOutput::new();
        let stdout_buffer = captured_output.stdout.clone();
        let stderr_buffer = captured_output.stderr.clone();

        // Create WASI Preview 1 context with custom stdout/stderr
        let wasi = if config.allow_console {
            debug!("Configuring WASI with captured stdout/stderr");
            wasmtime_wasi::WasiCtxBuilder::new()
                .inherit_stdin()
                .stdout(stdout_buffer)
                .stderr(stderr_buffer)
                .build_p1()
        } else {
            debug!("Configuring WASI with inherited stdout/stderr (console disabled)");
            wasmtime_wasi::WasiCtxBuilder::new()
                .inherit_stdin()
                .inherit_stdout()
                .inherit_stderr()
                .build_p1()
        };

        // Create store with WASI context
        let mut store = Store::new(engine, wasi);

        // Set fuel based on execution time limit
        let fuel = Self::calculate_fuel(config.max_execution_time);
        store
            .set_fuel(fuel)
            .context("Failed to set execution fuel")?;

        debug!(
            "Set fuel to {} for max execution time {:?}",
            fuel, config.max_execution_time
        );

        // Create linker and add WASI Preview 1
        let mut linker = Linker::new(engine);
        add_to_linker_sync(&mut linker, |ctx: &mut WasiP1Ctx| ctx)
            .context("Failed to add WASI to linker")?;

        // Instantiate module
        let instance = linker
            .instantiate(&mut store, &module)
            .context("Failed to instantiate WASM module")?;

        // Get the main export function
        let main_func = instance
            .get_typed_func::<(), i32>(&mut store, "main")
            .context("No main function found")?;

        // Execute
        let exec_start = Instant::now();
        let call_result = main_func.call(&mut store, ());
        let execution_time_ms = exec_start.elapsed().as_millis() as u64;

        // Check remaining fuel
        let remaining_fuel = store.get_fuel().unwrap_or(0);
        debug!("Execution completed with {} fuel remaining", remaining_fuel);

        // Extract captured output
        let (stdout, stderr) = captured_output.into_strings();

        match call_result {
            Ok(_result) => {
                debug!(
                    "WASM execution successful, captured {} bytes of stdout, {} bytes of stderr",
                    stdout.len(),
                    stderr.len()
                );
                Ok(ExecutionResult::Success {
                    output: "WASM execution completed successfully".to_string(),
                    stdout,
                    stderr,
                    execution_time_ms,
                })
            }
            Err(e) => {
                // Check if it was a timeout (out of fuel)
                // wasmtime returns a Trap with OutOfFuel variant
                if let Some(trap) = e.downcast_ref::<Trap>() {
                    if matches!(trap, Trap::OutOfFuel) {
                        warn!("Execution timed out due to fuel exhaustion");
                        return Ok(ExecutionResult::Timeout {
                            elapsed_ms: execution_time_ms,
                            partial_output: if stdout.is_empty() {
                                None
                            } else {
                                Some(stdout)
                            },
                        });
                    }
                }

                // Fall through to runtime error
                debug!("WASM execution failed: {}", e);
                Ok(ExecutionResult::Error {
                    error_type: ErrorType::Runtime,
                    message: e.to_string(),
                    stdout,
                    stderr,
                })
            }
        }
    }

    /// Calculate fuel amount based on execution time limit
    ///
    /// Heuristic: 1 million fuel units per second of allowed execution time
    fn calculate_fuel(max_time: Duration) -> u64 {
        let seconds = max_time.as_secs();
        let millis = max_time.subsec_millis() as u64;
        (seconds * 1_000_000) + (millis * 1_000)
    }

    /// Update execution metrics
    async fn update_metrics(&self, result: &ExecutionResult, _elapsed: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.total_executions += 1;

        match result {
            ExecutionResult::Success {
                execution_time_ms, ..
            } => {
                metrics.successful_executions += 1;

                // Update average execution time
                let total =
                    metrics.avg_execution_time_ms * (metrics.successful_executions - 1) as f64;
                metrics.avg_execution_time_ms =
                    (total + *execution_time_ms as f64) / metrics.successful_executions as f64;
            }
            ExecutionResult::Timeout { .. } => {
                metrics.timeout_count += 1;
                metrics.failed_executions += 1;
            }
            ExecutionResult::Error { .. } => {
                metrics.failed_executions += 1;
            }
            ExecutionResult::SecurityViolation { .. } => {
                metrics.security_violations += 1;
                metrics.failed_executions += 1;
            }
        }
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> WasmtimeMetrics {
        self.metrics.read().await.clone()
    }

    /// Get health status
    pub async fn health_check(&self) -> bool {
        let metrics = self.metrics.read().await;

        // Healthy if we have some successful executions and not too many failures
        if metrics.total_executions == 0 {
            return true; // No executions yet, consider healthy
        }

        let success_rate = metrics.successful_executions as f64 / metrics.total_executions as f64;
        success_rate > 0.5 // At least 50% success rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple WASM module that returns 42 (in WAT format)
    ///
    /// (module
    ///   (func $main (result i32)
    ///     i32.const 42)
    ///   (export "main" (func $main))
    /// )
    const SIMPLE_WASM: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d, // Magic number
        0x01, 0x00, 0x00, 0x00, // Version
        0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, // Type section
        0x03, 0x02, 0x01, 0x00, // Function section
        0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00, // Export section
        0x0a, 0x06, 0x01, 0x04, 0x00, 0x41, 0x2a, 0x0b, // Code section
    ];

    #[tokio::test]
    async fn test_wasmtime_sandbox_creation() -> Result<()> {
        let sandbox = WasmtimeSandbox::new(WasmtimeConfig::default())?;
        assert!(sandbox.health_check().await);
        Ok(())
    }

    #[tokio::test]
    async fn test_basic_wasm_execution() -> Result<()> {
        let sandbox = WasmtimeSandbox::new(WasmtimeConfig::default())?;
        let ctx = ExecutionContext::new("test".to_string(), serde_json::json!({}));

        let result = sandbox.execute(SIMPLE_WASM, &ctx).await?;

        match result {
            ExecutionResult::Success { .. } => Ok(()),
            other => panic!("Expected success, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_wasi_stdout_stderr_capture() -> Result<()> {
        // Create a WASM module that writes to stdout and stderr using WASI
        // This module uses fd_write system call to write "Hello, stdout!" to stdout (fd=1)
        // and "Hello, stderr!" to stderr (fd=2)
        let wat = r#"
            (module
              ;; Import WASI fd_write function
              (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
              
              ;; Memory export for WASI
              (memory (export "memory") 1)
              
              ;; Data section with our strings
              (data (i32.const 100) "Hello, stdout!\n")
              (data (i32.const 120) "Hello, stderr!\n")
              
              ;; Function to write to stdout
              (func $write_stdout
                ;; Set up iovec for stdout write
                i32.const 1    ;; fd = 1 (stdout)
                i32.const 116   ;; iov_ptr = address of iovec array
                i32.const 1     ;; iov_cnt = 1
                i32.const 132   ;; nwritten_ptr = address to store bytes written
                call $fd_write
                drop
              )
              
              ;; Function to write to stderr  
              (func $write_stderr
                ;; Set up iovec for stderr write
                i32.const 2    ;; fd = 2 (stderr)
                i32.const 124   ;; iov_ptr = address of iovec array
                i32.const 1     ;; iov_cnt = 1
                i32.const 136   ;; nwritten_ptr = address to store bytes written
                call $fd_write
                drop
              )
              
              ;; Main function
              (func $main (result i32)
                ;; Initialize iovec structures
                ;; stdout iovec at offset 116: [buf_ptr=100, buf_len=15]
                i32.const 100
                i32.const 116
                i32.store offset=0 align=4
                i32.const 15
                i32.const 116
                i32.store offset=4 align=4
                
                ;; stderr iovec at offset 124: [buf_ptr=120, buf_len=15]  
                i32.const 120
                i32.const 124
                i32.store offset=0 align=4
                i32.const 15
                i32.const 124
                i32.store offset=4 align=4
                
                ;; Write to stdout and stderr
                call $write_stdout
                call $write_stderr
                
                i32.const 0) ;; return 0
              
              (export "main" (func $main))
            )
        "#;

        // Parse WAT to WASM bytecode
        let wasm_bytecode = wat::parse_str(wat).context("Failed to parse WAT")?;

        // Test with console capture enabled
        let config = WasmtimeConfig {
            allow_console: true,
            ..Default::default()
        };
        let sandbox = WasmtimeSandbox::new(config)?;
        let ctx = ExecutionContext::new("wasi-test".to_string(), serde_json::json!({}));

        let result = sandbox.execute(&wasm_bytecode, &ctx).await?;

        match result {
            ExecutionResult::Success { stdout, stderr, .. } => {
                assert_eq!(stdout, "Hello, stdout!\n");
                assert_eq!(stderr, "Hello, stderr!\n");
            }
            other => panic!("Expected success with captured output, got: {:?}", other),
        }

        // Test with console capture disabled
        let config_disabled = WasmtimeConfig {
            allow_console: false,
            ..Default::default()
        };
        let sandbox_disabled = WasmtimeSandbox::new(config_disabled)?;

        let result_disabled = sandbox_disabled.execute(&wasm_bytecode, &ctx).await?;

        match result_disabled {
            ExecutionResult::Success { stdout, stderr, .. } => {
                // Should be empty when console is disabled
                assert!(stdout.is_empty());
                assert!(stderr.is_empty());
            }
            other => panic!("Expected success with empty output, got: {:?}", other),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_wasi_capture_with_timeout() -> Result<()> {
        // Create a WASM module that writes to stdout then enters infinite loop
        let wat = r#"
            (module
              ;; Import WASI fd_write function
              (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
              
              ;; Memory export for WASI
              (memory (export "memory") 1)
              
              ;; Data section with our string
              (data (i32.const 100) "Before infinite loop\n")
              
              ;; Main function
              (func $main (result i32)
                ;; Write to stdout first
                i32.const 1    ;; fd = 1 (stdout)
                i32.const 108   ;; iov_ptr = address of iovec array
                i32.const 1     ;; iov_cnt = 1
                i32.const 120   ;; nwritten_ptr = address to store bytes written
                call $fd_write
                drop
                
                ;; Initialize iovec
                i32.const 100
                i32.const 108
                i32.store offset=0 align=4
                i32.const 21
                i32.const 108
                i32.store offset=4 align=4
                
                ;; Infinite loop to trigger timeout
                (loop $forever
                  br $forever)
                
                i32.const 0) ;; unreachable
              
              (export "main" (func $main))
            )
        "#;

        let wasm_bytecode = wat::parse_str(wat).context("Failed to parse WAT")?;

        // Create sandbox with very short timeout
        let config = WasmtimeConfig {
            max_execution_time: Duration::from_millis(100),
            allow_console: true,
            ..Default::default()
        };

        let sandbox = WasmtimeSandbox::new(config)?;
        let ctx = ExecutionContext::new("timeout-test".to_string(), serde_json::json!({}));

        let result = sandbox.execute(&wasm_bytecode, &ctx).await?;

        match result {
            ExecutionResult::Timeout {
                elapsed_ms,
                partial_output,
            } => {
                assert!(elapsed_ms < 1000);
                assert_eq!(partial_output, Some("Before infinite loop\n".to_string()));
            }
            other => panic!("Expected timeout with partial output, got: {:?}", other),
        }

        Ok(())
    }
}

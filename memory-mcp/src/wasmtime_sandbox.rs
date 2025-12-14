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
//! - **WASI Support**: Stdout/stderr capture via WASI context
//! - **Fuel-Based Timeouts**: Deterministic execution time limits
//! - **JavaScript Support**: Future - Javy integration for JS→WASM compilation

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info, warn};
use wasmtime::*;
use wasmtime_wasi::preview1::{WasiP1Ctx, add_to_linker_sync};

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

/// Wasmtime-based WASM sandbox
pub struct WasmtimeSandbox {
    config: WasmtimeConfig,
    engine: Engine,
    pool_semaphore: Arc<Semaphore>,
    metrics: Arc<RwLock<WasmtimeMetrics>>,
}

impl WasmtimeSandbox {
    /// Create a new wasmtime sandbox
    pub fn new(config: WasmtimeConfig) -> Result<Self> {
        info!("Initializing wasmtime sandbox with config: {:?}", config);

        // Configure wasmtime engine with fuel support for timeout enforcement
        let mut engine_config = Config::new();
        engine_config.consume_fuel(true);

        let engine = Engine::new(&engine_config)
            .context("Failed to create wasmtime engine")?;

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
    /// Phase 2A: Accepts pre-compiled WASM bytecode
    /// Phase 2B: Will add JavaScript→WASM compilation via Javy
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

        // Create WASI Preview 1 context
        let wasi = wasmtime_wasi::WasiCtxBuilder::new()
            .inherit_stdin()
            .inherit_stdout()
            .inherit_stderr()
            .build_p1();

        // Create store with WASI context
        let mut store = Store::new(engine, wasi);

        // Set fuel based on execution time limit
        let fuel = Self::calculate_fuel(config.max_execution_time);
        store
            .set_fuel(fuel)
            .context("Failed to set execution fuel")?;

        debug!("Set fuel to {} for max execution time {:?}", fuel, config.max_execution_time);

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

        match call_result {
            Ok(_result) => Ok(ExecutionResult::Success {
                output: "WASM execution completed successfully".to_string(),
                stdout: String::new(), // TODO: Capture from WASI in Phase 2B.1
                stderr: String::new(), // TODO: Capture from WASI in Phase 2B.1
                execution_time_ms,
            }),
            Err(e) => {
                // Check if it was a timeout (out of fuel)
                // wasmtime returns a Trap with OutOfFuel variant
                if let Some(trap) = e.downcast_ref::<Trap>() {
                    if matches!(trap, Trap::OutOfFuel) {
                        warn!("Execution timed out due to fuel exhaustion");
                        return Ok(ExecutionResult::Timeout {
                            elapsed_ms: execution_time_ms,
                            partial_output: None,
                        });
                    }
                }

                // Fall through to runtime error
                Ok(ExecutionResult::Error {
                    error_type: ErrorType::Runtime,
                    message: e.to_string(),
                    stdout: String::new(),
                    stderr: String::new(),
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
    async fn test_concurrent_wasm_execution_no_crashes() -> Result<()> {
        let sandbox = Arc::new(WasmtimeSandbox::new(WasmtimeConfig::default())?);
        let mut handles = vec![];

        // Spawn 100 concurrent executions to stress test
        for i in 0..100 {
            let sandbox_clone = Arc::clone(&sandbox);
            let handle = tokio::spawn(async move {
                let ctx = ExecutionContext::new(
                    format!("concurrent-test-{}", i),
                    serde_json::json!({"iteration": i}),
                );
                sandbox_clone.execute(SIMPLE_WASM, &ctx).await
            });
            handles.push(handle);
        }

        // All should complete without SIGABRT crashes
        for handle in handles {
            let result = handle.await??;
            assert!(matches!(result, ExecutionResult::Success { .. }));
        }

        let metrics = sandbox.get_metrics().await;
        assert_eq!(metrics.total_executions, 100);
        assert_eq!(metrics.successful_executions, 100);
        assert_eq!(metrics.failed_executions, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_metrics_tracking() -> Result<()> {
        let sandbox = WasmtimeSandbox::new(WasmtimeConfig::default())?;
        let ctx = ExecutionContext::new("test".to_string(), serde_json::json!({}));

        // Execute a few times
        for _ in 0..5 {
            sandbox.execute(SIMPLE_WASM, &ctx).await?;
        }

        let metrics = sandbox.get_metrics().await;
        assert_eq!(metrics.total_executions, 5);
        assert_eq!(metrics.successful_executions, 5);

        Ok(())
    }

    #[tokio::test]
    async fn test_health_check() -> Result<()> {
        let sandbox = WasmtimeSandbox::new(WasmtimeConfig::default())?;

        // Should be healthy initially
        assert!(sandbox.health_check().await);

        // Execute successfully
        let ctx = ExecutionContext::new("test".to_string(), serde_json::json!({}));
        sandbox.execute(SIMPLE_WASM, &ctx).await?;

        // Should still be healthy
        assert!(sandbox.health_check().await);

        Ok(())
    }

    #[tokio::test]
    async fn test_fuel_calculation() {
        // Test fuel calculation heuristic
        let one_second = Duration::from_secs(1);
        let fuel = WasmtimeSandbox::calculate_fuel(one_second);
        assert_eq!(fuel, 1_000_000); // 1 million fuel per second

        let five_seconds = Duration::from_secs(5);
        let fuel = WasmtimeSandbox::calculate_fuel(five_seconds);
        assert_eq!(fuel, 5_000_000);

        let with_millis = Duration::from_millis(1500);
        let fuel = WasmtimeSandbox::calculate_fuel(with_millis);
        assert_eq!(fuel, 1_500_000); // 1.5 million fuel
    }

    #[tokio::test]
    async fn test_timeout_enforcement_via_fuel() -> Result<()> {
        // Create a WASM module with an expensive loop
        // This module runs a tight loop incrementing a counter, consuming fuel
        let wat = r#"
            (module
              (func $main (result i32)
                (local $i i32)
                (local.set $i (i32.const 0))
                (loop $forever
                  (local.set $i (i32.add (local.get $i) (i32.const 1)))
                  (br $forever))
                (i32.const 42))
              (export "main" (func $main)))
        "#;

        // Parse WAT to WASM bytecode
        let wasm_bytecode = wat::parse_str(wat)
            .context("Failed to parse WAT")?;

        // Create sandbox with very short timeout
        let mut config = WasmtimeConfig::default();
        config.max_execution_time = Duration::from_millis(100);

        let sandbox = WasmtimeSandbox::new(config)?;
        let ctx = ExecutionContext::new("timeout-test".to_string(), serde_json::json!({}));

        // Execute infinite loop - should timeout
        let result = sandbox.execute(&wasm_bytecode, &ctx).await?;

        match result {
            ExecutionResult::Timeout { elapsed_ms, .. } => {
                // Should have timed out quickly
                assert!(elapsed_ms < 1000, "Expected timeout < 1000ms, got {}", elapsed_ms);
            }
            other => panic!("Expected timeout, got: {:?}", other),
        }

        Ok(())
    }
}

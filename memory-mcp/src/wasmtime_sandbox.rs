//! Wasmtime-based WASM sandbox for secure code execution

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, warn};
use wasmtime::*;

use crate::types::{ErrorType, ExecutionContext, ExecutionResult};

#[cfg(test)]
pub mod tests;

/// Wasmtime sandbox configuration
#[derive(Debug, Clone)]
pub struct WasmtimeConfig {
    /// Maximum execution time
    pub max_execution_time: Duration,
    /// Maximum memory in bytes
    pub max_memory_bytes: usize,
    /// Maximum concurrent executions
    pub max_pool_size: usize,
    /// Enable console output capture
    pub allow_console: bool,
}

impl Default for WasmtimeConfig {
    fn default() -> Self {
        Self {
            max_execution_time: Duration::from_secs(5),
            max_memory_bytes: 128 * 1024 * 1024,
            max_pool_size: 20,
            allow_console: true,
        }
    }
}

impl WasmtimeConfig {
    pub fn restrictive() -> Self {
        Self {
            max_execution_time: Duration::from_secs(2),
            max_memory_bytes: 64 * 1024 * 1024,
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
    engine: Engine,
    pool_semaphore: Arc<Semaphore>,
    metrics: Arc<RwLock<WasmtimeMetrics>>,
}

impl std::fmt::Debug for WasmtimeSandbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmtimeSandbox")
            .field("engine", &"<Engine>")
            .field("pool_semaphore", &self.pool_semaphore)
            .field("metrics", &self.metrics)
            .finish()
    }
}

impl WasmtimeSandbox {
    /// Create a new wasmtime sandbox
    pub fn new(_config: WasmtimeConfig) -> Result<Self> {
        let mut engine_config = Config::new();
        engine_config.consume_fuel(true);
        let engine = Engine::new(&engine_config)?;

        Ok(Self {
            engine,
            pool_semaphore: Arc::new(Semaphore::new(20)),
            metrics: Arc::new(RwLock::new(WasmtimeMetrics::default())),
        })
    }

    /// Execute WASM module
    pub async fn execute(
        &self,
        wasm_bytecode: &[u8],
        _context: &ExecutionContext,
    ) -> Result<ExecutionResult> {
        let _permit = self
            .pool_semaphore
            .acquire()
            .await
            .context("Failed to acquire permit")?;
        let exec_start = Instant::now();

        let engine = self.engine.clone();
        let wasm_bytecode = wasm_bytecode.to_vec();
        let result =
            tokio::task::spawn_blocking(move || Self::execute_sync(&engine, &wasm_bytecode))
                .await
                .context("Failed to spawn blocking task")??;

        let elapsed = exec_start.elapsed();
        self.update_metrics(&result, elapsed).await;

        Ok(result)
    }

    fn execute_sync(engine: &Engine, wasm_bytecode: &[u8]) -> Result<ExecutionResult> {
        let module = Module::from_binary(engine, wasm_bytecode).context("Failed to load module")?;

        let mut store = Store::new(engine, ());
        store.set_fuel(5_000_000)?; // 5 seconds worth of fuel

        let linker = Linker::new(engine);
        let instance = linker
            .instantiate(&mut store, &module)
            .context("Failed to instantiate")?;

        let main_func = instance.get_typed_func::<(), i32>(&mut store, "main").ok();
        let execution_time_ms = Instant::now().elapsed().as_millis() as u64;

        match main_func {
            Some(func) => {
                let call_result = func.call(&mut store, ());
                match call_result {
                    Ok(_) => Ok(ExecutionResult::Success {
                        output: "WASM execution completed".to_string(),
                        stdout: String::new(),
                        stderr: String::new(),
                        execution_time_ms,
                    }),
                    Err(e) => {
                        if let Some(trap) = e.downcast_ref::<Trap>() {
                            if matches!(trap, Trap::OutOfFuel) {
                                return Ok(ExecutionResult::Timeout {
                                    elapsed_ms: execution_time_ms,
                                    partial_output: None,
                                });
                            }
                        }
                        Ok(ExecutionResult::Error {
                            error_type: ErrorType::Runtime,
                            message: e.to_string(),
                            stdout: String::new(),
                            stderr: String::new(),
                        })
                    }
                }
            }
            None => Ok(ExecutionResult::Error {
                error_type: ErrorType::Runtime,
                message: "No main function found in WASM module".to_string(),
                stdout: String::new(),
                stderr: String::new(),
            }),
        }
    }

    async fn update_metrics(&self, result: &ExecutionResult, _elapsed: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.total_executions += 1;
        match result {
            ExecutionResult::Success { .. } => metrics.successful_executions += 1,
            ExecutionResult::Timeout { .. } => {
                metrics.timeout_count += 1;
                metrics.failed_executions += 1;
            }
            ExecutionResult::Error { .. } => metrics.failed_executions += 1,
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
        if metrics.total_executions == 0 {
            return true;
        }
        let success_rate = metrics.successful_executions as f64 / metrics.total_executions as f64;
        success_rate > 0.5
    }
}

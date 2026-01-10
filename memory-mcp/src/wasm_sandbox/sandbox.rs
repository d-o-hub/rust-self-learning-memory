//! Main WASM sandbox implementation with pool management

use super::config::{PooledRuntime, WasmConfig};
use super::executor;
use super::types::{WasmHealthStatus, WasmMetrics};
use crate::types::{ExecutionContext, ExecutionResult};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use tracing::info;

/// WASM-based JavaScript execution sandbox
#[derive(Debug)]
#[cfg(feature = "wasm-rquickjs")]
pub struct WasmSandbox {
    config: WasmConfig,
    runtime_pool: Arc<RwLock<Vec<PooledRuntime>>>,
    pool_semaphore: Arc<Semaphore>,
    metrics: Arc<RwLock<WasmMetrics>>,
}

#[cfg(feature = "wasm-rquickjs")]
impl WasmSandbox {
    /// Create a new WASM sandbox
    pub fn new(config: WasmConfig) -> Result<Self> {
        let pool_semaphore = Arc::new(Semaphore::new(config.max_pool_size));

        info!(
            "Initialized WASM sandbox with pool size: {}",
            config.max_pool_size
        );

        Ok(Self {
            config,
            runtime_pool: Arc::new(RwLock::new(Vec::new())),
            pool_semaphore,
            metrics: Arc::new(RwLock::new(WasmMetrics::default())),
        })
    }

    /// Execute code in the sandbox
    pub async fn execute(&self, code: &str, context: &ExecutionContext) -> Result<ExecutionResult> {
        executor::execute(
            code,
            context,
            &self.config,
            &self.runtime_pool,
            &self.pool_semaphore,
            &self.metrics,
        )
        .await
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> WasmMetrics {
        self.metrics.read().await.clone()
    }

    /// Cleanup expired runtimes from the pool
    pub async fn cleanup_expired_runtimes(&self) {
        let mut pool = self.runtime_pool.write().await;
        let timeout = self.config.runtime_idle_timeout;

        pool.retain(|runtime| !runtime.is_expired(timeout));

        info!("Cleaned up expired runtimes, pool size: {}", pool.len());
    }

    /// Warmup the runtime pool
    pub async fn warmup_pool(&self) -> Result<()> {
        let target_size = self.config.max_pool_size / 2;

        info!("Warming up runtime pool to {} runtimes", target_size);

        let mut pool = self.runtime_pool.write().await;

        for i in 0..target_size {
            match PooledRuntime::new(&self.config) {
                Ok(runtime) => {
                    pool.push(runtime);
                    info!("Created warmup runtime {}/{}", i + 1, target_size);
                }
                Err(e) => {
                    tracing::warn!("Failed to create warmup runtime {}: {}", i + 1, e);
                }
            }
        }

        info!("Pool warmup complete: {} runtimes ready", pool.len());

        Ok(())
    }

    /// Get health status
    pub async fn get_health_status(&self) -> WasmHealthStatus {
        let pool = self.runtime_pool.read().await;
        let metrics = self.metrics.read().await;

        let success_rate = if metrics.total_executions > 0 {
            metrics.successful_executions as f64 / metrics.total_executions as f64
        } else {
            0.0
        };

        let avg_time_ms = metrics.average_execution_time.as_millis() as u64;

        WasmHealthStatus {
            pool_size: pool.len(),
            max_pool_size: self.config.max_pool_size,
            total_executions: metrics.total_executions,
            successful_executions: metrics.successful_executions,
            failed_executions: metrics.failed_executions,
            success_rate,
            pool_hits: metrics.pool_hits,
            pool_misses: metrics.pool_misses,
            average_execution_time_ms: avg_time_ms,
        }
    }
}

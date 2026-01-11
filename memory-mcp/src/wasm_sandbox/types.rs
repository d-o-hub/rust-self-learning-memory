//! WASM sandbox types and metrics

use std::time::Duration;

/// WASM sandbox metrics
#[derive(Debug, Default, Clone)]
pub struct WasmMetrics {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time: Duration,
    pub pool_hits: u64,
    pub pool_misses: u64,
    pub memory_usage_bytes: u64,
}

/// Health status of the WASM sandbox
#[derive(Debug, Clone)]
pub struct WasmHealthStatus {
    pub pool_size: usize,
    pub max_pool_size: usize,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub success_rate: f64,
    pub pool_hits: u64,
    pub pool_misses: u64,
    pub average_execution_time_ms: u64,
}

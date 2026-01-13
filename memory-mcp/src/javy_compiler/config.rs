//! Javy compiler configuration and metrics
//!
//! Provides configuration structs and metrics tracking for the Javy compiler.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Javy compiler configuration
#[derive(Debug, Clone)]
pub struct JavyConfig {
    /// Maximum compilation time
    pub max_compilation_time: Duration,
    /// Maximum execution time (after compilation)
    pub max_execution_time: Duration,
    /// Enable WASM optimization during compilation
    pub optimize_wasm: bool,
    /// Maximum memory for compiled WASM modules
    pub max_wasm_memory_bytes: usize,
    /// Cache compiled modules
    pub enable_caching: bool,
    /// Maximum cache size (number of modules)
    pub max_cache_size: usize,
}

impl Default for JavyConfig {
    fn default() -> Self {
        Self {
            max_compilation_time: Duration::from_secs(10),
            max_execution_time: Duration::from_secs(5),
            optimize_wasm: true,
            max_wasm_memory_bytes: 128 * 1024 * 1024, // 128MB
            enable_caching: true,
            max_cache_size: 100,
        }
    }
}

impl JavyConfig {
    pub fn restrictive() -> Self {
        Self {
            max_compilation_time: Duration::from_secs(5),
            max_execution_time: Duration::from_secs(2),
            optimize_wasm: true,
            max_wasm_memory_bytes: 64 * 1024 * 1024, // 64MB
            enable_caching: true,
            max_cache_size: 50,
        }
    }
}

/// Javy execution metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JavyMetrics {
    pub total_compilations: u64,
    pub successful_compilations: u64,
    pub failed_compilations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub timeout_count: u64,
    pub avg_compilation_time_ms: f64,
    pub avg_execution_time_ms: f64,
    pub cached_modules: usize,
}

impl JavyMetrics {
    pub fn compilation_success_rate(&self) -> f64 {
        if self.total_compilations == 0 {
            0.0
        } else {
            (self.successful_compilations as f64 / self.total_compilations as f64) * 100.0
        }
    }

    pub fn execution_success_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            (self.successful_executions as f64 / self.total_executions as f64) * 100.0
        }
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / total as f64) * 100.0
        }
    }
}

/// Calculate Exponential Moving Average
pub fn calculate_ema(current: f64, new_value: f64, count: u64) -> f64 {
    if count <= 1 {
        new_value
    } else {
        let alpha = 2.0 / (count as f64 + 1.0);
        current * (1.0 - alpha) + new_value * alpha
    }
}

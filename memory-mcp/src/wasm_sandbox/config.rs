//! WASM sandbox configuration

use anyhow::Result;
#[cfg(feature = "wasm-rquickjs")]
use rquickjs::Runtime;
use std::time::{Duration, Instant};

/// WASM sandbox configuration
#[derive(Debug, Clone)]
pub struct WasmConfig {
    /// Maximum execution time per script
    pub max_execution_time: Duration,
    /// Maximum memory usage per script (in MB)
    pub max_memory_mb: usize,
    /// Whether to allow console output
    pub allow_console: bool,
    /// Maximum number of runtimes in pool
    pub max_pool_size: usize,
    /// How long a runtime can be idle before cleanup
    pub runtime_idle_timeout: Duration,
}

impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            max_execution_time: Duration::from_secs(5),
            max_memory_mb: 64,
            allow_console: true,
            max_pool_size: 10,
            runtime_idle_timeout: Duration::from_secs(60),
        }
    }
}

impl WasmConfig {
    /// Create a restrictive configuration for untrusted code
    pub fn restrictive() -> Self {
        Self {
            max_execution_time: Duration::from_secs(2),
            max_memory_mb: 32,
            allow_console: false,
            max_pool_size: 5,
            runtime_idle_timeout: Duration::from_secs(30),
        }
    }

    /// Create a permissive configuration for trusted code
    pub fn permissive() -> Self {
        Self {
            max_execution_time: Duration::from_secs(30),
            max_memory_mb: 128,
            allow_console: true,
            max_pool_size: 20,
            runtime_idle_timeout: Duration::from_secs(300),
        }
    }
}

/// Pooled WASM runtime for efficient reuse
#[cfg(feature = "wasm-rquickjs")]
pub(super) struct PooledRuntime {
    pub runtime: Runtime,
    pub last_used: Instant,
}

#[cfg(feature = "wasm-rquickjs")]
impl std::fmt::Debug for PooledRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PooledRuntime")
            .field("last_used", &self.last_used)
            .finish()
    }
}

#[cfg(feature = "wasm-rquickjs")]
impl PooledRuntime {
    pub fn new(_config: &WasmConfig) -> Result<Self> {
        let runtime = Runtime::new()?;

        Ok(Self {
            runtime,
            last_used: Instant::now(),
        })
    }

    pub fn is_expired(&self, timeout: Duration) -> bool {
        self.last_used.elapsed() > timeout
    }

    pub fn touch(&mut self) {
        self.last_used = Instant::now();
    }
}

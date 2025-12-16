//! Javy-based JavaScript to WASM compiler for secure code execution
//!
//! This module provides JavaScript → WASM compilation using Javy,
//! enabling JavaScript execution in the same sandbox environment as WASM.
//!
//! ## Architecture
//!
//! - **Javy Engine**: Compile JavaScript source to optimized WASM
//! - **Pre-compilation**: Cache compiled WASM modules for reuse
//! - **WASM Execution**: Execute compiled JavaScript via wasmtime
//! - **Metrics**: Track compilation and execution statistics
//!
//! ## Features
//!
//! - JavaScript source to WASM compilation
//! - WASM module caching for performance
//! - Integration with existing UnifiedSandbox
//! - Comprehensive metrics and health monitoring
//! - Fuel-based execution timeouts

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::debug;

use crate::types::{ExecutionContext, ExecutionResult};
use crate::wasmtime_sandbox::{WasmtimeConfig, WasmtimeSandbox};

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

/// Compiled WASM module cache
#[derive(Debug)]
struct ModuleCache {
    modules: HashMap<String, Vec<u8>>,
    access_order: Vec<String>,
    max_size: usize,
}

impl ModuleCache {
    fn new(max_size: usize) -> Self {
        Self {
            modules: HashMap::new(),
            access_order: Vec::new(),
            max_size,
        }
    }

    fn get(&mut self, key: &str) -> Option<&Vec<u8>> {
        // Move to end of access order (most recently used)
        if let Some(pos) = self.access_order.iter().position(|k| k == key) {
            let key = self.access_order.remove(pos);
            self.access_order.push(key);
        }
        self.modules.get(key)
    }

    fn insert(&mut self, key: String, module: Vec<u8>) {
        // Remove oldest if at capacity
        if self.modules.len() >= self.max_size && !self.modules.contains_key(&key) {
            if let Some(oldest) = self.access_order.first().cloned() {
                self.modules.remove(&oldest);
                self.access_order.remove(0);
            }
        }

        self.modules.insert(key.clone(), module);

        // Update access order
        if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
            self.access_order.remove(pos);
        }
        self.access_order.push(key);
    }

    fn len(&self) -> usize {
        self.modules.len()
    }
}

/// Javy JavaScript to WASM compiler
#[derive(Debug)]
pub struct JavyCompiler {
    config: JavyConfig,
    #[allow(dead_code)]
    wasmtime_config: WasmtimeConfig,
    wasmtime_sandbox: Arc<WasmtimeSandbox>,
    metrics: Arc<RwLock<JavyMetrics>>,
    module_cache: Arc<Mutex<ModuleCache>>,
    compilation_semaphore: Arc<Semaphore>,
}

impl JavyCompiler {
    /// Create a new Javy compiler instance
    pub fn new(config: JavyConfig) -> Result<Self> {
        let wasmtime_config = WasmtimeConfig {
            max_execution_time: config.max_execution_time,
            max_memory_bytes: config.max_wasm_memory_bytes,
            max_pool_size: 20,
            allow_console: true,
        };

        let wasmtime_sandbox = Arc::new(WasmtimeSandbox::new(wasmtime_config.clone())?);

        let module_cache = if config.enable_caching {
            Arc::new(Mutex::new(ModuleCache::new(config.max_cache_size)))
        } else {
            Arc::new(Mutex::new(ModuleCache::new(0)))
        };

        let compilation_semaphore = Arc::new(Semaphore::new(5)); // Limit concurrent compilations

        Ok(Self {
            config,
            wasmtime_config,
            wasmtime_sandbox,
            metrics: Arc::new(RwLock::new(JavyMetrics::default())),
            module_cache,
            compilation_semaphore,
        })
    }

    /// Compile JavaScript source to WASM
    pub async fn compile_js_to_wasm(&self, js_source: &str) -> Result<Vec<u8>> {
        let _permit = self
            .compilation_semaphore
            .acquire()
            .await
            .context("Failed to acquire compilation semaphore")?;

        let start = Instant::now();

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_compilations += 1;
        }

        // Check cache first
        let cache_key = self.generate_cache_key(js_source);
        let cached_module_opt = {
            let cache = self.module_cache.lock().unwrap();
            cache.get(&cache_key).cloned()
        };

        if let Some(cached_module) = cached_module_opt {
            let mut metrics = self.metrics.write().await;
            metrics.cache_hits += 1;
            debug!("Using cached WASM module for JavaScript source");
            return Ok(cached_module);
        }

        // Perform compilation with timeout
        let compilation_result = tokio::time::timeout(
            self.config.max_compilation_time,
            self.perform_compilation(js_source),
        )
        .await
        .context("JavaScript compilation timed out")?;

        let wasm_bytes = match compilation_result {
            Ok(wasm) => {
                let mut metrics = self.metrics.write().await;
                metrics.successful_compilations += 1;
                metrics.avg_compilation_time_ms = calculate_ema(
                    metrics.avg_compilation_time_ms,
                    start.elapsed().as_millis() as f64,
                    metrics.successful_compilations,
                );

                // Cache the compiled module
                if self.config.enable_caching {
                    self.module_cache
                        .lock()
                        .unwrap()
                        .insert(cache_key, wasm.clone());
                }

                wasm
            }
            Err(e) => {
                let mut metrics = self.metrics.write().await;
                metrics.failed_compilations += 1;
                metrics.cache_misses += 1;
                return Err(e).context("JavaScript compilation failed");
            }
        };

        Ok(wasm_bytes)
    }

    /// Execute JavaScript code (compile + run)
    pub async fn execute_js(
        &self,
        js_source: String,
        context: ExecutionContext,
    ) -> Result<ExecutionResult> {
        debug!("Executing JavaScript code via Javy compiler");

        // Compile JavaScript to WASM
        let wasm_bytes = self.compile_js_to_wasm(&js_source).await?;

        // Execute the compiled WASM
        let result = self.wasmtime_sandbox.execute(&wasm_bytes, &context).await?;

        // Update execution metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_executions += 1;
            match &result {
                ExecutionResult::Success { .. } => {
                    metrics.successful_executions += 1;
                }
                _ => {
                    metrics.failed_executions += 1;
                }
            }
        }

        Ok(result)
    }

    /// Validate JavaScript syntax (basic validation)
    pub fn validate_js_syntax(&self, js_source: &str) -> Result<()> {
        // Basic syntax validation - check for balanced braces, brackets, parentheses
        let mut brace_count = 0;
        let mut bracket_count = 0;
        let mut paren_count = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for (i, ch) in js_source.char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }

            if ch == '\\' {
                escape_next = true;
                continue;
            }

            if ch == '"' || ch == '\'' {
                in_string = !in_string;
                continue;
            }

            if in_string {
                continue;
            }

            match ch {
                '{' => brace_count += 1,
                '}' => {
                    if brace_count == 0 {
                        return Err(anyhow::anyhow!("Unmatched closing brace at position {}", i));
                    }
                    brace_count -= 1;
                }
                '[' => bracket_count += 1,
                ']' => {
                    if bracket_count == 0 {
                        return Err(anyhow::anyhow!(
                            "Unmatched closing bracket at position {}",
                            i
                        ));
                    }
                    bracket_count -= 1;
                }
                '(' => paren_count += 1,
                ')' => {
                    if paren_count == 0 {
                        return Err(anyhow::anyhow!(
                            "Unmatched closing parenthesis at position {}",
                            i
                        ));
                    }
                    paren_count -= 1;
                }
                _ => {}
            }
        }

        if brace_count != 0 {
            return Err(anyhow::anyhow!("Unmatched opening braces: {}", brace_count));
        }
        if bracket_count != 0 {
            return Err(anyhow::anyhow!(
                "Unmatched opening brackets: {}",
                bracket_count
            ));
        }
        if paren_count != 0 {
            return Err(anyhow::anyhow!(
                "Unmatched opening parentheses: {}",
                paren_count
            ));
        }

        Ok(())
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> JavyMetrics {
        let metrics = self.metrics.read().await;
        JavyMetrics {
            cached_modules: self.module_cache.lock().unwrap().len(),
            ..metrics.clone()
        }
    }

    /// Check compiler health
    pub async fn health_check(&self) -> Result<()> {
        // Check if wasmtime sandbox is healthy
        let healthy = self.wasmtime_sandbox.health_check().await;
        if !healthy {
            return Err(anyhow::anyhow!(
                "Wasmtime sandbox reported unhealthy status"
            ));
        }

        // Check cache health
        let _ = self.module_cache.lock().unwrap().len();

        Ok(())
    }

    /// Clear the module cache
    pub async fn clear_cache(&self) {
        self.module_cache.lock().unwrap().modules.clear();
        self.module_cache.lock().unwrap().access_order.clear();
    }

    /// Perform the actual JavaScript to WASM compilation
    #[cfg(feature = "javy-backend")]
    async fn perform_compilation(&self, js_source: &str) -> Result<Vec<u8>> {
        use javy_codegen::{Generator, LinkingKind, JS};
        let source = js_source.to_string();
        let source_len = source.len();
        tokio::task::spawn_blocking(move || {
            let js = JS::from_string(source);
            let mut gen = Generator::default();
            gen.linking(LinkingKind::Dynamic);
            let wasm = gen.generate(&js).context("Failed to generate WASM")?;
            info!(
                "Compiled JS ({} bytes) to WASM ({} bytes)",
                source_len,
                wasm.len()
            );
            Ok(wasm)
        })
        .await
        .context("Compilation task panicked")?
    }

    #[cfg(not(feature = "javy-backend"))]
    async fn perform_compilation(&self, js_source: &str) -> Result<Vec<u8>> {
        Err(anyhow::anyhow!(
            "Javy backend not enabled. Compile with --features javy-backend\n\
             Source: {} bytes",
            js_source.len()
        ))
    }

    /// Generate a cache key for the JavaScript source
    fn generate_cache_key(&self, js_source: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        js_source.hash(&mut hasher);
        format!("js_{:x}", hasher.finish())
    }
}

/// Calculate Exponential Moving Average
fn calculate_ema(current: f64, new_value: f64, count: u64) -> f64 {
    if count <= 1 {
        new_value
    } else {
        let alpha = 2.0 / (count as f64 + 1.0);
        current * (1.0 - alpha) + new_value * alpha
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_javy_compiler_creation() {
        let config = JavyConfig::default();
        let compiler = JavyCompiler::new(config);
        assert!(compiler.is_ok());
    }

    #[tokio::test]
    async fn test_js_syntax_validation() {
        let compiler = JavyCompiler::new(JavyConfig::default()).unwrap();
        assert!(compiler.validate_js_syntax("const x = 1;").is_ok());
        assert!(compiler.validate_js_syntax("const x = {;").is_err());
    }

    #[tokio::test]
    async fn test_metrics_initialization() {
        let compiler = JavyCompiler::new(JavyConfig::default()).unwrap();
        let m = compiler.get_metrics().await;
        assert_eq!(m.total_compilations, 0);
        assert_eq!(m.successful_compilations, 0);
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let compiler = JavyCompiler::new(JavyConfig::default()).unwrap();
        let k1 = compiler.generate_cache_key("const x = 1;");
        let k2 = compiler.generate_cache_key("const x = 1;");
        let k3 = compiler.generate_cache_key("const x = 2;");
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
    }
}

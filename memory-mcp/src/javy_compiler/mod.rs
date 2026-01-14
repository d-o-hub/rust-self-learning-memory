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

mod cache;
mod config;
mod utils;

pub use config::{calculate_ema, JavyConfig, JavyMetrics};
pub use utils::{generate_cache_key, is_valid_wasm_file, validate_js_syntax};

use crate::types::{ExecutionContext, ExecutionResult};
use crate::wasmtime_sandbox::{WasmtimeConfig, WasmtimeSandbox};

use anyhow::{anyhow, Context, Result};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, warn};

use cache::ModuleCache;

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
    /// Create a new Javy compiler with the given configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the WASM sandbox cannot be initialized or if the configuration is invalid.
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

    /// Compile JavaScript source code to WASM bytecode
    ///
    /// # Arguments
    ///
    /// * `js_source` - The JavaScript source code to compile
    ///
    /// # Returns
    ///
    /// WASM bytecode as a vector of bytes
    ///
    /// # Errors
    ///
    /// Returns an error if compilation fails, times out, or if the JavaScript syntax is invalid.
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
        let cache_key = generate_cache_key(js_source);
        let cached_module_opt = {
            let mut cache = self
                .module_cache
                .lock()
                .map_err(|e| anyhow!("Failed to lock module cache: {}", e))?;
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
                        .map_err(|e| anyhow!("Failed to lock module cache for caching: {}", e))?
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
        validate_js_syntax(js_source)
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> JavyMetrics {
        let metrics = self.metrics.read().await;
        let cached_modules = self
            .module_cache
            .lock()
            .map(|cache| cache.len())
            .unwrap_or_else(|e| {
                warn!("Failed to lock module cache: {}", e);
                0
            });
        JavyMetrics {
            cached_modules,
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
        if let Err(e) = self.module_cache.lock() {
            warn!("Failed to lock module cache: {}", e);
        }

        Ok(())
    }

    /// Clear the module cache
    pub async fn clear_cache(&self) {
        let mut cache = self
            .module_cache
            .lock()
            .map_err(|e| {
                warn!("Failed to lock module cache for clearing: {}", e);
            })
            .ok();

        if let Some(ref mut cache) = cache {
            cache.clear();
        }
    }

    /// Perform the actual JavaScript to WASM compilation
    #[cfg(feature = "javy-backend")]
    async fn perform_compilation(&self, js_source: &str) -> Result<Vec<u8>> {
        use javy_codegen::{Generator, LinkingKind, JS};
        use std::env;
        // Ensure Javy plugin path is available for codegen. If not provided by the environment,
        // default to the plugin bundled with this crate.
        if env::var("JAVY_PLUGIN").is_err() {
            let default_path = format!("{}/javy-plugin.wasm", env!("CARGO_MANIFEST_DIR"));
            if std::fs::metadata(&default_path).is_ok() {
                env::set_var("JAVY_PLUGIN", &default_path);
                debug!(
                    "JAVY_PLUGIN not set; using bundled plugin at {}",
                    default_path
                );
            } else {
                // Fallback: write embedded plugin bytes to a temp file and use that
                const PLUGIN_BYTES: &[u8] = include_bytes!("../javy-plugin.wasm");
                let tmp_path = std::env::temp_dir().join("memory_mcp_javy_plugin.wasm");
                match std::fs::write(&tmp_path, PLUGIN_BYTES) {
                    Ok(_) => {
                        env::set_var("JAVY_PLUGIN", &tmp_path);
                        debug!(
                            "JAVY_PLUGIN not set; wrote embedded plugin to {}",
                            tmp_path.display()
                        );
                    }
                    Err(e) => {
                        debug!(
                            "Failed to write embedded Javy plugin to {}: {:?}",
                            tmp_path.display(),
                            e
                        );
                    }
                }
            }
        }

        let source = js_source.to_string();
        let source_len = source.len();

        // Try using Javy plugin (wasm plugin) first; if not available, fall back to the javy CLI.
        let js_clone = source.clone();
        tokio::task::spawn_blocking(move || {
            // If a JAVY_PLUGIN is provided and looks like a valid WASM, prefer the plugin + codegen path
            if let Ok(plugin_path) = std::env::var("JAVY_PLUGIN") {
                let plugin = Path::new(&plugin_path);
                if plugin.exists() && is_valid_wasm_file(plugin) {
                    let js = JS::from_string(js_clone);
                    let mut gen = Generator::default();
                    gen.linking(LinkingKind::Dynamic);
                    let wasm = gen.generate(&js).context("Failed to generate WASM")?;
                    debug!(
                        "Compiled JS ({} bytes) to WASM ({} bytes) via plugin {}",
                        source_len,
                        wasm.len(),
                        plugin_path
                    );
                    return Ok(wasm);
                } else if plugin.exists() {
                    debug!(
                        "JAVY_PLUGIN {} exists but is not a valid WASM file (too small or wrong format); attempting CLI fallback",
                        plugin_path
                    );
                }
            }

            // Try default bundled plugin path
            let default_path = format!("{}/javy-plugin.wasm", env!("CARGO_MANIFEST_DIR"));
            let default_plugin = Path::new(&default_path);
            if default_plugin.exists() && is_valid_wasm_file(default_plugin) {
                let js = JS::from_string(js_clone);
                let mut gen = Generator::default();
                gen.linking(LinkingKind::Dynamic);
                let wasm = gen.generate(&js).context("Failed to generate WASM")?;
                debug!(
                    "Compiled JS ({} bytes) to WASM ({} bytes) via bundled plugin {}",
                    source_len,
                    wasm.len(),
                    default_path
                );
                return Ok(wasm);
            } else if default_plugin.exists() {
                debug!(
                    "Bundled plugin {} exists but is not a valid WASM file (too small or wrong format); attempting CLI fallback",
                    default_path
                );
            }

            // Plugin not available or invalid — attempt CLI fallback
            debug!("Javy plugin not available or invalid; attempting javy CLI fallback");
            use std::io::Write;
            use std::process::{Command, Stdio};

            let mut child = Command::new("javy")
                .arg("compile")
                .arg("-o")
                .arg("-")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .context("Failed to spawn javy CLI. Please install javy: cargo install javy")?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(source.as_bytes())
                    .context("Writing JS to javy stdin failed")?;
            }

            let output = child
                .wait_with_output()
                .context("Failed to read javy CLI output")?;

            if !output.status.success() {
                return Err(anyhow::anyhow!(
                    "javy CLI compilation failed. Error: {}. Please install javy: cargo install javy",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            let wasm = output.stdout;
            if wasm.len() < 4 || &wasm[0..4] != b"\0asm" {
                return Err(anyhow::anyhow!(
                    "javy CLI produced non-wasm output ({} bytes)",
                    wasm.len()
                ));
            }

            debug!(
                "Compiled JS ({} bytes) to WASM ({} bytes) via javy CLI",
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
}

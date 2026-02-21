//! WASM sandbox execution logic
//!
//! This module contains all the execution-related methods for the WasmSandbox,
//! including retry logic, timeout handling, and secure environment setup.

use super::config::{PooledRuntime, WasmConfig};
use super::types::WasmMetrics;
use crate::types::{ExecutionContext, ExecutionResult};
use anyhow::{Result, anyhow};
#[cfg(feature = "wasm-rquickjs")]
use rquickjs::{Context, Ctx, Function, Object, Value};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{error, info};

/// Execute code in the WASM sandbox
#[cfg(feature = "wasm-rquickjs")]
pub async fn execute(
    code: &str,
    context: &ExecutionContext,
    config: &WasmConfig,
    runtime_pool: &Arc<RwLock<Vec<PooledRuntime>>>,
    pool_semaphore: &Arc<Semaphore>,
    metrics: &Arc<RwLock<WasmMetrics>>,
) -> Result<ExecutionResult> {
    let max_retries = 3;

    for attempt in 0..max_retries {
        match execute_with_retry(
            code,
            context,
            attempt,
            config,
            runtime_pool,
            pool_semaphore,
            metrics,
        )
        .await
        {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries - 1 => {
                let backoff_ms = 100 * (2u64.pow(attempt as u32));
                tracing::warn!(
                    "WASM execution attempt {} failed: {}, retrying in {}ms",
                    attempt + 1,
                    e,
                    backoff_ms
                );
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                continue;
            }
            Err(e) => {
                let mut metrics_guard = metrics.write().await;
                metrics_guard.total_executions += 1;
                metrics_guard.failed_executions += 1;
                error!(
                    "WASM execution failed after {} attempts: {}",
                    max_retries, e
                );
                return Err(e);
            }
        }
    }

    Err(anyhow!("internal: exhausted retry loop"))
}

/// Execute with retry attempt tracking
#[cfg(feature = "wasm-rquickjs")]
async fn execute_with_retry(
    code: &str,
    context: &ExecutionContext,
    attempt: usize,
    config: &WasmConfig,
    runtime_pool: &Arc<RwLock<Vec<PooledRuntime>>>,
    pool_semaphore: &Arc<Semaphore>,
    metrics: &Arc<RwLock<WasmMetrics>>,
) -> Result<ExecutionResult> {
    let _permit = pool_semaphore.acquire().await.map_err(|e| {
        anyhow!(
            "Failed to acquire runtime permit (attempt {}): {}",
            attempt,
            e
        )
    })?;

    let start_time = Instant::now();

    let result = execute_with_timeout(code, context, config, runtime_pool, metrics).await;

    let execution_time = start_time.elapsed();
    info!(
        "WASM execution attempt {} completed in {:?}",
        attempt + 1,
        execution_time
    );

    result
}

/// Execute with timeout enforcement
#[cfg(feature = "wasm-rquickjs")]
async fn execute_with_timeout(
    code: &str,
    context: &ExecutionContext,
    config: &WasmConfig,
    runtime_pool: &Arc<RwLock<Vec<PooledRuntime>>>,
    metrics: &Arc<RwLock<WasmMetrics>>,
) -> Result<ExecutionResult> {
    tokio::time::timeout(
        config.max_execution_time,
        execute_with_runtime(code, context, config, runtime_pool, metrics),
    )
    .await
    .map_err(|_| anyhow!("Execution timeout after {:?}", config.max_execution_time))?
}

/// Acquire a runtime from the pool
#[cfg(feature = "wasm-rquickjs")]
pub async fn acquire_runtime(
    config: &WasmConfig,
    runtime_pool: &Arc<RwLock<Vec<PooledRuntime>>>,
    metrics: &Arc<RwLock<WasmMetrics>>,
) -> Result<PooledRuntime> {
    let mut pool = runtime_pool.write().await;

    // Try to reuse an existing runtime
    if let Some(mut runtime) = pool.pop() {
        runtime.touch();
        let mut metrics_guard = metrics.write().await;
        metrics_guard.pool_hits += 1;
        drop(metrics_guard);
        drop(pool);
        return Ok(runtime);
    }

    // Create new runtime if pool is empty
    drop(pool);
    let mut metrics_guard = metrics.write().await;
    metrics_guard.pool_misses += 1;
    drop(metrics_guard);

    PooledRuntime::new(config)
}

/// Release a runtime back to the pool
#[cfg(feature = "wasm-rquickjs")]
pub async fn release_runtime(
    runtime: PooledRuntime,
    config: &WasmConfig,
    runtime_pool: &Arc<RwLock<Vec<PooledRuntime>>>,
) {
    let mut pool = runtime_pool.write().await;

    if pool.len() < config.max_pool_size && !runtime.is_expired(config.runtime_idle_timeout) {
        pool.push(runtime);
    }
}

/// Execute code with an acquired runtime
#[cfg(feature = "wasm-rquickjs")]
async fn execute_with_runtime(
    code: &str,
    context: &ExecutionContext,
    config: &WasmConfig,
    runtime_pool: &Arc<RwLock<Vec<PooledRuntime>>>,
    metrics: &Arc<RwLock<WasmMetrics>>,
) -> Result<ExecutionResult> {
    let runtime = acquire_runtime(config, runtime_pool, metrics).await?;

    let result = {
        let ctx = Context::full(&runtime.runtime)?;

        ctx.with(|ctx| -> Result<ExecutionResult> {
            // Setup secure environment
            setup_secure_environment(&ctx, config)?;

            // Inject context - use 'input' field instead of 'data'
            let global = ctx.globals();
            let context_str = serde_json::to_string(&context.input)?;
            global.set("__context", context_str)?;

            // Execute code
            let result_value: Value = ctx.eval(code)?;

            // Convert result to JSON string using stringify
            let result_json: String = if result_value.is_undefined() || result_value.is_null() {
                "null".to_string()
            } else if let Ok(Some(s)) = ctx.json_stringify(result_value.clone()) {
                // Convert rquickjs::String to std::String, handling potential error
                match s.to_string() {
                    Ok(str) => str,
                    Err(_) => "null".to_string(),
                }
            } else {
                "null".to_string()
            };

            // Return success result
            Ok(ExecutionResult::Success {
                output: result_json.clone(),
                stdout: result_json,
                stderr: String::new(),
                execution_time_ms: 0,
            })
        })?
    };

    // Release runtime back to pool
    release_runtime(runtime, config, runtime_pool).await;

    // Update metrics
    let mut metrics_guard = metrics.write().await;
    metrics_guard.total_executions += 1;
    metrics_guard.successful_executions += 1;

    Ok(result)
}

/// Setup secure environment with restricted globals
#[cfg(feature = "wasm-rquickjs")]
fn setup_secure_environment(ctx: &Ctx, config: &WasmConfig) -> Result<()> {
    let global = ctx.globals();

    // Remove dangerous globals
    let dangerous = ["eval", "Function", "require", "import", "process", "global"];
    for &name in &dangerous {
        let _ = global.remove(name);
    }

    // Setup console if allowed
    if config.allow_console {
        setup_safe_console(ctx)?;
    } else {
        let _ = global.remove("console");
    }

    Ok(())
}

/// Setup a safe console implementation
#[cfg(feature = "wasm-rquickjs")]
fn setup_safe_console(ctx: &Ctx) -> Result<()> {
    let global = ctx.globals();
    let console = Object::new(ctx.clone())?;

    // Create log function - simplify by just taking one string argument
    let log_fn = Function::new(ctx.clone(), |_ctx: Ctx, msg: String| {
        info!("[WASM Console] {}", msg);
        Ok::<(), rquickjs::Error>(())
    })?;

    console.set("log", log_fn.clone())?;
    console.set("info", log_fn.clone())?;
    console.set("warn", log_fn.clone())?;
    console.set("error", log_fn)?;

    global.set("console", console)?;

    Ok(())
}

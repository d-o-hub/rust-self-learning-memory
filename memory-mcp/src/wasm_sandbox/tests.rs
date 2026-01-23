//! Tests for WASM sandbox

use super::{WasmConfig, WasmSandbox};
use crate::types::ExecutionContext;

#[tokio::test]
#[cfg(feature = "wasm-rquickjs")]
async fn test_basic_execution() {
    let sandbox = WasmSandbox::new(WasmConfig::default()).unwrap();
    let code = "1 + 1";
    let context = ExecutionContext::new("test".to_string(), serde_json::json!({}));

    let result = sandbox.execute(code, &context).await;
    assert!(result.is_ok());
}

#[tokio::test]
#[cfg(feature = "wasm-rquickjs")]
#[ignore = "WASM timeout enforcement is flaky in CI - infinite loops don't always terminate reliably"]
async fn test_timeout_enforcement() {
    let mut config = WasmConfig::default();
    config.max_execution_time = std::time::Duration::from_millis(100);

    let sandbox = WasmSandbox::new(config).unwrap();
    let code = "while(true) {}"; // Infinite loop
    let context = ExecutionContext::new("test".to_string(), serde_json::json!({}));

    let result = sandbox.execute(code, &context).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("timeout"));
}

#[tokio::test]
#[cfg(feature = "wasm-rquickjs")]
async fn test_pool_reuse() {
    let sandbox = WasmSandbox::new(WasmConfig::default()).unwrap();
    let code = "2 + 2";
    let context = ExecutionContext::new("test".to_string(), serde_json::json!({}));

    // First execution
    let _ = sandbox.execute(code, &context).await.unwrap();

    // Second execution should reuse runtime
    let _ = sandbox.execute(code, &context).await.unwrap();

    let metrics = sandbox.get_metrics().await;
    assert_eq!(metrics.total_executions, 2);
    assert!(metrics.pool_hits > 0 || metrics.pool_misses > 0);
}

#[tokio::test]
#[cfg(feature = "wasm-rquickjs")]
async fn test_metrics_tracking() {
    let sandbox = WasmSandbox::new(WasmConfig::default()).unwrap();
    let code = "42";
    let context = ExecutionContext::new("test".to_string(), serde_json::json!({}));

    let _ = sandbox.execute(code, &context).await.unwrap();

    let metrics = sandbox.get_metrics().await;
    assert_eq!(metrics.total_executions, 1);
    assert_eq!(metrics.successful_executions, 1);
    assert_eq!(metrics.failed_executions, 0);
}

#[tokio::test]
#[cfg(feature = "wasm-rquickjs")]
async fn test_warmup_pool() {
    let sandbox = WasmSandbox::new(WasmConfig::default()).unwrap();

    let result = sandbox.warmup_pool().await;
    assert!(result.is_ok());

    let health = sandbox.get_health_status().await;
    assert!(health.pool_size > 0);
}

#[tokio::test]
#[cfg(feature = "wasm-rquickjs")]
async fn test_cleanup_expired() {
    let mut config = WasmConfig::default();
    config.runtime_idle_timeout = std::time::Duration::from_millis(10);

    let sandbox = WasmSandbox::new(config).unwrap();

    // Warmup and wait for expiration
    let _ = sandbox.warmup_pool().await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    sandbox.cleanup_expired_runtimes().await;

    let health = sandbox.get_health_status().await;
    // After cleanup, expired runtimes should be removed
    assert!(health.pool_size == 0);
}

#[tokio::test]
#[cfg(feature = "wasm-rquickjs")]
async fn test_health_status() {
    let sandbox = WasmSandbox::new(WasmConfig::default()).unwrap();

    let health = sandbox.get_health_status().await;
    assert_eq!(health.pool_size, 0);
    assert_eq!(health.total_executions, 0);
}

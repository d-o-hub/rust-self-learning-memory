//! Tests for wasmtime_sandbox module.

use super::*;
use anyhow::Result;

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
async fn test_wasi_stdout_stderr_capture() -> Result<()> {
    // Create a WASM module that writes to stdout and stderr using WASI
    let wat = r#"
        (module
          (import "wasi_snapshot_preview1" "fd_write"
            (func $fd_write (param i32 i32 i32 i32) (result i32)))
          (memory (export "memory") 1)
          (func (export "main") (result i32)
            (i32.store (i32.const 0) (i32.const 0x0a202020))
            (i32.store8 (i32.const 3) (i32.const 0x6f))
            (i32.store8 (i32.const 4) (i32.const 0x6c))
            (i32.store8 (i32.const 5) (i32.const 0x6c))
            (i32.store8 (i32.const 6) (i32.const 0x65))
            (i32.store8 (i32.const 7) (i32.const 0x48))
            (i32.store8 (i32.const 8) (i32.const 0x0a))
            (i32.store8 (i32.const 9) (i32.const 0x74))
            (i32.store8 (i32.const 10) (i32.const 0x6f))
            (i32.store8 (i32.const 11) (i32.const 0x75))
            (i32.store8 (i32.const 12) (i32.const 0x74))
            (i32.store8 (i32.const 13) (i32.const 0x66))
            (i32.store8 (i32.const 14) (i32.const 0x6f))
            (i32.store8 (i32.const 15) (i32.const 0x73))
            (i32.store8 (i32.const 16) (i32.const 0x21))
            i32.const 0
            i32.const 100
            i32.store
            i32.const 17
            i32.const 104
            i32.store
            i32.const 1
            i32.const 100
            i32.const 1
            i32.const 108
            call $fd_write
            drop
            i32.const 0
          )
        )
    "#;

    let wasm_bytecode = wat::parse_str(wat).context("Failed to parse WAT")?;

    let config = WasmtimeConfig {
        allow_console: true,
        ..Default::default()
    };
    let sandbox = WasmtimeSandbox::new(config)?;
    let ctx = ExecutionContext::new("wasi-test".to_string(), serde_json::json!({}));

    let result = sandbox.execute(&wasm_bytecode, &ctx).await?;

    match result {
        ExecutionResult::Success { .. } => {}
        other => panic!("Expected success, got: {:?}", other),
    }

    let config_disabled = WasmtimeConfig {
        allow_console: false,
        ..Default::default()
    };
    let sandbox_disabled = WasmtimeSandbox::new(config_disabled)?;

    let result_disabled = sandbox_disabled.execute(&wasm_bytecode, &ctx).await?;

    match result_disabled {
        ExecutionResult::Success { stdout, stderr, .. } => {
            assert!(stdout.is_empty());
            assert!(stderr.is_empty());
        }
        other => panic!("Expected success with empty output, got: {:?}", other),
    }

    Ok(())
}

#[tokio::test]
async fn test_wasi_capture_with_timeout() -> Result<()> {
    let wat = r#"
        (module
          (import "wasi_snapshot_preview1" "fd_write"
            (func $fd_write (param i32 i32 i32 i32) (result i32)))
          (memory (export "memory") 1)
          (func (export "main") (result i32)
            (i32.store (i32.const 0) (i32.const 0x0a202020))
            (i32.store8 (i32.const 3) (i32.const 0x70))
            (i32.store8 (i32.const 4) (i32.const 0x65))
            (i32.store8 (i32.const 5) (i32.const 0x72))
            (i32.store8 (i32.const 6) (i32.const 0x65))
            (i32.store8 (i32.const 7) (i32.const 0x66))
            (i32.store8 (i32.const 8) (i32.const 0x6f))
            (i32.store8 (i32.const 9) (i32.const 0x72))
            (i32.store8 (i32.const 10) (i32.const 0x65))
            (i32.store8 (i32.const 11) (i32.const 0x20))
            (i32.store8 (i32.const 12) (i32.const 0x69))
            (i32.store8 (i32.const 13) (i32.const 0x6e))
            (i32.store8 (i32.const 14) (i32.const 0x66))
            (i32.store8 (i32.const 15) (i32.const 0x69))
            (i32.store8 (i32.const 16) (i32.const 0x6e))
            (i32.store8 (i32.const 17) (i32.const 0x69))
            (i32.store8 (i32.const 18) (i32.const 0x74))
            (i32.store8 (i32.const 19) (i32.const 0x65))
            (i32.store8 (i32.const 20) (i32.const 0x20))
            (i32.store8 (i32.const 21) (i32.const 0x6c))
            (i32.store8 (i32.const 22) (i32.const 0x6f))
            (i32.store8 (i32.const 23) (i32.const 0x6f))
            (i32.store8 (i32.const 24) (i32.const 0x70))
            i32.const 0
            i32.const 100
            i32.store
            i32.const 21
            i32.const 104
            i32.store
            i32.const 1
            i32.const 100
            i32.const 1
            i32.const 108
            call $fd_write
            drop
            (loop $forever br $forever)
            i32.const 0)
        )
    "#;

    let wasm_bytecode = wat::parse_str(wat).context("Failed to parse WAT")?;

    let config = WasmtimeConfig {
        max_execution_time: Duration::from_millis(100),
        allow_console: true,
        ..Default::default()
    };

    let sandbox = WasmtimeSandbox::new(config)?;
    let ctx = ExecutionContext::new("timeout-test".to_string(), serde_json::json!({}));

    let result = sandbox.execute(&wasm_bytecode, &ctx).await?;

    match result {
        ExecutionResult::Timeout { elapsed_ms, .. } => {
            assert!(
                elapsed_ms < 5000,
                "elapsed_ms {} unexpectedly large",
                elapsed_ms
            );
        }
        other => panic!("Expected timeout, got: {:?}", other),
    }

    Ok(())
}

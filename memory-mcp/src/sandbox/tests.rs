//! Tests for the sandbox module.

use serde_json::json;

use super::*;
use crate::types::{ErrorType, ExecutionContext, ExecutionResult, SecurityViolationType};

/// Create a test execution context
fn create_test_context() -> ExecutionContext {
    ExecutionContext::new("test".to_string(), json!({}))
}

/// Set environment once for tests to disable WASM
fn set_once() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        std::env::set_var("MCP_USE_WASM", "false");
    });
}

#[tokio::test]
#[cfg(not(feature = "wasm-rquickjs"))]
async fn test_simple_execution() {
    set_once();
    let sandbox = CodeSandbox::new(SandboxConfig::default()).unwrap();
    let code = "return 1 + 1;";
    let context = create_test_context();

    let result = sandbox.execute(code, context).await.unwrap();

    match result {
        ExecutionResult::Success { .. } => {
            // Success expected
        }
        other => panic!("Expected success, got: {:?}", other),
    }
}

#[tokio::test]
#[cfg(not(feature = "wasm-rquickjs"))]
async fn test_console_output() {
    set_once();
    let sandbox = CodeSandbox::new(SandboxConfig::default()).unwrap();
    let code = r#"
        console.log("Hello");
        console.log("World");
        return "done";
    "#;
    let context = create_test_context();

    let result = sandbox.execute(code, context).await.unwrap();

    match result {
        ExecutionResult::Success { stdout, .. } => {
            assert!(stdout.contains("Hello"));
            assert!(stdout.contains("World"));
        }
        other => panic!("Expected success, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_timeout_enforcement() {
    set_once();
    let config = SandboxConfig {
        max_execution_time_ms: 500, // 500ms timeout
        ..Default::default()
    };

    let sandbox = CodeSandbox::new(config).unwrap();
    let code = r#"
        let sum = 0;
        for (let i = 0; i < 10000000000; i++) {
            sum += i;
            if (i % 1000000 === 0) {
                const x = sum;
            }
        }
        return sum;
    "#;
    let context = create_test_context();

    let result = sandbox.execute(code, context).await.unwrap();

    match result {
        ExecutionResult::Timeout { .. } => {
            // Timeout expected
        }
        other => panic!("Expected timeout, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_filesystem_blocking() {
    set_once();
    let sandbox = CodeSandbox::new(SandboxConfig::default()).unwrap();
    let code = r#"
        const fs = require('fs');
        fs.readFileSync('/etc/passwd');
    "#;
    let context = create_test_context();

    let result = sandbox.execute(code, context).await.unwrap();

    match result {
        ExecutionResult::SecurityViolation {
            violation_type: SecurityViolationType::FileSystemAccess,
            ..
        } => {
            // Security violation expected
        }
        other => panic!("Expected security violation, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_network_blocking() {
    set_once();
    let sandbox = CodeSandbox::new(SandboxConfig::default()).unwrap();
    let code = r#"
        const https = require('https');
        https.get('https://example.com');
    "#;
    let context = create_test_context();

    let result = sandbox.execute(code, context).await.unwrap();

    match result {
        ExecutionResult::SecurityViolation {
            violation_type: SecurityViolationType::NetworkAccess,
            ..
        } => {
            // Security violation expected
        }
        other => panic!("Expected security violation, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_process_execution_blocking() {
    set_once();
    let sandbox = CodeSandbox::new(SandboxConfig::default()).unwrap();
    let code = r#"
        const { exec } = require('child_process');
        exec('ls -la');
    "#;
    let context = create_test_context();

    let result = sandbox.execute(code, context).await.unwrap();

    match result {
        ExecutionResult::SecurityViolation {
            violation_type: SecurityViolationType::ProcessExecution,
            ..
        } => {
            // Security violation expected
        }
        other => panic!("Expected security violation, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_infinite_loop_detection() {
    set_once();
    let sandbox = CodeSandbox::new(SandboxConfig::default()).unwrap();
    let code = "while(true) {}";
    let context = create_test_context();

    let result = sandbox.execute(code, context).await.unwrap();

    match result {
        ExecutionResult::SecurityViolation {
            violation_type: SecurityViolationType::InfiniteLoop,
            ..
        } => {
            // Security violation expected
        }
        other => panic!("Expected security violation, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_eval_blocking() {
    set_once();
    let sandbox = CodeSandbox::new(SandboxConfig::default()).unwrap();
    let code = r#"eval("malicious code");"#;
    let context = create_test_context();

    let result = sandbox.execute(code, context).await.unwrap();

    match result {
        ExecutionResult::SecurityViolation {
            violation_type: SecurityViolationType::MaliciousCode,
            ..
        } => {
            // Security violation expected
        }
        other => panic!("Expected security violation, got: {:?}", other),
    }
}

#[tokio::test]
#[cfg(not(feature = "wasm-rquickjs"))]
async fn test_syntax_error() {
    set_once();
    let sandbox = CodeSandbox::new(SandboxConfig::default()).unwrap();
    let code = "const x = ;"; // Invalid syntax
    let context = create_test_context();

    let result = sandbox.execute(code, context).await.unwrap();

    match result {
        ExecutionResult::Error {
            error_type: ErrorType::Syntax,
            ..
        } => {
            // Syntax error expected
        }
        other => panic!("Expected syntax error, got: {:?}", other),
    }
}

#[tokio::test]
#[cfg(not(feature = "wasm-rquickjs"))]
async fn test_runtime_error() {
    set_once();
    let sandbox = CodeSandbox::new(SandboxConfig::default()).unwrap();
    let code = r#"
        throw new Error("Runtime error");
    "#;
    let context = create_test_context();

    let result = sandbox.execute(code, context).await.unwrap();

    match result {
        ExecutionResult::Error {
            error_type: ErrorType::Runtime,
            message,
            ..
        } => {
            assert!(message.contains("Runtime error"));
        }
        other => panic!("Expected runtime error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_code_length_limit() {
    set_once();
    let sandbox = CodeSandbox::new(SandboxConfig::default()).unwrap();
    let code = "a".repeat(100_001); // Exceeds 100KB limit
    let context = create_test_context();

    let result = sandbox.execute(&code, context).await.unwrap();

    match result {
        ExecutionResult::SecurityViolation { .. } => {
            // Security violation expected
        }
        other => panic!("Expected security violation, got: {:?}", other),
    }
}

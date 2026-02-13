//! Integration tests for Javy JavaScript compilation
//!
//! These tests verify that JavaScript code can be compiled to WASM
//! using the Javy backend. Tests are gated behind the `javy-backend` feature.

#[cfg(feature = "javy-backend")]
mod javy_tests {
    use memory_mcp::javy_compiler::{JavyCompiler, JavyConfig};
    #[allow(unused_imports)]
    use memory_mcp::types::{ExecutionContext, ExecutionResult};
    use std::io::Read;

    fn has_javy_plugin() -> bool {
        if let Ok(p) = std::env::var("JAVY_PLUGIN") {
            if let Ok(mut f) = std::fs::File::open(&p) {
                let mut magic = [0u8; 4];
                if f.read_exact(&mut magic).is_ok() && &magic == b"\0asm" {
                    return true;
                }
            }
        }
        let default = format!("{}/javy-plugin.wasm", env!("CARGO_MANIFEST_DIR"));
        if let Ok(mut f) = std::fs::File::open(&default) {
            let mut magic = [0u8; 4];
            if f.read_exact(&mut magic).is_ok() && &magic == b"\0asm" {
                return true;
            }
        }
        false
    }

    #[tokio::test]
    async fn test_basic_js_compilation() {
        if !has_javy_plugin() {
            eprintln!("Skipping test_basic_js_compilation: Javy plugin not available");
            return;
        }

        let compiler =
            JavyCompiler::new(JavyConfig::default()).expect("Failed to create Javy compiler");

        let js_source = r#"
            console.log("Hello from JavaScript");
            const x = 1 + 1;
        "#;

        let result = compiler.compile_js_to_wasm(js_source).await;
        assert!(
            result.is_ok(),
            "JavaScript compilation should succeed: {:?}",
            result.err()
        );

        let wasm_bytes = result.unwrap();
        assert!(!wasm_bytes.is_empty(), "WASM bytes should not be empty");
    }

    #[tokio::test]
    async fn test_js_syntax_error() {
        if !has_javy_plugin() {
            eprintln!("Skipping test_js_syntax_error: Javy plugin not available");
            return;
        }

        let compiler =
            JavyCompiler::new(JavyConfig::default()).expect("Failed to create Javy compiler");

        // Invalid JavaScript - missing closing brace
        let js_source = "function test() { console.log('test');";

        let result = compiler.compile_js_to_wasm(js_source).await;
        assert!(
            result.is_err(),
            "Compilation of invalid JavaScript should fail"
        );
    }

    #[tokio::test]
    async fn test_compilation_caching() {
        if !has_javy_plugin() {
            eprintln!("Skipping test_compilation_caching: Javy plugin not available");
            return;
        }

        let compiler =
            JavyCompiler::new(JavyConfig::default()).expect("Failed to create Javy compiler");

        let js_source = "const x = 42;";

        // First compilation
        let result1 = compiler.compile_js_to_wasm(js_source).await;
        assert!(result1.is_ok());

        // Second compilation (should hit cache)
        let result2 = compiler.compile_js_to_wasm(js_source).await;
        assert!(result2.is_ok());

        let metrics = compiler.get_metrics().await;
        assert_eq!(metrics.total_compilations, 2);
        assert_eq!(metrics.cache_hits, 1, "Second compilation should hit cache");
    }

    #[tokio::test]
    async fn test_js_execution_with_console_log() {
        use memory_mcp::ExecutionContext;

        if !has_javy_plugin() {
            eprintln!("Skipping test_js_execution_with_console_log: Javy plugin not available");
            return;
        }

        let compiler =
            JavyCompiler::new(JavyConfig::default()).expect("Failed to create Javy compiler");

        let js_source = r#"
            console.log("Test output");
            console.log("Another line");
        "#;

        let context = ExecutionContext {
            task: "Test JavaScript execution".to_string(),
            input: serde_json::json!({"test": "data"}),
            env: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
        };

        let result = compiler.execute_js(js_source.to_string(), context).await;
        assert!(
            result.is_ok(),
            "Execution should succeed: {:?}",
            result.err()
        );

        let exec_result = result.unwrap();
        match exec_result {
            ExecutionResult::Success { stdout, .. } => {
                assert!(
                    stdout.contains("Test output"),
                    "stdout should contain console.log output: got '{}'",
                    stdout
                );
            }
            other => panic!("Expected Success but got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_compilation_metrics() {
        if !has_javy_plugin() {
            eprintln!("Skipping test_compilation_metrics: Javy plugin not available");
            return;
        }

        let compiler =
            JavyCompiler::new(JavyConfig::default()).expect("Failed to create Javy compiler");

        let metrics_before = compiler.get_metrics().await;
        assert_eq!(metrics_before.total_compilations, 0);

        // Compile some JavaScript
        let _ = compiler.compile_js_to_wasm("const x = 1;").await;

        let metrics_after = compiler.get_metrics().await;
        assert_eq!(metrics_after.total_compilations, 1);
        assert!(metrics_after.avg_compilation_time_ms > 0.0);
    }
}

#[cfg(not(feature = "javy-backend"))]
mod no_javy_tests {
    /// This test module is empty when javy-backend is not enabled.
    /// The JavyCompiler type is not available without the javy-backend feature.
    #[allow(clippy::assertions_on_constants)]
    #[tokio::test]
    async fn test_javy_disabled_placeholder() {
        // Javy backend is disabled - compilation would fail
        // This test serves as a placeholder to verify the test suite compiles
        assert!(true, "javy-backend feature is not enabled");
    }
}

//! Integration tests for Javy JavaScript compilation
//!
//! These tests verify that JavaScript code can be compiled to WASM
//! using the Javy backend. Tests are gated behind the `javy-backend` feature.

#[cfg(feature = "javy-backend")]
mod javy_tests {
    use memory_mcp::javy_compiler::{JavyCompiler, JavyConfig};

    #[tokio::test]
    async fn test_basic_js_compilation() {
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

        let compiler =
            JavyCompiler::new(JavyConfig::default()).expect("Failed to create Javy compiler");

        let js_source = r#"
            console.log("Test output");
            console.log("Another line");
        "#;

        let context = ExecutionContext {
            timeout_ms: 5000,
            max_memory_mb: 64,
            ..Default::default()
        };

        let result = compiler.execute_js(js_source.to_string(), context).await;
        assert!(
            result.is_ok(),
            "Execution should succeed: {:?}",
            result.err()
        );

        let exec_result = result.unwrap();
        assert!(exec_result.success, "Execution should be successful");
        assert!(
            exec_result.stdout.contains("Test output"),
            "stdout should contain console.log output"
        );
    }

    #[tokio::test]
    async fn test_compilation_metrics() {
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
    use memory_mcp::javy_compiler::{JavyCompiler, JavyConfig};

    #[tokio::test]
    async fn test_javy_disabled_error() {
        let compiler =
            JavyCompiler::new(JavyConfig::default()).expect("Compiler creation should succeed");

        let result = compiler.compile_js_to_wasm("const x = 1;").await;
        assert!(
            result.is_err(),
            "Compilation should fail when javy-backend feature is disabled"
        );

        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("javy-backend"),
            "Error should mention the missing feature flag"
        );
    }
}

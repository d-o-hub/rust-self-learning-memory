//! Integration tests for MCP server and sandbox

use memory_core::{MemoryConfig, SelfLearningMemory};
use memory_mcp::{ExecutionContext, MemoryMCPServer, SandboxConfig};
use serde_json::json;
use std::sync::Arc;

/// Disable WASM sandbox for all tests to prevent rquickjs GC crashes
fn disable_wasm_for_tests() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        std::env::set_var("MCP_USE_WASM", "false");
    });
}

#[tokio::test]
async fn test_server_full_lifecycle() {
    disable_wasm_for_tests();
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            batch_config: None, // Disable batching for tests for test episodes
            ..Default::default()
        })),
    )
    .await
    .unwrap();

    // List tools
    let tools = server.list_tools().await;
    assert!(!tools.is_empty());

    // Execute code
    let code = r#"
        const data = context.input;
        const result = {
            processed: true,
            value: data.x + data.y
        };
        return result;
    "#;

    let ctx = ExecutionContext::new("test".to_string(), json!({"x": 10, "y": 20}));

    let result = server.execute_agent_code(code.to_string(), ctx).await;
    assert!(result.is_ok());

    // Check stats
    let stats = server.get_stats().await;
    assert!(stats.total_executions > 0);
}

#[tokio::test]
async fn test_query_memory_integration() {
    disable_wasm_for_tests();
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            batch_config: None, // Disable batching for tests for test episodes
            ..Default::default()
        })),
    )
    .await
    .unwrap();

    let result = server
        .query_memory(
            "test query".to_string(),
            "test-domain".to_string(),
            Some("code_generation".to_string()),
            5,
            "relevance".to_string(),
            None,
        )
        .await;

    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data.is_object());
}

#[tokio::test]
async fn test_analyze_patterns_integration() {
    disable_wasm_for_tests();
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            batch_config: None, // Disable batching for tests for test episodes
            ..Default::default()
        })),
    )
    .await
    .unwrap();

    let result = server
        .analyze_patterns("debugging".to_string(), 0.8, 10, None)
        .await;

    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data.is_object());
}

#[tokio::test]
async fn test_tool_management() {
    disable_wasm_for_tests();
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            batch_config: None, // Disable batching for tests for test episodes
            ..Default::default()
        })),
    )
    .await
    .unwrap();

    // Add custom tool
    let custom_tool = memory_mcp::types::Tool::new(
        "custom_test".to_string(),
        "Custom test tool".to_string(),
        json!({"type": "object"}),
    );

    server.add_tool(custom_tool).await.unwrap();

    // Verify it's in the list
    let tool = server.get_tool("custom_test").await;
    assert!(tool.is_some());

    // Remove it
    server.remove_tool("custom_test").await.unwrap();

    // Verify it's gone
    let tool = server.get_tool("custom_test").await;
    assert!(tool.is_none());
}

#[tokio::test]
async fn test_concurrent_executions() {
    disable_wasm_for_tests();
    let server = std::sync::Arc::new(
        MemoryMCPServer::new(
            SandboxConfig::default(),
            Arc::new(SelfLearningMemory::with_config(MemoryConfig {
                quality_threshold: 0.0,
                batch_config: None, // Disable batching for tests for test episodes
                ..Default::default()
            })),
        )
        .await
        .unwrap(),
    );

    let mut handles = vec![];

    for i in 0..5 {
        let server_clone = server.clone();
        let handle = tokio::spawn(async move {
            let code = format!("return {};", i);
            let ctx = ExecutionContext::new("test".to_string(), json!({}));
            server_clone.execute_agent_code(code, ctx).await
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    let stats = server.get_stats().await;
    assert_eq!(stats.total_executions, 5);
}

#[tokio::test]
async fn test_complex_code_execution() {
    disable_wasm_for_tests();
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            batch_config: None, // Disable batching for tests for test episodes
            ..Default::default()
        })),
    )
    .await
    .unwrap();

    let code = r#"
        // Complex calculation
        const fibonacci = (n) => {
            if (n <= 1) return n;
            return fibonacci(n - 1) + fibonacci(n - 2);
        };

        const result = {
            fib_10: fibonacci(10),
            timestamp: Date.now(),
            message: "Complex execution completed"
        };

        console.log("Fibonacci of 10:", result.fib_10);
        return result;
    "#;

    let ctx = ExecutionContext::new("fibonacci test".to_string(), json!({}));

    let result = server.execute_agent_code(code.to_string(), ctx).await;
    assert!(result.is_ok());

    match result.unwrap() {
        memory_mcp::ExecutionResult::Success { stdout, .. } => {
            assert!(stdout.contains("Fibonacci of 10:"));
        }
        other => panic!("Expected success, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_error_handling_in_code() {
    disable_wasm_for_tests();
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            batch_config: None, // Disable batching for tests for test episodes
            ..Default::default()
        })),
    )
    .await
    .unwrap();

    let code = r#"
        try {
            throw new Error("Test error");
        } catch (e) {
            console.error("Caught error:", e.message);
            return { error: e.message, handled: true };
        }
    "#;

    let ctx = ExecutionContext::new("error handling test".to_string(), json!({}));

    let result = server.execute_agent_code(code.to_string(), ctx).await;
    assert!(result.is_ok());

    // Should succeed because error is handled
    match result.unwrap() {
        memory_mcp::ExecutionResult::Success { .. } => {
            // Success expected
        }
        other => panic!("Expected success, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_async_code_execution() {
    disable_wasm_for_tests();
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            batch_config: None, // Disable batching for tests for test episodes
            ..Default::default()
        })),
    )
    .await
    .unwrap();

    let code = r#"
        const delay = (ms) => new Promise(resolve => setTimeout(resolve, ms));

        await delay(10);

        return {
            completed: true,
            message: "Async execution completed"
        };
    "#;

    let ctx = ExecutionContext::new("async test".to_string(), json!({}));

    let result = server.execute_agent_code(code.to_string(), ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_progressive_tool_disclosure() {
    disable_wasm_for_tests();
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            batch_config: None, // Disable batching for tests for test episodes
            ..Default::default()
        })),
    )
    .await
    .unwrap();

    // Use tools in different order
    let _ = server
        .analyze_patterns("test".to_string(), 0.7, 10, None)
        .await;

    let code = "return 1;";
    let ctx = ExecutionContext::new("test".to_string(), json!({}));
    let _ = server.execute_agent_code(code.to_string(), ctx).await;

    let _ = server
        .query_memory(
            "test".to_string(),
            "domain".to_string(),
            None,
            10,
            "relevance".to_string(),
            None,
        )
        .await;

    // Get usage stats
    let usage = server.get_tool_usage().await;
    assert!(usage.contains_key("execute_agent_code"));
    assert!(usage.contains_key("query_memory"));
    assert!(usage.contains_key("analyze_patterns"));
}
#[tokio::test]
async fn test_memory_integration_with_data() {
    disable_wasm_for_tests();
    use memory_core::{
        ExecutionResult, ExecutionStep, SelfLearningMemory, TaskContext, TaskOutcome, TaskType,
    };
    use memory_mcp::{MemoryMCPServer, SandboxConfig};
    use std::sync::Arc;

    // Create a shared memory instance
    let memory = Arc::new(SelfLearningMemory::with_config(MemoryConfig {
        quality_threshold: 0.0,
        batch_config: None, // Disable batching for tests for test episodes
        ..Default::default()
    }));

    // Populate memory with some episodes
    let context = TaskContext {
        domain: "web-api".to_string(),
        language: Some("rust".to_string()),
        ..Default::default()
    };

    let episode_id = memory
        .start_episode(
            "Build REST API".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;

    // Add some steps
    let mut step = ExecutionStep::new(1, "planner".to_string(), "Plan API structure".to_string());
    step.result = Some(ExecutionResult::Success {
        output: "API structure planned".to_string(),
    });
    memory.log_step(episode_id, step).await;

    // Complete the episode
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "API built successfully".to_string(),
                artifacts: vec!["api.rs".to_string()],
            },
        )
        .await
        .unwrap();

    // Now create MCP server with the same memory
    let server = MemoryMCPServer::new(SandboxConfig::default(), memory.clone())
        .await
        .unwrap();

    // Query memory through MCP
    let result = server
        .query_memory(
            "REST API".to_string(),
            "web-api".to_string(),
            None,
            10,
            "relevance".to_string(),
            None,
        )
        .await;

    assert!(result.is_ok());
    let data = result.unwrap();

    // Verify we get back our episode
    let episodes = data["episodes"].as_array().unwrap();
    assert_eq!(episodes.len(), 1);

    let retrieved_episode = &episodes[0];
    assert_eq!(retrieved_episode["task_description"], "Build REST API");

    // Verify insights
    assert!(data["insights"]["total_episodes"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn test_jsonrpc_response_format_execute_code() {
    disable_wasm_for_tests();
    use memory_core::SelfLearningMemory;
    use memory_mcp::{MemoryMCPServer, SandboxConfig};
    use serde_json::json;
    use std::sync::Arc;

    // Create server
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            batch_config: None, // Disable batching for tests for test episodes
            ..Default::default()
        })),
    )
    .await
    .unwrap();

    // Test execute_agent_code directly to verify the result format
    let code = "return { success: true, value: 42 };";
    let context =
        memory_mcp::ExecutionContext::new("test execution".to_string(), json!({ "test": "data" }));

    let result = server
        .execute_agent_code(code.to_string(), context)
        .await
        .unwrap();

    // Verify the result can be serialized to JSON
    let serialized = serde_json::to_string(&result).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    // Verify it has the expected structure for ExecutionResult::Success
    assert!(parsed.get("Success").is_some());
    let success_data = &parsed["Success"];
    assert_eq!(
        success_data["output"],
        json!("{\"success\":true,\"value\":42}")
    );
    assert!(success_data["execution_time_ms"].is_number());
    assert!(success_data["stdout"].is_string());
    assert!(success_data["stderr"].is_string());

    // Test error case
    let error_code = "throw new Error('test error');";
    let error_context = memory_mcp::ExecutionContext::new("test error".to_string(), json!({}));

    let error_result = server
        .execute_agent_code(error_code.to_string(), error_context)
        .await
        .unwrap();

    let error_serialized = serde_json::to_string(&error_result).unwrap();
    let error_parsed: serde_json::Value = serde_json::from_str(&error_serialized).unwrap();

    // Should be an Error variant
    assert!(error_parsed.get("Error").is_some());
    let error_data = &error_parsed["Error"];
    assert!(error_data["message"].is_string());
    assert!(error_data["stdout"].is_string());
    assert!(error_data["stderr"].is_string());
}

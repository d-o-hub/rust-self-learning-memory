//! Integration tests for MCP server and sandbox

use memory_core::SelfLearningMemory;
use memory_mcp::{ExecutionContext, MemoryMCPServer, SandboxConfig};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_server_full_lifecycle() {
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::new()),
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
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::new()),
    )
    .await
    .unwrap();

    let result = server
        .query_memory(
            "test query".to_string(),
            "test-domain".to_string(),
            Some("code_generation".to_string()),
            5,
        )
        .await;

    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data.is_object());
}

#[tokio::test]
async fn test_analyze_patterns_integration() {
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::new()),
    )
    .await
    .unwrap();

    let result = server
        .analyze_patterns("debugging".to_string(), 0.8, 10)
        .await;

    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data.is_object());
}

#[tokio::test]
async fn test_tool_management() {
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::new()),
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
    let server = std::sync::Arc::new(
        MemoryMCPServer::new(
            SandboxConfig::default(),
            Arc::new(SelfLearningMemory::new()),
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
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::new()),
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
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::new()),
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
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::new()),
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
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::new()),
    )
    .await
    .unwrap();

    // Use tools in different order
    let _ = server.analyze_patterns("test".to_string(), 0.7, 10).await;

    let code = "return 1;";
    let ctx = ExecutionContext::new("test".to_string(), json!({}));
    let _ = server.execute_agent_code(code.to_string(), ctx).await;

    let _ = server
        .query_memory("test".to_string(), "domain".to_string(), None, 10)
        .await;

    // Get usage stats
    let usage = server.get_tool_usage().await;
    assert!(usage.contains_key("execute_agent_code"));
    assert!(usage.contains_key("query_memory"));
    assert!(usage.contains_key("analyze_patterns"));
}
#[tokio::test]
async fn test_memory_integration_with_data() {
    use memory_core::{
        ExecutionResult, ExecutionStep, SelfLearningMemory, TaskContext, TaskOutcome, TaskType,
    };
    use memory_mcp::{MemoryMCPServer, SandboxConfig};
    use std::sync::Arc;

    // Create a shared memory instance
    let memory = Arc::new(SelfLearningMemory::new());

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
        .await
        .unwrap();

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
        .query_memory("REST API".to_string(), "web-api".to_string(), None, 10)
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

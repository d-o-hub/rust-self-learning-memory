//! Simple integration tests for memory MCP server
//!
//! These tests verify basic MCP server functionality and database integration.

use memory_core::{ComplexityLevel, SelfLearningMemory, TaskContext, TaskOutcome, TaskType};
use memory_mcp::{ExecutionContext, MemoryMCPServer, SandboxConfig};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_mcp_server_tools() {
    let memory = Arc::new(SelfLearningMemory::new());
    let sandbox_config = SandboxConfig::restrictive();
    let mcp_server = Arc::new(
        MemoryMCPServer::new(sandbox_config, memory.clone())
            .await
            .unwrap(),
    );

    // Test that server initializes with correct tools
    let tools = mcp_server.list_tools().await;
    assert_eq!(tools.len(), 5);

    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
    assert!(tool_names.contains(&"query_memory".to_string()));
    assert!(tool_names.contains(&"execute_agent_code".to_string()));
    assert!(tool_names.contains(&"analyze_patterns".to_string()));
    assert!(tool_names.contains(&"health_check".to_string()));
    assert!(tool_names.contains(&"get_metrics".to_string()));
}

#[tokio::test]
async fn test_memory_query_with_episode() {
    let memory = Arc::new(SelfLearningMemory::new());
    let sandbox_config = SandboxConfig::restrictive();
    let mcp_server = Arc::new(
        MemoryMCPServer::new(sandbox_config, memory.clone())
            .await
            .unwrap(),
    );

    // Create a test episode
    let episode_id = memory
        .start_episode(
            "Database test episode".to_string(),
            TaskContext {
                domain: "database".to_string(),
                language: Some("rust".to_string()),
                framework: Some("sqlx".to_string()),
                complexity: ComplexityLevel::Simple,
                tags: vec!["test".to_string()],
            },
            TaskType::CodeGeneration,
        )
        .await;

    // Complete the episode
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Test episode completed".to_string(),
                artifacts: vec!["test.rs".to_string()],
            },
        )
        .await;

    // Test MCP memory query
    let result = mcp_server
        .query_memory(
            "database test".to_string(),
            "database".to_string(),
            None,
            10,
        )
        .await
        .unwrap();

    let episodes = result["episodes"].as_array().unwrap();
    assert_eq!(episodes.len(), 1);

    let episode = &episodes[0];
    assert_eq!(episode["task_description"], "Database test episode");
    assert!(episode["outcome"].is_object());
}

#[tokio::test]
async fn test_tool_usage_tracking() {
    let memory = Arc::new(SelfLearningMemory::new());
    let sandbox_config = SandboxConfig::restrictive();
    let mcp_server = Arc::new(
        MemoryMCPServer::new(sandbox_config, memory.clone())
            .await
            .unwrap(),
    );

    // Perform some tool operations
    let _ = mcp_server
        .query_memory("test".to_string(), "test".to_string(), None, 5)
        .await
        .unwrap();

    let _ = mcp_server
        .analyze_patterns("test".to_string(), 0.7, 5)
        .await
        .unwrap();

    // Verify tool usage tracking
    let usage = mcp_server.get_tool_usage().await;
    assert!(usage.contains_key("query_memory"));
    assert!(usage.contains_key("analyze_patterns"));
    assert!(*usage.get("query_memory").unwrap_or(&0) >= 1);
    assert!(*usage.get("analyze_patterns").unwrap_or(&0) >= 1);
}

#[tokio::test]
async fn test_execution_attempt_tracking() {
    let memory = Arc::new(SelfLearningMemory::new());
    let sandbox_config = SandboxConfig::restrictive();
    let mcp_server = Arc::new(
        MemoryMCPServer::new(sandbox_config, memory.clone())
            .await
            .unwrap(),
    );

    // Execute some code (may fail, but should be tracked)
    let _ = mcp_server
        .execute_agent_code(
            "return 42;".to_string(),
            ExecutionContext::new("test".to_string(), json!({})),
        )
        .await;

    // Verify execution statistics are tracked
    let stats = mcp_server.get_stats().await;
    assert!(stats.total_executions >= 1);
}

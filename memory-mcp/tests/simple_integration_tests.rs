//! Simple integration tests for memory MCP server
//!
//! These tests verify basic MCP server functionality and database integration.

use memory_core::{
    ComplexityLevel, MemoryConfig, SelfLearningMemory, TaskContext, TaskOutcome, TaskType,
};
use memory_mcp::{ExecutionContext, MemoryMCPServer, SandboxConfig};
use serde_json::json;
use std::sync::Arc;

/// Disable WASM sandbox for all tests to ensure consistent tool counts across environments
fn disable_wasm_for_tests() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        std::env::set_var("MCP_USE_WASM", "false");
        std::env::set_var("MCP_CACHE_WARMING_ENABLED", "false");
    });
}

#[tokio::test]
async fn test_mcp_server_tools() {
    disable_wasm_for_tests();
    let memory = Arc::new(SelfLearningMemory::with_config(MemoryConfig {
        quality_threshold: 0.0, // Zero threshold for test episodes
        ..Default::default()
    }));
    let sandbox_config = SandboxConfig::restrictive();
    let mcp_server = Arc::new(
        MemoryMCPServer::new(sandbox_config, memory.clone())
            .await
            .unwrap(),
    );

    // Test that server initializes with correct tools
    // Note: With restrictive sandbox, execute_agent_code is not available
    // Available tools: query_memory, analyze_patterns, health_check, get_metrics,
    // advanced_pattern_analysis, quality_metrics, configure_embeddings, query_semantic_memory, test_embeddings
    let tools = mcp_server.list_tools().await;
    assert_eq!(tools.len(), 10);

    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();

    assert!(tool_names.contains(&"query_memory".to_string()));
    assert!(!tool_names.contains(&"execute_agent_code".to_string())); // Not available with restrictive sandbox
    assert!(tool_names.contains(&"analyze_patterns".to_string()));
    assert!(tool_names.contains(&"health_check".to_string()));
    assert!(tool_names.contains(&"get_metrics".to_string()));
    assert!(tool_names.contains(&"advanced_pattern_analysis".to_string()));
    assert!(tool_names.contains(&"quality_metrics".to_string()));
    assert!(tool_names.contains(&"configure_embeddings".to_string()));
    assert!(tool_names.contains(&"query_semantic_memory".to_string()));
    assert!(tool_names.contains(&"test_embeddings".to_string()));
}

#[tokio::test]
async fn test_memory_query_with_episode() {
    let memory = Arc::new(SelfLearningMemory::with_config(MemoryConfig {
        quality_threshold: 0.0, // Zero threshold for test episodes
        ..Default::default()
    }));
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
        .await
        .unwrap();

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
    let memory = Arc::new(SelfLearningMemory::with_config(MemoryConfig {
        quality_threshold: 0.0, // Zero threshold for test episodes
        ..Default::default()
    }));
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
    let memory = Arc::new(SelfLearningMemory::with_config(MemoryConfig {
        quality_threshold: 0.0, // Zero threshold for test episodes
        ..Default::default()
    }));
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

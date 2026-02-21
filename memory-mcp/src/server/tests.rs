//! Tests for the MemoryMCPServer

use crate::server::MemoryMCPServer;
use crate::server::sandbox;
use crate::types::{ExecutionContext, SandboxConfig};
use memory_core::SelfLearningMemory;
use serde_json::json;
use std::sync::Arc;

// Set once for all tests in this module
fn set_once() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        std::env::set_var("MCP_USE_WASM", "false");
    });
}

async fn create_test_server() -> MemoryMCPServer {
    set_once();
    let memory = Arc::new(SelfLearningMemory::new());
    MemoryMCPServer::new(SandboxConfig::default(), memory)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_server_creation() {
    let server = create_test_server().await;
    let tools = server.list_tools().await;

    assert!(!tools.is_empty());
    assert!(tools.iter().any(|t| t.name == "query_memory"));
    // execute_agent_code tool is only available if WASM sandbox is enabled
    if sandbox::is_wasm_sandbox_available() {
        assert!(tools.iter().any(|t| t.name == "execute_agent_code"));
    }
    assert!(tools.iter().any(|t| t.name == "analyze_patterns"));
}

#[tokio::test]
async fn test_get_tool() {
    let server = create_test_server().await;

    let tool = server.get_tool("query_memory").await;
    assert!(tool.is_some());

    let tool = tool.unwrap();
    assert_eq!(tool.name, "query_memory");
    assert!(!tool.description.is_empty());
}

#[tokio::test]
async fn test_execute_code() {
    if std::env::var("RUN_WASM_TESTS").is_err() || !sandbox::is_wasm_sandbox_available() {
        tracing::info!(
            "Skipping execute_agent_code test (set RUN_WASM_TESTS=1 and ensure WASM is available)"
        );
        return;
    }

    let server = create_test_server().await;

    let code = "return 1 + 1;";
    let context = ExecutionContext::new("test".to_string(), json!({}));

    let result = server.execute_agent_code(code.to_string(), context).await;
    assert!(result.is_ok());

    // Check stats were updated
    let stats = server.get_stats().await;
    assert_eq!(stats.total_executions, 1);
}

#[tokio::test]
async fn test_tool_usage_tracking() {
    let server = create_test_server().await;

    // Execute code multiple times
    for _ in 0..3 {
        let code = "return 1;";
        let context = ExecutionContext::new("test".to_string(), json!({}));
        let _ = server.execute_agent_code(code.to_string(), context).await;
    }

    // Check usage was tracked
    let usage = server.get_tool_usage().await;
    assert_eq!(usage.get("execute_agent_code"), Some(&3));
}

#[tokio::test]
async fn test_progressive_tool_disclosure() {
    if std::env::var("RUN_WASM_TESTS").is_err() || !sandbox::is_wasm_sandbox_available() {
        tracing::info!(
            "Skipping progressive tool disclosure test (set RUN_WASM_TESTS=1 and ensure WASM is available)"
        );
        return;
    }

    let server = create_test_server().await;

    // Use execute_agent_code multiple times
    for _ in 0..5 {
        let code = "return 1;";
        let context = ExecutionContext::new("test".to_string(), json!({}));
        let _ = server.execute_agent_code(code.to_string(), context).await;
    }

    // Use query_memory once
    let _ = server
        .query_memory(
            "test".to_string(),
            "test".to_string(),
            None,
            10,
            "relevance".to_string(),
            None,
        )
        .await;

    // List tools - execute_agent_code should be first (most used)
    let tools = server.list_tools().await;
    assert_eq!(tools[0].name, "execute_agent_code");
}

#[tokio::test]
async fn test_add_custom_tool() {
    let server = create_test_server().await;

    let custom_tool = crate::types::Tool::new(
        "custom_tool".to_string(),
        "A custom tool".to_string(),
        json!({"type": "object"}),
    );

    let result = server.add_tool(custom_tool).await;
    assert!(result.is_ok());

    let tool = server.get_tool("custom_tool").await;
    assert!(tool.is_some());
}

#[tokio::test]
async fn test_add_duplicate_tool_fails() {
    let server = create_test_server().await;

    let duplicate_tool = crate::types::Tool::new(
        "query_memory".to_string(),
        "Duplicate".to_string(),
        json!({"type": "object"}),
    );

    let result = server.add_tool(duplicate_tool).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_remove_tool() {
    let server = create_test_server().await;

    // Add and then remove a tool
    let custom_tool = crate::types::Tool::new(
        "temp_tool".to_string(),
        "Temporary".to_string(),
        json!({"type": "object"}),
    );

    server.add_tool(custom_tool).await.unwrap();
    assert!(server.get_tool("temp_tool").await.is_some());

    let result = server.remove_tool("temp_tool").await;
    assert!(result.is_ok());
    assert!(server.get_tool("temp_tool").await.is_none());
}

#[tokio::test]
async fn test_query_memory() {
    let server = create_test_server().await;

    let result = server
        .query_memory(
            "test query".to_string(),
            "web-api".to_string(),
            Some("code_generation".to_string()),
            10,
            "relevance".to_string(),
            None,
        )
        .await;

    assert!(result.is_ok());
    let json = result.unwrap();
    assert!(json.get("episodes").is_some());
    assert!(json.get("patterns").is_some());
}

#[tokio::test]
async fn test_query_memory_negative_returns_empty() {
    let server = create_test_server().await;

    let result = server
        .query_memory(
            "tmp_rovodev_negative_test_unique_string".to_string(),
            "verification".to_string(),
            Some("analysis".to_string()),
            10,
            "relevance".to_string(),
            None,
        )
        .await
        .unwrap();

    let episodes = result
        .get("episodes")
        .and_then(|v| v.as_array())
        .expect("episodes should be an array");

    assert!(
        episodes.is_empty(),
        "expected no episodes for unmatched query"
    );
}

#[tokio::test]
async fn test_query_memory_with_sort_options() {
    let server = create_test_server().await;

    // Test each sort option is accepted
    for sort in ["relevance", "newest", "oldest", "duration", "success"] {
        let result = server
            .query_memory(
                "test".to_string(),
                "test".to_string(),
                None,
                5,
                sort.to_string(),
                None,
            )
            .await;

        assert!(result.is_ok(), "Sort option '{}' should be accepted", sort);
    }
}

#[tokio::test]
async fn test_analyze_patterns() {
    let server = create_test_server().await;

    let result = server
        .analyze_patterns("code_generation".to_string(), 0.7, 20, None)
        .await;

    assert!(result.is_ok());
    let json = result.unwrap();
    assert!(json.get("patterns").is_some());
    assert!(json.get("statistics").is_some());
}

#[tokio::test]
async fn test_execution_stats() {
    let server = create_test_server().await;

    // Execute some code
    let code = "return 42;";
    let context = ExecutionContext::new("test".to_string(), json!({}));
    let _ = server.execute_agent_code(code.to_string(), context).await;

    let stats = server.get_stats().await;
    assert_eq!(stats.total_executions, 1);
    // avg_execution_time_ms can be 0.0 for very fast sub-millisecond executions
    assert!(stats.avg_execution_time_ms >= 0.0);
}

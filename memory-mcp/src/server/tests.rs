//! Tests for the MemoryMCPServer

use crate::server::MemoryMCPServer;
use crate::types::SandboxConfig;
use do_memory_core::SelfLearningMemory;
use serde_json::json;
use std::sync::Arc;

async fn create_test_server() -> MemoryMCPServer {
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
    // Core tools that are always available
    assert!(
        tools.iter().any(|t| t.name == "query_memory"),
        "query_memory tool should exist"
    );
    assert!(
        tools.iter().any(|t| t.name == "analyze_patterns"),
        "analyze_patterns tool should exist"
    );
    assert!(
        tools.iter().any(|t| t.name == "health_check"),
        "health_check tool should exist"
    );
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
async fn test_tool_usage_tracking() {
    let server = create_test_server().await;

    // Execute query_memory multiple times
    for _ in 0..3 {
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
    }

    // Check usage was tracked
    let usage = server.get_tool_usage().await;
    assert_eq!(usage.get("query_memory"), Some(&3));
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
async fn test_query_memory_with_provenance_redacted() {
    use crate::server::tools::core::QueryMemoryRequest;

    let server = create_test_server().await;
    let secret = "user password secret query about tokens".to_string();

    let result = server
        .query_memory_with_options(QueryMemoryRequest {
            query: secret.clone(),
            domain: "web-api".to_string(),
            task_type: Some("debugging".to_string()),
            limit: 5,
            sort: "relevance".to_string(),
            fields: None,
            with_provenance: true,
        })
        .await
        .expect("query_memory_with_options should succeed");

    let provenance = result
        .get("provenance")
        .expect("with_provenance=true must return provenance");
    assert!(
        provenance
            .get("fingerprint")
            .and_then(|v| v.as_str())
            .is_some_and(|s| !s.is_empty()),
        "fingerprint must be non-empty"
    );
    assert!(
        provenance
            .get("cache_hit")
            .and_then(|v| v.as_bool())
            .is_some()
    );
    assert!(provenance.get("index_generation").is_some());
    assert!(result.get("latency_ms").and_then(|v| v.as_u64()).is_some());

    let debug = result.to_string();
    assert!(
        !debug.contains("password") && !debug.contains(&secret),
        "provenance path must not embed raw query: {debug}"
    );

    // Default path must not include provenance
    let plain = server
        .query_memory(
            "plain query".to_string(),
            "web-api".to_string(),
            None,
            5,
            "relevance".to_string(),
            None,
        )
        .await
        .expect("plain query_memory should succeed");
    assert!(
        plain.get("provenance").is_none(),
        "default query_memory must omit provenance"
    );
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

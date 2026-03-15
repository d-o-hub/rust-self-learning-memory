#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! MCP Tool Coverage Tests (ACT-034)
//!
//! Comprehensive integration tests verifying MCP tool registration,
//! schema validation, uniqueness, and basic tool invocation.

use memory_core::{MemoryConfig, SelfLearningMemory};
use memory_mcp::{ExecutionStats, MemoryMCPServer, SandboxConfig};
use std::collections::HashSet;
use std::sync::Arc;

/// Disable WASM sandbox for all tests to ensure consistent tool counts across environments
#[allow(unsafe_code)]
fn disable_wasm_for_tests() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        // SAFETY: test-only env var manipulation
        unsafe {
            std::env::set_var("MCP_USE_WASM", "false");
            std::env::set_var("MCP_CACHE_WARMING_ENABLED", "false");
        }
    });
}

async fn create_test_server() -> MemoryMCPServer {
    disable_wasm_for_tests();
    let memory = Arc::new(SelfLearningMemory::with_config(MemoryConfig {
        quality_threshold: 0.0,
        batch_config: None,
        ..Default::default()
    }));
    MemoryMCPServer::new(SandboxConfig::restrictive(), memory)
        .await
        .expect("server creation should succeed")
}

// ── 1. Tool listing ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_mcp_tool_coverage_list_tools_returns_core_tools() {
    let server = create_test_server().await;
    let tools = server.list_tools().await;
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    let expected_core = [
        "query_memory",
        "analyze_patterns",
        "health_check",
        "get_metrics",
        "create_episode",
        "add_episode_step",
        "complete_episode",
        "get_episode",
    ];

    for name in &expected_core {
        assert!(
            names.contains(name),
            "Core tool '{}' should be in list_tools()",
            name
        );
    }
}

// ── 2. Tool count ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_mcp_tool_coverage_minimum_tool_count() {
    let server = create_test_server().await;
    let tools = server.list_tools().await;
    assert!(
        tools.len() >= 8,
        "Expected at least 8 core tools, got {}",
        tools.len()
    );
}

// ── 3. Tool schema validation ───────────────────────────────────────────

#[tokio::test]
async fn test_mcp_tool_coverage_tool_schemas_are_valid_json_objects() {
    let server = create_test_server().await;
    let tools = server.list_tools().await;

    for tool in &tools {
        assert!(
            tool.input_schema.is_object(),
            "Tool '{}' input_schema should be a JSON object, got: {}",
            tool.name,
            tool.input_schema
        );

        let schema_obj = tool.input_schema.as_object().unwrap();
        assert!(
            schema_obj.contains_key("type"),
            "Tool '{}' input_schema should have a \"type\" field",
            tool.name
        );
    }
}

// ── 4. Tool name uniqueness ─────────────────────────────────────────────

#[tokio::test]
async fn test_mcp_tool_coverage_no_duplicate_tool_names() {
    let server = create_test_server().await;
    let tools = server.list_tools().await;

    let mut seen = HashSet::new();
    for tool in &tools {
        assert!(
            seen.insert(&tool.name),
            "Duplicate tool name detected: '{}'",
            tool.name
        );
    }
}

// ── 5. Health check tool ────────────────────────────────────────────────

#[tokio::test]
async fn test_mcp_tool_coverage_health_check_returns_valid_response() {
    let server = create_test_server().await;
    let result = server
        .health_check()
        .await
        .expect("health_check should succeed");

    assert!(
        result.is_object(),
        "health_check should return a JSON object"
    );
}

// ── 6. Query memory tool schema ─────────────────────────────────────────

#[tokio::test]
async fn test_mcp_tool_coverage_query_memory_schema_has_required_properties() {
    let server = create_test_server().await;
    let tool = server
        .get_tool("query_memory")
        .await
        .expect("query_memory tool should exist");

    let schema = tool.input_schema.as_object().unwrap();
    assert_eq!(
        schema.get("type").and_then(|v| v.as_str()),
        Some("object"),
        "query_memory schema type should be 'object'"
    );

    let properties = schema
        .get("properties")
        .and_then(|v| v.as_object())
        .expect("query_memory schema should have 'properties'");

    assert!(
        properties.contains_key("query"),
        "query_memory should have 'query' property"
    );
    assert!(
        properties.contains_key("domain"),
        "query_memory should have 'domain' property"
    );
}

// ── 7. Analyze patterns tool schema ─────────────────────────────────────

#[tokio::test]
async fn test_mcp_tool_coverage_analyze_patterns_schema_has_required_properties() {
    let server = create_test_server().await;
    let tool = server
        .get_tool("analyze_patterns")
        .await
        .expect("analyze_patterns tool should exist");

    let schema = tool.input_schema.as_object().unwrap();
    assert_eq!(
        schema.get("type").and_then(|v| v.as_str()),
        Some("object"),
        "analyze_patterns schema type should be 'object'"
    );

    let properties = schema
        .get("properties")
        .and_then(|v| v.as_object())
        .expect("analyze_patterns schema should have 'properties'");

    assert!(
        properties.contains_key("domain"),
        "analyze_patterns should have 'domain' property"
    );
}

// ── 8. Server initialization ────────────────────────────────────────────

#[tokio::test]
async fn test_mcp_tool_coverage_server_initializes_without_panic() {
    let _server = create_test_server().await;
    // If we reach here, server initialization succeeded without panicking
}

// ── 9. Stats initial state ──────────────────────────────────────────────

#[tokio::test]
async fn test_mcp_tool_coverage_initial_stats_are_zero() {
    let server = create_test_server().await;
    let stats: ExecutionStats = server.get_stats().await;

    assert_eq!(
        stats.total_executions, 0,
        "initial total_executions should be 0"
    );
    assert_eq!(
        stats.successful_executions, 0,
        "initial successful_executions should be 0"
    );
    assert_eq!(
        stats.failed_executions, 0,
        "initial failed_executions should be 0"
    );
    assert_eq!(stats.timeout_count, 0, "initial timeout_count should be 0");
    assert_eq!(
        stats.security_violations, 0,
        "initial security_violations should be 0"
    );
}

// ── 10. Restrictive sandbox config ──────────────────────────────────────

#[tokio::test]
async fn test_mcp_tool_coverage_restrictive_sandbox_config() {
    let config = SandboxConfig::restrictive();

    assert!(
        !config.allow_network,
        "restrictive config should deny network"
    );
    assert!(
        !config.allow_filesystem,
        "restrictive config should deny filesystem"
    );
    assert!(
        !config.allow_subprocesses,
        "restrictive config should deny subprocesses"
    );
    assert!(
        config.allowed_paths.is_empty(),
        "restrictive config should have no allowed paths"
    );
    assert!(
        config.allowed_network.is_empty(),
        "restrictive config should have no allowed network hosts"
    );
    assert!(
        config.read_only_mode,
        "restrictive config should be read-only"
    );
    assert_eq!(config.max_memory_mb, 64);
    assert_eq!(config.max_execution_time_ms, 3000);
}

//! MCP Tool Contract Parity Tests
//!
//! This test module verifies that every tool advertised by the MCP server
//! (via the full-registry `tools/list` payload) has a corresponding handler
//! that can dispatch to it.
//!
//! This catches the issue where tools are defined in the schema but
//! their handlers are commented out or missing, which creates a broken
//! contract with clients.

#![allow(missing_docs)]
#![allow(clippy::doc_markdown)]
// Integration tests are separate crate roots and don't inherit the
// `allow-expect-in-tests` / `allow-unwrap-in-tests` settings.
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use do_memory_core::{MemoryConfig, SelfLearningMemory};
use do_memory_mcp::MemoryMCPServer;
use do_memory_mcp::jsonrpc::JsonRpcRequest;
use do_memory_mcp::protocol::{handle_describe_tool, handle_list_tools_with_lazy};
use do_memory_mcp::types::SandboxConfig;
use serde_json::json;
use std::collections::HashSet;
use std::sync::Arc;

/// Build a fresh MCP server with no tools loaded beyond the core set.
async fn fresh_server() -> MemoryMCPServer {
    MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            batch_config: None,
            ..Default::default()
        })),
    )
    .await
    .expect("Failed to create MCP server")
}

/// Build a `tools/list` JSON-RPC request.
fn list_request(id: i64, params: Option<serde_json::Value>) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: Some("2.0".to_string()),
        id: Some(json!(id)),
        method: "tools/list".to_string(),
        params,
    }
}

/// Get the list of dispatchable tool names from the handlers.rs match statement.
///
/// This is a static list that must be kept in sync with the actual dispatch
/// table in memory-mcp/src/bin/server_impl/handlers.rs.
///
/// IMPORTANT: When adding a new tool handler, add the tool name here too.
fn get_dispatchable_tool_names() -> Vec<&'static str> {
    vec![
        // Core tools
        "query_memory",
        "analyze_patterns",
        "health_check",
        "get_metrics",
        // Extended tools
        "advanced_pattern_analysis",
        "quality_metrics",
        "configure_embeddings",
        "query_semantic_memory",
        "test_embeddings",
        "generate_embedding",
        "search_by_embedding",
        "embedding_provider_status",
        "search_patterns",
        "recommend_patterns",
        "recommend_playbook",
        "explain_pattern",
        "record_recommendation_session",
        "record_recommendation_feedback",
        "get_recommendation_stats",
        "checkpoint_episode",
        "get_handoff_pack",
        "resume_from_handoff",
        // Episode lifecycle
        "bulk_episodes",
        "create_episode",
        "add_episode_step",
        "complete_episode",
        "get_episode",
        "delete_episode",
        "update_episode",
        "get_episode_timeline",
        // Episode tags
        "add_episode_tags",
        "remove_episode_tags",
        "set_episode_tags",
        "get_episode_tags",
        "search_episodes_by_tags",
        // Episode relationships
        "add_episode_relationship",
        "remove_episode_relationship",
        "get_episode_relationships",
        "find_related_episodes",
        "check_relationship_exists",
        "get_dependency_graph",
        "validate_no_cycles",
        "get_topological_order",
        // External signal provider
        "configure_agentfs",
        "external_signal_status",
        "test_agentfs_connection",
        // Checkpoint resume (issue #965)
        "resume_from_compact",
    ]
}

/// Test that all advertised tools have dispatch handlers.
///
/// This test creates an MCP server, lists all tools it advertises,
/// and verifies that each tool has a corresponding handler in the
/// dispatch table.
#[tokio::test]
async fn test_all_advertised_tools_are_dispatchable() {
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            batch_config: None,
            ..Default::default()
        })),
    )
    .await
    .expect("Failed to create MCP server");

    // Get all tools advertised by the server
    let advertised_tools = server.list_all_tools();
    let advertised_names: Vec<String> = advertised_tools.iter().map(|t| t.name.clone()).collect();

    // Get the dispatchable tool names
    let dispatchable_names = get_dispatchable_tool_names();

    // Find any tools that are advertised but not dispatchable
    let mut missing_handlers: Vec<String> = Vec::new();
    for name in &advertised_names {
        if !dispatchable_names.contains(&name.as_str()) {
            missing_handlers.push(name.clone());
        }
    }

    // Report the issue with helpful context
    if !missing_handlers.is_empty() {
        eprintln!("\n=== TOOL CONTRACT PARITY FAILURE ===");
        eprintln!("The following tools are advertised but have no dispatch handlers:");
        for name in &missing_handlers {
            eprintln!("  - {name}");
        }
        eprintln!("\nThis means clients can see these tools in tools/list but");
        eprintln!("will get 'Tool not found' error when calling tools/call.");
        eprintln!("\nTo fix this:");
        eprintln!("1. Add the handler to handlers.rs match statement, OR");
        eprintln!("2. Remove the tool definition from tool_definitions_extended.rs");
        eprintln!("=====================================\n");
    }

    assert!(
        missing_handlers.is_empty(),
        "Tools advertised but not dispatchable: {missing_handlers:?}"
    );
}

/// Test that the dispatch table covers all advertised tools.
///
/// This is the inverse check - verifying that our static list of
/// dispatchable tools is in sync with the server's advertised tools.
#[tokio::test]
async fn test_dispatch_table_covers_advertised_tools() {
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            batch_config: None,
            ..Default::default()
        })),
    )
    .await
    .expect("Failed to create MCP server");

    let advertised_tools = server.list_all_tools();
    let advertised_names: Vec<&str> = advertised_tools.iter().map(|t| t.name.as_str()).collect();

    let dispatchable_names = get_dispatchable_tool_names();

    // Find any dispatchable tools that are not advertised
    // (This is fine - just informational)
    let not_advertised: Vec<&&str> = dispatchable_names
        .iter()
        .filter(|name| !advertised_names.contains(name))
        .collect();

    if !not_advertised.is_empty() {
        // This is not necessarily an error - some tools may be conditionally advertised
        println!("\nInfo: Some dispatchable tools are not currently advertised:");
        for name in &not_advertised {
            println!("  - {name}");
        }
    }

    // The main check is that all advertised tools are dispatchable
    // (covered by the other test)
}

/// Test that deferred batch-analysis tools are NOT advertised.
///
/// WG-053 decision: these tool names are intentionally absent from MCP
/// `tools/list` until handlers exist and are wired into dispatch.
#[tokio::test]
async fn test_unimplemented_batch_tools_not_advertised() {
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            batch_config: None,
            ..Default::default()
        })),
    )
    .await
    .expect("Failed to create MCP server");

    let advertised_tools = server.list_all_tools();
    let advertised_names: Vec<&str> = advertised_tools.iter().map(|t| t.name.as_str()).collect();

    // These tools should NOT be advertised while intentionally deferred
    let unimplemented_tools = [
        "batch_query_episodes",
        "batch_pattern_analysis",
        "batch_compare_episodes",
    ];

    let mut incorrectly_advertised: Vec<&str> = Vec::new();
    for tool in &unimplemented_tools {
        if advertised_names.contains(tool) {
            incorrectly_advertised.push(tool);
        }
    }

    if !incorrectly_advertised.is_empty() {
        eprintln!("\n=== UNIMPLEMENTED TOOLS INCORRECTLY ADVERTISED ===");
        eprintln!("The following tools are advertised but have no implementation:");
        for name in &incorrectly_advertised {
            eprintln!("  - {name}");
        }
        eprintln!("\nThese tools are intentionally deferred in WG-053.");
        eprintln!("\nTo re-enable these tools:");
        eprintln!("1. Implement the handlers in the appropriate module");
        eprintln!("2. Add them to the dispatch table in handlers.rs");
        eprintln!("3. Add them to get_dispatchable_tool_names() in this test");
        eprintln!("===================================================\n");
    }

    assert!(
        incorrectly_advertised.is_empty(),
        "Unimplemented tools should not be advertised: {incorrectly_advertised:?}"
    );
}

/// Test that deferred batch-analysis tools cannot be resolved by name.
///
/// This guards against docs/tests drift by asserting the runtime contract:
/// these names are currently unsupported and absent from the tool registry.
#[tokio::test]
async fn test_deferred_batch_tools_cannot_be_resolved() {
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            batch_config: None,
            ..Default::default()
        })),
    )
    .await
    .expect("Failed to create MCP server");

    let deferred_tools = [
        "batch_query_episodes",
        "batch_pattern_analysis",
        "batch_compare_episodes",
    ];

    for tool in deferred_tools {
        let result = server.get_tool(tool).await;
        assert!(
            result.is_none(),
            "Deferred tool '{tool}' should not resolve from tool registry"
        );
    }
}

/// Test that the server's advertised tools match what's expected.
///
/// This test verifies consistency between the core tools and the full tool list.
#[tokio::test]
async fn test_core_tools_always_available() {
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            batch_config: None,
            ..Default::default()
        })),
    )
    .await
    .expect("Failed to create MCP server");

    let advertised_tools = server.list_all_tools();
    let advertised_names: Vec<&str> = advertised_tools.iter().map(|t| t.name.as_str()).collect();

    // Core tools should always be available
    let core_tools = [
        "query_memory",
        "health_check",
        "get_metrics",
        "analyze_patterns",
        "create_episode",
        "add_episode_step",
        "complete_episode",
        "get_episode",
    ];

    for tool in &core_tools {
        assert!(
            advertised_names.contains(tool),
            "Core tool '{tool}' should always be advertised"
        );
    }
}

/// Verify that tools listed in AGENTS.md categories are registered.
#[tokio::test]
async fn test_agents_md_categories_registered() {
    let server = MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            batch_config: None,
            ..Default::default()
        })),
    )
    .await
    .expect("Failed to create MCP server");

    // We want to check all available tools, including extended ones that might not be loaded initially
    let all_tool_names = server.list_all_tool_names();

    // Tools documented in AGENTS.md under "MCP Server Interaction Patterns"
    let expected_tools = vec![
        // Core/Monitoring
        "query_memory",
        "analyze_patterns",
        "health_check",
        "get_metrics",
        // Patterns/Recommendations
        "advanced_pattern_analysis",
        "search_patterns",
        "recommend_patterns",
        "recommend_playbook",
        "explain_pattern",
        // Checkpoints/Handoff
        "checkpoint_episode",
        "get_handoff_pack",
        "resume_from_handoff",
        // Embeddings
        "configure_embeddings",
        "query_semantic_memory",
        "search_by_embedding",
        "embedding_provider_status",
        // Episode Lifecycle
        "create_episode",
        "add_episode_step",
        "complete_episode",
        "bulk_episodes",
    ];

    for tool in expected_tools {
        assert!(
            all_tool_names.contains(&tool.to_string()),
            "Tool '{tool}' is documented in AGENTS.md but not registered in the server"
        );
    }
}

// =============================================================================
// ADR-024 registry parity (issue #1083)
//
// ADR-024 specifies that the default (`lazy=false`) `tools/list` response
// returns full schemas for *all registered* tools, while `lazy=true` returns
// name/description stubs for the same set. These tests pin that contract to the
// registry contents on a freshly created server (no extended tool loaded).
// =============================================================================

/// Fresh default listing exposes every registered tool with its `inputSchema`.
#[tokio::test]
async fn test_fresh_default_listing_covers_full_registry() {
    let server = fresh_server().await;

    let registered = server.list_all_tool_names();
    let listed = server.list_all_tools();

    assert_eq!(
        listed.len(),
        registered.len(),
        "default listing must expose the whole registry"
    );

    let listed_names: HashSet<&str> = listed.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(
        listed_names.len(),
        listed.len(),
        "default listing must not contain duplicate tool names"
    );

    for name in &registered {
        assert!(
            listed_names.contains(name.as_str()),
            "registered tool '{name}' missing from the default tools/list payload"
        );
    }

    for tool in &listed {
        assert!(
            tool.input_schema.is_object(),
            "tool '{}' must expose a full object inputSchema in default listing",
            tool.name
        );
    }
}

/// Core tools lead the full listing and the listing does not load extended tools.
#[tokio::test]
async fn test_default_listing_preserves_core_order_and_metrics() {
    let server = fresh_server().await;

    // Fresh server: progressive disclosure exposes exactly the core tools.
    let core = server.list_tools().await;
    let all = server.list_all_tools();

    assert!(
        all.len() > core.len(),
        "registry must contain extended tools"
    );

    let core_names: Vec<&str> = core.iter().map(|t| t.name.as_str()).collect();
    let leading_names: Vec<&str> = all
        .iter()
        .take(core.len())
        .map(|t| t.name.as_str())
        .collect();
    assert_eq!(
        leading_names, core_names,
        "core tools must lead the full listing in their existing order"
    );

    // Enumerating the full registry must not mutate progressive-disclosure state.
    let after = server.list_tools().await;
    assert_eq!(
        after.len(),
        core.len(),
        "default listing must not load extended tools"
    );
    let after_names: Vec<&str> = after.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(after_names, core_names, "core ordering must be unchanged");
}

/// Lazy listing exposes every registered name/description without loading state.
#[tokio::test]
async fn test_lazy_listing_parity_with_full_listing() {
    let server = fresh_server().await;
    let core_count = server.list_tools().await.len();
    let full = server.list_all_tools();

    // Registry-level stubs (what the handler consults for lazy listings).
    let stubs = server.list_all_tool_stubs();
    assert_eq!(stubs.len(), full.len());
    for (tool, stub) in full.iter().zip(stubs.iter()) {
        assert_eq!(tool.name, stub.name);
        assert_eq!(tool.description, stub.description);
    }

    // Protocol-level lazy response omits schemas but keeps names + descriptions.
    let lazy_response = handle_list_tools_with_lazy(
        list_request(1, Some(json!({"lazy": true}))),
        server.list_all_tools(),
    )
    .expect("lazy response should exist");
    let lazy_result = lazy_response.result.expect("lazy result should be present");
    let lazy_tools = lazy_result["tools"]
        .as_array()
        .expect("lazy tools array should exist");

    assert_eq!(lazy_tools.len(), full.len());
    let lazy_names: HashSet<String> = lazy_tools
        .iter()
        .map(|t| t["name"].as_str().expect("name").to_string())
        .collect();
    let full_names: HashSet<String> = full.iter().map(|t| t.name.clone()).collect();
    assert_eq!(
        lazy_names, full_names,
        "lazy names must match the full registry"
    );

    for tool in lazy_tools {
        assert!(tool.get("inputSchema").is_none(), "stubs omit inputSchema");
        assert!(
            tool.get("description").is_some(),
            "stubs include descriptions"
        );
    }

    // Neither lazy nor full listing loads execution state.
    assert_eq!(
        server.list_tools().await.len(),
        core_count,
        "lazy listing must not load extended tools"
    );
}

/// `tools/describe` returns the same schema as the full listing entry.
#[tokio::test]
async fn test_describe_matches_full_listing_schema() {
    let server = fresh_server().await;
    let full = server.list_all_tools();

    // One core tool and two extended tools that a fresh server has never loaded.
    for name in ["query_memory", "quality_metrics", "resume_from_compact"] {
        let expected = full
            .iter()
            .find(|t| t.name == name)
            .unwrap_or_else(|| panic!("tool '{name}' must be registered"));

        let response = handle_describe_tool(
            JsonRpcRequest {
                jsonrpc: Some("2.0".to_string()),
                id: Some(json!(1)),
                method: "tools/describe".to_string(),
                params: Some(json!({"name": name})),
            },
            |n: &str| full.iter().find(|t| t.name == n).cloned(),
        )
        .expect("describe response should exist");

        let result = response.result.expect("describe result should be present");
        let tool = result.get("tool").expect("tool object should exist");
        assert_eq!(tool["name"], json!(expected.name));
        assert_eq!(tool["description"], json!(expected.description));
        assert_eq!(
            tool["inputSchema"], expected.input_schema,
            "describe schema for '{name}' must match the full listing schema"
        );
    }
}

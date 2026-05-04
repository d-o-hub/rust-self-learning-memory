//! MCP Tool Contract Parity Tests
//!
//! This test module verifies that every tool advertised by the MCP server
//! (via `list_tools`) has a corresponding handler that can dispatch to it.
//!
//! This catches the issue where tools are defined in the schema but
//! their handlers are commented out or missing, which creates a broken
//! contract with clients.

#![allow(missing_docs)]
#![allow(clippy::doc_markdown)]

use do_memory_core::{MemoryConfig, SelfLearningMemory};
use do_memory_mcp::MemoryMCPServer;
use do_memory_mcp::types::SandboxConfig;
use std::sync::Arc;

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
    let advertised_tools = server.list_tools().await;
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

    let advertised_tools = server.list_tools().await;
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

    let advertised_tools = server.list_tools().await;
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

    let advertised_tools = server.list_tools().await;
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

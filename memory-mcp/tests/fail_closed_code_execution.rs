//! Fail-closed agent code execution (AGENTS.md core invariant).
//!
//! `tools/call` refuses `execute_agent_code` only because the tool is absent
//! from the registry. An advertised definition would silently re-enable
//! execution, so this pins the absence on every listing surface.

#![cfg(not(feature = "sandbox-dev"))]
#![allow(clippy::expect_used, clippy::panic)]

use do_memory_core::{MemoryConfig, SelfLearningMemory};
use do_memory_mcp::MemoryMCPServer;
use do_memory_mcp::types::SandboxConfig;
use std::sync::Arc;

const CODE_EXECUTION_TOOL: &str = "execute_agent_code";

async fn production_server() -> MemoryMCPServer {
    MemoryMCPServer::new(
        SandboxConfig::default(),
        Arc::new(SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            batch_config: None,
            ..Default::default()
        })),
    )
    .await
    .expect("production MCP server must construct")
}

/// Keeps the absence assertions below from passing on an empty registry.
#[tokio::test]
async fn test_production_registry_is_populated() {
    let server = production_server().await;

    let names = server.list_all_tool_names();
    assert!(names.iter().any(|name| name == "query_memory"));
    assert!(names.iter().any(|name| name == "create_episode"));
    assert!(
        server
            .list_all_tools()
            .iter()
            .any(|tool| tool.name == "query_memory")
    );
    assert!(
        server
            .list_all_tool_stubs()
            .iter()
            .any(|tool| tool.name == "query_memory")
    );
    assert!(
        server
            .list_tools()
            .await
            .iter()
            .any(|tool| tool.name == "query_memory")
    );
}

#[tokio::test]
async fn test_production_registry_never_advertises_code_execution() {
    let server = production_server().await;

    assert!(
        !server
            .list_all_tool_names()
            .iter()
            .any(|name| name == CODE_EXECUTION_TOOL),
        "{CODE_EXECUTION_TOOL} is registered; tools/call would then execute agent code"
    );
    assert!(
        !server
            .list_all_tools()
            .iter()
            .any(|tool| tool.name == CODE_EXECUTION_TOOL)
    );
    assert!(
        !server
            .list_all_tool_stubs()
            .iter()
            .any(|tool| tool.name == CODE_EXECUTION_TOOL)
    );
    assert!(
        !server
            .list_tools()
            .await
            .iter()
            .any(|tool| tool.name == CODE_EXECUTION_TOOL)
    );
}

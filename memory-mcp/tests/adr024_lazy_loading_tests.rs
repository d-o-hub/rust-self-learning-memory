//! ADR-024 Integration Tests: MCP Lazy Tool Loading
//!
//! These tests verify the lazy loading feature for MCP tools as specified in ADR-024:
//! - `tools/list` with `lazy=true` returns lightweight stubs (90-96% token reduction)
//! - `tools/list` with `lazy=false` returns full tool schemas (backward compatible)
//! - `tools/list` without `lazy` parameter defaults to full schemas
//! - `tools/describe` endpoint returns full schema for a single tool
//! - `tools/describe_batch` endpoint returns full schemas for multiple tools
//!
//! See: plans/adr/ADR-024-MCP-Lazy-Tool-Loading.md

// Integration tests are separate crate roots and don't inherit .clippy.toml settings
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use memory_core::{MemoryConfig, SelfLearningMemory};
use memory_mcp::{
    MemoryMCPServer, SandboxConfig, Tool,
    jsonrpc::JsonRpcRequest,
    protocol::{handle_describe_tool, handle_describe_tools, handle_list_tools_with_lazy},
};
use serde_json::json;
use std::sync::Arc;

/// Disable WASM sandbox for all tests to ensure consistent tool counts
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

async fn setup_test_server() -> Arc<MemoryMCPServer> {
    disable_wasm_for_tests();
    let memory = Arc::new(SelfLearningMemory::with_config(MemoryConfig {
        quality_threshold: 0.0,
        batch_config: None,
        ..Default::default()
    }));
    let sandbox_config = SandboxConfig::restrictive();
    Arc::new(
        MemoryMCPServer::new(sandbox_config, memory)
            .await
            .expect("Failed to create MCP server"),
    )
}

/// Helper to create a tool lookup function from a pre-loaded tool list
fn make_tool_lookup(tools: Vec<Tool>) -> impl Fn(&str) -> Option<Tool> {
    move |name| tools.iter().find(|t| t.name == name).cloned()
}

// =============================================================================
// Section 1: tools/list lazy parameter behavior
// =============================================================================

mod tools_list_lazy_parameter {
    use super::*;

    /// Test that tools/list with lazy=true returns lightweight stubs
    #[tokio::test]
    async fn test_lazy_true_returns_stubs_without_schema() {
        let mcp_server = setup_test_server().await;
        let tools = mcp_server.list_tools().await;

        let request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(1)),
            method: "tools/list".to_string(),
            params: Some(json!({"lazy": true})),
        };

        let response = handle_list_tools_with_lazy(request, tools).expect("Response should exist");

        assert!(response.result.is_some(), "Result should be present");
        assert!(response.error.is_none(), "Error should be None");

        let result = response.result.unwrap();
        let tools = result["tools"]
            .as_array()
            .expect("tools array should exist");

        assert!(!tools.is_empty(), "Should have at least one tool");

        // Verify each tool is a stub (no inputSchema)
        for tool in tools {
            assert!(tool.get("name").is_some(), "Tool should have name");
            assert!(
                tool.get("description").is_some(),
                "Tool should have description"
            );
            assert!(
                tool.get("inputSchema").is_none(),
                "Lazy stubs should NOT have inputSchema field"
            );
        }
    }

    /// Test that tools/list with lazy=false returns full tool schemas
    #[tokio::test]
    async fn test_lazy_false_returns_full_schemas() {
        let mcp_server = setup_test_server().await;
        let tools = mcp_server.list_tools().await;

        let request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(2)),
            method: "tools/list".to_string(),
            params: Some(json!({"lazy": false})),
        };

        let response = handle_list_tools_with_lazy(request, tools).expect("Response should exist");

        assert!(response.result.is_some(), "Result should be present");
        assert!(response.error.is_none(), "Error should be None");

        let result = response.result.unwrap();
        let tools = result["tools"]
            .as_array()
            .expect("tools array should exist");

        assert!(!tools.is_empty(), "Should have at least one tool");

        // Verify each tool has full schema (with inputSchema)
        for tool in tools {
            assert!(tool.get("name").is_some(), "Tool should have name");
            assert!(
                tool.get("description").is_some(),
                "Tool should have description"
            );
            assert!(
                tool.get("inputSchema").is_some(),
                "Full schemas should have inputSchema field"
            );
        }
    }

    /// Test that tools/list without lazy parameter defaults to full schemas
    #[tokio::test]
    async fn test_lazy_default_returns_full_schemas() {
        let mcp_server = setup_test_server().await;
        let tools = mcp_server.list_tools().await;

        let request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(3)),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handle_list_tools_with_lazy(request, tools).expect("Response should exist");

        assert!(response.result.is_some(), "Result should be present");
        assert!(response.error.is_none(), "Error should be None");

        let result = response.result.unwrap();
        let tools = result["tools"]
            .as_array()
            .expect("tools array should exist");

        for tool in tools {
            assert!(
                tool.get("inputSchema").is_some(),
                "Default behavior should return full schemas with inputSchema"
            );
        }
    }

    /// Test that lazy=true provides significant token reduction
    #[tokio::test]
    async fn test_lazy_mode_token_reduction() {
        let mcp_server = setup_test_server().await;
        let tools = mcp_server.list_tools().await;

        let lazy_request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(4)),
            method: "tools/list".to_string(),
            params: Some(json!({"lazy": true})),
        };
        let lazy_response =
            handle_list_tools_with_lazy(lazy_request, mcp_server.list_tools().await)
                .expect("Response should exist");
        let lazy_size = serde_json::to_string(&lazy_response.result).unwrap().len();

        let full_request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(5)),
            method: "tools/list".to_string(),
            params: Some(json!({"lazy": false})),
        };
        let full_response =
            handle_list_tools_with_lazy(full_request, tools).expect("Response should exist");
        let full_size = serde_json::to_string(&full_response.result).unwrap().len();

        let reduction = ((full_size - lazy_size) as f64 / full_size as f64) * 100.0;

        println!("Full schema size: {} bytes", full_size);
        println!("Lazy stubs size: {} bytes", lazy_size);
        println!("Token reduction: {:.1}%", reduction);

        assert!(
            reduction > 50.0,
            "Lazy mode should provide significant token reduction (got {:.1}%)",
            reduction
        );
        assert!(lazy_size < full_size, "Lazy response should be smaller");
    }

    /// Test that lazy responses contain correct tool names
    #[tokio::test]
    async fn test_lazy_mode_preserves_tool_names() {
        let mcp_server = setup_test_server().await;
        let tools = mcp_server.list_tools().await;

        let lazy_request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(6)),
            method: "tools/list".to_string(),
            params: Some(json!({"lazy": true})),
        };
        let lazy_response =
            handle_list_tools_with_lazy(lazy_request, mcp_server.list_tools().await)
                .expect("Response should exist");

        let full_request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(7)),
            method: "tools/list".to_string(),
            params: Some(json!({"lazy": false})),
        };
        let full_response =
            handle_list_tools_with_lazy(full_request, tools).expect("Response should exist");

        let lazy_result = lazy_response.result.unwrap();
        let lazy_tools = lazy_result["tools"].as_array().unwrap();

        let full_result = full_response.result.unwrap();
        let full_tools = full_result["tools"].as_array().unwrap();

        let lazy_names: std::collections::HashSet<String> = lazy_tools
            .iter()
            .map(|t| t["name"].as_str().unwrap().to_string())
            .collect();

        let full_names: std::collections::HashSet<String> = full_tools
            .iter()
            .map(|t| t["name"].as_str().unwrap().to_string())
            .collect();

        assert_eq!(lazy_names, full_names, "Tool names should match");
    }
}

// =============================================================================
// Section 2: tools/describe endpoint tests
// =============================================================================

mod tools_describe_endpoint {
    use super::*;

    /// Test that tools/describe returns full schema for a valid tool
    #[tokio::test]
    async fn test_describe_returns_full_schema() {
        let mcp_server = setup_test_server().await;
        let tools = mcp_server.list_tools().await;
        let lookup = make_tool_lookup(tools);

        let request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(1)),
            method: "tools/describe".to_string(),
            params: Some(json!({"name": "query_memory"})),
        };

        let response = handle_describe_tool(request, lookup).expect("Response should exist");

        assert!(response.result.is_some(), "Result should be present");
        assert!(response.error.is_none(), "Error should be None");

        let result = response.result.unwrap();
        let tool = result.get("tool").expect("tool object should exist");

        assert_eq!(
            tool.get("name").and_then(|v| v.as_str()),
            Some("query_memory"),
            "Tool name should match"
        );
        assert!(
            tool.get("description").is_some(),
            "Tool should have description"
        );
        assert!(
            tool.get("inputSchema").is_some(),
            "Tool should have inputSchema"
        );
    }

    /// Test that tools/describe returns error for non-existent tool
    #[tokio::test]
    async fn test_describe_nonexistent_tool_returns_error() {
        let mcp_server = setup_test_server().await;
        let tools = mcp_server.list_tools().await;
        let lookup = make_tool_lookup(tools);

        let request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(2)),
            method: "tools/describe".to_string(),
            params: Some(json!({"name": "nonexistent_tool_xyz"})),
        };

        let response = handle_describe_tool(request, lookup).expect("Response should exist");

        assert!(response.result.is_none(), "Result should be None for error");
        assert!(response.error.is_some(), "Error should be present");

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602, "Error code should be Invalid params");
        assert!(
            error.message.contains("not found"),
            "Error should indicate tool not found"
        );
    }

    /// Test that tools/describe returns error when name parameter is missing
    #[tokio::test]
    async fn test_describe_missing_name_parameter() {
        let mcp_server = setup_test_server().await;
        let tools = mcp_server.list_tools().await;
        let lookup = make_tool_lookup(tools);

        let request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(3)),
            method: "tools/describe".to_string(),
            params: Some(json!({})),
        };

        let response = handle_describe_tool(request, lookup).expect("Response should exist");

        assert!(response.result.is_none(), "Result should be None for error");
        assert!(response.error.is_some(), "Error should be present");

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602, "Error code should be Invalid params");
    }

    /// Test that tools/describe works for extended tools
    #[tokio::test]
    async fn test_describe_extended_tool() {
        let mcp_server = setup_test_server().await;

        // Load extended tools first
        let _ = mcp_server.get_tool("quality_metrics").await;
        let tools = mcp_server.list_tools().await;
        let lookup = make_tool_lookup(tools);

        let request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(4)),
            method: "tools/describe".to_string(),
            params: Some(json!({"name": "quality_metrics"})),
        };

        let response = handle_describe_tool(request, lookup).expect("Response should exist");

        assert!(
            response.result.is_some(),
            "Result should be present for extended tool"
        );
        assert!(response.error.is_none(), "Error should be None");

        let result = response.result.unwrap();
        let tool = result.get("tool").expect("tool object should exist");
        assert_eq!(
            tool.get("name").and_then(|v| v.as_str()),
            Some("quality_metrics"),
            "Extended tool should be loadable"
        );
    }

    /// Test that describing same tool twice works (idempotent)
    #[tokio::test]
    async fn test_describe_idempotent() {
        let mcp_server = setup_test_server().await;
        let tools = mcp_server.list_tools().await;
        let lookup1 = make_tool_lookup(tools.clone());
        let lookup2 = make_tool_lookup(tools);

        let request1 = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(5)),
            method: "tools/describe".to_string(),
            params: Some(json!({"name": "health_check"})),
        };

        let response1 = handle_describe_tool(request1, lookup1).expect("Response should exist");

        let request2 = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(6)),
            method: "tools/describe".to_string(),
            params: Some(json!({"name": "health_check"})),
        };

        let response2 = handle_describe_tool(request2, lookup2).expect("Response should exist");

        assert!(response1.result.is_some());
        assert!(response2.result.is_some());

        let tool1 = response1.result.unwrap();
        let tool2 = response2.result.unwrap();

        assert_eq!(
            tool1["tool"]["name"], tool2["tool"]["name"],
            "Both calls should return same tool"
        );
    }
}

// =============================================================================
// Section 3: tools/describe_batch endpoint tests
// =============================================================================

mod tools_describe_batch_endpoint {
    use super::*;

    /// Test that tools/describe_batch returns full schemas for multiple tools
    #[tokio::test]
    async fn test_describe_batch_returns_full_schemas() {
        let mcp_server = setup_test_server().await;
        let tools = mcp_server.list_tools().await;
        let lookup = make_tool_lookup(tools);

        let request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(1)),
            method: "tools/describe_batch".to_string(),
            params: Some(json!({
                "names": ["query_memory", "health_check", "analyze_patterns"]
            })),
        };

        let response = handle_describe_tools(request, lookup).expect("Response should exist");

        assert!(response.result.is_some(), "Result should be present");
        assert!(response.error.is_none(), "Error should be None");

        let result = response.result.unwrap();
        let tools = result["tools"]
            .as_array()
            .expect("tools array should exist");

        assert_eq!(tools.len(), 3, "Should have 3 tools");

        for tool in tools {
            assert!(tool.get("name").is_some(), "Tool should have name");
            assert!(
                tool.get("description").is_some(),
                "Tool should have description"
            );
            assert!(
                tool.get("inputSchema").is_some(),
                "Tool should have inputSchema"
            );
        }
    }

    /// Test that tools/describe_batch with empty array returns empty result
    #[tokio::test]
    async fn test_describe_batch_empty_array() {
        let mcp_server = setup_test_server().await;
        let tools = mcp_server.list_tools().await;
        let lookup = make_tool_lookup(tools);

        let request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(2)),
            method: "tools/describe_batch".to_string(),
            params: Some(json!({"names": []})),
        };

        let response = handle_describe_tools(request, lookup).expect("Response should exist");

        assert!(response.result.is_some(), "Result should be present");
        assert!(response.error.is_none(), "Error should be None");

        let result = response.result.unwrap();
        let tools = result["tools"]
            .as_array()
            .expect("tools array should exist");

        assert_eq!(tools.len(), 0, "Should have 0 tools");
    }

    /// Test that tools/describe_batch with missing names parameter returns error
    #[tokio::test]
    async fn test_describe_batch_missing_names_parameter() {
        let mcp_server = setup_test_server().await;
        let tools = mcp_server.list_tools().await;
        let lookup = make_tool_lookup(tools);

        let request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(3)),
            method: "tools/describe_batch".to_string(),
            params: Some(json!({})),
        };

        let response = handle_describe_tools(request, lookup).expect("Response should exist");

        assert!(response.result.is_none(), "Result should be None for error");
        assert!(response.error.is_some(), "Error should be present");

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602, "Error code should be Invalid params");
    }

    /// Test that tools/describe_batch with mixed valid/invalid names returns only valid tools
    #[tokio::test]
    async fn test_describe_batch_mixed_valid_invalid() {
        let mcp_server = setup_test_server().await;
        let tools = mcp_server.list_tools().await;
        let lookup = make_tool_lookup(tools);

        let request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(4)),
            method: "tools/describe_batch".to_string(),
            params: Some(json!({
                "names": [
                    "query_memory",
                    "nonexistent_tool_xyz",
                    "health_check"
                ]
            })),
        };

        let response = handle_describe_tools(request, lookup).expect("Response should exist");

        assert!(response.result.is_some(), "Result should be present");
        assert!(response.error.is_none(), "Error should be None");

        let result = response.result.unwrap();
        let tools = result["tools"]
            .as_array()
            .expect("tools array should exist");

        assert_eq!(tools.len(), 2, "Should have only 2 valid tools");

        let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();

        assert!(
            names.contains(&"query_memory"),
            "Should contain query_memory"
        );
        assert!(
            names.contains(&"health_check"),
            "Should contain health_check"
        );
        assert!(
            !names.contains(&"nonexistent_tool_xyz"),
            "Should NOT contain nonexistent tool"
        );
    }

    /// Test that tools/describe_batch works with extended tools
    #[tokio::test]
    async fn test_describe_batch_extended_tools() {
        let mcp_server = setup_test_server().await;

        // Load extended tools first
        let _ = mcp_server.get_tool("quality_metrics").await;
        let _ = mcp_server.get_tool("configure_embeddings").await;
        let _ = mcp_server.get_tool("bulk_episodes").await;
        let tools = mcp_server.list_tools().await;
        let lookup = make_tool_lookup(tools);

        let request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(5)),
            method: "tools/describe_batch".to_string(),
            params: Some(json!({
                "names": ["quality_metrics", "configure_embeddings", "bulk_episodes"]
            })),
        };

        let response = handle_describe_tools(request, lookup).expect("Response should exist");

        assert!(response.result.is_some(), "Result should be present");
        assert!(response.error.is_none(), "Error should be None");

        let result = response.result.unwrap();
        let tools = result["tools"]
            .as_array()
            .expect("tools array should exist");

        assert_eq!(tools.len(), 3, "Should have 3 extended tools");

        for tool in tools {
            assert!(
                tool.get("inputSchema").is_some(),
                "Extended tool should have inputSchema"
            );
        }
    }

    /// Test that describe_batch preserves input order
    #[tokio::test]
    async fn test_describe_batch_preserves_order() {
        let mcp_server = setup_test_server().await;
        let tools = mcp_server.list_tools().await;
        let lookup = make_tool_lookup(tools);

        let expected_order = ["analyze_patterns", "health_check", "query_memory"];

        let request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(6)),
            method: "tools/describe_batch".to_string(),
            params: Some(json!({ "names": expected_order })),
        };

        let response = handle_describe_tools(request, lookup).expect("Response should exist");

        let result = response.result.unwrap();
        let tools = result["tools"]
            .as_array()
            .expect("tools array should exist");

        let actual_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();

        assert_eq!(
            actual_names, expected_order,
            "Tools should be returned in the order requested"
        );
    }
}

// =============================================================================
// Section 4: Integration tests combining lazy loading with describe
// =============================================================================

mod lazy_describe_integration {
    use super::*;

    /// Test the typical workflow: lazy list -> describe specific tool
    #[tokio::test]
    async fn test_lazy_list_then_describe_workflow() {
        let mcp_server = setup_test_server().await;

        // Step 1: Get lazy list to find tool names
        let list_request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(1)),
            method: "tools/list".to_string(),
            params: Some(json!({"lazy": true})),
        };

        let list_response =
            handle_list_tools_with_lazy(list_request, mcp_server.list_tools().await)
                .expect("List response should exist");

        let list_result = list_response.result.unwrap();
        let tools_list = list_result["tools"].as_array().unwrap();

        let tool_name = tools_list[0]["name"]
            .as_str()
            .expect("Tool should have name");

        // Step 2: Describe the specific tool to get full schema
        let all_tools = mcp_server.list_tools().await;
        let lookup = make_tool_lookup(all_tools);

        let describe_request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(2)),
            method: "tools/describe".to_string(),
            params: Some(json!({"name": tool_name})),
        };

        let describe_response =
            handle_describe_tool(describe_request, lookup).expect("Describe response should exist");

        let describe_result = describe_response.result.unwrap();
        let tool = &describe_result["tool"];

        assert_eq!(tool["name"].as_str().unwrap(), tool_name);
        assert!(
            tool.get("inputSchema").is_some(),
            "Described tool should have inputSchema"
        );
    }

    /// Test token savings calculation
    #[tokio::test]
    async fn test_token_savings_calculation() {
        let mcp_server = setup_test_server().await;

        // Get lazy list size
        let lazy_request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(1)),
            method: "tools/list".to_string(),
            params: Some(json!({"lazy": true})),
        };
        let lazy_response =
            handle_list_tools_with_lazy(lazy_request, mcp_server.list_tools().await)
                .expect("Response should exist");
        let lazy_size = serde_json::to_string(&lazy_response.result).unwrap().len();

        // Get describe for one tool
        let all_tools = mcp_server.list_tools().await;
        let lookup = make_tool_lookup(all_tools);

        let describe_request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(2)),
            method: "tools/describe".to_string(),
            params: Some(json!({"name": "query_memory"})),
        };
        let describe_response =
            handle_describe_tool(describe_request, lookup).expect("Response should exist");
        let describe_size = serde_json::to_string(&describe_response.result)
            .unwrap()
            .len();

        // Get full list size
        let full_request = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(3)),
            method: "tools/list".to_string(),
            params: Some(json!({"lazy": false})),
        };
        let full_response =
            handle_list_tools_with_lazy(full_request, mcp_server.list_tools().await)
                .expect("Response should exist");
        let full_size = serde_json::to_string(&full_response.result).unwrap().len();

        let lazy_plus_describe = lazy_size + describe_size;

        println!("=== Token Savings Analysis ===");
        println!("Full list: {} bytes", full_size);
        println!("Lazy list: {} bytes", lazy_size);
        println!("Describe one tool: {} bytes", describe_size);
        println!("Lazy + describe one: {} bytes", lazy_plus_describe);
        println!(
            "Savings vs full: {:.1}%",
            ((full_size - lazy_plus_describe) as f64 / full_size as f64) * 100.0
        );

        // The pattern "lazy list + describe one" should be smaller than full list for typical toolsets
        if full_size > 2000 {
            assert!(
                lazy_plus_describe < full_size,
                "Lazy + describe should be smaller than full list for typical toolsets"
            );
        }
    }
}

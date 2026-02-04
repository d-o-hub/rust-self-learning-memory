//! Core MCP protocol handlers
//!
//! This module contains the core MCP protocol handlers:
//! - handle_initialize: Initialize request handler (from library)
//! - handle_protected_resource_metadata: RFC 9728 protected resource metadata
//! - handle_list_tools: List available tools (bin-specific implementation)
//! - handle_call_tool: Execute a tool call
//! - handle_shutdown: Shutdown the server (from library)

use memory_mcp::jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
pub use memory_mcp::protocol::{
    handle_initialize, handle_shutdown, OAuthConfig, ProtectedResourceMetadata,
};
use memory_mcp::protocol::{
    DescribeToolResult, DescribeToolsResult, ListToolStubsResult, ListToolsResult, McpTool,
    ToolStub,
};
use memory_mcp::MemoryMCPServer;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

/// Handle protected resource metadata request (RFC 9728)
pub async fn handle_protected_resource_metadata(
    request: JsonRpcRequest,
    oauth_config: &OAuthConfig,
) -> Option<JsonRpcResponse> {
    request.id.as_ref()?;
    info!("Handling protected resource metadata request");

    // RFC 9728: Protected Resource Metadata
    let resource_uri = std::env::var("MCP_RESOURCE_URI")
        .unwrap_or_else(|_| "https://memory-mcp.example.com".to_string());

    let resource_uri_clone = resource_uri.clone();
    let metadata = ProtectedResourceMetadata {
        authorization_servers: oauth_config
            .issuer
            .clone()
            .map(|iss| vec![iss])
            .unwrap_or_default(),
        resource: resource_uri,
        scopes_supported: oauth_config.scopes.clone(),
        resource_metadata: Some(format!(
            "{}/.well-known/oauth-protected-resource",
            resource_uri_clone
        )),
    };

    match serde_json::to_value(metadata) {
        Ok(value) => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(value),
            error: None,
        }),
        Err(e) => {
            error!("Failed to serialize protected resource metadata: {}", e);
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: "Internal error".to_string(),
                    data: Some(json!({"details": format!("Failed to serialize metadata: {}", e)})),
                }),
            })
        }
    }
}

/// Handle tools/list request
///
/// Supports lazy loading via the `lazy` parameter:
/// - `lazy=true` or `lazy` not specified: Returns lightweight tool stubs (90-96% token reduction)
/// - `lazy=false`: Returns full tool schemas (backward compatible)
pub async fn handle_list_tools(
    request: JsonRpcRequest,
    mcp_server: &Arc<Mutex<MemoryMCPServer>>,
) -> Option<JsonRpcResponse> {
    // Notifications must not produce a response
    request.id.as_ref()?;
    info!("Handling tools/list request");

    // Check if lazy loading is enabled (default: true for token optimization)
    let lazy = request
        .params
        .as_ref()
        .and_then(|p| p.get("lazy"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let server = mcp_server.lock().await;
    let tools = server.list_tools().await;

    if lazy {
        // Return lightweight stubs (90-96% token reduction)
        let tool_stubs: Vec<ToolStub> = tools
            .into_iter()
            .map(|tool| ToolStub {
                name: tool.name,
                title: None,
                description: tool.description,
            })
            .collect();

        let result = ListToolStubsResult { tools: tool_stubs };

        match serde_json::to_value(result) {
            Ok(value) => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(value),
                error: None,
            }),
            Err(e) => {
                error!("Failed to serialize list_tools response: {}", e);
                Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32603,
                        message: "Internal error".to_string(),
                        data: Some(json!({"details": format!("Response serialization failed: {}", e)})),
                    }),
                })
            }
        }
    } else {
        // Return full schemas (backward compatible)
        let mcp_tools: Vec<McpTool> = tools
            .into_iter()
            .map(|tool| McpTool {
                name: tool.name,
                title: None,
                description: tool.description,
                input_schema: tool.input_schema,
            })
            .collect();

        let result = ListToolsResult { tools: mcp_tools };

        match serde_json::to_value(result) {
            Ok(value) => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(value),
                error: None,
            }),
            Err(e) => {
                error!("Failed to serialize list_tools response: {}", e);
                Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32603,
                        message: "Internal error".to_string(),
                        data: Some(json!({"details": format!("Response serialization failed: {}", e)})),
                    }),
                })
            }
        }
    }
}

/// Handle tools/describe request (for on-demand schema loading)
///
/// Parameters:
/// - `name` (required): Tool name to describe
///
/// Returns full tool schema including inputSchema
pub async fn handle_describe_tool(
    request: JsonRpcRequest,
    mcp_server: &Arc<Mutex<MemoryMCPServer>>,
) -> Option<JsonRpcResponse> {
    request.id.as_ref()?;
    info!("Handling tools/describe request");

    // Extract tool name from params
    let tool_name = request
        .params
        .as_ref()
        .and_then(|p| p.get("name"))
        .and_then(|v| v.as_str());

    let tool_name = match tool_name {
        Some(name) => name,
        None => {
            return Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Invalid params".to_string(),
                    data: Some(json!({"details": "Missing required parameter: name"})),
                }),
            });
        }
    };

    let server = mcp_server.lock().await;
    let tool = server.get_tool(tool_name).await;

    match tool {
        Some(tool) => {
            let mcp_tool = McpTool {
                name: tool.name,
                title: None,
                description: tool.description,
                input_schema: tool.input_schema,
            };

            let result = DescribeToolResult { tool: mcp_tool };

            match serde_json::to_value(result) {
                Ok(value) => Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(value),
                    error: None,
                }),
                Err(e) => {
                    error!("Failed to serialize describe_tool response: {}", e);
                    Some(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32603,
                            message: "Internal error".to_string(),
                            data: Some(json!({"details": format!("Response serialization failed: {}", e)})),
                        }),
                    })
                }
            }
        }
        None => {
            warn!("Tool not found: {}", tool_name);
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Tool not found".to_string(),
                    data: Some(json!({"tool_name": tool_name})),
                }),
            })
        }
    }
}

/// Handle tools/describe_batch request (for batch on-demand schema loading)
///
/// Parameters:
/// - `names` (required): Array of tool names to describe
///
/// Returns array of full tool schemas
pub async fn handle_describe_tools(
    request: JsonRpcRequest,
    mcp_server: &Arc<Mutex<MemoryMCPServer>>,
) -> Option<JsonRpcResponse> {
    request.id.as_ref()?;
    info!("Handling tools/describe_batch request");

    // Extract tool names from params
    let tool_names = request
        .params
        .as_ref()
        .and_then(|p| p.get("names"))
        .and_then(|v| v.as_array());

    let tool_names = match tool_names {
        Some(names) => names
            .iter()
            .filter_map(|v| v.as_str())
            .map(String::from)
            .collect::<Vec<_>>(),
        None => {
            return Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Invalid params".to_string(),
                    data: Some(json!({"details": "Missing required parameter: names (array)"})),
                }),
            });
        }
    };

    let server = mcp_server.lock().await;
    let tools = server.tool_registry.load_tools(&tool_names).await;

    let mcp_tools: Vec<McpTool> = tools
        .into_iter()
        .map(|tool| McpTool {
            name: tool.name,
            title: None,
            description: tool.description,
            input_schema: tool.input_schema,
        })
        .collect();

    let result = DescribeToolsResult { tools: mcp_tools };

    match serde_json::to_value(result) {
        Ok(value) => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(value),
            error: None,
        }),
        Err(e) => {
            error!("Failed to serialize describe_tools response: {}", e);
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: "Internal error".to_string(),
                    data: Some(json!({"details": format!("Response serialization failed: {}", e)})),
                }),
            })
        }
    }
}

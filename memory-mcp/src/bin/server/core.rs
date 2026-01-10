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
use memory_mcp::protocol::{ListToolsResult, McpTool};
use memory_mcp::MemoryMCPServer;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

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
pub async fn handle_list_tools(
    request: JsonRpcRequest,
    mcp_server: &Arc<Mutex<MemoryMCPServer>>,
) -> Option<JsonRpcResponse> {
    // Notifications must not produce a response
    request.id.as_ref()?;
    info!("Handling tools/list request");

    let server = mcp_server.lock().await;
    let tools = server.list_tools().await;

    let mcp_tools: Vec<McpTool> = tools
        .into_iter()
        .map(|tool| McpTool {
            name: tool.name,
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

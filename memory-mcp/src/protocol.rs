//! MCP Protocol handlers
//!
//! This module contains core MCP protocol handlers:
//! - handle_initialize: Initialize request handler
//! - handle_list_tools: List available tools
//! - handle_shutdown: Shutdown the server
//!
//! These handlers are used by both the library and binary crate.

use crate::jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use tracing::{error, info};

/// Supported MCP protocol versions (in order of preference, latest first)
pub const SUPPORTED_VERSIONS: &[&str] = &["2025-11-25", "2024-11-05"];

/// OAuth 2.1 Configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OAuthConfig {
    /// Whether authorization is enabled
    pub enabled: bool,
    /// Expected audience for tokens
    pub audience: Option<String>,
    /// Expected issuer for tokens
    pub issuer: Option<String>,
    /// Supported scopes
    pub scopes: Vec<String>,
    /// JWKS URI for token validation
    pub jwks_uri: Option<String>,
}

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            audience: None,
            issuer: None,
            scopes: vec!["mcp:read".to_string(), "mcp:write".to_string()],
            jwks_uri: None,
        }
    }
}

/// MCP Initialize response payload
#[derive(Debug, Serialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: Value,
    #[serde(rename = "serverInfo")]
    pub server_info: Value,
}

/// MCP Tool structure for listing (full schema)
#[derive(Debug, Serialize)]
pub struct McpTool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// MCP Tool stub for lightweight listing (lazy loading optimization)
/// This reduces token usage by 90-96% by omitting the large inputSchema field
#[derive(Debug, Serialize, Clone)]
pub struct ToolStub {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub description: String,
}

impl From<McpTool> for ToolStub {
    fn from(tool: McpTool) -> Self {
        Self {
            name: tool.name,
            title: tool.title,
            description: tool.description,
        }
    }
}

/// MCP ListTools response
#[derive(Debug, Serialize)]
pub struct ListToolsResult {
    pub tools: Vec<McpTool>,
}

/// MCP ListTools response with stubs (for lazy loading)
#[derive(Debug, Serialize)]
pub struct ListToolStubsResult {
    pub tools: Vec<ToolStub>,
}

/// MCP DescribeTool response (for on-demand schema loading)
#[derive(Debug, Serialize)]
pub struct DescribeToolResult {
    pub tool: McpTool,
}

/// MCP DescribeTools (batch) response
#[derive(Debug, Serialize)]
pub struct DescribeToolsResult {
    pub tools: Vec<McpTool>,
}

/// Protected Resource Metadata (RFC 9728)
#[derive(Debug, Serialize)]
pub struct ProtectedResourceMetadata {
    #[serde(rename = "authorizationServers", skip_serializing_if = "Vec::is_empty")]
    pub authorization_servers: Vec<String>,
    pub resource: String,
    #[serde(rename = "scopesSupported", skip_serializing_if = "Vec::is_empty")]
    pub scopes_supported: Vec<String>,
    #[serde(rename = "resourceMetadata")]
    pub resource_metadata: Option<String>,
}

/// Handle initialize request
pub async fn handle_initialize(
    request: JsonRpcRequest,
    oauth_config: &OAuthConfig,
) -> Option<JsonRpcResponse> {
    // Notifications must not produce a response
    request.id.as_ref()?;

    // Extract client's requested protocol version
    let requested_version = request
        .params
        .as_ref()
        .and_then(|params| params.get("protocolVersion").and_then(|v| v.as_str()));

    // Negotiate protocol version
    let protocol_version = match requested_version {
        Some(version) => {
            if SUPPORTED_VERSIONS.contains(&version) {
                version.to_string()
            } else {
                // Client requested unsupported version, return the latest we support
                info!(
                    "Client requested unsupported protocol version '{}', using latest '{}'",
                    version, SUPPORTED_VERSIONS[0]
                );
                SUPPORTED_VERSIONS[0].to_string()
            }
        }
        None => {
            // No version requested, use latest
            SUPPORTED_VERSIONS[0].to_string()
        }
    };

    info!("Negotiated protocol version: {}", protocol_version);

    // Build capabilities object
    let mut capabilities = json!({
        "tools": {
            "listChanged": false
        },
        "completions": {},
        "elicitation": {},
        "tasks": {
            "list": {},
            "create": {},
            "update": {}
        }
    });

    // Add OAuth 2.1 authorization capability if enabled
    if oauth_config.enabled {
        capabilities["authorization"] = json!({
            "enabled": true,
            "issuer": oauth_config.issuer.clone().unwrap_or_default(),
            "audience": oauth_config.audience.clone().unwrap_or_default(),
            "scopes": oauth_config.scopes
        });
    }

    let result = InitializeResult {
        protocol_version,
        capabilities,
        server_info: json!({
            "name": "memory-mcp-server",
            "version": env!("CARGO_PKG_VERSION")
        }),
    };

    match serde_json::to_value(result) {
        Ok(value) => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(value),
            error: None,
        }),
        Err(e) => {
            error!("Failed to serialize initialize response: {}", e);
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

/// Handle tools/list request
pub async fn handle_list_tools(
    request: JsonRpcRequest,
    tools: Vec<McpTool>,
) -> Option<JsonRpcResponse> {
    // Notifications must not produce a response
    request.id.as_ref()?;
    info!("Handling tools/list request");

    let result = ListToolsResult { tools };

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

/// Handle shutdown request
pub async fn handle_shutdown(request: JsonRpcRequest) -> Option<JsonRpcResponse> {
    // Notifications must not produce a response
    request.id.as_ref()?;
    info!("Handling shutdown request");

    Some(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result: Some(json!(null)),
        error: None,
    })
}

/// Handle tools/list request with lazy loading support (ADR-024)
///
/// Supports lazy loading via the `lazy` parameter:
/// - `lazy=true`: Returns lightweight tool stubs (90-96% token reduction)
/// - `lazy=false` or `lazy` not specified: Returns full tool schemas (backward compatible)
///
/// # Arguments
///
/// * `request` - The JSON-RPC request
/// * `tools` - The tools to list (full Tool objects)
///
/// # Returns
///
/// JSON-RPC response with either `ListToolStubsResult` (lazy=true) or `ListToolsResult` (lazy=false)
pub fn handle_list_tools_with_lazy(
    request: JsonRpcRequest,
    tools: Vec<crate::types::Tool>,
) -> Option<JsonRpcResponse> {
    // Notifications must not produce a response
    request.id.as_ref()?;
    info!("Handling tools/list request");

    // Check if lazy loading is enabled (default: false for compatibility)
    let lazy = request
        .params
        .as_ref()
        .and_then(|p| p.get("lazy"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

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
                        data: Some(
                            json!({"details": format!("Response serialization failed: {}", e)}),
                        ),
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
                        data: Some(
                            json!({"details": format!("Response serialization failed: {}", e)}),
                        ),
                    }),
                })
            }
        }
    }
}

/// Handle tools/describe request (ADR-024)
///
/// Returns full schema for a single tool (on-demand loading after lazy list).
///
/// # Arguments
///
/// * `request` - The JSON-RPC request with `name` parameter
/// * `get_tool` - Function to get a tool by name (returns `Option<Tool>`)
///
/// # Returns
///
/// JSON-RPC response with `DescribeToolResult` or error if tool not found
pub fn handle_describe_tool<F>(request: JsonRpcRequest, get_tool: F) -> Option<JsonRpcResponse>
where
    F: FnOnce(&str) -> Option<crate::types::Tool>,
{
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

    let tool = get_tool(tool_name);

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
                            data: Some(
                                json!({"details": format!("Response serialization failed: {}", e)}),
                            ),
                        }),
                    })
                }
            }
        }
        None => {
            info!("Tool not found: {}", tool_name);
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

/// Handle tools/describe_batch request (ADR-024)
///
/// Returns full schemas for multiple tools (batch on-demand loading).
///
/// # Arguments
///
/// * `request` - The JSON-RPC request with `names` array parameter
/// * `get_tool` - Function to get a tool by name (returns `Option<Tool>`)
///
/// # Returns
///
/// JSON-RPC response with `DescribeToolsResult` containing found tools
pub fn handle_describe_tools<F>(request: JsonRpcRequest, get_tool: F) -> Option<JsonRpcResponse>
where
    F: Fn(&str) -> Option<crate::types::Tool>,
{
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

    // Load tools by name
    let mut mcp_tools = Vec::new();
    for tool_name in &tool_names {
        if let Some(tool) = get_tool(tool_name) {
            mcp_tools.push(McpTool {
                name: tool.name,
                title: None,
                description: tool.description,
                input_schema: tool.input_schema,
            });
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_versions() {
        assert_eq!(SUPPORTED_VERSIONS, &["2025-11-25", "2024-11-05"]);
    }

    #[test]
    fn test_oauth_config_default() {
        let config = OAuthConfig::default();
        assert!(!config.enabled);
        assert!(config.audience.is_none());
        assert!(config.issuer.is_none());
        assert_eq!(config.scopes.len(), 2);
    }

    #[tokio::test]
    async fn test_initialize_protocol_version_latest() {
        let req = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(1)),
            method: "initialize".into(),
            params: Some(json!({
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"}
            })),
        };

        let resp = handle_initialize(req, &OAuthConfig::default()).await;
        assert!(resp.is_some());
        let resp = resp.unwrap();
        assert!(resp.result.is_some());

        let result = resp.result.unwrap();
        let protocol_version = result.get("protocolVersion").and_then(|v| v.as_str());
        assert_eq!(protocol_version, Some("2025-11-25"));
    }

    #[tokio::test]
    async fn test_initialize_protocol_version_older() {
        let req = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(2)),
            method: "initialize".into(),
            params: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"}
            })),
        };

        let resp = handle_initialize(req, &OAuthConfig::default()).await;
        assert!(resp.is_some());
        let resp = resp.unwrap();
        assert!(resp.result.is_some());

        let result = resp.result.unwrap();
        let protocol_version = result.get("protocolVersion").and_then(|v| v.as_str());
        assert_eq!(protocol_version, Some("2024-11-05"));
    }

    #[tokio::test]
    async fn test_initialize_protocol_version_unsupported() {
        let req = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(3)),
            method: "initialize".into(),
            params: Some(json!({
                "protocolVersion": "2020-01-01",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"}
            })),
        };

        let resp = handle_initialize(req, &OAuthConfig::default()).await;
        assert!(resp.is_some());
        let resp = resp.unwrap();
        assert!(resp.result.is_some());

        let result = resp.result.unwrap();
        let protocol_version = result.get("protocolVersion").and_then(|v| v.as_str());
        // Should return latest supported version
        assert_eq!(protocol_version, Some("2025-11-25"));
    }

    #[tokio::test]
    async fn test_initialize_no_version() {
        let req = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(4)),
            method: "initialize".into(),
            params: Some(json!({
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"}
            })),
        };

        let resp = handle_initialize(req, &OAuthConfig::default()).await;
        assert!(resp.is_some());
        let resp = resp.unwrap();
        assert!(resp.result.is_some());

        let result = resp.result.unwrap();
        let protocol_version = result.get("protocolVersion").and_then(|v| v.as_str());
        assert_eq!(protocol_version, Some("2025-11-25"));
    }
}

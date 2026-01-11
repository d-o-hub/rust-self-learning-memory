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
use serde_json::json;
use serde_json::Value;
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

/// MCP Tool structure for listing
#[derive(Debug, Serialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// MCP ListTools response
#[derive(Debug, Serialize)]
pub struct ListToolsResult {
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

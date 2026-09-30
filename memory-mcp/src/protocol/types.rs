//! MCP Protocol types and constants
//!
//! This module contains type definitions for MCP protocol messages.

use serde::Serialize;
use serde_json::Value;

/// Supported MCP protocol versions (in order of preference, latest first)
pub const SUPPORTED_VERSIONS: &[&str] = &["2025-11-25", "2024-11-05"];

/// OAuth 2.1 Configuration
#[derive(Debug, Clone)]
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
    /// Secret key for HMAC token validation
    pub token_secret: Option<String>,
}

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            audience: None,
            issuer: None,
            scopes: vec!["mcp:read".to_string(), "mcp:write".to_string()],
            jwks_uri: None,
            token_secret: None,
        }
    }
}

impl OAuthConfig {
    /// Diagnostic explaining why OAuth cannot be enforced, if that is the case
    ///
    /// Returns `None` when OAuth is disabled or when the configuration is
    /// sufficient to verify tokens. Callers MUST treat `Some(_)` as fail-closed:
    /// never dispatch requests while authorization is advertised but cannot be
    /// verified.
    pub fn enforcement_error(&self) -> Option<&'static str> {
        if !self.enabled {
            return None;
        }

        #[cfg(not(feature = "oauth"))]
        {
            Some(
                "MCP_OAUTH_ENABLED is set but this server was built without the `oauth` feature, \
                 so token signatures cannot be verified",
            )
        }

        #[cfg(feature = "oauth")]
        {
            if self.token_secret.is_none() {
                Some(
                    "MCP_OAUTH_ENABLED is set but MCP_OAUTH_TOKEN_SECRET is missing, \
                     so token signatures cannot be verified",
                )
            } else {
                None
            }
        }
    }

    /// Whether authorization is enabled *and* actually enforceable
    pub fn is_enforced(&self) -> bool {
        self.enabled && self.enforcement_error().is_none()
    }
}

/// Validate an OAuth configuration before serving requests (issue #1082)
///
/// Returns `Err` with an actionable diagnostic when `MCP_OAUTH_ENABLED` is set
/// but tokens cannot be verified. Callers MUST fail closed (refuse to start or
/// reject the request) rather than serving unauthenticated traffic.
pub fn validate_oauth_config(config: &OAuthConfig) -> Result<(), &'static str> {
    match config.enforcement_error() {
        None => Ok(()),
        Some(reason) => Err(reason),
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

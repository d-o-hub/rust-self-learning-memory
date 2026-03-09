//! MCP Protocol handlers
//!
//! This module contains core MCP protocol handlers:
//! - handle_initialize: Initialize request handler
//! - handle_list_tools: List available tools
//! - handle_shutdown: Shutdown the server
//!
//! These handlers are used by both the library and binary crate.

mod handlers;
mod types;

pub use handlers::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jsonrpc::JsonRpcRequest;
    use serde_json::json;

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

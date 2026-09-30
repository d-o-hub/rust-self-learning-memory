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

    #[tokio::test]
    async fn test_initialize_omits_authorization_capability_when_disabled() {
        let req = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(5)),
            method: "initialize".into(),
            params: Some(json!({"protocolVersion": "2025-11-25"})),
        };

        let resp = handle_initialize(req, &OAuthConfig::default())
            .await
            .unwrap();
        let capabilities = resp.result.unwrap()["capabilities"].clone();
        assert!(
            capabilities.get("authorization").is_none(),
            "authorization must not be advertised while OAuth is disabled"
        );
        assert!(capabilities.get("tools").is_some());
    }

    #[cfg(feature = "oauth")]
    #[tokio::test]
    async fn test_initialize_advertises_authorization_only_when_enforced() {
        let enforced = OAuthConfig {
            enabled: true,
            issuer: Some("https://issuer.example".to_string()),
            audience: Some("mcp-server".to_string()),
            token_secret: Some("secret".to_string()),
            ..OAuthConfig::default()
        };
        let req = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(6)),
            method: "initialize".into(),
            params: Some(json!({"protocolVersion": "2025-11-25"})),
        };

        let value = handle_initialize(req, &enforced)
            .await
            .unwrap()
            .result
            .unwrap();
        let authorization = &value["capabilities"]["authorization"];
        assert_eq!(authorization["enabled"], json!(true));
        assert_eq!(authorization["issuer"], json!("https://issuer.example"));
        assert_eq!(authorization["audience"], json!("mcp-server"));

        // Enabled but unenforceable (no secret): the capability must be absent
        let misconfigured = OAuthConfig {
            enabled: true,
            token_secret: None,
            ..OAuthConfig::default()
        };
        let req = JsonRpcRequest {
            jsonrpc: Some("2.0".to_string()),
            id: Some(json!(7)),
            method: "initialize".into(),
            params: Some(json!({"protocolVersion": "2025-11-25"})),
        };
        let value = handle_initialize(req, &misconfigured)
            .await
            .unwrap()
            .result
            .unwrap();
        assert!(
            value["capabilities"].get("authorization").is_none(),
            "an unenforceable configuration must not advertise authorization"
        );
    }
}

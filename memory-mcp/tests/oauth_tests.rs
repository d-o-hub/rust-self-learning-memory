//! `OAuth` 2.1 enforcement tests for the MCP server (issue #1082)
//!
//! These tests exercise the real request path: `handle_request` must reject an
//! unauthenticated call when `OAuth` is enabled, must fail closed when token
//! verification is impossible, and must only advertise the authorization
//! capability when enforcement is real.

#![cfg(feature = "oauth")]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::doc_markdown)]
#![allow(missing_docs)]

#[path = "../src/bin/server_impl/mod.rs"]
mod server_impl;

use do_memory_mcp::MemoryMCPServer;
use do_memory_mcp::protocol::OAuthConfig;
use do_memory_mcp::server::rate_limiter::{ClientId, OperationType, RateLimitConfig, RateLimiter};
use do_memory_mcp::types::SandboxConfig;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use server_impl::{
    AuthContext, EmbeddingEnvConfig, JsonRpcRequest, RequestAuthorization, authorize_request,
    extract_request_bearer_token, handle_request, run_jsonrpc_server,
};
use std::sync::Arc;
use tokio::sync::Mutex;

const SECRET: &str = "test-secret-key";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TestClaims {
    iss: Option<String>,
    aud: Option<String>,
    exp: Option<u64>,
    sub: String,
    scope: Option<String>,
}

fn config() -> OAuthConfig {
    OAuthConfig {
        enabled: true,
        issuer: Some("https://auth.example.com".to_string()),
        audience: Some("mcp-server".to_string()),
        token_secret: Some(SECRET.to_string()),
        ..OAuthConfig::default()
    }
}

fn token(scope: &str, subject: &str, expires_in: u64) -> anyhow::Result<String> {
    token_expiring_at(
        scope,
        subject,
        jsonwebtoken::get_current_timestamp() + expires_in,
    )
}

fn token_expiring_at(scope: &str, subject: &str, exp: u64) -> anyhow::Result<String> {
    let claims = TestClaims {
        iss: Some("https://auth.example.com".to_string()),
        aud: Some("mcp-server".to_string()),
        exp: Some(exp),
        sub: subject.to_string(),
        scope: Some(scope.to_string()),
    };
    Ok(encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(SECRET.as_bytes()),
    )?)
}

fn embedding_config() -> EmbeddingEnvConfig {
    EmbeddingEnvConfig {
        provider: "local".to_string(),
        api_key: None,
        api_key_env: "OPENAI_API_KEY".to_string(),
        model: None,
        similarity_threshold: 0.7,
        batch_size: 32,
    }
}

fn disabled_limiter() -> RateLimiter {
    RateLimiter::new(RateLimitConfig {
        enabled: false,
        ..RateLimitConfig::default()
    })
}

/// Context with no transport credential (credentials must come from the request)
fn auth_ctx(config: OAuthConfig) -> AuthContext {
    AuthContext::new(config, ClientId::process(), None)
}

/// Context carrying a stdio transport credential
fn auth_ctx_with_token(config: OAuthConfig, token: &str) -> AuthContext {
    AuthContext::new(config, ClientId::process(), Some(token.to_string()))
}

async fn test_server() -> anyhow::Result<Arc<Mutex<MemoryMCPServer>>> {
    let memory = Arc::new(do_memory_core::SelfLearningMemory::new());
    let server = MemoryMCPServer::new(SandboxConfig::restrictive(), memory).await?;
    Ok(Arc::new(Mutex::new(server)))
}

fn request(method: &str, params: Option<serde_json::Value>) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: Some("2.0".to_string()),
        id: Some(json!(1)),
        method: method.to_string(),
        params,
    }
}

fn tool_call(params_extra: &serde_json::Value) -> JsonRpcRequest {
    let mut params = json!({
        "name": "query_memory",
        "arguments": {"query": "anything"}
    });
    if let (Some(base), Some(extra)) = (params.as_object_mut(), params_extra.as_object()) {
        for (key, value) in extra {
            base.insert(key.clone(), value.clone());
        }
    }
    request("tools/call", Some(params))
}

async fn call(
    server: &Arc<Mutex<MemoryMCPServer>>,
    request: JsonRpcRequest,
    auth_context: &AuthContext,
) -> Option<server_impl::JsonRpcResponse> {
    let elicitations: Arc<Mutex<Vec<server_impl::ActiveElicitation>>> =
        Arc::new(Mutex::new(Vec::new()));
    let tasks: Arc<Mutex<Vec<server_impl::ActiveTask>>> = Arc::new(Mutex::new(Vec::new()));
    let embedding = embedding_config();
    let limiter = disabled_limiter();
    handle_request(
        request,
        server,
        auth_context,
        &elicitations,
        &tasks,
        &embedding,
        &limiter,
    )
    .await
}

#[tokio::test]
async fn test_oauth_enabled_unauthenticated_tool_call_rejected() -> anyhow::Result<()> {
    let server = test_server().await?;
    let ctx = auth_ctx(config());

    let response = call(&server, tool_call(&json!({})), &ctx)
        .await
        .ok_or_else(|| anyhow::anyhow!("expected an error response"))?;

    let error = response
        .error
        .ok_or_else(|| anyhow::anyhow!("tool call must not succeed without a token"))?;
    assert_eq!(error.code, server_impl::oauth::UNAUTHORIZED_CODE);
    assert_eq!(error.message, "Unauthorized");
    assert!(response.result.is_none());

    // No credential material in the error payload
    let rendered = serde_json::to_string(&error.data)?;
    assert!(!rendered.contains(SECRET));
    assert!(
        !rendered.contains("test-secret"),
        "error payload must not echo secrets"
    );
    Ok(())
}

#[tokio::test]
async fn test_spoofed_client_id_does_not_authenticate() -> anyhow::Result<()> {
    let server = test_server().await?;
    let ctx = auth_ctx(config());

    // Caller-supplied identity fields must not substitute for credentials.
    let spoofed = tool_call(&json!({
        "_meta": {"client_id": "trusted-admin", "headers": {"Authorization": "Bearer not-a-jwt"}},
        "client_id": "trusted-admin"
    }));
    let response = call(&server, spoofed, &ctx)
        .await
        .ok_or_else(|| anyhow::anyhow!("expected an error response"))?;

    let error = response
        .error
        .ok_or_else(|| anyhow::anyhow!("spoofed identity must not authorize"))?;
    assert_eq!(error.code, server_impl::oauth::UNAUTHORIZED_CODE);
    Ok(())
}

#[tokio::test]
async fn test_valid_token_is_dispatched_and_scopes_enforced() -> anyhow::Result<()> {
    let server = test_server().await?;
    let ctx = auth_ctx(config());
    let read_token = token("mcp:read", "user-1", 600)?;

    // Read scope on a read method reaches dispatch
    let list_request = request(
        "tools/list",
        Some(json!({"_meta": {"authorization": format!("Bearer {read_token}")}})),
    );
    let response = call(&server, list_request, &ctx)
        .await
        .ok_or_else(|| anyhow::anyhow!("expected a response"))?;
    let result = response
        .result
        .ok_or_else(|| anyhow::anyhow!("read scope must be accepted for tools/list"))?;
    assert!(result.get("tools").is_some());

    // Same read token cannot perform a write operation
    let response = call(
        &server,
        tool_call(&json!({"_meta": {"authorization": format!("Bearer {read_token}")}})),
        &ctx,
    )
    .await
    .ok_or_else(|| anyhow::anyhow!("expected an error response"))?;
    let error = response
        .error
        .ok_or_else(|| anyhow::anyhow!("read scope must not authorize a write"))?;
    assert_eq!(error.code, server_impl::oauth::INSUFFICIENT_SCOPE_CODE);
    assert_eq!(error.message, "Insufficient scope");
    Ok(())
}

#[tokio::test]
async fn test_write_scope_allows_write_operation() -> anyhow::Result<()> {
    let server = test_server().await?;
    let ctx = auth_ctx(config());
    let write_token = token("mcp:write", "user-2", 600)?;

    let response = call(
        &server,
        tool_call(&json!({"_meta": {"authorization": format!("Bearer {write_token}")}})),
        &ctx,
    )
    .await
    .ok_or_else(|| anyhow::anyhow!("expected a response"))?;

    // The token is authorized, so the call reaches the tool dispatcher (it may
    // fail on tool input, but it is no longer an authorization failure).
    if let Some(error) = response.error {
        assert_ne!(error.code, server_impl::oauth::UNAUTHORIZED_CODE);
        assert_ne!(error.code, server_impl::oauth::INSUFFICIENT_SCOPE_CODE);
    }
    Ok(())
}

#[tokio::test]
async fn test_stdio_transport_token_authenticates() -> anyhow::Result<()> {
    let server = test_server().await?;
    let read_token = token("mcp:read", "user-3", 600)?;
    // stdio carries the credential out-of-band (server process environment)
    let ctx = auth_ctx_with_token(config(), &read_token);

    let response = call(&server, request("tools/list", None), &ctx)
        .await
        .ok_or_else(|| anyhow::anyhow!("expected a response"))?;
    assert!(
        response.result.is_some(),
        "transport token must authenticate the request"
    );
    Ok(())
}

#[tokio::test]
async fn test_expired_token_rejected() -> anyhow::Result<()> {
    let server = test_server().await?;
    let ctx = auth_ctx(config());
    // Expired well beyond the 60s validation leeway
    let expired = token_expiring_at(
        "mcp:write",
        "user-4",
        jsonwebtoken::get_current_timestamp() - 3600,
    )?;

    let response = call(
        &server,
        tool_call(&json!({"_meta": {"authorization": format!("Bearer {expired}")}})),
        &ctx,
    )
    .await
    .ok_or_else(|| anyhow::anyhow!("expected an error response"))?;
    let error = response
        .error
        .ok_or_else(|| anyhow::anyhow!("expired token must be rejected"))?;
    assert_eq!(error.code, server_impl::oauth::UNAUTHORIZED_CODE);
    Ok(())
}

#[tokio::test]
async fn test_initialize_without_token_still_available() -> anyhow::Result<()> {
    let server = test_server().await?;
    let ctx = auth_ctx(config());

    let response = call(
        &server,
        request("initialize", Some(json!({"protocolVersion": "2025-11-25"}))),
        &ctx,
    )
    .await
    .ok_or_else(|| anyhow::anyhow!("expected a response"))?;
    assert!(
        response.result.is_some(),
        "discovery methods stay reachable without a token"
    );
    Ok(())
}

#[tokio::test]
async fn test_missing_secret_fails_closed() -> anyhow::Result<()> {
    let server = test_server().await?;
    let misconfigured = OAuthConfig {
        enabled: true,
        token_secret: None,
        ..OAuthConfig::default()
    };

    // Startup must refuse to serve
    let startup = run_jsonrpc_server(Arc::clone(&server), misconfigured.clone()).await;
    let err = startup
        .err()
        .ok_or_else(|| anyhow::anyhow!("server must refuse to start without a token secret"))?;
    assert!(
        err.to_string().contains("MCP_OAUTH_TOKEN_SECRET"),
        "diagnostic must name the missing secret, got: {err}"
    );

    // And no request may be authorized while configuration is unenforceable
    let outcome = authorize_request(None, OperationType::Write, &misconfigured, None);
    assert!(
        matches!(
            &outcome,
            RequestAuthorization::Rejected { error, description, .. }
                if error == "invalid_config" && description.contains("MCP_OAUTH_TOKEN_SECRET")
        ),
        "expected fail-closed invalid_config, got {outcome:?}"
    );

    let response = call(&server, tool_call(&json!({})), &auth_ctx(misconfigured))
        .await
        .ok_or_else(|| anyhow::anyhow!("expected an error response"))?;
    let error = response
        .error
        .ok_or_else(|| anyhow::anyhow!("requests must fail closed"))?;
    assert_eq!(error.code, server_impl::oauth::UNAUTHORIZED_CODE);
    Ok(())
}

#[tokio::test]
async fn test_authorization_capability_only_advertised_when_enforced() -> anyhow::Result<()> {
    let server = test_server().await?;
    let ctx = auth_ctx(config());

    // Enforced configuration advertises the capability
    let response = call(
        &server,
        request("initialize", Some(json!({"protocolVersion": "2025-11-25"}))),
        &ctx,
    )
    .await
    .ok_or_else(|| anyhow::anyhow!("expected a response"))?;
    let result = response
        .result
        .ok_or_else(|| anyhow::anyhow!("initialize must succeed"))?;
    assert!(
        result
            .get("capabilities")
            .and_then(|c| c.get("authorization"))
            .is_some(),
        "enforced OAuth must be advertised"
    );

    // Enabled but unenforceable configuration advertises nothing (and rejects)
    let misconfigured = OAuthConfig {
        enabled: true,
        token_secret: None,
        ..OAuthConfig::default()
    };
    assert!(!misconfigured.is_enforced());
    assert!(misconfigured.enforcement_error().is_some());

    // Disabled configuration never advertises authorization
    let disabled = OAuthConfig::default();
    let response = call(
        &server,
        request("initialize", Some(json!({"protocolVersion": "2025-11-25"}))),
        &auth_ctx(disabled),
    )
    .await
    .ok_or_else(|| anyhow::anyhow!("expected a response"))?;
    let result = response
        .result
        .ok_or_else(|| anyhow::anyhow!("initialize must succeed"))?;
    assert!(
        result
            .get("capabilities")
            .and_then(|c| c.get("authorization"))
            .is_none(),
        "disabled OAuth must not be advertised"
    );
    Ok(())
}

#[test]
fn test_extract_request_bearer_token_sources() -> anyhow::Result<()> {
    let params = json!({"_meta": {"headers": {"Authorization": "Bearer from-headers"}}});
    assert_eq!(
        extract_request_bearer_token(Some(&params), None),
        Some("from-headers".to_string())
    );

    let params = json!({"authorization": "Bearer from-param"});
    assert_eq!(
        extract_request_bearer_token(Some(&params), None),
        Some("from-param".to_string())
    );

    // Transport credential is used when the request carries none
    assert_eq!(
        extract_request_bearer_token(None, Some("transport-token")),
        Some("transport-token".to_string())
    );
    assert_eq!(
        extract_request_bearer_token(None, Some("Bearer transport-token")),
        Some("transport-token".to_string())
    );
    assert_eq!(extract_request_bearer_token(None, None), None);
    Ok(())
}

#[test]
fn test_identity_fields_are_not_credentials() {
    // The request may carry identity-like fields; the authorization gate must
    // ignore them entirely (checked through the public behavior in
    // `test_spoofed_client_id_does_not_authenticate`).
    let params = json!({"_meta": {"client_id": "spoofed"}, "client_id": "spoofed"});
    assert_eq!(
        extract_request_bearer_token(Some(&params), None),
        None,
        "identity fields are not credentials"
    );
}

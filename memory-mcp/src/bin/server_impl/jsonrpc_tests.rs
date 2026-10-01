//! Tests for the `jsonrpc` module (split out of it to keep the source file under the LOC ceiling).

use super::*;

async fn test_server() -> anyhow::Result<Arc<Mutex<MemoryMCPServer>>> {
    let memory = Arc::new(do_memory_core::SelfLearningMemory::new());
    let server = MemoryMCPServer::new(do_memory_mcp::SandboxConfig::restrictive(), memory).await?;
    Ok(Arc::new(Mutex::new(server)))
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

async fn dispatch(
    server: &Arc<Mutex<MemoryMCPServer>>,
    request: JsonRpcRequest,
    auth_context: &AuthContext,
) -> Option<JsonRpcResponse> {
    let elicitations: Arc<Mutex<Vec<ActiveElicitation>>> = Arc::new(Mutex::new(Vec::new()));
    let tasks: Arc<Mutex<Vec<ActiveTask>>> = Arc::new(Mutex::new(Vec::new()));
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

fn request(method: &str, params: Option<serde_json::Value>) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: Some("2.0".to_string()),
        id: Some(json!(1)),
        method: method.to_string(),
        params,
    }
}

#[test]
fn test_load_rate_limit_config_bounds_identities() {
    let config = load_rate_limit_config();
    // Stale identity buckets must still be evicted after five minutes
    assert_eq!(config.stale_threshold, std::time::Duration::from_secs(300));
    assert!(config.max_identities > 0);
}

#[tokio::test]
async fn test_notifications_produce_no_response() -> anyhow::Result<()> {
    let server = test_server().await?;
    let auth = AuthContext::new(
        OAuthConfig::default(),
        do_memory_mcp::server::rate_limiter::ClientId::process(),
        None,
    );
    let mut notification = request("tools/list", None);
    notification.id = None;

    assert!(dispatch(&server, notification, &auth).await.is_none());
    Ok(())
}

#[tokio::test]
async fn test_startup_refuses_unenforceable_oauth() -> anyhow::Result<()> {
    let server = test_server().await?;
    let misconfigured = OAuthConfig {
        enabled: true,
        token_secret: None,
        ..OAuthConfig::default()
    };

    let err = run_jsonrpc_server(server, misconfigured)
        .await
        .err()
        .ok_or_else(|| anyhow::anyhow!("server must refuse to start"))?;
    assert!(
        err.to_string().contains("cannot be enforced"),
        "diagnostic must explain that enforcement is impossible, got: {err}"
    );
    Ok(())
}

#[cfg(feature = "oauth")]
#[tokio::test]
async fn test_handle_request_rejects_unauthenticated_tool_call() -> anyhow::Result<()> {
    let server = test_server().await?;
    let enforced = OAuthConfig {
        enabled: true,
        token_secret: Some("jsonrpc-test-secret".to_string()),
        ..OAuthConfig::default()
    };
    let auth = AuthContext::new(
        enforced,
        do_memory_mcp::server::rate_limiter::ClientId::process(),
        None,
    );

    let response = dispatch(
        &server,
        request(
            "tools/call",
            Some(json!({"name": "query_memory", "arguments": {"query": "x"}})),
        ),
        &auth,
    )
    .await
    .ok_or_else(|| anyhow::anyhow!("expected an error response"))?;

    let error = response
        .error
        .ok_or_else(|| anyhow::anyhow!("unauthenticated call must be rejected"))?;
    assert_eq!(error.code, super::super::oauth::UNAUTHORIZED_CODE);
    assert_eq!(error.message, "Unauthorized");
    Ok(())
}

#[tokio::test]
async fn test_handle_request_dispatches_when_oauth_disabled() -> anyhow::Result<()> {
    let server = test_server().await?;
    let auth = AuthContext::new(
        OAuthConfig::default(),
        do_memory_mcp::server::rate_limiter::ClientId::process(),
        None,
    );

    let response = dispatch(&server, request("tools/list", None), &auth)
        .await
        .ok_or_else(|| anyhow::anyhow!("expected a response"))?;
    assert!(
        response.result.is_some(),
        "requests must be dispatched when OAuth is disabled"
    );
    Ok(())
}

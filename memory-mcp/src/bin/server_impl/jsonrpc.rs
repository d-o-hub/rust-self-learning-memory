//! JSON-RPC server infrastructure

use super::core::{
    handle_describe_tool, handle_describe_tools, handle_initialize, handle_list_tools,
    handle_protected_resource_metadata, handle_shutdown,
};
use super::handlers::{handle_batch_execute, handle_call_tool};
use super::mcp::{
    handle_completion_complete, handle_elicitation_cancel, handle_elicitation_data,
    handle_elicitation_request, handle_task_cancel, handle_task_complete, handle_task_create,
    handle_task_list, handle_task_update,
};
use super::types::{ActiveElicitation, ActiveTask, EmbeddingEnvConfig, RateLimitEnvConfig};
use memory_mcp::jsonrpc::{
    read_next_message, write_response_with_length, JsonRpcError, JsonRpcRequest, JsonRpcResponse,
};
use memory_mcp::protocol::OAuthConfig;
use memory_mcp::server::rate_limiter::{ClientId, OperationType, RateLimitConfig, RateLimiter};
use memory_mcp::MemoryMCPServer;
use serde_json::json;
use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// Load embedding configuration from environment variables
pub fn load_embedding_config() -> EmbeddingEnvConfig {
    let provider = std::env::var("EMBEDDING_PROVIDER")
        .unwrap_or_else(|_| "local".to_string())
        .to_lowercase();
    let api_key = std::env::var("OPENAI_API_KEY").ok();
    let api_key_env =
        std::env::var("OPENAI_API_KEY_ENV").unwrap_or_else(|_| "OPENAI_API_KEY".to_string());
    let model = std::env::var("EMBEDDING_MODEL")
        .ok()
        .filter(|m| !m.is_empty());
    let similarity_threshold: f32 = std::env::var("EMBEDDING_SIMILARITY_THRESHOLD")
        .unwrap_or_else(|_| "0.7".to_string())
        .parse()
        .unwrap_or(0.7);
    let batch_size: usize = std::env::var("EMBEDDING_BATCH_SIZE")
        .unwrap_or_else(|_| "32".to_string())
        .parse()
        .unwrap_or(32);
    EmbeddingEnvConfig {
        provider,
        api_key,
        api_key_env,
        model,
        similarity_threshold,
        batch_size,
    }
}

/// Load rate limit configuration from environment variables
pub fn load_rate_limit_config() -> RateLimitConfig {
    let env_config = RateLimitEnvConfig::from_env();
    RateLimitConfig {
        enabled: env_config.enabled,
        read_requests_per_second: env_config.read_rps,
        read_burst_size: env_config.read_burst,
        write_requests_per_second: env_config.write_rps,
        write_burst_size: env_config.write_burst,
        cleanup_interval: std::time::Duration::from_secs(env_config.cleanup_interval_secs),
        client_id_header: env_config.client_id_header,
    }
}

/// Extract client ID from request parameters or use default
fn extract_client_id(params: Option<&serde_json::Value>, client_id_header: &str) -> ClientId {
    if let Some(params) = params {
        // Try to extract from meta field (common in MCP requests)
        if let Some(meta) = params.get("_meta") {
            if let Some(client_id) = meta.get("client_id").and_then(|v| v.as_str()) {
                return ClientId::from_string(client_id);
            }

            // Try to extract from headers in meta
            if let Some(headers) = meta.get("headers") {
                if let Some(client_id) = headers.get(client_id_header).and_then(|v| v.as_str()) {
                    return ClientId::from_string(client_id);
                }
            }
        }

        // Try to extract from client_id field directly
        if let Some(client_id) = params.get("client_id").and_then(|v| v.as_str()) {
            return ClientId::from_string(client_id);
        }
    }

    // Fallback to unknown client
    ClientId::Unknown
}

/// Handle embedding/config request
pub async fn handle_embedding_config(
    request: JsonRpcRequest,
    embedding_config: &EmbeddingEnvConfig,
) -> Option<JsonRpcResponse> {
    request.id.as_ref()?;
    info!("Handling embedding/config");
    let dimension = match embedding_config.provider.as_str() {
        "openai" => match embedding_config.model.as_deref() {
            Some("text-embedding-3-small") => 1536,
            Some("text-embedding-3-large") => 3072,
            Some("text-embedding-ada-002") => 1536,
            _ => 1536,
        },
        _ => 384,
    };
    let model_name = embedding_config.model.clone().unwrap_or_else(|| {
        match embedding_config.provider.as_str() {
            "openai" => "text-embedding-3-small".to_string(),
            _ => "all-MiniLM-L6-v2".to_string(),
        }
    });
    let has_api_key = embedding_config.api_key.is_some();
    Some(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result: Some(json!({
            "success": true, "provider": embedding_config.provider, "model": model_name,
            "dimension": dimension, "hasApiKey": has_api_key,
            "similarityThreshold": embedding_config.similarity_threshold,
            "batchSize": embedding_config.batch_size,
            "message": if has_api_key { format!("{} embeddings configured", embedding_config.provider) }
            else { format!("{} embeddings configured (no API key set)", embedding_config.provider) }
        })),
        error: None,
    })
}

/// Main message loop for JSON-RPC
#[allow(clippy::excessive_nesting)]
pub async fn run_jsonrpc_server(
    mcp_server: Arc<Mutex<MemoryMCPServer>>,
    oauth_config: OAuthConfig,
) -> anyhow::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut handle = stdin.lock();
    let elicitation_tracker: Arc<Mutex<Vec<ActiveElicitation>>> = Arc::new(Mutex::new(Vec::new()));
    let task_tracker: Arc<Mutex<Vec<ActiveTask>>> = Arc::new(Mutex::new(Vec::new()));
    let embedding_config = load_embedding_config();
    let rate_limit_config = load_rate_limit_config();
    let rate_limiter = Arc::new(RateLimiter::new(rate_limit_config));

    if rate_limiter.is_enabled() {
        info!("Rate limiting enabled");
    } else {
        info!("Rate limiting disabled");
    }

    #[allow(unused_assignments)]
    let mut last_input_was_lsp = false;
    loop {
        match read_next_message(&mut handle) {
            Ok(None) => {
                info!("Received EOF, shutting down");
                break;
            }
            Ok(Some((line, is_lsp))) => {
                last_input_was_lsp = is_lsp;
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                #[allow(clippy::excessive_nesting)]
                match serde_json::from_str::<JsonRpcRequest>(line) {
                    Ok(request) => {
                        let response = handle_request(
                            request,
                            &mcp_server,
                            &oauth_config,
                            &elicitation_tracker,
                            &task_tracker,
                            &embedding_config,
                            &rate_limiter,
                        )
                        .await;
                        if let Some(response_json) = response {
                            let response_str = serde_json::to_string(&response_json)?;
                            if last_input_was_lsp {
                                write_response_with_length(&mut stdout, &response_str)?;
                            } else {
                                writeln!(stdout, "{}", response_str)?;
                                stdout.flush()?;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse JSON-RPC request: {}", e);
                        let error_response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: None,
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32700,
                                message: "Parse error".to_string(),
                                data: Some(json!({"details": e.to_string()})),
                            }),
                        };
                        let response_str = serde_json::to_string(&error_response)?;
                        if last_input_was_lsp {
                            write_response_with_length(&mut stdout, &response_str)?;
                        } else {
                            writeln!(stdout, "{}", response_str)?;
                            stdout.flush()?;
                        }
                    }
                }
            }
            Err(e) => {
                error!("Error reading from stdin: {}", e);
                break;
            }
        }
    }
    info!("Memory MCP Server shutting down");
    Ok(())
}

/// Handle a JSON-RPC request with rate limiting
#[allow(clippy::excessive_nesting)]
pub async fn handle_request(
    request: JsonRpcRequest,
    mcp_server: &Arc<Mutex<MemoryMCPServer>>,
    oauth_config: &OAuthConfig,
    elicitation_tracker: &Arc<Mutex<Vec<ActiveElicitation>>>,
    task_tracker: &Arc<Mutex<Vec<ActiveTask>>>,
    embedding_config: &EmbeddingEnvConfig,
    rate_limiter: &RateLimiter,
) -> Option<JsonRpcResponse> {
    if request.id.is_none() || matches!(request.id, Some(serde_json::Value::Null)) {
        return None;
    }
    if oauth_config.enabled {
        debug!("OAuth enabled");
    }

    // Normalize method name
    let compat_env = std::env::var("MCP_COMPAT_ALIASES").unwrap_or_else(|_| "true".to_string());
    let compat = compat_env.to_lowercase();
    let compat_enabled = !(compat == "false" || compat == "0" || compat == "no");
    let mut method = request.method.clone();
    if compat_enabled {
        method = match request.method.as_str() {
            "tools.get" | "tools/get" | "list_tools" | "list-tools" => "tools/list".to_string(),
            "call_tool" | "tool/call" | "tools.call" => "tools/call".to_string(),
            _ => request.method.clone(),
        };
    }

    // Check rate limit
    let client_id = extract_client_id(
        request.params.as_ref(),
        &rate_limiter.config.client_id_header,
    );
    let operation_type = OperationType::from_method(&method);
    let rate_limit_result = rate_limiter.check_rate_limit(&client_id, operation_type);

    if !rate_limit_result.allowed {
        warn!(
            "Rate limit exceeded for client: {} (method: {})",
            client_id, method
        );
        let retry_after_secs = rate_limit_result
            .retry_after
            .map(|d| d.as_secs())
            .unwrap_or(1);
        return Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32000,
                message: "Rate limit exceeded".to_string(),
                data: Some(json!({
                    "retry_after": retry_after_secs,
                    "limit": rate_limit_result.limit,
                    "remaining": rate_limit_result.remaining,
                })),
            }),
        });
    }

    // Process the request
    let response = match method.as_str() {
        "initialize" => handle_initialize(request, oauth_config).await,
        "tools/list" => handle_list_tools(request, mcp_server).await,
        "tools/describe" => handle_describe_tool(request, mcp_server).await,
        "tools/describe_batch" => handle_describe_tools(request, mcp_server).await,
        "tools/call" => handle_call_tool(request, mcp_server).await,
        "batch/execute" => handle_batch_execute(request, mcp_server).await,
        "shutdown" => handle_shutdown(request).await,
        "completion/complete" => handle_completion_complete(request).await,
        "elicitation/request" => handle_elicitation_request(request, elicitation_tracker).await,
        "elicitation/data" => handle_elicitation_data(request, elicitation_tracker).await,
        "elicitation/cancel" => handle_elicitation_cancel(request, elicitation_tracker).await,
        "task/create" => handle_task_create(request, task_tracker).await,
        "task/update" => handle_task_update(request, task_tracker).await,
        "task/complete" => handle_task_complete(request, task_tracker).await,
        "task/cancel" => handle_task_cancel(request, task_tracker).await,
        "task/list" => handle_task_list(request, task_tracker).await,
        "embedding/config" => handle_embedding_config(request, embedding_config).await,
        ".well-known/oauth-protected-resource" => {
            handle_protected_resource_metadata(request, oauth_config).await
        }
        _ => {
            warn!("Unknown method: {}", method);
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: "Method not found".to_string(),
                    data: None,
                }),
            })
        }
    };

    // Note: Rate limit headers would typically be added here for HTTP-based protocols
    // For stdio-based JSON-RPC, we include rate limit info in the response data
    response
}

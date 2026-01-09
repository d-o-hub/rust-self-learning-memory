//! JSON-RPC server infrastructure
//!
//! This module contains the JSON-RPC server loop and request routing:
//! - run_jsonrpc_server: Main server loop
//! - handle_request: Request routing and method normalization

use super::core::{
    handle_initialize, handle_list_tools, handle_protected_resource_metadata, handle_shutdown,
};
use super::mcp::{
    handle_completion_complete, handle_elicitation_cancel, handle_elicitation_data,
    handle_elicitation_request, handle_task_cancel, handle_task_complete, handle_task_create,
    handle_task_list, handle_task_update,
};
use super::tools::{
    handle_advanced_pattern_analysis, handle_analyze_patterns, handle_configure_embeddings,
    handle_execute_code, handle_get_metrics, handle_health_check, handle_quality_metrics,
    handle_query_memory, handle_query_semantic_memory, handle_test_embeddings,
};
use super::types::{
    ActiveElicitation, ActiveTask, CallToolParams, CallToolResult, EmbeddingEnvConfig, OAuthConfig,
};
use memory_mcp::jsonrpc::{
    read_next_message, write_response_with_length, JsonRpcError, JsonRpcRequest, JsonRpcResponse,
};
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

/// Handle embedding/config - query embedding configuration
pub async fn handle_embedding_config(
    request: JsonRpcRequest,
    embedding_config: &EmbeddingEnvConfig,
) -> Option<JsonRpcResponse> {
    request.id.as_ref()?;
    info!("Handling embedding/config");

    // Determine dimension based on provider/model
    let dimension = match embedding_config.provider.as_str() {
        "openai" => match embedding_config.model.as_deref() {
            Some("text-embedding-3-small") => 1536,
            Some("text-embedding-3-large") => 3072,
            Some("text-embedding-ada-002") => 1536,
            _ => 1536, // Default for OpenAI
        },
        _ => 384, // Default for local or other providers (all-MiniLM-L6-v2)
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
            "success": true,
            "provider": embedding_config.provider,
            "model": model_name,
            "dimension": dimension,
            "hasApiKey": has_api_key,
            "similarityThreshold": embedding_config.similarity_threshold,
            "batchSize": embedding_config.batch_size,
            "message": if has_api_key {
                format!("{} embeddings configured", embedding_config.provider)
            } else {
                format!(
                    "{} embeddings configured (no API key set)",
                    embedding_config.provider
                )
            }
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
    // Main message loop for JSON-RPC
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut handle = stdin.lock();

    // Track active elicitation requests (MCP 2025-11-25)
    let elicitation_tracker: Arc<Mutex<Vec<ActiveElicitation>>> = Arc::new(Mutex::new(Vec::new()));

    // Track active tasks (MCP 2025-11-25)
    let task_tracker: Arc<Mutex<Vec<ActiveTask>>> = Arc::new(Mutex::new(Vec::new()));

    // Load embedding configuration from environment
    let embedding_config = load_embedding_config();

    // Track last input framing to respond with the same framing style
    #[allow(unused_assignments)]
    let mut last_input_was_lsp = false;
    loop {
        match read_next_message(&mut handle) {
            Ok(None) => {
                // EOF reached
                info!("Received EOF, shutting down");
                break;
            }
            Ok(Some((line, is_lsp))) => {
                last_input_was_lsp = is_lsp;
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                // Parse JSON-RPC request
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
                        )
                        .await;

                        // Send response, matching the input framing style
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
                        // Send error response
                        let error_response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: None, // We don't have an ID if parsing failed
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

/// Handle a JSON-RPC request
#[allow(clippy::excessive_nesting)]
pub async fn handle_request(
    request: JsonRpcRequest,
    mcp_server: &Arc<Mutex<MemoryMCPServer>>,
    oauth_config: &OAuthConfig,
    elicitation_tracker: &Arc<Mutex<Vec<ActiveElicitation>>>,
    task_tracker: &Arc<Mutex<Vec<ActiveTask>>>,
    embedding_config: &EmbeddingEnvConfig,
) -> Option<JsonRpcResponse> {
    // Notifications (no id) must not produce a response per JSON-RPC
    // Treat both missing id and explicit null id as notifications (no response)
    if request.id.is_none() || matches!(request.id, Some(serde_json::Value::Null)) {
        return None;
    }

    // Check authorization for protected methods
    if oauth_config.enabled {
        // For HTTP transport, extract Bearer token from Authorization header
        // For stdio, token would need to be passed via request metadata
        // This is a placeholder for HTTP transport mode
        debug!("OAuth enabled - authorization check would occur here for HTTP transport");
    }

    // Normalize method name if compatibility aliases are enabled
    // Enable compatibility aliases by default for broader client support.
    // Can be disabled by setting MCP_COMPAT_ALIASES to "false"/"0"/"no".
    let compat_env = std::env::var("MCP_COMPAT_ALIASES").unwrap_or_else(|_| "true".to_string());
    let compat = compat_env.to_lowercase();
    let compat_enabled = !(compat == "false" || compat == "0" || compat == "no");

    let mut method = request.method.clone();
    if compat_enabled {
        method = match request.method.as_str() {
            // Common variants observed in some clients
            "tools.get" | "tools/get" | "list_tools" | "list-tools" => "tools/list".to_string(),
            "call_tool" | "tool/call" | "tools.call" => "tools/call".to_string(),
            // Pass through known methods unchanged
            _ => request.method.clone(),
        };
    }

    match method.as_str() {
        "initialize" => handle_initialize(request, oauth_config).await,
        "tools/list" => handle_list_tools(request, mcp_server).await,
        "tools/call" => handle_call_tool(request, mcp_server).await,
        "shutdown" => handle_shutdown(request).await,
        "completion/complete" => handle_completion_complete(request).await,
        // Elicitation handlers (MCP 2025-11-25)
        "elicitation/request" => handle_elicitation_request(request, elicitation_tracker).await,
        "elicitation/data" => handle_elicitation_data(request, elicitation_tracker).await,
        "elicitation/cancel" => handle_elicitation_cancel(request, elicitation_tracker).await,
        // Tasks handlers (MCP 2025-11-25)
        "task/create" => handle_task_create(request, task_tracker).await,
        "task/update" => handle_task_update(request, task_tracker).await,
        "task/complete" => handle_task_complete(request, task_tracker).await,
        "task/cancel" => handle_task_cancel(request, task_tracker).await,
        "task/list" => handle_task_list(request, task_tracker).await,
        // Embedding configuration
        "embedding/config" => handle_embedding_config(request, embedding_config).await,
        // OAuth 2.1 protected resource metadata endpoint (MCP specification)
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
    }
}

/// Handle tools/call request
#[allow(clippy::excessive_nesting)]
async fn handle_call_tool(
    request: JsonRpcRequest,
    mcp_server: &Arc<Mutex<MemoryMCPServer>>,
) -> Option<JsonRpcResponse> {
    // Notifications must not produce a response
    request.id.as_ref()?;
    let params: CallToolParams = match request.params {
        Some(params) => match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => {
                return Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Invalid params".to_string(),
                        data: Some(json!({"details": e.to_string()})),
                    }),
                });
            }
        },
        None => {
            return Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Missing params".to_string(),
                    data: None,
                }),
            });
        }
    };

    info!("Handling tools/call request for tool: {}", params.name);

    let mut server = mcp_server.lock().await;
    let result = match params.name.as_str() {
        "query_memory" => handle_query_memory(&mut server, params.arguments).await,
        "execute_agent_code" => {
            // Check if execute_agent_code tool is available
            let tools = server.list_tools().await;
            if tools.iter().any(|t| t.name == "execute_agent_code") {
                handle_execute_code(&mut server, params.arguments).await
            } else {
                return Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32000,
                        message: "Tool execution failed".to_string(),
                        data: Some(json!({
                            "details": "execute_agent_code tool is not available due to WASM sandbox compilation issues"
                        })),
                    }),
                });
            }
        }
        "analyze_patterns" => handle_analyze_patterns(&mut server, params.arguments).await,
        "advanced_pattern_analysis" => {
            handle_advanced_pattern_analysis(&mut server, params.arguments).await
        }
        "health_check" => handle_health_check(&mut server, params.arguments).await,
        "get_metrics" => handle_get_metrics(&mut server, params.arguments).await,
        "quality_metrics" => handle_quality_metrics(&mut server, params.arguments).await,
        "configure_embeddings" => handle_configure_embeddings(&mut server, params.arguments).await,
        "query_semantic_memory" => {
            handle_query_semantic_memory(&mut server, params.arguments).await
        }
        "test_embeddings" => handle_test_embeddings(&mut server, params.arguments).await,
        _ => {
            return Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: "Tool not found".to_string(),
                    data: Some(json!({"tool": params.name})),
                }),
            });
        }
    };

    // Process the tool result
    let response = match result {
        Ok(content) => {
            let call_result = CallToolResult { content };
            match serde_json::to_value(call_result) {
                Ok(value) => Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(value),
                    error: None,
                }),
                Err(e) => {
                    error!("Failed to serialize call_tool response: {}", e);
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
        Err(e) => {
            error!("Tool execution failed: {}", e);
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32000,
                    message: "Tool execution failed".to_string(),
                    data: Some(json!({"details": e.to_string()})),
                }),
            })
        }
    };

    response
}

//! Batch execution handler

use super::tool_call::{DispatchError, dispatch_tool};
use do_memory_mcp::MemoryMCPServer;
use do_memory_mcp::jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

/// Handle batch/execute request
#[allow(clippy::excessive_nesting)]
pub async fn handle_batch_execute(
    request: JsonRpcRequest,
    mcp_server: &Arc<Mutex<MemoryMCPServer>>,
) -> Option<JsonRpcResponse> {
    use do_memory_mcp::{BatchExecutor, BatchRequest};

    request.id.as_ref()?;

    let batch_request: BatchRequest = match request.params {
        Some(params) => match serde_json::from_value(params) {
            Ok(br) => br,
            Err(e) => {
                return Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Invalid batch request params".to_string(),
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
                    message: "Missing batch request params".to_string(),
                    data: None,
                }),
            });
        }
    };

    info!(
        "Handling batch/execute request with {} operations (mode: {:?})",
        batch_request.operations.len(),
        batch_request.mode
    );

    let executor = BatchExecutor::new();
    let server_clone = Arc::clone(mcp_server);

    let executor_fn = move |tool_name: String, arguments: serde_json::Value| {
        let server_for_task = Arc::clone(&server_clone);
        async move {
            let mut server = server_for_task.lock().await;

            let result = match dispatch_tool(&mut server, &tool_name, Some(arguments)).await {
                Ok(r) => r,
                Err(DispatchError::ToolNotFound(name)) => {
                    Err(anyhow::anyhow!("Unknown tool: {}", name))
                }
                Err(DispatchError::ToolUnavailable(details)) => Err(anyhow::anyhow!("{}", details)),
            };

            match result {
                Ok(content_vec) => match serde_json::to_value(&content_vec) {
                    Ok(value) => Ok(value),
                    Err(e) => Err((-32603, format!("Failed to serialize result: {}", e))),
                },
                Err(e) => Err((-32000, e.to_string())),
            }
        }
    };

    let batch_result = executor.execute(batch_request, executor_fn).await;

    match batch_result {
        Ok(response) => {
            info!(
                "Batch execution completed: {} successful, {} failed, {}ms total",
                response.success_count, response.failure_count, response.total_duration_ms
            );

            let client_id = "batch-client";
            {
                let server = mcp_server.lock().await;
                server
                    .audit_logger()
                    .log_batch_execution(
                        client_id,
                        response.success_count + response.failure_count,
                        response.success_count,
                        response.failure_count,
                        true,
                    )
                    .await;
            }

            match serde_json::to_value(response) {
                Ok(value) => Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(value),
                    error: None,
                }),
                Err(e) => {
                    error!("Failed to serialize batch response: {}", e);
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
            error!("Batch execution failed: {}", e);
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: "Batch execution failed".to_string(),
                    data: Some(json!({"details": e})),
                }),
            })
        }
    }
}

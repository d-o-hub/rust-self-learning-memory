//! JSON-RPC request handlers
//!
//! This module contains the tool call and batch execution handlers:
//! - handle_call_tool: Route tools/call requests to appropriate handlers
//! - handle_batch_execute: Handle batch/execute requests

use super::tools::{
    handle_advanced_pattern_analysis, handle_analyze_patterns, handle_bulk_episodes,
    handle_configure_embeddings, handle_execute_code, handle_get_metrics, handle_health_check,
    handle_quality_metrics, handle_query_memory, handle_query_semantic_memory,
    handle_recommend_patterns, handle_search_patterns, handle_test_embeddings,
};
use super::types::{CallToolParams, CallToolResult, Content};
use memory_mcp::jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use memory_mcp::MemoryMCPServer;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

/// Handle tools/call request
#[allow(clippy::excessive_nesting)]
pub async fn handle_call_tool(
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
        "search_patterns" => handle_search_patterns(&mut server, params.arguments).await,
        "recommend_patterns" => handle_recommend_patterns(&mut server, params.arguments).await,
        "bulk_episodes" => handle_bulk_episodes(&mut server, params.arguments).await,
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
            let call_result = CallToolResult::success(content);
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
            let error_content = vec![Content::Text {
                text: format!("Tool execution failed: {}", e),
            }];
            let call_result = CallToolResult::error(error_content);
            match serde_json::to_value(call_result) {
                Ok(value) => Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(value),
                    error: None,
                }),
                Err(ser_e) => {
                    error!("Failed to serialize error response: {}", ser_e);
                    Some(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32603,
                            message: "Internal error".to_string(),
                            data: Some(
                                json!({"details": format!("Response serialization failed: {}", ser_e)}),
                            ),
                        }),
                    })
                }
            }
        }
    };

    response
}

/// Handle batch/execute request
#[allow(clippy::excessive_nesting)]
pub async fn handle_batch_execute(
    request: JsonRpcRequest,
    mcp_server: &Arc<Mutex<MemoryMCPServer>>,
) -> Option<JsonRpcResponse> {
    use memory_mcp::{BatchExecutor, BatchRequest};

    // Notifications must not produce a response
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

    // Create batch executor
    let executor = BatchExecutor::new();

    // Clone the server Arc for use in the executor function
    let server_clone = Arc::clone(mcp_server);

    // Define executor function that calls tools
    let executor_fn = move |tool_name: String, arguments: serde_json::Value| {
        let server_for_task = Arc::clone(&server_clone);
        async move {
            let mut server = server_for_task.lock().await;

            // Route to appropriate tool handler
            let result = match tool_name.as_str() {
                "query_memory" => handle_query_memory(&mut server, Some(arguments)).await,
                "execute_agent_code" => {
                    // Check if execute_agent_code tool is available
                    let tools = server.list_tools().await;
                    if tools.iter().any(|t| t.name == "execute_agent_code") {
                        handle_execute_code(&mut server, Some(arguments)).await
                    } else {
                        Err(anyhow::anyhow!(
                            "execute_agent_code tool is not available due to WASM sandbox issues"
                        ))
                    }
                }
                "analyze_patterns" => handle_analyze_patterns(&mut server, Some(arguments)).await,
                "advanced_pattern_analysis" => {
                    handle_advanced_pattern_analysis(&mut server, Some(arguments)).await
                }
                "health_check" => handle_health_check(&mut server, Some(arguments)).await,
                "get_metrics" => handle_get_metrics(&mut server, Some(arguments)).await,
                "quality_metrics" => handle_quality_metrics(&mut server, Some(arguments)).await,
                "configure_embeddings" => {
                    handle_configure_embeddings(&mut server, Some(arguments)).await
                }
                "query_semantic_memory" => {
                    handle_query_semantic_memory(&mut server, Some(arguments)).await
                }
                "test_embeddings" => handle_test_embeddings(&mut server, Some(arguments)).await,
                "search_patterns" => handle_search_patterns(&mut server, Some(arguments)).await,
                "recommend_patterns" => {
                    handle_recommend_patterns(&mut server, Some(arguments)).await
                }
                "bulk_episodes" => handle_bulk_episodes(&mut server, Some(arguments)).await,
                _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
            };

            // Convert result to expected format (Content array -> JSON Value)
            match result {
                Ok(content_vec) => {
                    // Serialize Content array to JSON Value
                    match serde_json::to_value(&content_vec) {
                        Ok(value) => Ok(value),
                        Err(e) => Err((-32603, format!("Failed to serialize result: {}", e))),
                    }
                }
                Err(e) => Err((-32000, e.to_string())),
            }
        }
    };

    // Execute batch
    let batch_result = executor.execute(batch_request, executor_fn).await;

    match batch_result {
        Ok(response) => {
            info!(
                "Batch execution completed: {} successful, {} failed, {}ms total",
                response.success_count, response.failure_count, response.total_duration_ms
            );

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

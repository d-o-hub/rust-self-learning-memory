//! # Elicitation Handlers (MCP 2025-11-25)
//!
//! Handlers for elicitation/request, elicitation/data, and elicitation/cancel.

use memory_mcp::jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use super::super::ActiveElicitation;
use super::super::types::{
    ElicitationCancelParams, ElicitationDataParams, ElicitationParams, ElicitationResult,
};

/// Handle elicitation/request - server asks client for user input
pub async fn handle_elicitation_request(
    request: JsonRpcRequest,
    elicitation_tracker: &Arc<Mutex<Vec<ActiveElicitation>>>,
) -> Option<JsonRpcResponse> {
    request.id.as_ref()?;
    info!("Handling elicitation/request");

    let params: ElicitationParams = match request.params {
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
                })
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
            })
        }
    };

    // Store active elicitation
    let active = ActiveElicitation {
        id: params.elicitation_id.clone(),
        prompt: params.prompt.clone(),
        trigger: params.trigger.clone(),
        created_at: std::time::Instant::now(),
    };

    let mut tracker = elicitation_tracker.lock().await;
    tracker.push(active);

    info!(
        "Elicitation {} created for tool: {}",
        params.elicitation_id, params.trigger
    );

    Some(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result: Some(json!({
            "elicitationId": params.elicitation_id,
            "status": "pending"
        })),
        error: None,
    })
}

/// Handle elicitation/data - client responds with user input
pub async fn handle_elicitation_data(
    request: JsonRpcRequest,
    elicitation_tracker: &Arc<Mutex<Vec<ActiveElicitation>>>,
) -> Option<JsonRpcResponse> {
    request.id.as_ref()?;
    info!("Handling elicitation/data");

    let params: ElicitationDataParams = match request.params {
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
                })
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
            })
        }
    };

    // Find and remove matching elicitation
    let mut tracker = elicitation_tracker.lock().await;
    let index = tracker.iter().position(|e| e.id == params.elicitation_id);

    match index {
        Some(i) => {
            let elicitation = tracker.remove(i);
            info!(
                "Elicitation {} resolved from tool: {}",
                params.elicitation_id, elicitation.trigger
            );

            let result = ElicitationResult {
                elicitation_id: params.elicitation_id,
                data: params.data,
            };

            match serde_json::to_value(result) {
                Ok(value) => Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(value),
                    error: None,
                }),
                Err(e) => {
                    error!("Failed to serialize elicitation response: {}", e);
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
        None => {
            warn!("Elicitation {} not found", params.elicitation_id);
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Elicitation not found".to_string(),
                    data: Some(
                        json!({"details": format!("No active elicitation with id: {}", params.elicitation_id)}),
                    ),
                }),
            })
        }
    }
}

/// Cancel an active elicitation
pub async fn handle_elicitation_cancel(
    request: JsonRpcRequest,
    elicitation_tracker: &Arc<Mutex<Vec<ActiveElicitation>>>,
) -> Option<JsonRpcResponse> {
    request.id.as_ref()?;
    info!("Handling elicitation/cancel");

    let params: ElicitationCancelParams = match request.params {
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
                })
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
            })
        }
    };

    let mut tracker = elicitation_tracker.lock().await;
    let index = tracker.iter().position(|e| e.id == params.elicitation_id);

    match index {
        Some(i) => {
            let elicitation = tracker.remove(i);
            info!(
                "Elicitation {} cancelled for tool: {}",
                params.elicitation_id, elicitation.trigger
            );

            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(json!({
                    "elicitationId": params.elicitation_id,
                    "status": "cancelled"
                })),
                error: None,
            })
        }
        None => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32602,
                message: "Elicitation not found".to_string(),
                data: Some(
                    json!({"details": format!("No active elicitation with id: {}", params.elicitation_id)}),
                ),
            }),
        }),
    }
}

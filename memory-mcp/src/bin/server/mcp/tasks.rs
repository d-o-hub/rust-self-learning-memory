//! # Task Handlers (MCP 2025-11-25)
//!
//! Handlers for task/create, task/update, task/complete, task/cancel, and task/list.

use memory_mcp::jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use super::super::ActiveTask;
use super::super::types::{
    TaskCancelParams, TaskCompleteParams, TaskCreateParams, TaskStatus, TaskUpdateParams,
};

/// Handle task/create - create a new long-running task
pub async fn handle_task_create(
    request: JsonRpcRequest,
    task_tracker: &Arc<Mutex<Vec<ActiveTask>>>,
) -> Option<JsonRpcResponse> {
    request.id.as_ref()?;
    info!("Handling task/create");

    let params: TaskCreateParams = match request.params {
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

    let active = ActiveTask {
        id: params.task_id.clone(),
        name: params.task.name.clone(),
        status: TaskStatus::Pending,
        input: params.task.input.clone(),
        metadata: params.task.metadata.clone(),
        progress: 0,
        result: None,
        created_at: std::time::Instant::now(),
    };

    let mut tracker = task_tracker.lock().await;
    tracker.push(active);

    info!("Task {} created: {}", params.task_id, params.task.name);

    Some(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result: Some(json!({
            "taskId": params.task_id,
            "status": "pending"
        })),
        error: None,
    })
}

/// Handle task/update - update task status
pub async fn handle_task_update(
    request: JsonRpcRequest,
    task_tracker: &Arc<Mutex<Vec<ActiveTask>>>,
) -> Option<JsonRpcResponse> {
    request.id.as_ref()?;
    info!("Handling task/update");

    let params: TaskUpdateParams = match request.params {
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

    let mut tracker = task_tracker.lock().await;
    let index = tracker.iter().position(|t| t.id == params.task_id);

    match index {
        Some(i) => {
            let task = &mut tracker[i];
            task.status = params.status.clone();
            if let Some(progress) = params.progress {
                task.progress = progress;
            }
            if let Some(_partial) = params.partial_result {
                info!("Task {} progress: {}%", params.task_id, task.progress);
            }

            let status_str = format!("{:?}", params.status).to_lowercase();

            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(json!({
                    "taskId": params.task_id,
                    "status": status_str,
                    "progress": task.progress
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
                message: "Task not found".to_string(),
                data: Some(
                    json!({"details": format!("No active task with id: {}", params.task_id)}),
                ),
            }),
        }),
    }
}

/// Handle task/complete - complete a task with result
pub async fn handle_task_complete(
    request: JsonRpcRequest,
    task_tracker: &Arc<Mutex<Vec<ActiveTask>>>,
) -> Option<JsonRpcResponse> {
    request.id.as_ref()?;
    info!("Handling task/complete");

    let params: TaskCompleteParams = match request.params {
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

    let mut tracker = task_tracker.lock().await;
    let index = tracker.iter().position(|t| t.id == params.task_id);

    match index {
        Some(i) => {
            let elapsed = tracker[i].created_at.elapsed();
            let elapsed_ms = elapsed.as_millis() as u64;
            tracker[i].status = TaskStatus::Completed;
            tracker[i].result = Some(params.result);

            info!("Task {} completed in {}ms", params.task_id, elapsed_ms);

            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(json!({
                    "taskId": params.task_id,
                    "status": "completed",
                    "elapsedMs": elapsed_ms
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
                message: "Task not found".to_string(),
                data: Some(
                    json!({"details": format!("No active task with id: {}", params.task_id)}),
                ),
            }),
        }),
    }
}

/// Handle task/cancel - cancel a task
pub async fn handle_task_cancel(
    request: JsonRpcRequest,
    task_tracker: &Arc<Mutex<Vec<ActiveTask>>>,
) -> Option<JsonRpcResponse> {
    request.id.as_ref()?;
    info!("Handling task/cancel");

    let params: TaskCancelParams = match request.params {
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

    let mut tracker = task_tracker.lock().await;
    let index = tracker.iter().position(|t| t.id == params.task_id);

    match index {
        Some(i) => {
            let _task = tracker.remove(i);
            info!(
                "Task {} cancelled: {}",
                params.task_id,
                params
                    .reason
                    .clone()
                    .unwrap_or_else(|| "No reason provided".to_string())
            );

            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(json!({
                    "taskId": params.task_id,
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
                message: "Task not found".to_string(),
                data: Some(
                    json!({"details": format!("No active task with id: {}", params.task_id)}),
                ),
            }),
        }),
    }
}

/// Handle task/list - list all active tasks
pub async fn handle_task_list(
    request: JsonRpcRequest,
    task_tracker: &Arc<Mutex<Vec<ActiveTask>>>,
) -> Option<JsonRpcResponse> {
    request.id.as_ref()?;
    info!("Handling task/list");

    let tracker = task_tracker.lock().await;
    let now = std::time::Instant::now();

    let tasks: Vec<_> = tracker
        .iter()
        .map(|t| {
            json!({
                "taskId": t.id,
                "name": t.name,
                "status": format!("{:?}", t.status).to_lowercase(),
                "progress": t.progress,
                "createdAtSecsAgo": now.duration_since(t.created_at).as_secs()
            })
        })
        .collect();

    let total = tasks.len();

    Some(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result: Some(json!({
            "tasks": tasks,
            "total": total
        })),
        error: None,
    })
}

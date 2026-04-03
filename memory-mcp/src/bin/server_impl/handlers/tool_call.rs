//! Tool call handler and dispatch logic

use super::super::tools::{
    handle_add_episode_relationship, handle_add_episode_step, handle_add_episode_tags,
    handle_advanced_pattern_analysis, handle_analyze_patterns, handle_bulk_episodes,
    handle_check_relationship_exists, handle_checkpoint_episode, handle_complete_episode,
    handle_configure_agentfs, handle_configure_embeddings, handle_create_episode,
    handle_delete_episode, handle_embedding_provider_status, handle_execute_code,
    handle_explain_pattern, handle_external_signal_status, handle_find_related_episodes,
    handle_generate_embedding, handle_get_dependency_graph, handle_get_episode,
    handle_get_episode_relationships, handle_get_episode_tags, handle_get_episode_timeline,
    handle_get_handoff_pack, handle_get_metrics, handle_get_recommendation_stats,
    handle_get_topological_order, handle_health_check, handle_quality_metrics, handle_query_memory,
    handle_query_semantic_memory, handle_recommend_patterns, handle_recommend_playbook,
    handle_record_recommendation_feedback, handle_record_recommendation_session,
    handle_remove_episode_relationship, handle_remove_episode_tags, handle_resume_from_handoff,
    handle_search_by_embedding, handle_search_episodes_by_tags, handle_search_patterns,
    handle_set_episode_tags, handle_test_agentfs_connection, handle_test_embeddings,
    handle_update_episode, handle_validate_no_cycles,
};
use super::super::types::{CallToolParams, CallToolResult, Content};
use do_memory_mcp::MemoryMCPServer;
use do_memory_mcp::jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

pub enum DispatchError {
    ToolNotFound(String),
    ToolUnavailable(String),
}

pub async fn dispatch_tool(
    server: &mut MemoryMCPServer,
    tool_name: &str,
    arguments: Option<serde_json::Value>,
) -> Result<Result<Vec<Content>, anyhow::Error>, DispatchError> {
    let result = match tool_name {
        "query_memory" => handle_query_memory(server, arguments).await,
        "execute_agent_code" => {
            let tools = server.list_tools().await;
            if tools.iter().any(|t| t.name == "execute_agent_code") {
                handle_execute_code(server, arguments).await
            } else {
                return Err(DispatchError::ToolUnavailable(
                    "execute_agent_code tool is not available due to WASM sandbox compilation issues".to_string(),
                ));
            }
        }
        "analyze_patterns" => handle_analyze_patterns(server, arguments).await,
        "advanced_pattern_analysis" => handle_advanced_pattern_analysis(server, arguments).await,
        "health_check" => handle_health_check(server, arguments).await,
        "get_metrics" => handle_get_metrics(server, arguments).await,
        "quality_metrics" => handle_quality_metrics(server, arguments).await,
        "configure_embeddings" => handle_configure_embeddings(server, arguments).await,
        "query_semantic_memory" => handle_query_semantic_memory(server, arguments).await,
        "test_embeddings" => handle_test_embeddings(server, arguments).await,
        "generate_embedding" => handle_generate_embedding(server, arguments).await,
        "search_by_embedding" => handle_search_by_embedding(server, arguments).await,
        "embedding_provider_status" => handle_embedding_provider_status(server, arguments).await,
        "search_patterns" => handle_search_patterns(server, arguments).await,
        "recommend_patterns" => handle_recommend_patterns(server, arguments).await,
        "bulk_episodes" => handle_bulk_episodes(server, arguments).await,
        "create_episode" => handle_create_episode(server, arguments).await,
        "add_episode_step" => handle_add_episode_step(server, arguments).await,
        "complete_episode" => handle_complete_episode(server, arguments).await,
        "get_episode" => handle_get_episode(server, arguments).await,
        "delete_episode" => handle_delete_episode(server, arguments).await,
        "update_episode" => handle_update_episode(server, arguments).await,
        "get_episode_timeline" => handle_get_episode_timeline(server, arguments).await,
        "add_episode_tags" => handle_add_episode_tags(server, arguments).await,
        "remove_episode_tags" => handle_remove_episode_tags(server, arguments).await,
        "set_episode_tags" => handle_set_episode_tags(server, arguments).await,
        "get_episode_tags" => handle_get_episode_tags(server, arguments).await,
        "search_episodes_by_tags" => handle_search_episodes_by_tags(server, arguments).await,
        "add_episode_relationship" => handle_add_episode_relationship(server, arguments).await,
        "remove_episode_relationship" => {
            handle_remove_episode_relationship(server, arguments).await
        }
        "get_episode_relationships" => handle_get_episode_relationships(server, arguments).await,
        "find_related_episodes" => handle_find_related_episodes(server, arguments).await,
        "check_relationship_exists" => handle_check_relationship_exists(server, arguments).await,
        "get_dependency_graph" => handle_get_dependency_graph(server, arguments).await,
        "validate_no_cycles" => handle_validate_no_cycles(server, arguments).await,
        "get_topological_order" => handle_get_topological_order(server, arguments).await,
        "recommend_playbook" => handle_recommend_playbook(server, arguments).await,
        "explain_pattern" => handle_explain_pattern(server, arguments).await,
        "record_recommendation_session" => {
            handle_record_recommendation_session(server, arguments).await
        }
        "record_recommendation_feedback" => {
            handle_record_recommendation_feedback(server, arguments).await
        }
        "get_recommendation_stats" => handle_get_recommendation_stats(server, arguments).await,
        "checkpoint_episode" => handle_checkpoint_episode(server, arguments).await,
        "get_handoff_pack" => handle_get_handoff_pack(server, arguments).await,
        "resume_from_handoff" => handle_resume_from_handoff(server, arguments).await,
        "configure_agentfs" => handle_configure_agentfs(server, arguments).await,
        "external_signal_status" => handle_external_signal_status(server, arguments).await,
        "test_agentfs_connection" => handle_test_agentfs_connection(server, arguments).await,
        _ => return Err(DispatchError::ToolNotFound(tool_name.to_string())),
    };
    Ok(result)
}

/// Handle tools/call request
pub async fn handle_call_tool(
    request: JsonRpcRequest,
    mcp_server: &Arc<Mutex<MemoryMCPServer>>,
) -> Option<JsonRpcResponse> {
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
    let result = match dispatch_tool(&mut server, &params.name, params.arguments).await {
        Ok(r) => r,
        Err(DispatchError::ToolNotFound(name)) => {
            return Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: "Tool not found".to_string(),
                    data: Some(json!({"tool": name})),
                }),
            });
        }
        Err(DispatchError::ToolUnavailable(details)) => {
            return Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32000,
                    message: "Tool execution failed".to_string(),
                    data: Some(json!({"details": details})),
                }),
            });
        }
    };

    match result {
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
    }
}

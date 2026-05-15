//! Tool call handler

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
        "generate_embedding" => handle_generate_embedding(&mut server, params.arguments).await,
        "search_by_embedding" => handle_search_by_embedding(&mut server, params.arguments).await,
        "embedding_provider_status" => {
            handle_embedding_provider_status(&mut server, params.arguments).await
        }
        "search_patterns" => handle_search_patterns(&mut server, params.arguments).await,
        "recommend_patterns" => handle_recommend_patterns(&mut server, params.arguments).await,
        "bulk_episodes" => handle_bulk_episodes(&mut server, params.arguments).await,
        "create_episode" => handle_create_episode(&mut server, params.arguments).await,
        "add_episode_step" => handle_add_episode_step(&mut server, params.arguments).await,
        "complete_episode" => handle_complete_episode(&mut server, params.arguments).await,
        "get_episode" => handle_get_episode(&mut server, params.arguments).await,
        "delete_episode" => handle_delete_episode(&mut server, params.arguments).await,
        "update_episode" => handle_update_episode(&mut server, params.arguments).await,
        "get_episode_timeline" => handle_get_episode_timeline(&mut server, params.arguments).await,
        "add_episode_tags" => handle_add_episode_tags(&mut server, params.arguments).await,
        "remove_episode_tags" => handle_remove_episode_tags(&mut server, params.arguments).await,
        "set_episode_tags" => handle_set_episode_tags(&mut server, params.arguments).await,
        "get_episode_tags" => handle_get_episode_tags(&mut server, params.arguments).await,
        "search_episodes_by_tags" => {
            handle_search_episodes_by_tags(&mut server, params.arguments).await
        }
        "add_episode_relationship" => {
            handle_add_episode_relationship(&mut server, params.arguments).await
        }
        "remove_episode_relationship" => {
            handle_remove_episode_relationship(&mut server, params.arguments).await
        }
        "get_episode_relationships" => {
            handle_get_episode_relationships(&mut server, params.arguments).await
        }
        "find_related_episodes" => {
            handle_find_related_episodes(&mut server, params.arguments).await
        }
        "check_relationship_exists" => {
            handle_check_relationship_exists(&mut server, params.arguments).await
        }
        "get_dependency_graph" => handle_get_dependency_graph(&mut server, params.arguments).await,
        "validate_no_cycles" => handle_validate_no_cycles(&mut server, params.arguments).await,
        "get_topological_order" => {
            handle_get_topological_order(&mut server, params.arguments).await
        }
        // ADR-044 Feature 1: Playbook tools
        "recommend_playbook" => handle_recommend_playbook(&mut server, params.arguments).await,
        "explain_pattern" => handle_explain_pattern(&mut server, params.arguments).await,
        // ADR-044 Feature 2: Recommendation feedback tools
        "record_recommendation_session" => {
            handle_record_recommendation_session(&mut server, params.arguments).await
        }
        "record_recommendation_feedback" => {
            handle_record_recommendation_feedback(&mut server, params.arguments).await
        }
        "get_recommendation_stats" => {
            handle_get_recommendation_stats(&mut server, params.arguments).await
        }
        // ADR-044 Feature 3: Checkpoint tools
        "checkpoint_episode" => handle_checkpoint_episode(&mut server, params.arguments).await,
        "get_handoff_pack" => handle_get_handoff_pack(&mut server, params.arguments).await,
        "resume_from_handoff" => handle_resume_from_handoff(&mut server, params.arguments).await,
        // External signal provider tools
        "configure_agentfs" => handle_configure_agentfs(&mut server, params.arguments).await,
        "external_signal_status" => {
            handle_external_signal_status(&mut server, params.arguments).await
        }
        "test_agentfs_connection" => {
            handle_test_agentfs_connection(&mut server, params.arguments).await
        }
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

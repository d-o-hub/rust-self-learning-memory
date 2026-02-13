//! Memory tool handlers with audit logging
//!
//! This module contains individual tool handler functions for all MCP tools.
//! All security-relevant operations are logged to the audit logger.
//!
//! ## Audit Logging
//!
//! The following operations are logged:
//! - Episode creation/modification/deletion
//! - Relationship changes
//! - Configuration changes
//! - Authentication events
//! - Rate limit violations

use super::types::Content;
use memory_mcp::mcp::tools::embeddings::{ConfigureEmbeddingsInput, QuerySemanticMemoryInput};
use memory_mcp::mcp::tools::pattern_search::{RecommendPatternsInput, SearchPatternsInput};
use memory_mcp::mcp::tools::quality_metrics::QualityMetricsInput;
use memory_mcp::server::rate_limiter::OperationType;
use memory_mcp::ExecutionContext;
use memory_mcp::MemoryMCPServer;
use serde_json::{json, Value};

/// Extract client ID from arguments or use default
fn get_client_id(args: &Value) -> String {
    args.get("client_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "anonymous".to_string())
}

/// Get length from a JSON Value (array length or 0)
fn json_value_len(value: &Value) -> usize {
    value.as_array().map(|a| a.len()).unwrap_or(0)
}

/// Handle query_memory tool
pub async fn handle_query_memory(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
    let client_id = get_client_id(&args);
    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let domain = args
        .get("domain")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let task_type = args
        .get("task_type")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
    let sort = args
        .get("sort")
        .and_then(|v| v.as_str())
        .unwrap_or("relevance")
        .to_string();
    let fields = args.get("fields").and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
    });

    let result = server
        .query_memory(query.clone(), domain, task_type, limit, sort, fields)
        .await;

    // Audit log the operation
    let result_count = result.as_ref().map(json_value_len).unwrap_or(0);
    server
        .audit_logger()
        .log_memory_query(&client_id, &query, result_count, result.is_ok())
        .await;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }];

    Ok(content)
}

/// Handle execute_agent_code tool
pub async fn handle_execute_code(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
    let client_id = get_client_id(&args);
    let code = args
        .get("code")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'code' parameter"))?
        .to_string();

    let context_obj = args
        .get("context")
        .ok_or_else(|| anyhow::anyhow!("Missing 'context' parameter"))?;

    let task = context_obj
        .get("task")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'task' in context"))?
        .to_string();

    let input = context_obj.get("input").cloned().unwrap_or(json!({}));

    let context = ExecutionContext::new(task, input);

    // Check if WASM sandbox is available by attempting a simple test
    // If it fails, return a proper error instead of crashing
    match server
        .execute_agent_code(
            "console.log('test');".to_string(),
            ExecutionContext::new("test".to_string(), json!({})),
        )
        .await
    {
        Ok(_) => {
            // WASM sandbox is working, proceed with actual execution
            let start_time = std::time::Instant::now();
            let result = server.execute_agent_code(code, context).await;
            let execution_time_ms = start_time.elapsed().as_millis() as u64;

            // Audit log the execution
            let success = result.is_ok();
            let error = result.as_ref().err().map(|e| e.to_string());
            server
                .audit_logger()
                .log_code_execution(
                    &client_id,
                    "wasmtime",
                    execution_time_ms,
                    success,
                    error.as_deref(),
                )
                .await;

            let content = vec![Content::Text {
                text: serde_json::to_string_pretty(&result?)?,
            }];
            Ok(content)
        }
        Err(e) => {
            // WASM sandbox is not available, return proper error
            server
                .audit_logger()
                .log_code_execution(&client_id, "wasmtime", 0, false, Some("WASM unavailable"))
                .await;
            Err(anyhow::anyhow!(
                "Code execution is currently unavailable due to WASM sandbox compilation issues. Error: {}",
                e
            ))
        }
    }
}

/// Handle analyze_patterns tool
pub async fn handle_analyze_patterns(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
    let client_id = get_client_id(&args);
    let task_type = args
        .get("task_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'task_type' parameter"))?
        .to_string();
    let min_success_rate = args
        .get("min_success_rate")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.7) as f32;
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;

    let result = server
        .analyze_patterns(task_type.clone(), min_success_rate, limit, None)
        .await;

    // Audit log the operation
    let result_count = result.as_ref().map(json_value_len).unwrap_or(0);
    server
        .audit_logger()
        .log_pattern_analysis(&client_id, &task_type, result_count, result.is_ok())
        .await;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }];

    Ok(content)
}

/// Handle advanced_pattern_analysis tool
pub async fn handle_advanced_pattern_analysis(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
    let client_id = get_client_id(&args);

    // Parse analysis type
    let analysis_type_str = args
        .get("analysis_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'analysis_type' parameter"))?;

    let analysis_type = match analysis_type_str {
        "statistical" => {
            memory_mcp::mcp::tools::advanced_pattern_analysis::AnalysisType::Statistical
        }
        "predictive" => memory_mcp::mcp::tools::advanced_pattern_analysis::AnalysisType::Predictive,
        "comprehensive" => {
            memory_mcp::mcp::tools::advanced_pattern_analysis::AnalysisType::Comprehensive
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid analysis_type: {}",
                analysis_type_str
            ))
        }
    };

    // Parse time series data
    let time_series_data_value = args
        .get("time_series_data")
        .ok_or_else(|| anyhow::anyhow!("Missing 'time_series_data' parameter"))?;

    let time_series_data: std::collections::HashMap<String, Vec<f64>> =
        serde_json::from_value(time_series_data_value.clone())?;

    // Parse optional config
    let config = args
        .get("config")
        .and_then(|c| serde_json::from_value(c.clone()).ok());

    let input = memory_mcp::mcp::tools::advanced_pattern_analysis::AdvancedPatternAnalysisInput {
        analysis_type: analysis_type.clone(),
        time_series_data,
        config,
    };

    let result = server.execute_advanced_pattern_analysis(input).await;

    // Audit log the operation
    let success = result.is_ok();
    server
        .audit_logger()
        .log_advanced_pattern_analysis(&client_id, analysis_type_str, success)
        .await;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }];

    Ok(content)
}

/// Handle health_check tool
pub async fn handle_health_check(
    server: &mut MemoryMCPServer,
    _arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let result = server.health_check().await?;
    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];

    Ok(content)
}

/// Handle get_metrics tool
pub async fn handle_get_metrics(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
    let metric_type = args
        .get("metric_type")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let result = server.get_metrics(metric_type).await?;
    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];

    Ok(content)
}

/// Handle quality_metrics tool
pub async fn handle_quality_metrics(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
    let input: QualityMetricsInput = serde_json::from_value(args)?;
    let result = server.execute_quality_metrics(input).await?;
    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];
    Ok(content)
}

/// Handle configure_embeddings tool
pub async fn handle_configure_embeddings(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
    let client_id = get_client_id(&args);
    let input: ConfigureEmbeddingsInput = serde_json::from_value(args)?;

    let provider = input.provider.clone();
    let model = input.model.clone();

    let result = server.execute_configure_embeddings(input).await;

    // Audit log the configuration change
    let success = result.is_ok();
    server
        .audit_logger()
        .log_embedding_config(
            &client_id,
            &format!("{:?}", provider),
            model.as_deref(),
            success,
        )
        .await;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }];
    Ok(content)
}

/// Handle query_semantic_memory tool
pub async fn handle_query_semantic_memory(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
    let client_id = get_client_id(&args);
    let input: QuerySemanticMemoryInput = serde_json::from_value(args)?;
    let query = input.query.clone();

    let result = server.execute_query_semantic_memory(input).await;

    // Audit log the operation
    let result_count = result.as_ref().map(json_value_len).unwrap_or(0);
    server
        .audit_logger()
        .log_semantic_query(&client_id, &query, result_count, result.is_ok())
        .await;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }];
    Ok(content)
}

/// Handle test_embeddings tool
pub async fn handle_test_embeddings(
    server: &mut MemoryMCPServer,
    _arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let result = server.execute_test_embeddings().await?;
    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];
    Ok(content)
}

/// Handle search_patterns tool
pub async fn handle_search_patterns(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let client_id = get_client_id(&args);
    let input: SearchPatternsInput = serde_json::from_value(args)?;
    let domain = input.domain.clone();

    // Access memory through the server's memory field
    let result = server.execute_search_patterns(input).await;

    // Audit log the operation
    let result_count = result.as_ref().map(json_value_len).unwrap_or(0);
    server
        .audit_logger()
        .log_pattern_search(&client_id, &domain, result_count, result.is_ok())
        .await;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }];
    Ok(content)
}

/// Handle recommend_patterns tool
pub async fn handle_recommend_patterns(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let client_id = get_client_id(&args);
    let input: RecommendPatternsInput = serde_json::from_value(args)?;
    let domain = input.domain.clone();

    // Access memory through the server's memory field
    let result = server.execute_recommend_patterns(input).await;

    // Audit log the operation
    let recommendation_count = result.as_ref().map(json_value_len).unwrap_or(0);
    server
        .audit_logger()
        .log_recommend_patterns(&client_id, &domain, recommendation_count, result.is_ok())
        .await;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }];
    Ok(content)
}

/// Handle bulk_episodes tool
pub async fn handle_bulk_episodes(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use uuid::Uuid;

    let args: Value = arguments.unwrap_or(json!({}));
    let client_id = get_client_id(&args);

    // Parse episode_ids as a comma-separated string or array
    let episode_ids_value = args
        .get("episode_ids")
        .ok_or_else(|| anyhow::anyhow!("Missing 'episode_ids' parameter"))?;

    let episode_ids: Vec<Uuid> = match episode_ids_value {
        Value::String(s) => {
            // Parse comma-separated UUIDs
            s.split(',')
                .map(|id| {
                    let id = id.trim();
                    Uuid::parse_str(id).map_err(|_| anyhow::anyhow!("Invalid UUID: {}", id))
                })
                .collect::<anyhow::Result<Vec<_>>>()?
        }
        Value::Array(arr) => {
            // Parse array of UUID strings
            arr.iter()
                .map(|v| {
                    let id = v
                        .as_str()
                        .ok_or_else(|| anyhow::anyhow!("Expected string in episode_ids array"))?;
                    Uuid::parse_str(id).map_err(|_| anyhow::anyhow!("Invalid UUID: {}", id))
                })
                .collect::<anyhow::Result<Vec<_>>>()?
        }
        _ => {
            return Err(anyhow::anyhow!(
                "episode_ids must be a string (comma-separated) or array of UUIDs"
            ))
        }
    };

    let result = server.get_episodes_by_ids(&episode_ids).await;

    // Audit log the operation
    let episode_count = result.as_ref().map(|r| r.len()).unwrap_or(0);
    let success = result.is_ok();
    server
        .audit_logger()
        .log_bulk_episodes(&client_id, episode_count, success)
        .await;

    let mut episodes_json = Vec::with_capacity(result.as_ref().map(|r| r.len()).unwrap_or(0));
    for ep in result?.iter() {
        episodes_json.push(
            serde_json::to_value(ep)
                .map_err(|e| anyhow::anyhow!("Failed to serialize episode: {}", e))?,
        );
    }

    #[derive(serde::Serialize)]
    struct BulkEpisodeResult {
        requested_count: usize,
        found_count: usize,
        missing_count: usize,
        episodes: Vec<serde_json::Value>,
    }

    let bulk_result = BulkEpisodeResult {
        requested_count: episode_ids.len(),
        found_count: episodes_json.len(),
        missing_count: episode_ids.len() - episodes_json.len(),
        episodes: episodes_json,
    };

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&bulk_result)?,
    }];

    Ok(content)
}

/// Handle create_episode tool
pub async fn handle_create_episode(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let client_id = get_client_id(&args);
    let task_description = args
        .get("task_description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let result = server.create_episode_tool(args).await;

    // Audit log the operation
    let episode_id = result
        .as_ref()
        .ok()
        .and_then(|r| r.get("episode_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let success = result.is_ok();
    let error = result.as_ref().err().map(|e| e.to_string());
    server
        .audit_logger()
        .log_episode_creation(
            &client_id,
            episode_id,
            &task_description,
            success,
            error.as_deref(),
        )
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

/// Handle add_episode_step tool
pub async fn handle_add_episode_step(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let client_id = get_client_id(&args);
    let episode_id = args
        .get("episode_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let step_number = args
        .get("step_number")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let tool = args
        .get("tool")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let result = server.add_episode_step_tool(args).await;

    // Audit log the operation
    let success = result.is_ok();
    server
        .audit_logger()
        .log_episode_step(&client_id, &episode_id, step_number, &tool, success)
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

/// Handle complete_episode tool
pub async fn handle_complete_episode(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let client_id = get_client_id(&args);
    let episode_id = args
        .get("episode_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let outcome = args
        .get("outcome_type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let result = server.complete_episode_tool(args).await;

    // Audit log the operation
    let success = result.is_ok();
    server
        .audit_logger()
        .log_episode_completion(&client_id, &episode_id, &outcome, success)
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

/// Handle get_episode tool
pub async fn handle_get_episode(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let result = server.get_episode_tool(args).await?;
    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }])
}

/// Handle delete_episode tool
pub async fn handle_delete_episode(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let client_id = get_client_id(&args);
    let episode_id = args
        .get("episode_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let result = server.delete_episode_tool(args).await;

    // Audit log the operation
    let success = result.is_ok();
    let error = result.as_ref().err().map(|e| e.to_string());
    server
        .audit_logger()
        .log_episode_deletion(&client_id, &episode_id, success, error.as_deref())
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

/// Handle update_episode tool
pub async fn handle_update_episode(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let client_id = get_client_id(&args);
    let episode_id = args
        .get("episode_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let result = server.update_episode_tool(args).await;

    // Audit log the operation
    let success = result.is_ok();
    let error = result.as_ref().err().map(|e| e.to_string());
    server
        .audit_logger()
        .log_episode_modification(&client_id, &episode_id, "update", success, error.as_deref())
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

/// Handle get_episode_timeline tool
pub async fn handle_get_episode_timeline(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let result = server.get_episode_timeline_tool(args).await?;
    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }])
}

// TODO: Re-enable batch tools when batch module is fixed
// See: memory-mcp/src/server/tools/mod.rs:7-8
// Commented out: handle_batch_query_episodes, handle_batch_pattern_analysis, handle_batch_compare_episodes

/// Handle add_episode_tags tool
pub async fn handle_add_episode_tags(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::mcp::tools::episode_tags::{AddEpisodeTagsInput, EpisodeTagTools};

    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let client_id = get_client_id(&args);
    let input: AddEpisodeTagsInput = serde_json::from_value(args)?;
    let episode_id = input.episode_id.to_string();
    let tags: Vec<String> = input.tags.clone();

    let tools = EpisodeTagTools::new(server.memory());
    let result = tools.add_tags(input).await;

    // Audit log the operation
    let success = result.is_ok();
    server
        .audit_logger()
        .log_add_tags(&client_id, &episode_id, &tags, success)
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

/// Handle remove_episode_tags tool
pub async fn handle_remove_episode_tags(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::mcp::tools::episode_tags::{EpisodeTagTools, RemoveEpisodeTagsInput};

    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let client_id = get_client_id(&args);
    let input: RemoveEpisodeTagsInput = serde_json::from_value(args)?;
    let episode_id = input.episode_id.to_string();
    let tags: Vec<String> = input.tags.clone();

    let tools = EpisodeTagTools::new(server.memory());
    let result = tools.remove_tags(input).await;

    // Audit log the operation
    let success = result.is_ok();
    server
        .audit_logger()
        .log_remove_tags(&client_id, &episode_id, &tags, success)
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

/// Handle set_episode_tags tool
pub async fn handle_set_episode_tags(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::mcp::tools::episode_tags::{EpisodeTagTools, SetEpisodeTagsInput};

    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let client_id = get_client_id(&args);
    let input: SetEpisodeTagsInput = serde_json::from_value(args)?;
    let episode_id = input.episode_id.to_string();
    let tags: Vec<String> = input.tags.clone();

    let tools = EpisodeTagTools::new(server.memory());
    let result = tools.set_tags(input).await;

    // Audit log the operation
    let success = result.is_ok();
    server
        .audit_logger()
        .log_set_tags(&client_id, &episode_id, &tags, success)
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

/// Handle get_episode_tags tool
pub async fn handle_get_episode_tags(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::mcp::tools::episode_tags::{EpisodeTagTools, GetEpisodeTagsInput};

    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let input: GetEpisodeTagsInput = serde_json::from_value(args)?;

    let tools = EpisodeTagTools::new(server.memory());
    let result = tools.get_tags(input).await?;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }])
}

/// Handle search_episodes_by_tags tool
pub async fn handle_search_episodes_by_tags(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::mcp::tools::episode_tags::{EpisodeTagTools, SearchEpisodesByTagsInput};

    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let client_id = get_client_id(&args);
    let input: SearchEpisodesByTagsInput = serde_json::from_value(args)?;
    let tags: Vec<String> = input.tags.clone();

    let tools = EpisodeTagTools::new(server.memory());
    let result = tools.search_by_tags(input).await;

    // Audit log the operation
    let result_count = result.as_ref().map(|r| r.count).unwrap_or(0);
    server
        .audit_logger()
        .log_search_tags(&client_id, &tags, result_count)
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

/// Handle add_episode_relationship tool
pub async fn handle_add_episode_relationship(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::mcp::tools::episode_relationships::{
        AddEpisodeRelationshipInput, EpisodeRelationshipTools,
    };

    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    // Check rate limit (WRITE operation)
    let client_id = server.client_id_from_args(&args);
    let rate_limit_result = server.check_rate_limit(&client_id, OperationType::Write);
    if !rate_limit_result.allowed {
        // Log rate limit violation
        let client_id_str = get_client_id(&args);
        server
            .audit_logger()
            .log_rate_limit_violation(
                &client_id_str,
                "add_episode_relationship",
                rate_limit_result.limit,
                rate_limit_result.remaining,
            )
            .await;

        // Return rate limited error
        return Err(anyhow::anyhow!(
            "Rate limit exceeded. Retry after {} seconds.",
            rate_limit_result
                .retry_after
                .map(|d| d.as_secs())
                .unwrap_or(60)
        ));
    }

    let client_id_str = get_client_id(&args);
    let input: AddEpisodeRelationshipInput = serde_json::from_value(args)?;
    let from_id = input.from_episode_id.clone();
    let to_id = input.to_episode_id.clone();
    let rel_type = input.relationship_type.clone();

    let tools = EpisodeRelationshipTools::new(server.memory());
    let result = tools.add_relationship(input).await;

    // Audit log the operation
    let success = result.is_ok();
    let relationship_id = result
        .as_ref()
        .map(|r| r.relationship_id.clone())
        .unwrap_or_default();
    server
        .audit_logger()
        .log_add_relationship(
            &client_id_str,
            &from_id,
            &to_id,
            &rel_type,
            &relationship_id,
            success,
        )
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

/// Handle remove_episode_relationship tool
pub async fn handle_remove_episode_relationship(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::mcp::tools::episode_relationships::{
        EpisodeRelationshipTools, RemoveEpisodeRelationshipInput,
    };

    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    // Check rate limit (WRITE operation)
    let client_id = server.client_id_from_args(&args);
    let rate_limit_result = server.check_rate_limit(&client_id, OperationType::Write);
    if !rate_limit_result.allowed {
        let client_id_str = get_client_id(&args);
        server
            .audit_logger()
            .log_rate_limit_violation(
                &client_id_str,
                "remove_episode_relationship",
                rate_limit_result.limit,
                rate_limit_result.remaining,
            )
            .await;
        return Err(anyhow::anyhow!(
            "Rate limit exceeded. Retry after {} seconds.",
            rate_limit_result
                .retry_after
                .map(|d| d.as_secs())
                .unwrap_or(60)
        ));
    }

    let client_id_str = get_client_id(&args);
    let input: RemoveEpisodeRelationshipInput = serde_json::from_value(args)?;
    let relationship_id = input.relationship_id.clone();

    let tools = EpisodeRelationshipTools::new(server.memory());
    let result = tools.remove_relationship(input).await;

    // Audit log the operation
    let success = result.is_ok();
    server
        .audit_logger()
        .log_remove_relationship(&client_id_str, &relationship_id, success)
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

/// Handle get_episode_relationships tool (Tool 3)
///
/// Get relationships for an episode with direction filtering (outgoing/incoming/both)
pub async fn handle_get_episode_relationships(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::mcp::tools::episode_relationships::{
        EpisodeRelationshipTools, GetEpisodeRelationshipsInput,
    };

    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    // Check rate limit (READ operation)
    let client_id = server.client_id_from_args(&args);
    let rate_limit_result = server.check_rate_limit(&client_id, OperationType::Read);
    if !rate_limit_result.allowed {
        let client_id_str = get_client_id(&args);
        server
            .audit_logger()
            .log_rate_limit_violation(
                &client_id_str,
                "get_episode_relationships",
                rate_limit_result.limit,
                rate_limit_result.remaining,
            )
            .await;
        return Err(anyhow::anyhow!(
            "Rate limit exceeded. Retry after {} seconds.",
            rate_limit_result
                .retry_after
                .map(|d| d.as_secs())
                .unwrap_or(60)
        ));
    }

    let client_id_str = get_client_id(&args);
    let input: GetEpisodeRelationshipsInput = serde_json::from_value(args)?;
    let episode_id = input.episode_id.clone();

    let tools = EpisodeRelationshipTools::new(server.memory());
    let result = tools.get_relationships(input).await;

    // Audit log the operation
    let success = result.is_ok();
    let total_count = result.as_ref().map(|r| r.total_count).unwrap_or(0);
    server
        .audit_logger()
        .log_get_relationships(&client_id_str, &episode_id, total_count, success)
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

/// Handle find_related_episodes tool (Tool 4)
///
/// Find episodes related to a given episode with optional type filtering
pub async fn handle_find_related_episodes(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::mcp::tools::episode_relationships::{
        EpisodeRelationshipTools, FindRelatedEpisodesInput,
    };

    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    // Check rate limit (READ operation)
    let client_id = server.client_id_from_args(&args);
    let rate_limit_result = server.check_rate_limit(&client_id, OperationType::Read);
    if !rate_limit_result.allowed {
        let client_id_str = get_client_id(&args);
        server
            .audit_logger()
            .log_rate_limit_violation(
                &client_id_str,
                "find_related_episodes",
                rate_limit_result.limit,
                rate_limit_result.remaining,
            )
            .await;
        return Err(anyhow::anyhow!(
            "Rate limit exceeded. Retry after {} seconds.",
            rate_limit_result
                .retry_after
                .map(|d| d.as_secs())
                .unwrap_or(60)
        ));
    }

    let client_id_str = get_client_id(&args);
    let input: FindRelatedEpisodesInput = serde_json::from_value(args)?;
    let episode_id = input.episode_id.clone();

    let tools = EpisodeRelationshipTools::new(server.memory());
    let result = tools.find_related(input).await;

    // Audit log the operation
    let success = result.is_ok();
    let count = result.as_ref().map(|r| r.count).unwrap_or(0);
    server
        .audit_logger()
        .log_find_related(&client_id_str, &episode_id, count, success)
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

/// Handle check_relationship_exists tool (Tool 5)
///
/// Check if a specific relationship exists between two episodes
pub async fn handle_check_relationship_exists(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::mcp::tools::episode_relationships::{
        CheckRelationshipExistsInput, EpisodeRelationshipTools,
    };

    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    // Check rate limit (READ operation)
    let client_id = server.client_id_from_args(&args);
    let rate_limit_result = server.check_rate_limit(&client_id, OperationType::Read);
    if !rate_limit_result.allowed {
        let client_id_str = get_client_id(&args);
        server
            .audit_logger()
            .log_rate_limit_violation(
                &client_id_str,
                "check_relationship_exists",
                rate_limit_result.limit,
                rate_limit_result.remaining,
            )
            .await;
        return Err(anyhow::anyhow!(
            "Rate limit exceeded. Retry after {} seconds.",
            rate_limit_result
                .retry_after
                .map(|d| d.as_secs())
                .unwrap_or(60)
        ));
    }

    let client_id_str = get_client_id(&args);
    let input: CheckRelationshipExistsInput = serde_json::from_value(args)?;
    let from_id = input.from_episode_id.clone();
    let to_id = input.to_episode_id.clone();
    let rel_type = input.relationship_type.clone();

    let tools = EpisodeRelationshipTools::new(server.memory());
    let result = tools.check_exists(input).await;

    // Audit log the operation
    let success = result.is_ok();
    let exists = result.as_ref().map(|r| r.exists).unwrap_or(false);
    server
        .audit_logger()
        .log_check_relationship(&client_id_str, &from_id, &to_id, &rel_type, exists, success)
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

/// Handle get_dependency_graph tool (Tool 6)
///
/// Get relationship graph for visualization in JSON or DOT format
pub async fn handle_get_dependency_graph(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::mcp::tools::episode_relationships::{
        DependencyGraphInput, EpisodeRelationshipTools,
    };

    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    // Check rate limit (READ operation)
    let client_id = server.client_id_from_args(&args);
    let rate_limit_result = server.check_rate_limit(&client_id, OperationType::Read);
    if !rate_limit_result.allowed {
        let client_id_str = get_client_id(&args);
        server
            .audit_logger()
            .log_rate_limit_violation(
                &client_id_str,
                "get_dependency_graph",
                rate_limit_result.limit,
                rate_limit_result.remaining,
            )
            .await;
        return Err(anyhow::anyhow!(
            "Rate limit exceeded. Retry after {} seconds.",
            rate_limit_result
                .retry_after
                .map(|d| d.as_secs())
                .unwrap_or(60)
        ));
    }

    let client_id_str = get_client_id(&args);
    let input: DependencyGraphInput = serde_json::from_value(args)?;
    let episode_id = input.episode_id.clone();

    let tools = EpisodeRelationshipTools::new(server.memory());
    let result = tools.get_dependency_graph(input).await;

    // Audit log the operation
    let success = result.is_ok();
    let node_count = result.as_ref().map(|r| r.node_count).unwrap_or(0);
    let edge_count = result.as_ref().map(|r| r.edge_count).unwrap_or(0);
    server
        .audit_logger()
        .log_dependency_graph(&client_id_str, &episode_id, node_count, edge_count, success)
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

/// Handle validate_no_cycles tool (Tool 7)
///
/// Validate that adding a relationship would not create a cycle
pub async fn handle_validate_no_cycles(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::mcp::tools::episode_relationships::{
        EpisodeRelationshipTools, ValidateNoCyclesInput,
    };

    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    // Check rate limit (READ operation - validates before potential write)
    let client_id = server.client_id_from_args(&args);
    let rate_limit_result = server.check_rate_limit(&client_id, OperationType::Read);
    if !rate_limit_result.allowed {
        let client_id_str = get_client_id(&args);
        server
            .audit_logger()
            .log_rate_limit_violation(
                &client_id_str,
                "validate_no_cycles",
                rate_limit_result.limit,
                rate_limit_result.remaining,
            )
            .await;
        return Err(anyhow::anyhow!(
            "Rate limit exceeded. Retry after {} seconds.",
            rate_limit_result
                .retry_after
                .map(|d| d.as_secs())
                .unwrap_or(60)
        ));
    }

    let client_id_str = get_client_id(&args);
    let input: ValidateNoCyclesInput = serde_json::from_value(args)?;
    let from_id = input.from_episode_id.clone();
    let to_id = input.to_episode_id.clone();
    let rel_type = input.relationship_type.clone();

    let tools = EpisodeRelationshipTools::new(server.memory());
    let result = tools.validate_no_cycles(input).await;

    // Audit log the operation
    let would_create_cycle = result
        .as_ref()
        .map(|r| r.would_create_cycle)
        .unwrap_or(false);
    server
        .audit_logger()
        .log_validate_cycles(
            &client_id_str,
            &from_id,
            &to_id,
            &rel_type,
            would_create_cycle,
            result.is_ok(),
        )
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

/// Handle get_topological_order tool (Tool 8)
///
/// Get topological ordering of episodes based on dependency relationships
pub async fn handle_get_topological_order(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::mcp::tools::episode_relationships::{
        EpisodeRelationshipTools, GetTopologicalOrderInput,
    };

    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    // Check rate limit (READ operation)
    let client_id = server.client_id_from_args(&args);
    let rate_limit_result = server.check_rate_limit(&client_id, OperationType::Read);
    if !rate_limit_result.allowed {
        let client_id_str = get_client_id(&args);
        server
            .audit_logger()
            .log_rate_limit_violation(
                &client_id_str,
                "get_topological_order",
                rate_limit_result.limit,
                rate_limit_result.remaining,
            )
            .await;
        return Err(anyhow::anyhow!(
            "Rate limit exceeded. Retry after {} seconds.",
            rate_limit_result
                .retry_after
                .map(|d| d.as_secs())
                .unwrap_or(60)
        ));
    }

    let client_id_str = get_client_id(&args);
    let input: GetTopologicalOrderInput = serde_json::from_value(args)?;
    let episode_count = input.episode_ids.len();

    let tools = EpisodeRelationshipTools::new(server.memory());
    let result = tools.get_topological_order(input).await;

    // Audit log the operation
    let has_cycles = result.as_ref().map(|r| r.has_cycles).unwrap_or(false);
    let output_count = result.as_ref().map(|r| r.count).unwrap_or(0);
    server
        .audit_logger()
        .log_topological_order(
            &client_id_str,
            episode_count,
            output_count,
            has_cycles,
            result.is_ok(),
        )
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

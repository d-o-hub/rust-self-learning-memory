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
use memory_mcp::ExecutionContext;
use memory_mcp::MemoryMCPServer;
use memory_mcp::mcp::tools::embeddings::{ConfigureEmbeddingsInput, QuerySemanticMemoryInput};
use memory_mcp::mcp::tools::pattern_search::{RecommendPatternsInput, SearchPatternsInput};
use memory_mcp::mcp::tools::quality_metrics::QualityMetricsInput;
use memory_mcp::server::rate_limiter::OperationType;
use serde_json::{Value, json};

mod episode_handlers;
mod relationship_handlers;
mod tag_handlers;

pub use episode_handlers::*;
pub use relationship_handlers::*;
pub use tag_handlers::*;

/// Extract client ID from arguments or use default
pub(super) fn get_client_id(args: &Value) -> String {
    args.get("client_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "anonymous".to_string())
}

/// Get length from a JSON Value (array length or 0)
pub(super) fn json_value_len(value: &Value) -> usize {
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
            ));
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

// Episode, tag, and relationship handlers are implemented in submodules.

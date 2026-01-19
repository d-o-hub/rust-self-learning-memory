//! Memory tool handlers
//!
//! This module contains handlers for all memory-related tools:
//! - handle_query_memory: Query memories
//! - handle_execute_code: Execute agent code (WASM)
//! - handle_analyze_patterns: Analyze patterns
//! - handle_advanced_pattern_analysis: Advanced pattern analysis
//! - handle_health_check: Health check
//! - handle_get_metrics: Get metrics
//! - handle_quality_metrics: Quality metrics
//! - handle_configure_embeddings: Configure embedding provider
//! - handle_query_semantic_memory: Semantic memory search
//! - handle_test_embeddings: Test embedding provider

use super::types::Content;
use memory_mcp::mcp::tools::embeddings::{ConfigureEmbeddingsInput, QuerySemanticMemoryInput};
use memory_mcp::mcp::tools::pattern_search::{RecommendPatternsInput, SearchPatternsInput};
use memory_mcp::mcp::tools::quality_metrics::QualityMetricsInput;
use memory_mcp::ExecutionContext;
use memory_mcp::MemoryMCPServer;
use serde_json::{json, Value};

/// Handle query_memory tool
pub async fn handle_query_memory(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
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

    let result = server.query_memory(query, domain, task_type, limit).await?;
    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];

    Ok(content)
}

/// Handle execute_agent_code tool
pub async fn handle_execute_code(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
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
            let result = server.execute_agent_code(code, context).await?;
            let content = vec![Content::Text {
                text: serde_json::to_string_pretty(&result)?,
            }];
            Ok(content)
        }
        Err(e) => {
            // WASM sandbox is not available, return proper error
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
        .analyze_patterns(task_type, min_success_rate, limit)
        .await?;
    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];

    Ok(content)
}

/// Handle advanced_pattern_analysis tool
pub async fn handle_advanced_pattern_analysis(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));

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
        analysis_type,
        time_series_data,
        config,
    };

    let result = server.execute_advanced_pattern_analysis(input).await?;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
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
    let input: ConfigureEmbeddingsInput = serde_json::from_value(args)?;
    let result = server.execute_configure_embeddings(input).await?;
    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];
    Ok(content)
}

/// Handle query_semantic_memory tool
pub async fn handle_query_semantic_memory(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
    let input: QuerySemanticMemoryInput = serde_json::from_value(args)?;
    let result = server.execute_query_semantic_memory(input).await?;
    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
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
    let input: SearchPatternsInput = serde_json::from_value(args)?;

    // Access memory through the server's memory field
    let result = server.execute_search_patterns(input).await?;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];
    Ok(content)
}

/// Handle recommend_patterns tool
pub async fn handle_recommend_patterns(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let input: RecommendPatternsInput = serde_json::from_value(args)?;

    // Access memory through the server's memory field
    let result = server.execute_recommend_patterns(input).await?;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
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

    let result = server.get_episodes_by_ids(&episode_ids).await?;

    #[derive(serde::Serialize)]
    struct BulkEpisodeResult {
        requested_count: usize,
        found_count: usize,
        missing_count: usize,
        episodes: Vec<serde_json::Value>,
    }

    let mut episodes_json = Vec::with_capacity(result.len());
    for ep in result.iter() {
        episodes_json.push(
            serde_json::to_value(ep)
                .map_err(|e| anyhow::anyhow!("Failed to serialize episode: {}", e))?,
        );
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

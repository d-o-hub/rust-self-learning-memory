//! Memory and pattern handler functions
//!
//! This module contains handlers for memory queries, pattern analysis,
//! and related operations.

use super::{Content, MemoryMCPServer, Value, get_client_id, json_value_len};
use do_memory_mcp::mcp::tools::embeddings::{
    ConfigureEmbeddingsInput, EmbeddingProviderStatusInput, GenerateEmbeddingInput,
    QuerySemanticMemoryInput, SearchByEmbeddingInput,
};
use do_memory_mcp::mcp::tools::quality_metrics::QualityMetricsInput;
use serde_json::json;

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
///
/// Note: WASM sandbox has been removed in v0.1.29. This tool is deprecated.
pub async fn handle_execute_code(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
    let client_id = get_client_id(&args);

    // Audit log the execution attempt
    server
        .audit_logger()
        .log_code_execution(
            &client_id,
            "deprecated",
            0,
            false,
            Some("WASM sandbox removed"),
        )
        .await;

    Err(anyhow::anyhow!(
        "Code execution is no longer available. The WASM sandbox was removed in v0.1.29. \
         See ADR-052 for details."
    ))
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
            do_memory_mcp::mcp::tools::advanced_pattern_analysis::AnalysisType::Statistical
        }
        "predictive" => {
            do_memory_mcp::mcp::tools::advanced_pattern_analysis::AnalysisType::Predictive
        }
        "comprehensive" => {
            do_memory_mcp::mcp::tools::advanced_pattern_analysis::AnalysisType::Comprehensive
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

    let input =
        do_memory_mcp::mcp::tools::advanced_pattern_analysis::AdvancedPatternAnalysisInput {
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

/// Handle generate_embedding tool
pub async fn handle_generate_embedding(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
    let client_id = get_client_id(&args);
    let input: GenerateEmbeddingInput = serde_json::from_value(args)?;

    let result = server.execute_generate_embedding(input).await;

    // Audit log the operation
    let success = result.is_ok();
    server
        .audit_logger()
        .log_embedding_generation(&client_id, success)
        .await;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }];
    Ok(content)
}

/// Handle search_by_embedding tool
pub async fn handle_search_by_embedding(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
    let client_id = get_client_id(&args);
    let input: SearchByEmbeddingInput = serde_json::from_value(args)?;

    let result = server.execute_search_by_embedding(input).await;

    // Audit log the operation
    let result_count = result
        .as_ref()
        .ok()
        .and_then(|v| v.as_object())
        .map(|o| o.len())
        .unwrap_or(0);
    server
        .audit_logger()
        .log_embedding_search(&client_id, result_count, result.is_ok())
        .await;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }];
    Ok(content)
}

/// Handle embedding_provider_status tool
pub async fn handle_embedding_provider_status(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.unwrap_or(json!({}));
    let input: EmbeddingProviderStatusInput = serde_json::from_value(args)?;

    let result = server.execute_embedding_provider_status_tool(input).await?;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];
    Ok(content)
}

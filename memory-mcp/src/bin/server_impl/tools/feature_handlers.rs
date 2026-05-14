//! Feature handler functions (ADR-044 and pattern search)
//!
//! This module contains handlers for newer features including
//! playbook recommendations, pattern explanations, and pattern search/recommend.

use super::{Content, MemoryMCPServer, Value, get_client_id, json_value_len};
use do_memory_mcp::mcp::tools::pattern_search::{RecommendPatternsInput, SearchPatternsInput};

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

/// Handle recommend_playbook tool (ADR-044 Feature 1)
pub async fn handle_recommend_playbook(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let client_id = get_client_id(&args);

    let task_description = args
        .get("task_description")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'task_description' parameter"))?
        .to_string();

    let domain = args
        .get("domain")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'domain' parameter"))?
        .to_string();

    let task_type_str = args
        .get("task_type")
        .and_then(|v| v.as_str())
        .unwrap_or("code_generation");

    let task_type = match task_type_str {
        "code_generation" => do_memory_core::TaskType::CodeGeneration,
        "debugging" => do_memory_core::TaskType::Debugging,
        "refactoring" => do_memory_core::TaskType::Refactoring,
        "testing" => do_memory_core::TaskType::Testing,
        "analysis" => do_memory_core::TaskType::Analysis,
        "documentation" => do_memory_core::TaskType::Documentation,
        _ => do_memory_core::TaskType::CodeGeneration,
    };

    let max_steps = args.get("max_steps").and_then(|v| v.as_u64()).unwrap_or(5) as usize;

    let language = args
        .get("language")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let framework = args
        .get("framework")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let tags = args
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let context = do_memory_core::TaskContext {
        domain: domain.clone(),
        language,
        framework,
        complexity: do_memory_core::ComplexityLevel::Moderate,
        tags,
    };

    // Retrieve playbooks from memory
    let playbooks = server
        .memory()
        .retrieve_playbooks(
            &task_description,
            &domain,
            task_type,
            context,
            1, // max_playbooks
            max_steps,
        )
        .await;

    let result = serde_json::to_value(&playbooks)?;

    // Audit log the operation
    server
        .audit_logger()
        .log_playbook_recommendation(&client_id, &task_description, playbooks.len(), true)
        .await;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];
    Ok(content)
}

/// Handle explain_pattern tool (ADR-044 Feature 1)
pub async fn handle_explain_pattern(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let client_id = get_client_id(&args);

    let pattern_id_str = args
        .get("pattern_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'pattern_id' parameter"))?;

    let pattern_id = uuid::Uuid::parse_str(pattern_id_str)
        .map_err(|e| anyhow::anyhow!("Invalid pattern_id: {}", e))?;

    // Get pattern explanation from memory
    let explanation = server
        .memory()
        .explain_pattern(pattern_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("Pattern not found: {}", pattern_id))?;

    let result = serde_json::json!({
        "pattern_id": pattern_id_str,
        "explanation": explanation
    });

    // Audit log the operation
    server
        .audit_logger()
        .log_pattern_explanation(&client_id, pattern_id_str, true)
        .await;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];
    Ok(content)
}

/// Handle record_recommendation_session tool (ADR-044 Feature 2)
pub async fn handle_record_recommendation_session(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let _client_id = get_client_id(&args);
    let input: do_memory_mcp::mcp::tools::recommendation_feedback::RecordRecommendationSessionInput =
        serde_json::from_value(args)?;

    let tools =
        do_memory_mcp::mcp::tools::recommendation_feedback::RecommendationFeedbackTools::new(
            server.memory(),
        );
    let result = tools.record_session(input).await?;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];
    Ok(content)
}

/// Handle record_recommendation_feedback tool (ADR-044 Feature 2)
pub async fn handle_record_recommendation_feedback(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let _client_id = get_client_id(&args);
    let input: do_memory_mcp::mcp::tools::recommendation_feedback::RecordRecommendationFeedbackInput =
        serde_json::from_value(args)?;

    let tools =
        do_memory_mcp::mcp::tools::recommendation_feedback::RecommendationFeedbackTools::new(
            server.memory(),
        );
    let result = tools.record_feedback(input).await?;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];
    Ok(content)
}

/// Handle get_recommendation_stats tool (ADR-044 Feature 2)
pub async fn handle_get_recommendation_stats(
    server: &mut MemoryMCPServer,
    _arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let tools =
        do_memory_mcp::mcp::tools::recommendation_feedback::RecommendationFeedbackTools::new(
            server.memory(),
        );
    let result = tools.get_stats().await?;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];
    Ok(content)
}

/// Handle checkpoint_episode tool (ADR-044 Feature 3)
pub async fn handle_checkpoint_episode(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let _client_id = get_client_id(&args);
    let input: do_memory_mcp::mcp::tools::checkpoint::CheckpointEpisodeInput =
        serde_json::from_value(args)?;

    let tools = do_memory_mcp::mcp::tools::checkpoint::CheckpointTools::new(server.memory());
    let result = tools.checkpoint_episode(input).await?;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];
    Ok(content)
}

/// Handle get_handoff_pack tool (ADR-044 Feature 3)
pub async fn handle_get_handoff_pack(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let _client_id = get_client_id(&args);
    let input: do_memory_mcp::mcp::tools::checkpoint::GetHandoffPackInput =
        serde_json::from_value(args)?;

    let tools = do_memory_mcp::mcp::tools::checkpoint::CheckpointTools::new(server.memory());
    let result = tools.get_handoff_pack(input).await?;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];
    Ok(content)
}

/// Handle resume_from_handoff tool (ADR-044 Feature 3)
pub async fn handle_resume_from_handoff(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    let args: Value = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
    let _client_id = get_client_id(&args);
    let input: do_memory_mcp::mcp::tools::checkpoint::ResumeFromHandoffInput =
        serde_json::from_value(args)?;

    let tools = do_memory_mcp::mcp::tools::checkpoint::CheckpointTools::new(server.memory());
    let result = tools.resume_from_handoff(input).await?;

    let content = vec![Content::Text {
        text: serde_json::to_string_pretty(&result)?,
    }];
    Ok(content)
}

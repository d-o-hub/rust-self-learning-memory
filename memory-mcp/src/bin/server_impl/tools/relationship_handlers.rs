use super::{Content, MemoryMCPServer, OperationType, Value, get_client_id};

pub async fn handle_add_episode_relationship(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::mcp::tools::episode_relationships::{
        AddEpisodeRelationshipInput, EpisodeRelationshipTools,
    };

    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

    let client_id = server.client_id_from_args(&args);
    let rate_limit_result = server.check_rate_limit(&client_id, OperationType::Write);
    if !rate_limit_result.allowed {
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

pub async fn handle_remove_episode_relationship(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::mcp::tools::episode_relationships::{
        EpisodeRelationshipTools, RemoveEpisodeRelationshipInput,
    };

    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

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

    let success = result.is_ok();
    server
        .audit_logger()
        .log_remove_relationship(&client_id_str, &relationship_id, success)
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

pub async fn handle_get_episode_relationships(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::mcp::tools::episode_relationships::{
        EpisodeRelationshipTools, GetEpisodeRelationshipsInput,
    };

    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

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

pub async fn handle_find_related_episodes(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::mcp::tools::episode_relationships::{
        EpisodeRelationshipTools, FindRelatedEpisodesInput,
    };

    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

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

pub async fn handle_check_relationship_exists(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::mcp::tools::episode_relationships::{
        CheckRelationshipExistsInput, EpisodeRelationshipTools,
    };

    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

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

pub async fn handle_get_dependency_graph(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::mcp::tools::episode_relationships::{
        DependencyGraphInput, EpisodeRelationshipTools,
    };

    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

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

pub async fn handle_validate_no_cycles(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::mcp::tools::episode_relationships::{
        EpisodeRelationshipTools, ValidateNoCyclesInput,
    };

    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

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

pub async fn handle_get_topological_order(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use memory_mcp::mcp::tools::episode_relationships::{
        EpisodeRelationshipTools, GetTopologicalOrderInput,
    };

    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

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

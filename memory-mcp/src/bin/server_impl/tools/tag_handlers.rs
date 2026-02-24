use super::{Content, MemoryMCPServer, Value, get_client_id};

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

    let success = result.is_ok();
    server
        .audit_logger()
        .log_add_tags(&client_id, &episode_id, &tags, success)
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

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

    let success = result.is_ok();
    server
        .audit_logger()
        .log_remove_tags(&client_id, &episode_id, &tags, success)
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

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

    let success = result.is_ok();
    server
        .audit_logger()
        .log_set_tags(&client_id, &episode_id, &tags, success)
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

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

    let result_count = result.as_ref().map(|r| r.count).unwrap_or(0);
    server
        .audit_logger()
        .log_search_tags(&client_id, &tags, result_count)
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

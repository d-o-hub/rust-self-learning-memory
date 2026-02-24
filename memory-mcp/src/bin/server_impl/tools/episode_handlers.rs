use super::{Content, MemoryMCPServer, Value, get_client_id};
use serde_json::json;

pub async fn handle_bulk_episodes(
    server: &mut MemoryMCPServer,
    arguments: Option<Value>,
) -> anyhow::Result<Vec<Content>> {
    use uuid::Uuid;

    let args: Value = arguments.unwrap_or(json!({}));
    let client_id = get_client_id(&args);

    let episode_ids_value = args
        .get("episode_ids")
        .ok_or_else(|| anyhow::anyhow!("Missing 'episode_ids' parameter"))?;

    let episode_ids: Vec<Uuid> = match episode_ids_value {
        Value::String(s) => s
            .split(',')
            .map(|id| {
                let id = id.trim();
                Uuid::parse_str(id).map_err(|_| anyhow::anyhow!("Invalid UUID: {}", id))
            })
            .collect::<anyhow::Result<Vec<_>>>()?,
        Value::Array(arr) => arr
            .iter()
            .map(|v| {
                let id = v
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Expected string in episode_ids array"))?;
                Uuid::parse_str(id).map_err(|_| anyhow::anyhow!("Invalid UUID: {}", id))
            })
            .collect::<anyhow::Result<Vec<_>>>()?,
        _ => {
            return Err(anyhow::anyhow!(
                "episode_ids must be a string (comma-separated) or array of UUIDs"
            ));
        }
    };

    let result = server.get_episodes_by_ids(&episode_ids).await;

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

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&bulk_result)?,
    }])
}

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
    let success = result.is_ok();
    server
        .audit_logger()
        .log_episode_step(&client_id, &episode_id, step_number, &tool, success)
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

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
    let success = result.is_ok();
    server
        .audit_logger()
        .log_episode_completion(&client_id, &episode_id, &outcome, success)
        .await;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&result?)?,
    }])
}

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

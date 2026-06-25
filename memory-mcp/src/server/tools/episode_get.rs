//! Episode retrieval and deletion tools for MCP server
//!
//! This module provides tools for getting episode details and deleting
//! episodes permanently.

use crate::server::MemoryMCPServer;
use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use tracing::debug;
use tracing::warn;
use uuid::Uuid;

impl MemoryMCPServer {
    /// Get episode details by ID
    ///
    /// This tool retrieves complete details of an episode including
    /// all steps, outcome, reflection, and extracted patterns.
    ///
    /// # Arguments (from JSON)
    ///
    /// * `episode_id` - UUID of the episode to retrieve
    /// * `fields` - Optional array of field paths to return (e.g., ["episode.id", "episode.task_description"])
    ///
    /// # Field Selection
    ///
    /// Clients can request specific fields to reduce token usage:
    /// ```json
    /// {
    ///   "episode_id": "123e4567-e89b-12d3-a456-426614174000",
    ///   "fields": ["episode.id", "episode.task_description", "episode.outcome"]
    /// }
    /// ```
    pub async fn get_episode_tool(&self, args: Value) -> Result<Value> {
        debug!("Getting episode with args: {}", args);

        let episode_id_str = args
            .get("episode_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: episode_id"))?;

        let episode_id = Uuid::parse_str(episode_id_str)
            .map_err(|e| anyhow!("Invalid episode_id format: {}", e))?;

        let episode = self
            .memory
            .get_episode(episode_id)
            .await
            .map_err(|e| anyhow!("Failed to get episode: {}", e))?;

        // Convert episode to JSON
        let episode_json = serde_json::to_value(&episode)
            .map_err(|e| anyhow!("Failed to serialize episode: {}", e))?;

        let result = json!({
            "success": true,
            "episode": episode_json
        });

        // Apply field projection if requested
        if let Some(fields_value) = args.get("fields") {
            if let Some(field_array) = fields_value.as_array() {
                let field_list: Vec<String> = field_array
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                if !field_list.is_empty() {
                    use crate::server::tools::field_projection::FieldSelector;
                    let selector = FieldSelector::new(field_list.into_iter().collect());
                    return selector.apply(&result);
                }
            }
        }

        Ok(result)
    }

    /// Delete an episode permanently
    ///
    /// This tool removes an episode from all storage backends.
    /// **Warning**: This operation cannot be undone.
    ///
    /// # Arguments (from JSON)
    ///
    /// * `episode_id` - UUID of the episode to delete
    /// * `confirm` - Must be set to true to confirm deletion
    pub async fn delete_episode_tool(&self, args: Value) -> Result<Value> {
        debug!("Deleting episode with args: {}", args);

        let episode_id_str = args
            .get("episode_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: episode_id"))?;

        let episode_id = Uuid::parse_str(episode_id_str)
            .map_err(|e| anyhow!("Invalid episode_id format: {}", e))?;

        // Require explicit confirmation
        let confirm = args
            .get("confirm")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !confirm {
            return Err(anyhow!(
                "Deletion requires explicit confirmation. Set 'confirm' to true."
            ));
        }

        self.memory
            .delete_episode(episode_id)
            .await
            .map_err(|e| anyhow!("Failed to delete episode: {}", e))?;

        warn!(
            episode_id = %episode_id,
            "Deleted episode via MCP (permanent)"
        );

        Ok(json!({
            "success": true,
            "episode_id": episode_id.to_string(),
            "message": "Episode deleted permanently"
        }))
    }
}

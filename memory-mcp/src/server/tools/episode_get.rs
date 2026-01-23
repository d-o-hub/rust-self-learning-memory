//! Episode retrieval and deletion tools for MCP server
//!
//! This module provides tools for getting episode details and deleting
//! episodes permanently.

use crate::server::MemoryMCPServer;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
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

        Ok(json!({
            "success": true,
            "episode": episode_json
        }))
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

//! Episode update MCP tool
//!
//! Provides the `update_episode` tool for modifying episodes through the MCP server.

use crate::server::MemoryMCPServer;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use tracing::debug;
use tracing::info;
use uuid::Uuid;

impl MemoryMCPServer {
    /// Update an existing episode with new information
    ///
    /// This tool allows AI agents to programmatically update episodes
    /// to modify descriptions, tags, and metadata.
    ///
    /// # Arguments (from JSON)
    ///
    /// * `episode_id` - UUID of the episode to update
    /// * `description` - Optional new task description
    /// * `add_tags` - Optional tags to add to the episode
    /// * `remove_tags` - Optional tags to remove from the episode
    /// * `set_tags` - Optional tags to replace all existing tags
    /// * `metadata` - Optional metadata key-value pairs to merge
    pub async fn update_episode_tool(&self, args: Value) -> Result<Value> {
        debug!("Updating episode with args: {}", args);

        // Extract episode_id
        let episode_id_str = args
            .get("episode_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: episode_id"))?
            .to_string();

        let uuid = Uuid::parse_str(&episode_id_str)
            .map_err(|_| anyhow!("Invalid episode ID format: {}", episode_id_str))?;

        // Extract optional fields
        let description = args
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let add_tags = args.get("add_tags").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        });

        let remove_tags = args
            .get("remove_tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            });

        let set_tags = args.get("set_tags").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        });

        let metadata = args.get("metadata").and_then(|v| v.as_object()).map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        });

        // Track updated fields
        let mut updated_fields = Vec::new();

        // Update description if provided
        if let Some(desc) = description {
            self.memory
                .update_episode(uuid, Some(desc.clone()), None)
                .await
                .map_err(|e| anyhow!("Failed to update description: {}", e))?;
            updated_fields.push("description");
        }

        // Update metadata if provided
        if let Some(meta) = metadata {
            self.memory
                .update_episode(uuid, None, Some(meta))
                .await
                .map_err(|e| anyhow!("Failed to update metadata: {}", e))?;
            updated_fields.push("metadata");
        }

        // Add tags
        if let Some(tags) = add_tags {
            self.memory
                .add_episode_tags(uuid, tags)
                .await
                .map_err(|e| anyhow!("Failed to add tags: {}", e))?;
            updated_fields.push("tags (added)");
        }

        // Remove tags
        if let Some(tags) = remove_tags {
            self.memory
                .remove_episode_tags(uuid, tags)
                .await
                .map_err(|e| anyhow!("Failed to remove tags: {}", e))?;
            updated_fields.push("tags (removed)");
        }

        // Set tags (replace all)
        if let Some(tags) = set_tags {
            self.memory
                .set_episode_tags(uuid, tags)
                .await
                .map_err(|e| anyhow!("Failed to set tags: {}", e))?;
            updated_fields.push("tags (set)");
        }

        if updated_fields.is_empty() {
            return Ok(json!({
                "success": true,
                "episode_id": episode_id_str,
                "message": "No changes specified. Episode unchanged.",
                "updated_fields": []
            }));
        }

        info!(
            episode_id = %episode_id_str,
            fields = %updated_fields.join(", "),
            "Updated episode via MCP"
        );

        Ok(json!({
            "success": true,
            "episode_id": episode_id_str,
            "message": format!("Successfully updated episode {}. Updated fields: {}", episode_id_str, updated_fields.join(", ")),
            "updated_fields": updated_fields
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SandboxConfig;
    use memory_core::{TaskContext, TaskType};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_update_episode_description() {
        let memory = memory_core::SelfLearningMemory::new();
        let server = MemoryMCPServer::new(SandboxConfig::default(), Arc::new(memory))
            .await
            .unwrap();

        // Create an episode first
        let episode_id = server
            .memory()
            .start_episode(
                "Original description".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        // Update description
        let result = server
            .update_episode_tool(json!({
                "episode_id": episode_id.to_string(),
                "description": "Updated description"
            }))
            .await
            .unwrap();

        assert!(result["success"].as_bool().unwrap());
        assert!(result["message"]
            .as_str()
            .unwrap()
            .contains("Successfully updated"));

        // Verify update
        let episode = server.memory().get_episode(episode_id).await.unwrap();
        assert_eq!(episode.task_description, "Updated description");
    }

    #[tokio::test]
    async fn test_update_episode_tags() {
        let memory = memory_core::SelfLearningMemory::new();
        let server = MemoryMCPServer::new(SandboxConfig::default(), Arc::new(memory))
            .await
            .unwrap();

        // Create an episode
        let episode_id = server
            .memory()
            .start_episode(
                "Test task".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        // Add tags
        let result = server
            .update_episode_tool(json!({
                "episode_id": episode_id.to_string(),
                "add_tags": ["tag1", "tag2"]
            }))
            .await
            .unwrap();

        assert!(result["success"].as_bool().unwrap());

        // Verify tags
        let tags = server.memory().get_episode_tags(episode_id).await.unwrap();
        assert_eq!(tags.len(), 2);
    }

    #[tokio::test]
    async fn test_update_episode_invalid_id() {
        let memory = memory_core::SelfLearningMemory::new();
        let server = MemoryMCPServer::new(SandboxConfig::default(), Arc::new(memory))
            .await
            .unwrap();

        let result = server
            .update_episode_tool(json!({
                "episode_id": "invalid-uuid",
                "description": "Test"
            }))
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid episode ID format"));
    }
}

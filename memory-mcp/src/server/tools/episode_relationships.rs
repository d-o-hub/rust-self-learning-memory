//! Episode relationship tool handlers for MCP server
//!
//! This module provides handlers for episode relationship tools:
//! - add_episode_relationship: Add a relationship between two episodes
//! - remove_episode_relationship: Remove a relationship by ID

use crate::server::MemoryMCPServer;
use anyhow::Result;
use memory_mcp::mcp::tools::episode_relationships::{
    AddEpisodeRelationshipInput, EpisodeRelationshipTools, RemoveEpisodeRelationshipInput,
};
use serde_json::Value;
use tracing::{debug, info};

impl MemoryMCPServer {
    /// Add a relationship between two episodes
    ///
    /// This tool creates a directed relationship from one episode to another
    /// with validation. Supports relationship types: parent_child, depends_on,
    /// follows, related_to, blocks, duplicates, references.
    ///
    /// # Arguments (from JSON)
    ///
    /// * `from_episode_id` - Source episode UUID
    /// * `to_episode_id` - Target episode UUID
    /// * `relationship_type` - Type of relationship
    /// * `reason` - Optional explanation
    /// * `priority` - Optional priority (1-10)
    /// * `created_by` - Optional creator identifier
    pub async fn add_episode_relationship_tool(&self, args: Value) -> Result<Value> {
        debug!("Adding episode relationship with args: {}", args);

        let input: AddEpisodeRelationshipInput = serde_json::from_value(args)?;
        let from_id = input.from_episode_id.clone();
        let to_id = input.to_episode_id.clone();
        let rel_type = input.relationship_type.clone();

        let tools = EpisodeRelationshipTools::new(self.memory());
        let result = tools.add_relationship(input).await?;

        info!(
            relationship_id = %result.relationship_id,
            from_episode_id = %from_id,
            to_episode_id = %to_id,
            relationship_type = %rel_type,
            "Created episode relationship via MCP"
        );

        Ok(serde_json::to_value(result)?)
    }

    /// Remove a relationship by ID
    ///
    /// This tool removes an existing episode relationship.
    ///
    /// # Arguments (from JSON)
    ///
    /// * `relationship_id` - UUID of the relationship to remove
    pub async fn remove_episode_relationship_tool(&self, args: Value) -> Result<Value> {
        debug!("Removing episode relationship with args: {}", args);

        let input: RemoveEpisodeRelationshipInput = serde_json::from_value(args)?;
        let relationship_id = input.relationship_id.clone();

        let tools = EpisodeRelationshipTools::new(self.memory());
        let result = tools.remove_relationship(input).await?;

        info!(
            relationship_id = %relationship_id,
            "Removed episode relationship via MCP"
        );

        Ok(serde_json::to_value(result)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_add_relationship_input_parsing() {
        let json = json!({
            "from_episode_id": "550e8400-e29b-41d4-a716-446655440000",
            "to_episode_id": "550e8400-e29b-41d4-a716-446655440001",
            "relationship_type": "depends_on",
            "reason": "Test reason",
            "priority": 5,
            "created_by": "test_user"
        });

        let input: AddEpisodeRelationshipInput = serde_json::from_value(json).unwrap();
        assert_eq!(
            input.from_episode_id,
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(input.to_episode_id, "550e8400-e29b-41d4-a716-446655440001");
        assert_eq!(input.relationship_type, "depends_on");
        assert_eq!(input.reason, Some("Test reason".to_string()));
        assert_eq!(input.priority, Some(5));
        assert_eq!(input.created_by, Some("test_user".to_string()));
    }

    #[test]
    fn test_add_relationship_input_minimal() {
        let json = json!({
            "from_episode_id": "550e8400-e29b-41d4-a716-446655440000",
            "to_episode_id": "550e8400-e29b-41d4-a716-446655440001",
            "relationship_type": "parent_child"
        });

        let input: AddEpisodeRelationshipInput = serde_json::from_value(json).unwrap();
        assert_eq!(
            input.from_episode_id,
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(input.to_episode_id, "550e8400-e29b-41d4-a716-446655440001");
        assert_eq!(input.relationship_type, "parent_child");
        assert_eq!(input.reason, None);
        assert_eq!(input.priority, None);
        assert_eq!(input.created_by, None);
    }

    #[test]
    fn test_remove_relationship_input_parsing() {
        let json = json!({
            "relationship_id": "550e8400-e29b-41d4-a716-446655440000"
        });

        let input: RemoveEpisodeRelationshipInput = serde_json::from_value(json).unwrap();
        assert_eq!(
            input.relationship_id,
            "550e8400-e29b-41d4-a716-446655440000"
        );
    }

    #[test]
    fn test_all_relationship_types() {
        let types = vec![
            "parent_child",
            "depends_on",
            "follows",
            "related_to",
            "blocks",
            "duplicates",
            "references",
        ];

        for rel_type in types {
            let json = json!({
                "from_episode_id": "550e8400-e29b-41d4-a716-446655440000",
                "to_episode_id": "550e8400-e29b-41d4-a716-446655440001",
                "relationship_type": rel_type
            });

            let input: AddEpisodeRelationshipInput = serde_json::from_value(json).unwrap();
            assert_eq!(input.relationship_type, rel_type);
        }
    }
}

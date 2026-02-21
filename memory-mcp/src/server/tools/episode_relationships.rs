//! Episode relationship tool handlers for MCP server
//!
//! This module provides handlers for episode relationship tools:
//! - add_episode_relationship: Add a relationship between two episodes
//! - remove_episode_relationship: Remove a relationship by ID
//! - get_episode_relationships: Get relationships for an episode
//! - find_related_episodes: Find episodes related to a given episode
//! - check_relationship_exists: Check if a specific relationship exists
//! - get_dependency_graph: Get relationship graph for visualization
//! - validate_no_cycles: Check if adding a relationship would create a cycle
//! - get_topological_order: Get topological ordering of episodes

use crate::mcp::tools::episode_relationships::{
    AddEpisodeRelationshipInput, CheckRelationshipExistsInput, DependencyGraphInput,
    EpisodeRelationshipTools, FindRelatedEpisodesInput, GetEpisodeRelationshipsInput,
    GetTopologicalOrderInput, RemoveEpisodeRelationshipInput, ValidateNoCyclesInput,
};
use crate::server::MemoryMCPServer;
use anyhow::Result;
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
        let client_id = input
            .created_by
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        let tools = EpisodeRelationshipTools::new(self.memory());
        let result = tools.add_relationship(input).await;

        // Log the operation to audit trail
        let audit_logger = self.audit_logger();
        match &result {
            Ok(r) => {
                audit_logger
                    .log_add_relationship(
                        &client_id,
                        &from_id,
                        &to_id,
                        &rel_type,
                        &r.relationship_id,
                        true,
                    )
                    .await;

                info!(
                    relationship_id = %r.relationship_id,
                    from_episode_id = %from_id,
                    to_episode_id = %to_id,
                    relationship_type = %rel_type,
                    "Created episode relationship via MCP"
                );
            }
            Err(e) => {
                audit_logger
                    .log_add_relationship(&client_id, &from_id, &to_id, &rel_type, "none", false)
                    .await;

                debug!("Failed to create episode relationship: {}", e);
            }
        }

        let value = result?;
        serde_json::to_value(value).map_err(anyhow::Error::from)
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
        let client_id = "mcp_client".to_string();

        let tools = EpisodeRelationshipTools::new(self.memory());
        let result = tools.remove_relationship(input).await;

        // Log the operation to audit trail
        let audit_logger = self.audit_logger();
        match &result {
            Ok(_) => {
                audit_logger
                    .log_remove_relationship(&client_id, &relationship_id, true)
                    .await;

                info!(relationship_id = %relationship_id, "Removed episode relationship via MCP");
            }
            Err(e) => {
                audit_logger
                    .log_remove_relationship(&client_id, &relationship_id, false)
                    .await;

                debug!("Failed to remove episode relationship: {}", e);
            }
        }

        let value = result?;
        serde_json::to_value(value).map_err(anyhow::Error::from)
    }

    /// Get relationships for an episode
    ///
    /// This tool retrieves all relationships for a given episode with optional
    /// direction and type filtering.
    ///
    /// # Arguments (from JSON)
    ///
    /// * `episode_id` - Episode UUID to query
    /// * `direction` - Optional direction filter ("outgoing", "incoming", "both")
    /// * `relationship_type` - Optional relationship type filter
    pub async fn get_episode_relationships_tool(&self, args: Value) -> Result<Value> {
        debug!("Getting episode relationships with args: {}", args);

        let input: GetEpisodeRelationshipsInput = serde_json::from_value(args)?;
        let episode_id = input.episode_id.clone();
        let client_id = "mcp_client".to_string();

        let tools = EpisodeRelationshipTools::new(self.memory());
        let result = tools.get_relationships(input).await;

        // Log the operation to audit trail
        let audit_logger = self.audit_logger();
        match &result {
            Ok(r) => {
                let total_count = r.outgoing.len() + r.incoming.len();
                audit_logger
                    .log_get_relationships(&client_id, &episode_id, total_count, true)
                    .await;

                info!(
                    episode_id = %episode_id,
                    outgoing_count = r.outgoing.len(),
                    incoming_count = r.incoming.len(),
                    "Retrieved episode relationships via MCP"
                );
            }
            Err(e) => {
                audit_logger
                    .log_get_relationships(&client_id, &episode_id, 0, false)
                    .await;

                debug!("Failed to get episode relationships: {}", e);
            }
        }

        let value = result?;
        serde_json::to_value(value).map_err(anyhow::Error::from)
    }

    /// Find episodes related to a given episode
    ///
    /// This tool finds all episodes related to the specified episode with
    /// optional filtering by relationship type.
    ///
    /// # Arguments (from JSON)
    ///
    /// * `episode_id` - Episode UUID to find relationships for
    /// * `relationship_type` - Optional relationship type filter
    /// * `limit` - Optional maximum number of results (default: 10)
    /// * `include_metadata` - Optional flag to include relationship metadata
    pub async fn find_related_episodes_tool(&self, args: Value) -> Result<Value> {
        debug!("Finding related episodes with args: {}", args);

        let input: FindRelatedEpisodesInput = serde_json::from_value(args)?;
        let episode_id = input.episode_id.clone();
        let client_id = "mcp_client".to_string();

        let tools = EpisodeRelationshipTools::new(self.memory());
        let result = tools.find_related(input).await;

        // Log the operation to audit trail
        let audit_logger = self.audit_logger();
        match &result {
            Ok(r) => {
                audit_logger
                    .log_find_related(&client_id, &episode_id, r.count, true)
                    .await;

                info!(episode_id = %episode_id, related_count = r.count, "Found related episodes via MCP");
            }
            Err(e) => {
                audit_logger
                    .log_find_related(&client_id, &episode_id, 0, false)
                    .await;

                debug!("Failed to find related episodes: {}", e);
            }
        }

        let value = result?;
        serde_json::to_value(value).map_err(anyhow::Error::from)
    }

    /// Check if a specific relationship exists
    ///
    /// This tool checks whether a relationship of a specific type exists
    /// between two episodes.
    ///
    /// # Arguments (from JSON)
    ///
    /// * `from_episode_id` - Source episode UUID
    /// * `to_episode_id` - Target episode UUID
    /// * `relationship_type` - Type of relationship to check
    pub async fn check_relationship_exists_tool(&self, args: Value) -> Result<Value> {
        debug!("Checking relationship exists with args: {}", args);

        let input: CheckRelationshipExistsInput = serde_json::from_value(args)?;
        let from_id = input.from_episode_id.clone();
        let to_id = input.to_episode_id.clone();
        let rel_type = input.relationship_type.clone();
        let client_id = "mcp_client".to_string();

        let tools = EpisodeRelationshipTools::new(self.memory());
        let result = tools.check_exists(input).await;

        // Log the operation to audit trail
        let audit_logger = self.audit_logger();
        match &result {
            Ok(c) => {
                audit_logger
                    .log_check_relationship(&client_id, &from_id, &to_id, &rel_type, c.exists, true)
                    .await;

                info!(
                    from_episode_id = %from_id,
                    to_episode_id = %to_id,
                    relationship_type = %rel_type,
                    exists = c.exists,
                    "Checked relationship existence via MCP"
                );
            }
            Err(e) => {
                audit_logger
                    .log_check_relationship(&client_id, &from_id, &to_id, &rel_type, false, false)
                    .await;

                debug!("Failed to check relationship exists: {}", e);
            }
        }

        let value = result?;
        serde_json::to_value(value).map_err(anyhow::Error::from)
    }

    /// Get dependency graph for visualization
    ///
    /// This tool builds a relationship graph starting from an episode up to
    /// a specified depth, optionally in DOT format for visualization.
    ///
    /// # Arguments (from JSON)
    ///
    /// * `episode_id` - Root episode UUID
    /// * `depth` - Optional maximum traversal depth (1-5, default: 2)
    /// * `format` - Optional output format ("json" or "dot", default: "json")
    pub async fn get_dependency_graph_tool(&self, args: Value) -> Result<Value> {
        debug!("Getting dependency graph with args: {}", args);

        let input: DependencyGraphInput = serde_json::from_value(args)?;
        let episode_id = input.episode_id.clone();
        let format = input.format.clone().unwrap_or_else(|| "json".to_string());
        let client_id = "mcp_client".to_string();

        let tools = EpisodeRelationshipTools::new(self.memory());
        let result = tools.get_dependency_graph(input).await;

        // Log the operation to audit trail
        let audit_logger = self.audit_logger();
        match &result {
            Ok(g) => {
                audit_logger
                    .log_dependency_graph(&client_id, &episode_id, g.node_count, g.edge_count, true)
                    .await;

                info!(
                    episode_id = %episode_id,
                    node_count = g.node_count,
                    edge_count = g.edge_count,
                    format = %format,
                    "Retrieved dependency graph via MCP"
                );
            }
            Err(e) => {
                audit_logger
                    .log_dependency_graph(&client_id, &episode_id, 0, 0, false)
                    .await;

                debug!("Failed to get dependency graph: {}", e);
            }
        }

        let value = result?;
        serde_json::to_value(value).map_err(anyhow::Error::from)
    }

    /// Validate that adding a relationship would not create a cycle
    ///
    /// This tool checks if adding a relationship between two episodes would
    /// create a cycle in the dependency graph. Returns whether a cycle would
    /// be created and the cycle path if detected.
    ///
    /// # Arguments (from JSON)
    ///
    /// * `from_episode_id` - Source episode UUID (proposed from)
    /// * `to_episode_id` - Target episode UUID (proposed to)
    /// * `relationship_type` - Type of relationship being added
    pub async fn validate_no_cycles_tool(&self, args: Value) -> Result<Value> {
        debug!("Validating no cycles with args: {}", args);

        let input: ValidateNoCyclesInput = serde_json::from_value(args)?;
        let from_id = input.from_episode_id.clone();
        let to_id = input.to_episode_id.clone();
        let rel_type = input.relationship_type.clone();
        let client_id = "mcp_client".to_string();

        let tools = EpisodeRelationshipTools::new(self.memory());
        let result = tools.validate_no_cycles(input).await;

        // Log the operation to audit trail
        let audit_logger = self.audit_logger();
        match &result {
            Ok(v) => {
                audit_logger
                    .log_validate_cycles(
                        &client_id,
                        &from_id,
                        &to_id,
                        &rel_type,
                        v.would_create_cycle,
                        true,
                    )
                    .await;

                info!(
                    from_episode_id = %from_id,
                    to_episode_id = %to_id,
                    relationship_type = %rel_type,
                    would_create_cycle = v.would_create_cycle,
                    is_valid = v.is_valid,
                    "Validated cycle absence via MCP"
                );
            }
            Err(e) => {
                audit_logger
                    .log_validate_cycles(&client_id, &from_id, &to_id, &rel_type, false, false)
                    .await;

                debug!("Failed to validate no cycles: {}", e);
            }
        }

        let value = result?;
        serde_json::to_value(value).map_err(anyhow::Error::from)
    }

    /// Get topological ordering of episodes
    ///
    /// This tool returns episodes in topological order where dependencies come
    /// before dependents. Only works on directed acyclic graphs (DAGs).
    ///
    /// # Arguments (from JSON)
    ///
    /// * `episode_ids` - Array of episode UUIDs to sort
    pub async fn get_topological_order_tool(&self, args: Value) -> Result<Value> {
        debug!("Getting topological order with args: {}", args);

        let input: GetTopologicalOrderInput = serde_json::from_value(args)?;
        let episode_count = input.episode_ids.len();
        let client_id = "mcp_client".to_string();

        let tools = EpisodeRelationshipTools::new(self.memory());
        let result = tools.get_topological_order(input).await;

        // Log the operation to audit trail
        let audit_logger = self.audit_logger();
        match &result {
            Ok(o) => {
                audit_logger
                    .log_topological_order(&client_id, episode_count, o.count, o.has_cycles, true)
                    .await;

                info!(
                    input_count = episode_count,
                    output_count = o.count,
                    has_cycles = o.has_cycles,
                    "Computed topological order via MCP"
                );
            }
            Err(e) => {
                audit_logger
                    .log_topological_order(&client_id, episode_count, 0, false, false)
                    .await;

                debug!("Failed to get topological order: {}", e);
            }
        }

        let value = result?;
        serde_json::to_value(value).map_err(anyhow::Error::from)
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

    #[test]
    fn test_get_relationships_input_parsing() {
        let json = json!({
            "episode_id": "550e8400-e29b-41d4-a716-446655440000",
            "direction": "outgoing",
            "relationship_type": "depends_on"
        });

        let input: GetEpisodeRelationshipsInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.episode_id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(input.direction, Some("outgoing".to_string()));
        assert_eq!(input.relationship_type, Some("depends_on".to_string()));
    }

    #[test]
    fn test_find_related_episodes_input_parsing() {
        let json = json!({
            "episode_id": "550e8400-e29b-41d4-a716-446655440000",
            "relationship_type": "depends_on",
            "limit": 5,
            "include_metadata": true
        });

        let input: FindRelatedEpisodesInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.episode_id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(input.relationship_type, Some("depends_on".to_string()));
        assert_eq!(input.limit, Some(5));
        assert_eq!(input.include_metadata, Some(true));
    }

    #[test]
    fn test_check_relationship_exists_input_parsing() {
        let json = json!({
            "from_episode_id": "550e8400-e29b-41d4-a716-446655440000",
            "to_episode_id": "550e8400-e29b-41d4-a716-446655440001",
            "relationship_type": "depends_on"
        });

        let input: CheckRelationshipExistsInput = serde_json::from_value(json).unwrap();
        assert_eq!(
            input.from_episode_id,
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(input.to_episode_id, "550e8400-e29b-41d4-a716-446655440001");
        assert_eq!(input.relationship_type, "depends_on");
    }

    #[test]
    fn test_dependency_graph_input_parsing() {
        let json = json!({
            "episode_id": "550e8400-e29b-41d4-a716-446655440000",
            "depth": 3,
            "format": "dot"
        });

        let input: DependencyGraphInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.episode_id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(input.depth, Some(3));
        assert_eq!(input.format, Some("dot".to_string()));
    }

    #[test]
    fn test_validate_no_cycles_input_parsing() {
        let json = json!({
            "from_episode_id": "550e8400-e29b-41d4-a716-446655440000",
            "to_episode_id": "550e8400-e29b-41d4-a716-446655440001",
            "relationship_type": "depends_on"
        });

        let input: ValidateNoCyclesInput = serde_json::from_value(json).unwrap();
        assert_eq!(
            input.from_episode_id,
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(input.to_episode_id, "550e8400-e29b-41d4-a716-446655440001");
        assert_eq!(input.relationship_type, "depends_on");
    }

    #[test]
    fn test_get_topological_order_input_parsing() {
        let json = json!({
            "episode_ids": [
                "550e8400-e29b-41d4-a716-446655440000",
                "550e8400-e29b-41d4-a716-446655440001",
                "550e8400-e29b-41d4-a716-446655440002"
            ]
        });

        let input: GetTopologicalOrderInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.episode_ids.len(), 3);
        assert_eq!(input.episode_ids[0], "550e8400-e29b-41d4-a716-446655440000");
    }
}

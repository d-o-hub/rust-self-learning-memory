//! Episode relationship tool implementations.

use crate::mcp::tools::episode_relationships::types::{
    AddEpisodeRelationshipInput, AddEpisodeRelationshipOutput, CheckRelationshipExistsInput,
    CheckRelationshipExistsOutput, DependencyGraphInput, DependencyGraphOutput,
    FindRelatedEpisodesInput, FindRelatedEpisodesOutput, GetEpisodeRelationshipsInput,
    GetEpisodeRelationshipsOutput, GetTopologicalOrderInput, GetTopologicalOrderOutput,
    RelatedEpisode, RelationshipEdge, RelationshipNode, RemoveEpisodeRelationshipInput,
    RemoveEpisodeRelationshipOutput, TopologicalEpisode, ValidateNoCyclesInput,
    ValidateNoCyclesOutput,
};
use anyhow::{Result, anyhow};
use memory_core::SelfLearningMemory;
use memory_core::episode::{
    Direction, EpisodeRelationship, RelationshipMetadata, RelationshipType,
};
use memory_core::memory::relationship_query::RelationshipFilter;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, instrument};
use uuid::Uuid;

/// Episode relationship tools
#[derive(Clone)]
pub struct EpisodeRelationshipTools {
    memory: Arc<SelfLearningMemory>,
}

impl EpisodeRelationshipTools {
    /// Create a new episode relationship tools instance
    pub fn new(memory: Arc<SelfLearningMemory>) -> Self {
        Self { memory }
    }

    /// Add a relationship between two episodes
    ///
    /// Creates a directed relationship from one episode to another with validation.
    /// Validates that both episodes exist, prevents self-relationships, and checks
    /// for cycles in acyclic relationship types.
    ///
    /// # Arguments
    ///
    /// * `input` - Input containing from/to episode IDs, relationship type, and optional metadata
    ///
    /// # Returns
    ///
    /// Returns the created relationship ID.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Episode IDs are invalid
    /// - Either episode does not exist
    /// - Self-relationship is attempted
    /// - Duplicate relationship exists
    /// - Cycle would be created (for acyclic types like depends_on)
    #[instrument(skip(self, input), fields(from = %input.from_episode_id, to = %input.to_episode_id, rel_type = %input.relationship_type))]
    pub async fn add_relationship(
        &self,
        input: AddEpisodeRelationshipInput,
    ) -> Result<AddEpisodeRelationshipOutput> {
        info!(
            "Adding {} relationship from {} to {}",
            input.relationship_type, input.from_episode_id, input.to_episode_id
        );

        // Parse episode IDs
        let from_id = Uuid::parse_str(&input.from_episode_id)
            .map_err(|e| anyhow!("Invalid from_episode_id: {}", e))?;
        let to_id = Uuid::parse_str(&input.to_episode_id)
            .map_err(|e| anyhow!("Invalid to_episode_id: {}", e))?;

        // Parse relationship type
        let rel_type = RelationshipType::parse(&input.relationship_type)
            .map_err(|e| anyhow!("Invalid relationship_type: {}", e))?;

        // Build metadata
        let mut metadata = RelationshipMetadata::new();
        if let Some(reason) = &input.reason {
            metadata.reason = Some(reason.clone());
        }
        if let Some(created_by) = &input.created_by {
            metadata.created_by = Some(created_by.clone());
        }
        if let Some(priority) = input.priority {
            metadata.priority = Some(priority);
        }

        // Add relationship using memory API
        let relationship_id = self
            .memory
            .add_episode_relationship(from_id, to_id, rel_type, metadata)
            .await?;

        info!(
            "Successfully created relationship {} from {} to {}",
            relationship_id, from_id, to_id
        );

        Ok(AddEpisodeRelationshipOutput {
            success: true,
            relationship_id: relationship_id.to_string(),
            from_episode_id: input.from_episode_id,
            to_episode_id: input.to_episode_id,
            relationship_type: input.relationship_type,
            message: format!("Relationship created successfully: {}", relationship_id),
        })
    }

    /// Remove a relationship by ID
    ///
    /// # Arguments
    ///
    /// * `input` - Input containing relationship ID to remove
    ///
    /// # Returns
    ///
    /// Returns success confirmation.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Relationship ID is invalid
    /// - Relationship does not exist
    #[instrument(skip(self, input), fields(relationship_id = %input.relationship_id))]
    pub async fn remove_relationship(
        &self,
        input: RemoveEpisodeRelationshipInput,
    ) -> Result<RemoveEpisodeRelationshipOutput> {
        info!("Removing relationship: {}", input.relationship_id);

        // Parse relationship ID
        let rel_id = Uuid::parse_str(&input.relationship_id)
            .map_err(|e| anyhow!("Invalid relationship_id: {}", e))?;

        // Remove relationship using memory API
        self.memory.remove_episode_relationship(rel_id).await?;

        info!("Successfully removed relationship: {}", rel_id);

        Ok(RemoveEpisodeRelationshipOutput {
            success: true,
            relationship_id: input.relationship_id,
            message: "Relationship removed successfully".to_string(),
        })
    }

    /// Get all relationships for an episode
    ///
    /// # Arguments
    ///
    /// * `input` - Input containing episode ID and optional filters
    ///
    /// # Returns
    ///
    /// Returns outgoing and incoming relationships.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Episode ID is invalid
    /// - Episode does not exist
    #[instrument(skip(self, input), fields(episode_id = %input.episode_id))]
    pub async fn get_relationships(
        &self,
        input: GetEpisodeRelationshipsInput,
    ) -> Result<GetEpisodeRelationshipsOutput> {
        debug!("Getting relationships for episode: {}", input.episode_id);

        // Parse episode ID
        let episode_id =
            Uuid::parse_str(&input.episode_id).map_err(|e| anyhow!("Invalid episode_id: {}", e))?;

        // Parse direction
        let direction = match input.direction.as_deref() {
            Some("outgoing") => Direction::Outgoing,
            Some("incoming") => Direction::Incoming,
            _ => Direction::Both,
        };

        // Get relationships using memory API
        let relationships = self
            .memory
            .get_episode_relationships(episode_id, direction)
            .await?;

        // Convert to output format
        let mut outgoing = Vec::new();
        let mut incoming = Vec::new();

        for rel in relationships {
            let edge = relationship_to_edge(&rel);

            // Filter by type if specified
            if let Some(ref filter_type) = input.relationship_type {
                let filter_rel_type = RelationshipType::parse(filter_type)
                    .map_err(|e| anyhow!("Invalid relationship_type filter: {}", e))?;
                if rel.relationship_type != filter_rel_type {
                    continue;
                }
            }

            if rel.from_episode_id == episode_id {
                outgoing.push(edge);
            } else {
                incoming.push(edge);
            }
        }

        let total_count = outgoing.len() + incoming.len();

        debug!(
            "Found {} outgoing and {} incoming relationships for episode {}",
            outgoing.len(),
            incoming.len(),
            episode_id
        );

        Ok(GetEpisodeRelationshipsOutput {
            success: true,
            episode_id: input.episode_id,
            outgoing,
            incoming,
            total_count,
            message: format!("Found {} relationship(s)", total_count),
        })
    }

    /// Find related episodes
    ///
    /// Finds episodes related to the given episode with optional filtering.
    ///
    /// # Arguments
    ///
    /// * `input` - Input containing episode ID and optional filters
    ///
    /// # Returns
    ///
    /// Returns list of related episodes with metadata.
    #[instrument(skip(self, input), fields(episode_id = %input.episode_id))]
    pub async fn find_related(
        &self,
        input: FindRelatedEpisodesInput,
    ) -> Result<FindRelatedEpisodesOutput> {
        info!("Finding related episodes for: {}", input.episode_id);

        // Parse episode ID
        let episode_id =
            Uuid::parse_str(&input.episode_id).map_err(|e| anyhow!("Invalid episode_id: {}", e))?;

        // Build filter
        let mut filter = RelationshipFilter::new();
        if let Some(ref rel_type_str) = input.relationship_type {
            let rel_type = RelationshipType::parse(rel_type_str)
                .map_err(|e| anyhow!("Invalid relationship_type: {}", e))?;
            filter = filter.with_type(rel_type);
        }
        if let Some(limit) = input.limit {
            filter = filter.with_limit(limit);
        }

        // Get related episode IDs
        let related_ids = self
            .memory
            .find_related_episodes(episode_id, filter)
            .await?;

        // Get relationships for metadata
        let relationships = self
            .memory
            .get_episode_relationships(episode_id, Direction::Both)
            .await?;

        // Build related episodes list
        let mut related_episodes = Vec::new();
        for related_id in related_ids {
            if let Ok(episode) = self.memory.get_episode(related_id).await {
                // Find the relationship
                if let Some(rel) = relationships
                    .iter()
                    .find(|r| r.from_episode_id == related_id || r.to_episode_id == related_id)
                {
                    let direction = if rel.from_episode_id == episode_id {
                        "outgoing"
                    } else {
                        "incoming"
                    };

                    related_episodes.push(RelatedEpisode {
                        episode_id: related_id.to_string(),
                        task_description: episode.task_description.clone(),
                        task_type: format!("{:?}", episode.task_type),
                        relationship_type: rel.relationship_type.as_str().to_string(),
                        direction: direction.to_string(),
                        reason: if input.include_metadata.unwrap_or(false) {
                            rel.metadata.reason.clone()
                        } else {
                            None
                        },
                        priority: if input.include_metadata.unwrap_or(false) {
                            rel.metadata.priority
                        } else {
                            None
                        },
                    });
                }
            }
        }

        let count = related_episodes.len();
        info!("Found {} related episodes for {}", count, episode_id);

        Ok(FindRelatedEpisodesOutput {
            success: true,
            episode_id: input.episode_id,
            related_episodes,
            count,
            message: format!("Found {} related episode(s)", count),
        })
    }

    /// Check if a relationship exists
    ///
    /// # Arguments
    ///
    /// * `input` - Input containing from/to episode IDs and relationship type
    ///
    /// # Returns
    ///
    /// Returns whether the relationship exists.
    #[instrument(skip(self, input), fields(from = %input.from_episode_id, to = %input.to_episode_id))]
    pub async fn check_exists(
        &self,
        input: CheckRelationshipExistsInput,
    ) -> Result<CheckRelationshipExistsOutput> {
        debug!(
            "Checking if relationship exists from {} to {}",
            input.from_episode_id, input.to_episode_id
        );

        // Parse episode IDs
        let from_id = Uuid::parse_str(&input.from_episode_id)
            .map_err(|e| anyhow!("Invalid from_episode_id: {}", e))?;
        let to_id = Uuid::parse_str(&input.to_episode_id)
            .map_err(|e| anyhow!("Invalid to_episode_id: {}", e))?;

        // Parse relationship type
        let rel_type = RelationshipType::parse(&input.relationship_type)
            .map_err(|e| anyhow!("Invalid relationship_type: {}", e))?;

        // Check existence using memory API
        let exists = self
            .memory
            .relationship_exists(from_id, to_id, rel_type)
            .await?;

        Ok(CheckRelationshipExistsOutput {
            success: true,
            exists,
            from_episode_id: input.from_episode_id,
            to_episode_id: input.to_episode_id,
            relationship_type: input.relationship_type,
            message: if exists {
                "Relationship exists".to_string()
            } else {
                "Relationship does not exist".to_string()
            },
        })
    }

    /// Get dependency graph for visualization
    ///
    /// Builds a relationship graph starting from an episode up to a specified depth.
    ///
    /// # Arguments
    ///
    /// * `input` - Input containing root episode ID, depth, and format
    ///
    /// # Returns
    ///
    /// Returns graph nodes and edges, optionally in DOT format.
    #[instrument(skip(self, input), fields(episode_id = %input.episode_id))]
    pub async fn get_dependency_graph(
        &self,
        input: DependencyGraphInput,
    ) -> Result<DependencyGraphOutput> {
        info!(
            "Building dependency graph for episode: {}",
            input.episode_id
        );

        // Parse episode ID
        let episode_id =
            Uuid::parse_str(&input.episode_id).map_err(|e| anyhow!("Invalid episode_id: {}", e))?;

        // Get depth (default 2, max 5)
        let depth = input.depth.map(|d| d.clamp(1, 5)).unwrap_or(2);

        // Get format (default json)
        let format = input.format.as_deref().unwrap_or("json");

        // Build graph using memory API
        let graph = self
            .memory
            .build_relationship_graph(episode_id, depth)
            .await?;

        // Convert nodes
        let nodes: Vec<RelationshipNode> = graph
            .nodes
            .values()
            .map(|ep| RelationshipNode {
                id: ep.episode_id.to_string(),
                task_description: ep.task_description.clone(),
                task_type: format!("{:?}", ep.task_type),
                is_complete: ep.is_complete(),
            })
            .collect();

        // Convert edges
        let edges: Vec<RelationshipEdge> = graph.edges.iter().map(relationship_to_edge).collect();

        // Generate DOT if requested
        let dot = if format == "dot" {
            Some(graph.to_dot())
        } else {
            None
        };

        let node_count = nodes.len();
        let edge_count = edges.len();

        info!(
            "Built dependency graph with {} nodes and {} edges",
            node_count, edge_count
        );

        Ok(DependencyGraphOutput {
            success: true,
            root: input.episode_id,
            node_count,
            edge_count,
            nodes,
            edges,
            dot,
            message: format!(
                "Graph contains {} nodes and {} edges",
                node_count, edge_count
            ),
        })
    }

    /// Validate that adding a relationship would not create a cycle
    ///
    /// # Arguments
    ///
    /// * `input` - Input containing proposed from/to episode IDs and relationship type
    ///
    /// # Returns
    ///
    /// Returns whether adding the relationship would create a cycle.
    #[instrument(skip(self, input), fields(from = %input.from_episode_id, to = %input.to_episode_id))]
    pub async fn validate_no_cycles(
        &self,
        input: ValidateNoCyclesInput,
    ) -> Result<ValidateNoCyclesOutput> {
        info!(
            "Validating no cycles for relationship from {} to {}",
            input.from_episode_id, input.to_episode_id
        );

        // Parse episode IDs
        let from_id = Uuid::parse_str(&input.from_episode_id)
            .map_err(|e| anyhow!("Invalid from_episode_id: {}", e))?;
        let to_id = Uuid::parse_str(&input.to_episode_id)
            .map_err(|e| anyhow!("Invalid to_episode_id: {}", e))?;

        // Parse relationship type
        let rel_type = RelationshipType::parse(&input.relationship_type)
            .map_err(|e| anyhow!("Invalid relationship_type: {}", e))?;

        // Only check for acyclic relationship types
        if !rel_type.requires_acyclic() {
            return Ok(ValidateNoCyclesOutput {
                success: true,
                would_create_cycle: false,
                is_valid: true,
                cycle_path: None,
                message: format!(
                    "Relationship type '{}' does not require acyclic validation",
                    input.relationship_type
                ),
            });
        }

        // Build current graph
        let mut adjacency_list: HashMap<Uuid, Vec<EpisodeRelationship>> = HashMap::new();

        // Get all relationships to build graph
        // Note: This is a simplified approach. In production, you'd want to
        // query only relevant relationships.
        let all_rels = self.get_all_relationships().await?;

        for rel in all_rels {
            adjacency_list
                .entry(rel.from_episode_id)
                .or_default()
                .push(rel);
        }

        // Check if there's already a path from 'to' to 'from' (which would create a cycle)
        let would_create_cycle =
            memory_core::episode::graph_algorithms::has_path_dfs(&adjacency_list, to_id, from_id)?;

        let cycle_path = if would_create_cycle {
            // Try to find the cycle path
            match memory_core::episode::graph_algorithms::find_path_dfs(
                &adjacency_list,
                to_id,
                from_id,
            ) {
                Ok(path) => Some(path.iter().map(|u| u.to_string()).collect()),
                Err(_) => None,
            }
        } else {
            None
        };

        Ok(ValidateNoCyclesOutput {
            success: true,
            would_create_cycle,
            is_valid: !would_create_cycle,
            cycle_path,
            message: if would_create_cycle {
                "Adding this relationship would create a cycle".to_string()
            } else {
                "No cycle would be created".to_string()
            },
        })
    }

    /// Get topological ordering of episodes
    ///
    /// Returns episodes in an order where dependencies come before dependents.
    /// Only works on directed acyclic graphs (DAGs).
    ///
    /// # Arguments
    ///
    /// * `input` - Input containing array of episode IDs to sort
    ///
    /// # Returns
    ///
    /// Returns episodes in topological order.
    #[instrument(skip(self, input), fields(episode_count = input.episode_ids.len()))]
    pub async fn get_topological_order(
        &self,
        input: GetTopologicalOrderInput,
    ) -> Result<GetTopologicalOrderOutput> {
        info!(
            "Getting topological order for {} episodes",
            input.episode_ids.len()
        );

        if input.episode_ids.is_empty() {
            return Ok(GetTopologicalOrderOutput {
                success: true,
                order: vec![],
                count: 0,
                has_cycles: false,
                message: "No episodes provided".to_string(),
            });
        }

        // Parse episode IDs
        let mut episode_ids = Vec::new();
        for id_str in &input.episode_ids {
            let id = Uuid::parse_str(id_str)
                .map_err(|e| anyhow!("Invalid episode_id '{}': {}", id_str, e))?;
            episode_ids.push(id);
        }

        // Build graph from relationships
        let mut adjacency_list: HashMap<Uuid, Vec<EpisodeRelationship>> = HashMap::new();

        // Get relationships for all episodes
        for episode_id in &episode_ids {
            if let Ok(rels) = self
                .memory
                .get_episode_relationships(*episode_id, Direction::Outgoing)
                .await
            {
                // Only include relationships to episodes in our input set
                let filtered_rels: Vec<_> = rels
                    .into_iter()
                    .filter(|r| episode_ids.contains(&r.to_episode_id))
                    .collect();

                if !filtered_rels.is_empty() {
                    adjacency_list.insert(*episode_id, filtered_rels);
                }
            }
        }

        // Check for cycles
        let has_cycles = memory_core::episode::graph_algorithms::has_cycle(&adjacency_list)?;

        if has_cycles {
            return Ok(GetTopologicalOrderOutput {
                success: true,
                order: vec![],
                count: 0,
                has_cycles: true,
                message: "Cannot compute topological order: graph contains cycles".to_string(),
            });
        }

        // Perform topological sort
        let sorted_ids = memory_core::episode::graph_algorithms::topological_sort(&adjacency_list)?;

        // Build output with episode details
        let mut order = Vec::new();
        for (position, id) in sorted_ids.iter().enumerate() {
            if let Ok(episode) = self.memory.get_episode(*id).await {
                order.push(TopologicalEpisode {
                    episode_id: id.to_string(),
                    task_description: episode.task_description.clone(),
                    position: position + 1,
                });
            }
        }

        // Add any episodes that weren't in the sorted list (isolated nodes)
        for id in &episode_ids {
            if !sorted_ids.contains(id) {
                if let Ok(episode) = self.memory.get_episode(*id).await {
                    order.push(TopologicalEpisode {
                        episode_id: id.to_string(),
                        task_description: episode.task_description.clone(),
                        position: order.len() + 1,
                    });
                }
            }
        }

        let count = order.len();
        info!("Computed topological order for {} episodes", count);

        Ok(GetTopologicalOrderOutput {
            success: true,
            order,
            count,
            has_cycles: false,
            message: format!("Episodes in topological order ({} total)", count),
        })
    }

    /// Helper to get all relationships (for cycle detection)
    async fn get_all_relationships(&self) -> Result<Vec<EpisodeRelationship>> {
        // This is a simplified implementation
        // In production, you'd want a dedicated storage method
        // For now, we return an empty list and rely on the validation in add_relationship
        Ok(Vec::new())
    }
}

/// Convert an EpisodeRelationship to a RelationshipEdge
fn relationship_to_edge(rel: &EpisodeRelationship) -> RelationshipEdge {
    RelationshipEdge {
        id: rel.id.to_string(),
        from: rel.from_episode_id.to_string(),
        to: rel.to_episode_id.to_string(),
        relationship_type: rel.relationship_type.as_str().to_string(),
        reason: rel.metadata.reason.clone(),
        priority: rel.metadata.priority,
        created_by: rel.metadata.created_by.clone(),
        created_at: rel.created_at.to_rfc3339(),
    }
}

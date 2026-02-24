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

mod graph_ops;
use graph_ops::relationship_to_edge;

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
}

//! # Storage Abstraction
//!
//! Unified trait for storage backends (Turso, redb, etc.)
//!
//! This allows the memory system to work with different storage implementations
//! transparently, supporting both durable (Turso) and cache (redb) layers.

pub mod circuit_breaker;

use crate::episode::{Direction, EpisodeRelationship, PatternId, RelationshipType};
use crate::{Episode, Heuristic, Pattern, Result};
use async_trait::async_trait;
use uuid::Uuid;

/// Unified storage backend trait
///
/// Provides a common interface for different storage implementations.
/// All operations are async to support both async (Turso) and sync (redb via `spawn_blocking`).
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Store an episode
    ///
    /// # Arguments
    ///
    /// * `episode` - Episode to store
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn store_episode(&self, episode: &Episode) -> Result<()>;

    /// Retrieve an episode by ID
    ///
    /// # Arguments
    ///
    /// * `id` - Episode UUID
    ///
    /// # Returns
    ///
    /// `Some(Episode)` if found, `None` if not found
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn get_episode(&self, id: Uuid) -> Result<Option<Episode>>;

    /// Delete an episode by ID
    ///
    /// # Arguments
    ///
    /// * `id` - Episode UUID
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn delete_episode(&self, id: Uuid) -> Result<()>;

    /// Store a pattern
    ///
    /// # Arguments
    ///
    /// * `pattern` - Pattern to store
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn store_pattern(&self, pattern: &Pattern) -> Result<()>;

    /// Retrieve a pattern by ID
    ///
    /// # Arguments
    ///
    /// * `id` - Pattern ID
    ///
    /// # Returns
    ///
    /// `Some(Pattern)` if found, `None` if not found
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn get_pattern(&self, id: PatternId) -> Result<Option<Pattern>>;

    /// Store a heuristic
    ///
    /// # Arguments
    ///
    /// * `heuristic` - Heuristic to store
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn store_heuristic(&self, heuristic: &Heuristic) -> Result<()>;

    /// Retrieve a heuristic by ID
    ///
    /// # Arguments
    ///
    /// * `id` - Heuristic UUID
    ///
    /// # Returns
    ///
    /// `Some(Heuristic)` if found, `None` if not found
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn get_heuristic(&self, id: Uuid) -> Result<Option<Heuristic>>;

    /// Query episodes modified since a given timestamp
    ///
    /// Used for incremental synchronization between storage layers.
    ///
    /// # Arguments
    ///
    /// * `since` - Timestamp to query from
    ///
    /// # Returns
    ///
    /// Vector of episodes with `start_time` >= since
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn query_episodes_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Episode>>;

    /// Query episodes by metadata key-value pair
    ///
    /// Used for specialized queries like monitoring data retrieval.
    ///
    /// # Arguments
    ///
    /// * `key` - Metadata key to search for
    /// * `value` - Metadata value to match
    ///
    /// # Returns
    ///
    /// Vector of episodes matching the metadata criteria
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn query_episodes_by_metadata(&self, key: &str, value: &str) -> Result<Vec<Episode>>;

    // ========== Embedding Storage Methods ==========

    /// Store embedding for an episode or pattern
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the embedding (e.g., `episode_id` or `pattern_id`)
    /// * `embedding` - Vector of f32 values representing the embedding
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn store_embedding(&self, id: &str, embedding: Vec<f32>) -> Result<()>;

    /// Retrieve embedding by ID
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the embedding
    ///
    /// # Returns
    ///
    /// `Some(Vec<f32>)` if found, `None` if not found
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn get_embedding(&self, id: &str) -> Result<Option<Vec<f32>>>;

    /// Delete embedding by ID
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the embedding
    ///
    /// # Returns
    ///
    /// `true` if deleted, `false` if not found
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn delete_embedding(&self, id: &str) -> Result<bool>;

    /// Store multiple embeddings in batch
    ///
    /// # Arguments
    ///
    /// * `embeddings` - Vector of (id, embedding) tuples
    ///
    /// # Errors
    ///
    /// Returns error if any storage operation fails
    async fn store_embeddings_batch(&self, embeddings: Vec<(String, Vec<f32>)>) -> Result<()>;

    /// Get embeddings for multiple IDs
    ///
    /// # Arguments
    ///
    /// * `ids` - Vector of embedding IDs
    ///
    /// # Returns
    ///
    /// Vector of `Option<Vec<f32>>` corresponding to each ID (None if not found)
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn get_embeddings_batch(&self, ids: &[String]) -> Result<Vec<Option<Vec<f32>>>>;

    // ========== Relationship Storage Methods ==========

    /// Store a relationship between two episodes
    ///
    /// # Arguments
    ///
    /// * `relationship` - The relationship to store
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn store_relationship(&self, relationship: &EpisodeRelationship) -> Result<()> {
        let _ = relationship;
        Ok(())
    }

    /// Remove a relationship by ID
    ///
    /// # Arguments
    ///
    /// * `relationship_id` - The UUID of the relationship to remove
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn remove_relationship(&self, relationship_id: Uuid) -> Result<()> {
        let _ = relationship_id;
        Ok(())
    }

    /// Get relationships for an episode
    ///
    /// # Arguments
    ///
    /// * `episode_id` - The episode to query
    /// * `direction` - Which relationships to return (Outgoing, Incoming, or Both)
    ///
    /// # Returns
    ///
    /// Vector of relationships matching the query
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn get_relationships(
        &self,
        episode_id: Uuid,
        direction: Direction,
    ) -> Result<Vec<EpisodeRelationship>> {
        let _ = (episode_id, direction);
        Ok(Vec::new())
    }

    /// Check if a relationship exists
    ///
    /// # Arguments
    ///
    /// * `from_episode_id` - Source episode
    /// * `to_episode_id` - Target episode  
    /// * `relationship_type` - Type of relationship
    ///
    /// # Returns
    ///
    /// `true` if the relationship exists
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn relationship_exists(
        &self,
        from_episode_id: Uuid,
        to_episode_id: Uuid,
        relationship_type: RelationshipType,
    ) -> Result<bool> {
        let _ = (from_episode_id, to_episode_id, relationship_type);
        Ok(false)
    }
}

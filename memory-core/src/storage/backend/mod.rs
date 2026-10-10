//! Storage backend trait definitions.
//!
//! Unified async interface implemented by Turso, redb, and in-memory backends.
//!
//! Optional operations are gated by [`StorageBackendCapabilities`]: their
//! default implementations return [`Error::CapabilityUnavailable`]
//! instead of a fabricated success, and callers can ask the capability
//! predicate before invoking them.

mod capabilities;

pub use capabilities::StorageBackendCapabilities;

use crate::episode::{
    CleanupResult, Direction, EpisodePatternRelationship, EpisodeRelationship,
    EpisodeRetentionPolicy, PatternId, RelationshipType,
};
use crate::memory::attribution::{
    RecommendationFeedback, RecommendationSession, RecommendationStats,
};
use crate::procedural::ProceduralMemory;
use crate::{Episode, Error, Heuristic, Pattern, Result};
use async_trait::async_trait;
use uuid::Uuid;

/// Unified storage backend trait
///
/// Provides a common interface for different storage implementations.
/// All operations are async to support both async (Turso) and sync (redb via `spawn_blocking`).
///
/// Operations that are not universally available are optional methods whose
/// default returns [`Error::CapabilityUnavailable`] rather than a fabricated
/// success. [`StorageBackendCapabilities`] advertises which of them a backend
/// truly persists.
///
/// Implementing this trait therefore also requires
/// [`StorageBackendCapabilities`]: the predicates default to `false`, so an
/// implementor adds `impl StorageBackendCapabilities for T {}` and overrides
/// `true` only for the domains it really persists.
#[async_trait]
pub trait StorageBackend: StorageBackendCapabilities + Send + Sync {
    /// Store an episode
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn store_episode(&self, episode: &Episode) -> Result<()>;

    /// Store a batch of episodes in one call.
    ///
    /// Backends with transactional batching (Turso) override this with a
    /// single-transaction implementation. The default loops over
    /// [`store_episode`](Self::store_episode), so existing implementors keep
    /// compiling and behaving identically. Episodes use `INSERT OR REPLACE`
    /// semantics wherever supported, making retried batches idempotent.
    ///
    /// # Arguments
    ///
    /// * `episodes` - Episodes to store
    ///
    /// # Errors
    ///
    /// Returns error if any storage operation fails
    async fn store_episodes_batch(&self, episodes: &[Episode]) -> Result<()> {
        for episode in episodes {
            self.store_episode(episode).await?;
        }
        Ok(())
    }

    /// Retrieve an episode by ID.
    ///
    /// Returns `Some(Episode)` if found, `None` if not found.
    async fn get_episode(&self, id: Uuid) -> Result<Option<Episode>>;

    /// Bounded liveness probe: one cheap read through the backend's real I/O path.
    ///
    /// Health reporting must not treat configuration as proof of connectivity (#1085).
    /// The default reads a known-absent episode; backends with a native ping override it.
    async fn health_check(&self) -> Result<()> {
        self.get_episode(Uuid::nil()).await.map(|_| ())
    }

    /// Delete an episode by ID
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn delete_episode(&self, id: Uuid) -> Result<()>;

    /// Store a pattern
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn store_pattern(&self, pattern: &Pattern) -> Result<()>;

    /// Retrieve a pattern by ID.
    ///
    /// Returns `Some(Pattern)` if found, `None` if not found.
    async fn get_pattern(&self, id: PatternId) -> Result<Option<Pattern>>;

    /// Retrieve all stored patterns.
    ///
    /// Backends that persist patterns should override this so that
    /// `pattern list` / `pattern search` work across process boundaries
    /// (the in-memory `patterns_fallback` is empty in a fresh CLI process).
    /// The default returns an empty list to avoid breaking backends that
    /// only implement single-pattern retrieval.
    async fn get_all_patterns(&self) -> Result<Vec<Pattern>> {
        Ok(Vec::new())
    }

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

    /// Retrieve a heuristic by ID.
    ///
    /// Returns `Some(Heuristic)` if found, `None` if not found.
    async fn get_heuristic(&self, id: Uuid) -> Result<Option<Heuristic>>;

    /// Query episodes modified since a given timestamp
    ///
    /// Used for incremental synchronization between storage layers.
    ///
    /// # Arguments
    ///
    /// * `since` - Timestamp to query from
    /// * `limit` - Maximum number of episodes to return (default: 100, max: 1000)
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
        limit: Option<usize>,
    ) -> Result<Vec<Episode>>;

    /// Query episodes by metadata key-value pair
    ///
    /// Used for specialized queries like monitoring data retrieval.
    ///
    /// # Arguments
    ///
    /// * `key` - Metadata key to search for
    /// * `value` - Metadata value to match
    /// * `limit` - Maximum number of episodes to return (default: 100, max: 1000)
    ///
    /// # Returns
    ///
    /// Vector of episodes matching the metadata criteria
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn query_episodes_by_metadata(
        &self,
        key: &str,
        value: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Episode>>;

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

    /// List the ids of embeddings persisted through [`store_embedding`](Self::store_embedding).
    ///
    /// Backends that persist generic embeddings must override this so the
    /// identity-scoped embedding adapter can enumerate persisted vectors. The
    /// default reports "this backend cannot enumerate" instead of an empty
    /// list: an empty list means "nothing is stored" and would make a similarity
    /// search over a non-enumerating backend look like a successful empty
    /// result.
    ///
    /// # Errors
    ///
    /// The default implementation returns [`Error::CapabilityUnavailable`].
    /// Returns error if the backend cannot enumerate its embeddings.
    async fn list_embedding_ids(&self) -> Result<Vec<String>> {
        Err(Error::CapabilityUnavailable {
            operation: "list_embedding_ids",
        })
    }

    // ========== Relationship Storage Methods ==========

    /// Store a relationship between two episodes.
    /// Optional: see [`StorageBackendCapabilities::supports_relationship_persistence`];
    /// the default returns [`Error::CapabilityUnavailable`].
    async fn store_relationship(&self, relationship: &EpisodeRelationship) -> Result<()> {
        let _ = relationship;
        Err(Error::capability_unavailable("store_relationship"))
    }

    /// Remove a relationship by ID.
    /// Optional: see [`StorageBackendCapabilities::supports_relationship_persistence`];
    /// the default returns [`Error::CapabilityUnavailable`].
    async fn remove_relationship(&self, relationship_id: Uuid) -> Result<()> {
        let _ = relationship_id;
        Err(Error::capability_unavailable("remove_relationship"))
    }

    /// Get an episode's relationships filtered by `direction` (Outgoing,
    /// Incoming, or Both).
    /// Optional: see [`StorageBackendCapabilities::supports_relationship_persistence`];
    /// the default returns [`Error::CapabilityUnavailable`].
    async fn get_relationships(
        &self,
        episode_id: Uuid,
        direction: Direction,
    ) -> Result<Vec<EpisodeRelationship>> {
        let _ = (episode_id, direction);
        Err(Error::capability_unavailable("get_relationships"))
    }

    /// Fetch every relationship across the entire store (WG-150 / WG-151, ADR-055).
    /// Optional: see [`StorageBackendCapabilities::supports_relationship_persistence`];
    /// the default returns [`Error::CapabilityUnavailable`].
    async fn get_all_relationships(&self) -> Result<Vec<EpisodeRelationship>> {
        Err(Error::capability_unavailable("get_all_relationships"))
    }

    /// Look up a single relationship by its ID (WG-150, ADR-055).
    /// Optional: see [`StorageBackendCapabilities::supports_relationship_persistence`];
    /// the default returns [`Error::CapabilityUnavailable`].
    async fn get_relationship_by_id(
        &self,
        relationship_id: Uuid,
    ) -> Result<Option<EpisodeRelationship>> {
        let _ = relationship_id;
        Err(Error::capability_unavailable("get_relationship_by_id"))
    }

    /// Check whether a directed relationship of `relationship_type` exists
    /// between two episodes.
    /// Optional: see [`StorageBackendCapabilities::supports_relationship_persistence`];
    /// the default returns [`Error::CapabilityUnavailable`].
    async fn relationship_exists(
        &self,
        from_episode_id: Uuid,
        to_episode_id: Uuid,
        relationship_type: RelationshipType,
    ) -> Result<bool> {
        let _ = (from_episode_id, to_episode_id, relationship_type);
        Err(Error::capability_unavailable("relationship_exists"))
    }

    /// Store a relationship between an episode and a pattern.
    /// Optional: see [`StorageBackendCapabilities::supports_relationship_persistence`];
    /// the default returns [`Error::CapabilityUnavailable`].
    async fn store_episode_pattern_relationship(
        &self,
        relationship: &EpisodePatternRelationship,
    ) -> Result<()> {
        let _ = relationship;
        Err(Error::capability_unavailable(
            "store_episode_pattern_relationship",
        ))
    }

    /// Get pattern relationships for an episode.
    /// Optional: see [`StorageBackendCapabilities::supports_relationship_persistence`];
    /// the default returns [`Error::CapabilityUnavailable`].
    async fn get_episode_pattern_relationships(
        &self,
        episode_id: Uuid,
    ) -> Result<Vec<EpisodePatternRelationship>> {
        let _ = episode_id;
        Err(Error::capability_unavailable(
            "get_episode_pattern_relationships",
        ))
    }

    /// Get weighted neighbors `(target_id, weight, is_pattern)` for an episode.
    /// Optional: see [`StorageBackendCapabilities::supports_relationship_persistence`];
    /// the default returns [`Error::CapabilityUnavailable`].
    async fn get_weighted_neighbors(&self, episode_id: Uuid) -> Result<Vec<(Uuid, f32, bool)>> {
        let _ = episode_id;
        Err(Error::capability_unavailable("get_weighted_neighbors"))
    }

    // ========== Recommendation Attribution (ADR-044) ==========

    // Recommendation-attribution and ranking-adaptation capabilities are
    // advertised through `StorageBackendCapabilities`.

    /// Persist a recommendation session for durability and analytics.
    async fn store_recommendation_session(&self, session: &RecommendationSession) -> Result<()> {
        let _ = session;
        Ok(())
    }

    /// Retrieve a recommendation session by ID.
    async fn get_recommendation_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<RecommendationSession>> {
        let _ = session_id;
        Ok(None)
    }

    /// Retrieve the most recent recommendation session for an episode.
    async fn get_recommendation_session_for_episode(
        &self,
        episode_id: Uuid,
    ) -> Result<Option<RecommendationSession>> {
        let _ = episode_id;
        Ok(None)
    }

    /// Persist feedback associated with a recommendation session.
    async fn store_recommendation_feedback(&self, feedback: &RecommendationFeedback) -> Result<()> {
        let _ = feedback;
        Ok(())
    }

    /// Retrieve feedback for a recommendation session.
    async fn get_recommendation_feedback(
        &self,
        session_id: Uuid,
    ) -> Result<Option<RecommendationFeedback>> {
        let _ = session_id;
        Ok(None)
    }

    /// Compute global recommendation statistics.
    async fn get_recommendation_stats(&self) -> Result<RecommendationStats> {
        Ok(RecommendationStats::default())
    }

    /// List persisted `RecommendationSession` (ADR-082); empty by default so
    /// non-capable backends contribute nothing.
    async fn list_recommendation_sessions(&self) -> Result<Vec<RecommendationSession>> {
        Ok(Vec::new())
    }

    /// List persisted `RecommendationFeedback` (ADR-082); empty by default so
    /// non-capable backends contribute nothing.
    async fn list_recommendation_feedback(&self) -> Result<Vec<RecommendationFeedback>> {
        Ok(Vec::new())
    }

    // ========== Episode GC/TTL (WG-075) ==========

    /// Clean up expired episodes based on `policy` (WG-075).
    /// Optional: see [`StorageBackendCapabilities::supports_episode_cleanup`];
    /// the default returns [`Error::CapabilityUnavailable`].
    async fn cleanup_episodes(&self, policy: &EpisodeRetentionPolicy) -> Result<CleanupResult> {
        let _ = policy;
        Err(Error::capability_unavailable("cleanup_episodes"))
    }

    /// Count episodes `policy` would clean up without deleting (dry run).
    /// Optional: see [`StorageBackendCapabilities::supports_episode_cleanup`];
    /// the default returns [`Error::CapabilityUnavailable`].
    async fn count_cleanup_candidates(&self, policy: &EpisodeRetentionPolicy) -> Result<usize> {
        let _ = policy;
        Err(Error::capability_unavailable("count_cleanup_candidates"))
    }

    // ========== Procedural Memory Methods ==========

    /// Store a procedural memory.
    /// Optional: see [`StorageBackendCapabilities::supports_procedural_memory`];
    /// the default returns [`Error::CapabilityUnavailable`].
    async fn store_procedural(&self, procedural: &ProceduralMemory) -> Result<()> {
        let _ = procedural;
        Err(Error::capability_unavailable("store_procedural"))
    }

    /// Retrieve a procedural memory by ID.
    /// Optional: see [`StorageBackendCapabilities::supports_procedural_memory`];
    /// the default returns [`Error::CapabilityUnavailable`].
    async fn get_procedural(&self, id: Uuid) -> Result<Option<ProceduralMemory>> {
        let _ = id;
        Err(Error::capability_unavailable("get_procedural"))
    }

    /// Delete a procedural memory by ID.
    /// Optional: see [`StorageBackendCapabilities::supports_procedural_memory`];
    /// the default returns [`Error::CapabilityUnavailable`].
    async fn delete_procedural(&self, id: Uuid) -> Result<()> {
        let _ = id;
        Err(Error::capability_unavailable("delete_procedural"))
    }

    /// Query procedural memories.
    /// Optional: see [`StorageBackendCapabilities::supports_procedural_memory`];
    /// the default returns [`Error::CapabilityUnavailable`].
    async fn query_procedural(&self, limit: Option<usize>) -> Result<Vec<ProceduralMemory>> {
        let _ = limit;
        Err(Error::capability_unavailable("query_procedural"))
    }
}

#[cfg(test)]
#[path = "backend_default_tests.rs"]
mod backend_default_tests;

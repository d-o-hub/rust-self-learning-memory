//! # Storage Abstraction
//!
//! Unified trait for storage backends (Turso, redb, etc.)
//!
//! This allows the memory system to work with different storage implementations
//! transparently, supporting both durable (Turso) and cache (redb) layers.

pub mod circuit_breaker;

use crate::episode::PatternId;
use crate::{Episode, Heuristic, Pattern, Result};
use async_trait::async_trait;
use uuid::Uuid;

/// Unified storage backend trait
///
/// Provides a common interface for different storage implementations.
/// All operations are async to support both async (Turso) and sync (redb via spawn_blocking).
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
    /// Vector of episodes with start_time >= since
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    async fn query_episodes_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Episode>>;
}

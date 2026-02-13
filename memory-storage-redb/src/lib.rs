#![allow(clippy::excessive_nesting)]

//! # Memory Storage - redb
//!
//! redb embedded database for fast cache layer.
//!
//! This crate provides:
//! - High-performance key-value storage using redb
//! - Zero-copy reads for fast retrieval
//! - Async wrappers for synchronous redb operations
//! - Episode and pattern caching
//! - Bincode serialization for efficient storage
//!
//! ## Example
//!
//! ```no_run
//! use memory_storage_redb::RedbStorage;
//! use std::path::Path;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let storage = RedbStorage::new(Path::new("./memory.redb")).await?;
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use memory_core::{episode::PatternId, Episode, Error, Heuristic, Pattern, Result, StorageBackend};
use redb::{Database, ReadableTable, ReadableTableMetadata, TableDefinition};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;
use uuid::Uuid;

mod cache;
mod embeddings;
mod embeddings_backend;
mod embeddings_impl;
mod episodes;
mod episodes_queries;
mod episodes_summaries;
mod heuristics;
mod patterns;
mod persistence;
mod relationships;
mod storage;
mod tables;

pub use cache::{
    AdaptiveCache, AdaptiveCacheConfig, AdaptiveCacheMetrics, CacheConfig, CacheMetrics, LRUCache,
};
pub use persistence::{
    CachePersistence, CacheSnapshot, PersistedCacheEntry, PersistenceConfig, PersistenceManager,
    PersistenceMode, PersistenceStats, PersistenceStrategy,
};
pub use storage::RedbQuery;

// ============================================================================
// Deserialization Limits (Security)
// ============================================================================

/// Maximum size for episode deserialization (10MB).
///
/// Prevents OOM attacks from maliciously large bincode payloads.
pub const MAX_EPISODE_SIZE: u64 = 10_000_000;

/// Maximum size for pattern deserialization (1MB).
///
/// Limits pattern data size to prevent resource exhaustion.
pub const MAX_PATTERN_SIZE: u64 = 1_000_000;

/// Maximum size for heuristic deserialization (100KB).
///
/// Restricts heuristic data size for security.
pub const MAX_HEURISTIC_SIZE: u64 = 100_000;

/// Maximum size for embedding deserialization (1MB).
///
/// Limits embedding vector size to prevent resource exhaustion.
/// Typical embedding dimensions (384-1536) * 4 bytes/f32 = ~1.5KB-6KB.
pub const MAX_EMBEDDING_SIZE: u64 = 1_000_000;

// Table definitions
pub(crate) const EPISODES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("episodes");
pub(crate) const PATTERNS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("patterns");
pub(crate) const HEURISTICS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("heuristics");
pub(crate) const EMBEDDINGS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("embeddings");
pub(crate) const METADATA_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("metadata");
pub(crate) const SUMMARIES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("summaries");
pub(crate) const RELATIONSHIPS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("relationships");

// ============================================================================
// Timeout Helper Functions
// ============================================================================

/// Timeout duration for database operations (10 seconds)
const DB_OPERATION_TIMEOUT: Duration = Duration::from_secs(10);

/// Execute a spawn_blocking operation with timeout
async fn with_db_timeout<T, F>(operation: F) -> crate::Result<T>
where
    F: FnOnce() -> crate::Result<T> + Send + 'static,
    T: Send + 'static,
{
    // spawn_blocking returns Result<T, JoinError>
    // timeout wraps that, so we get Result<Result<T, JoinError>, Elapsed>
    match tokio::time::timeout(DB_OPERATION_TIMEOUT, tokio::task::spawn_blocking(operation)).await {
        Ok(Ok(result)) => result, // Inner Ok is JoinError, outer Ok is timeout success
        Ok(Err(join_err)) => Err(Error::Storage(format!("Task join error: {}", join_err))),
        Err(_) => Err(Error::Storage(format!(
            "Database operation timed out after {:?}",
            DB_OPERATION_TIMEOUT
        ))),
    }
}

/// redb storage backend for fast caching
pub struct RedbStorage {
    pub(crate) db: Arc<Database>,
    pub(crate) cache: LRUCache,
}

impl RedbStorage {
    /// Create a new redb storage instance with default cache configuration
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the redb database file
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_storage_redb::RedbStorage;
    /// # use std::path::Path;
    /// # async fn example() -> anyhow::Result<()> {
    /// let storage = RedbStorage::new(Path::new("./memory.redb")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(path: &Path) -> Result<Self> {
        Self::new_with_cache_config(path, CacheConfig::default()).await
    }

    /// Create a new redb storage instance with custom cache configuration
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the redb database file
    /// * `cache_config` - Cache configuration settings
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_storage_redb::{RedbStorage, CacheConfig};
    /// # use std::path::Path;
    /// # async fn example() -> anyhow::Result<()> {
    /// let config = CacheConfig {
    ///     max_size: 500,
    ///     default_ttl_secs: 1800,
    ///     cleanup_interval_secs: 600,
    ///     enable_background_cleanup: true,
    /// };
    /// let storage = RedbStorage::new_with_cache_config(Path::new("./memory.redb"), config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new_with_cache_config(path: &Path, cache_config: CacheConfig) -> Result<Self> {
        info!("Opening redb database at {}", path.display());

        // Use spawn_blocking for synchronous redb initialization with timeout
        let path_buf = path.to_path_buf();
        let db = with_db_timeout(move || {
            Database::create(&path_buf)
                .map_err(|e| Error::Storage(format!("Failed to create redb database: {}", e)))
        })
        .await?;

        let cache = LRUCache::new(cache_config);
        let storage = Self {
            db: Arc::new(db),
            cache,
        };

        // Initialize tables
        storage.initialize_tables().await?;

        info!("Successfully opened redb database with LRU cache");
        Ok(storage)
    }

    /// Initialize database tables
    async fn initialize_tables(&self) -> Result<()> {
        let db = Arc::clone(&self.db);

        with_db_timeout(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            // Open tables to ensure they exist
            {
                let _episodes = write_txn
                    .open_table(EPISODES_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open episodes table: {}", e)))?;
                let _patterns = write_txn
                    .open_table(PATTERNS_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open patterns table: {}", e)))?;
                let _heuristics = write_txn.open_table(HEURISTICS_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open heuristics table: {}", e))
                })?;
                let _embeddings = write_txn.open_table(EMBEDDINGS_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open embeddings table: {}", e))
                })?;
                let _metadata = write_txn
                    .open_table(METADATA_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open metadata table: {}", e)))?;
                let _summaries = write_txn.open_table(SUMMARIES_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open summaries table: {}", e))
                })?;
                let _relationships = write_txn.open_table(RELATIONSHIPS_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open relationships table: {}", e))
                })?;
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await?;

        info!("Initialized redb tables");
        Ok(())
    }

    /// Get database statistics
    pub async fn get_statistics(&self) -> Result<StorageStatistics> {
        let db = Arc::clone(&self.db);

        with_db_timeout(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let episodes_table = read_txn
                .open_table(EPISODES_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open episodes table: {}", e)))?;
            let patterns_table = read_txn
                .open_table(PATTERNS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open patterns table: {}", e)))?;
            let heuristics_table = read_txn
                .open_table(HEURISTICS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open heuristics table: {}", e)))?;

            let episode_count = episodes_table
                .len()
                .map_err(|e| Error::Storage(format!("Failed to get episodes count: {}", e)))?
                as usize;

            let pattern_count = patterns_table
                .len()
                .map_err(|e| Error::Storage(format!("Failed to get patterns count: {}", e)))?
                as usize;

            let heuristic_count = heuristics_table
                .len()
                .map_err(|e| Error::Storage(format!("Failed to get heuristics count: {}", e)))?
                as usize;

            Ok(StorageStatistics {
                episode_count,
                pattern_count,
                heuristic_count,
            })
        })
        .await
    }

    /// Health check - verify database accessibility
    pub async fn health_check(&self) -> Result<bool> {
        let db = Arc::clone(&self.db);

        with_db_timeout(move || match db.begin_read() {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        })
        .await
    }

    /// Get cache metrics
    ///
    /// Returns current cache performance metrics including hit rate, miss rate,
    /// eviction count, and size statistics.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_storage_redb::RedbStorage;
    /// # use std::path::Path;
    /// # async fn example() -> anyhow::Result<()> {
    /// # let storage = RedbStorage::new(Path::new("./memory.redb")).await?;
    /// let metrics = storage.get_cache_metrics().await;
    /// println!("Cache hit rate: {:.2}%", metrics.hit_rate * 100.0);
    /// println!("Cache size: {} items, {} bytes", metrics.item_count, metrics.total_size_bytes);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_cache_metrics(&self) -> CacheMetrics {
        self.cache.get_metrics().await
    }

    /// Manually trigger cache cleanup to remove expired entries
    ///
    /// Returns the number of expired entries removed.
    ///
    /// This is useful for testing or when you want to force cleanup
    /// without waiting for the background task.
    pub async fn cleanup_cache(&self) -> usize {
        self.cache.cleanup_expired().await
    }

    /// Clear all cached data (use with caution!)
    pub async fn clear_all(&self) -> Result<()> {
        info!("Clearing all cached data from redb");

        // Clear the LRU cache metadata
        self.cache.clear().await;

        let db = Arc::clone(&self.db);

        with_db_timeout(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                // Clear episodes table
                let mut episodes = write_txn
                    .open_table(EPISODES_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open episodes table: {}", e)))?;
                let keys: Vec<String> = episodes
                    .iter()
                    .map_err(|e| Error::Storage(format!("Failed to iterate episodes: {}", e)))?
                    .filter_map(|item| item.ok())
                    .map(|(k, _v)| k.value().to_string())
                    .collect();
                for key in keys {
                    episodes.remove(key.as_str()).map_err(|e| {
                        Error::Storage(format!("Failed to remove episode key: {}", e))
                    })?;
                }
                drop(episodes);

                // Clear patterns table
                let mut patterns = write_txn
                    .open_table(PATTERNS_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open patterns table: {}", e)))?;
                let keys: Vec<String> = patterns
                    .iter()
                    .map_err(|e| Error::Storage(format!("Failed to iterate patterns: {}", e)))?
                    .filter_map(|item| item.ok())
                    .map(|(k, _v)| k.value().to_string())
                    .collect();
                for key in keys {
                    patterns.remove(key.as_str()).map_err(|e| {
                        Error::Storage(format!("Failed to remove pattern key: {}", e))
                    })?;
                }
                drop(patterns);

                // Clear heuristics table
                let mut heuristics = write_txn.open_table(HEURISTICS_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open heuristics table: {}", e))
                })?;
                let keys: Vec<String> = heuristics
                    .iter()
                    .map_err(|e| Error::Storage(format!("Failed to iterate heuristics: {}", e)))?
                    .filter_map(|item| item.ok())
                    .map(|(k, _v)| k.value().to_string())
                    .collect();
                for key in keys {
                    heuristics.remove(key.as_str()).map_err(|e| {
                        Error::Storage(format!("Failed to remove heuristic key: {}", e))
                    })?;
                }
                drop(heuristics);

                // Clear embeddings table
                let mut embeddings = write_txn.open_table(EMBEDDINGS_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open embeddings table: {}", e))
                })?;
                let keys: Vec<String> = embeddings
                    .iter()
                    .map_err(|e| Error::Storage(format!("Failed to iterate embeddings: {}", e)))?
                    .filter_map(|item| item.ok())
                    .map(|(k, _v)| k.value().to_string())
                    .collect();
                for key in keys {
                    embeddings.remove(key.as_str()).map_err(|e| {
                        Error::Storage(format!("Failed to remove embedding key: {}", e))
                    })?;
                }
                drop(embeddings);

                // Clear metadata table
                let mut metadata = write_txn
                    .open_table(METADATA_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open metadata table: {}", e)))?;
                let keys: Vec<String> = metadata
                    .iter()
                    .map_err(|e| Error::Storage(format!("Failed to iterate metadata: {}", e)))?
                    .filter_map(|item| item.ok())
                    .map(|(k, _v)| k.value().to_string())
                    .collect();
                for key in keys {
                    metadata.remove(key.as_str()).map_err(|e| {
                        Error::Storage(format!("Failed to remove metadata key: {}", e))
                    })?;
                }
                drop(metadata);

                // Clear summaries table
                let mut summaries = write_txn.open_table(SUMMARIES_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open summaries table: {}", e))
                })?;
                let keys: Vec<String> = summaries
                    .iter()
                    .map_err(|e| Error::Storage(format!("Failed to iterate summaries: {}", e)))?
                    .filter_map(|item| item.ok())
                    .map(|(k, _v)| k.value().to_string())
                    .collect();
                for key in keys {
                    summaries.remove(key.as_str()).map_err(|e| {
                        Error::Storage(format!("Failed to remove summary key: {}", e))
                    })?;
                }
                drop(summaries);

                // Clear relationships table
                let mut relationships = write_txn.open_table(RELATIONSHIPS_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open relationships table: {}", e))
                })?;
                let keys: Vec<String> = relationships
                    .iter()
                    .map_err(|e| Error::Storage(format!("Failed to iterate relationships: {}", e)))?
                    .filter_map(|item| item.ok())
                    .map(|(k, _v)| k.value().to_string())
                    .collect();
                for key in keys {
                    relationships.remove(key.as_str()).map_err(|e| {
                        Error::Storage(format!("Failed to remove relationship key: {}", e))
                    })?;
                }
                drop(relationships);
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await?;

        info!("Successfully cleared all cached data");
        Ok(())
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStatistics {
    pub episode_count: usize,
    pub pattern_count: usize,
    pub heuristic_count: usize,
}

/// Implement the unified StorageBackend trait for RedbStorage
#[async_trait]
impl StorageBackend for RedbStorage {
    async fn store_episode(&self, episode: &Episode) -> Result<()> {
        self.store_episode(episode).await
    }

    async fn get_episode(&self, id: Uuid) -> Result<Option<Episode>> {
        self.get_episode(id).await
    }

    async fn delete_episode(&self, id: Uuid) -> Result<()> {
        self.delete_episode(id).await
    }

    async fn store_pattern(&self, pattern: &Pattern) -> Result<()> {
        self.store_pattern(pattern).await
    }

    async fn get_pattern(&self, id: PatternId) -> Result<Option<Pattern>> {
        self.get_pattern(id).await
    }

    async fn store_heuristic(&self, heuristic: &Heuristic) -> Result<()> {
        self.store_heuristic(heuristic).await
    }

    async fn get_heuristic(&self, id: Uuid) -> Result<Option<Heuristic>> {
        self.get_heuristic(id).await
    }

    async fn query_episodes_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
        limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        self.query_episodes_since(since, limit).await
    }

    async fn query_episodes_by_metadata(
        &self,
        key: &str,
        value: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        self.query_episodes_by_metadata(key, value, limit).await
    }

    async fn store_embedding(&self, id: &str, embedding: Vec<f32>) -> Result<()> {
        self.store_embedding_impl(id, embedding).await
    }

    async fn get_embedding(&self, id: &str) -> Result<Option<Vec<f32>>> {
        self.get_embedding_impl(id).await
    }

    async fn delete_embedding(&self, id: &str) -> Result<bool> {
        self.delete_embedding_impl(id).await
    }

    async fn store_embeddings_batch(&self, embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        self.store_embeddings_batch_impl(embeddings).await
    }

    async fn get_embeddings_batch(&self, ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        self.get_embeddings_batch_impl(ids).await
    }

    // ========== Relationship Storage Methods ==========

    async fn store_relationship(
        &self,
        relationship: &memory_core::episode::EpisodeRelationship,
    ) -> Result<()> {
        self.store_relationship(relationship).await
    }

    async fn remove_relationship(&self, relationship_id: Uuid) -> Result<()> {
        self.remove_relationship(relationship_id).await
    }

    async fn get_relationships(
        &self,
        episode_id: Uuid,
        direction: memory_core::episode::Direction,
    ) -> Result<Vec<memory_core::episode::EpisodeRelationship>> {
        self.get_relationships(episode_id, direction).await
    }

    async fn relationship_exists(
        &self,
        from_episode_id: Uuid,
        to_episode_id: Uuid,
        relationship_type: memory_core::episode::RelationshipType,
    ) -> Result<bool> {
        self.relationship_exists(from_episode_id, to_episode_id, relationship_type)
            .await
    }
}

#[cfg(test)]
mod tests;

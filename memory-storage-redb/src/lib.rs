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
mod storage;
mod tables;

pub use cache::{CacheConfig, CacheMetrics, LRUCache};
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

        // Use spawn_blocking for synchronous redb initialization
        let path_buf = path.to_path_buf();
        let db = tokio::task::spawn_blocking(move || {
            Database::create(&path_buf)
                .map_err(|e| Error::Storage(format!("Failed to create redb database: {}", e)))
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

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

        tokio::task::spawn_blocking(move || {
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
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        info!("Initialized redb tables");
        Ok(())
    }

    /// Get database statistics
    pub async fn get_statistics(&self) -> Result<StorageStatistics> {
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
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
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

    /// Health check - verify database accessibility
    pub async fn health_check(&self) -> Result<bool> {
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || match db.begin_read() {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
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

        tokio::task::spawn_blocking(move || {
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
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

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
    ) -> Result<Vec<Episode>> {
        self.query_episodes_since(since).await
    }

    async fn query_episodes_by_metadata(&self, key: &str, value: &str) -> Result<Vec<Episode>> {
        self.query_episodes_by_metadata(key, value).await
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_storage() -> Result<RedbStorage> {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.redb");
        RedbStorage::new(&db_path).await
    }

    #[tokio::test]
    async fn test_storage_creation() {
        let storage = create_test_storage().await;
        assert!(storage.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let storage = create_test_storage().await.unwrap();
        let healthy = storage.health_check().await.unwrap();
        assert!(healthy);
    }

    #[tokio::test]
    async fn test_statistics() {
        let storage = create_test_storage().await.unwrap();
        let stats = storage.get_statistics().await.unwrap();
        assert_eq!(stats.episode_count, 0);
        assert_eq!(stats.pattern_count, 0);
        assert_eq!(stats.heuristic_count, 0);
    }

    #[tokio::test]
    async fn test_clear_all() {
        let storage = create_test_storage().await.unwrap();
        let result = storage.clear_all().await;
        assert!(result.is_ok());
    }

    // ========== Embedding Storage Tests ==========

    #[tokio::test]
    async fn test_store_and_get_embedding() {
        let storage = create_test_storage().await.unwrap();

        let id = "test_embedding_1";
        let embedding = vec![0.1_f32, 0.2, 0.3, 0.4];

        // Store embedding
        storage
            .store_embedding(id, embedding.clone())
            .await
            .unwrap();

        // Retrieve embedding
        let retrieved = storage.get_embedding(id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), embedding);
    }

    #[tokio::test]
    async fn test_get_nonexistent_embedding() {
        let storage = create_test_storage().await.unwrap();

        let retrieved = storage.get_embedding("nonexistent").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_delete_embedding() {
        let storage = create_test_storage().await.unwrap();

        let id = "test_embedding_delete";
        let embedding = vec![0.1_f32, 0.2, 0.3];

        // Store embedding
        storage
            .store_embedding(id, embedding.clone())
            .await
            .unwrap();

        // Verify it exists
        let retrieved = storage.get_embedding(id).await.unwrap();
        assert!(retrieved.is_some());

        // Delete embedding
        let deleted = storage.delete_embedding(id).await.unwrap();
        assert!(deleted);

        // Verify it's gone
        let retrieved = storage.get_embedding(id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_embedding() {
        let storage = create_test_storage().await.unwrap();

        let deleted = storage.delete_embedding("nonexistent").await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_store_embeddings_batch() {
        let storage = create_test_storage().await.unwrap();

        let embeddings = vec![
            ("batch_1".to_string(), vec![0.1_f32, 0.2, 0.3]),
            ("batch_2".to_string(), vec![0.4_f32, 0.5, 0.6]),
            ("batch_3".to_string(), vec![0.7_f32, 0.8, 0.9]),
        ];

        // Store embeddings in batch
        storage
            .store_embeddings_batch(embeddings.clone())
            .await
            .unwrap();

        // Verify all embeddings were stored
        for (id, expected_embedding) in &embeddings {
            let retrieved = storage.get_embedding(id).await.unwrap();
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap(), *expected_embedding);
        }
    }

    #[tokio::test]
    async fn test_get_embeddings_batch() {
        let storage = create_test_storage().await.unwrap();

        let embeddings = vec![
            ("get_batch_1".to_string(), vec![0.1_f32, 0.2]),
            ("get_batch_2".to_string(), vec![0.3_f32, 0.4]),
            ("get_batch_3".to_string(), vec![0.5_f32, 0.6]),
        ];

        // Store embeddings
        storage
            .store_embeddings_batch(embeddings.clone())
            .await
            .unwrap();

        // Get embeddings in batch
        let ids = vec![
            "get_batch_1".to_string(),
            "get_batch_2".to_string(),
            "get_batch_3".to_string(),
            "nonexistent".to_string(),
        ];

        let results = storage.get_embeddings_batch(&ids).await.unwrap();

        // Verify results
        assert_eq!(results.len(), 4);

        assert!(results[0].is_some());
        assert_eq!(results[0].as_ref().unwrap(), &embeddings[0].1);

        assert!(results[1].is_some());
        assert_eq!(results[1].as_ref().unwrap(), &embeddings[1].1);

        assert!(results[2].is_some());
        assert_eq!(results[2].as_ref().unwrap(), &embeddings[2].1);

        assert!(results[3].is_none()); // Nonexistent embedding
    }

    #[tokio::test]
    async fn test_different_embedding_dimensions() {
        let storage = create_test_storage().await.unwrap();

        // Test different dimensions (384, 1024, 1536)
        let dim_384: Vec<f32> = (0..384).map(|i| i as f32 / 384.0).collect();
        let dim_1024: Vec<f32> = (0..1024).map(|i| i as f32 / 1024.0).collect();
        let dim_1536: Vec<f32> = (0..1536).map(|i| i as f32 / 1536.0).collect();

        // Store different dimensions
        storage.store_embedding("dim_384", dim_384).await.unwrap();

        storage.store_embedding("dim_1024", dim_1024).await.unwrap();

        storage.store_embedding("dim_1536", dim_1536).await.unwrap();

        // Retrieve and verify dimensions
        let retrieved_384 = storage.get_embedding("dim_384").await.unwrap();
        assert!(retrieved_384.is_some());
        assert_eq!(retrieved_384.unwrap().len(), 384);

        let retrieved_1024 = storage.get_embedding("dim_1024").await.unwrap();
        assert!(retrieved_1024.is_some());
        assert_eq!(retrieved_1024.unwrap().len(), 1024);

        let retrieved_1536 = storage.get_embedding("dim_1536").await.unwrap();
        assert!(retrieved_1536.is_some());
        assert_eq!(retrieved_1536.unwrap().len(), 1536);
    }

    #[tokio::test]
    async fn test_update_existing_embedding() {
        let storage = create_test_storage().await.unwrap();

        let id = "update_test";
        let embedding_v1 = vec![0.1_f32, 0.2, 0.3];
        let embedding_v2 = vec![0.9_f32, 0.8, 0.7];

        // Store initial embedding
        storage
            .store_embedding(id, embedding_v1.clone())
            .await
            .unwrap();

        // Verify initial embedding
        let retrieved = storage.get_embedding(id).await.unwrap();
        assert_eq!(retrieved.unwrap(), embedding_v1);

        // Update embedding
        storage
            .store_embedding(id, embedding_v2.clone())
            .await
            .unwrap();

        // Verify updated embedding
        let retrieved = storage.get_embedding(id).await.unwrap();
        assert_eq!(retrieved.unwrap(), embedding_v2);
    }

    #[tokio::test]
    async fn test_empty_embeddings_batch() {
        let storage = create_test_storage().await.unwrap();

        // Store empty batch
        storage.store_embeddings_batch(vec![]).await.unwrap();

        // Get empty batch
        let results = storage.get_embeddings_batch(&[]).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_embedding_size_limit() {
        let storage = create_test_storage().await.unwrap();

        // Test that embeddings larger than MAX_EMBEDDING_SIZE (1MB) are rejected
        let large_embedding: Vec<f32> = vec![0.0_f32; 1_000_000]; // ~4MB

        let result = storage.store_embedding("too_large", large_embedding).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
    }
}

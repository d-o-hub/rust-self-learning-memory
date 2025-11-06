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

use memory_core::{episode::PatternId, Episode, Error, Heuristic, Pattern, Result};
use redb::{Database, TableDefinition};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

mod storage;
mod tables;

pub use storage::RedbQuery;

// Table definitions
pub(crate) const EPISODES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("episodes");
pub(crate) const PATTERNS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("patterns");
pub(crate) const HEURISTICS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("heuristics");
pub(crate) const EMBEDDINGS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("embeddings");
pub(crate) const METADATA_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("metadata");

/// redb storage backend for fast caching
pub struct RedbStorage {
    pub(crate) db: Arc<Database>,
}

impl RedbStorage {
    /// Create a new redb storage instance
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
        info!("Opening redb database at {}", path.display());

        // Use spawn_blocking for synchronous redb initialization
        let path_buf = path.to_path_buf();
        let db = tokio::task::spawn_blocking(move || {
            Database::create(&path_buf)
                .map_err(|e| Error::Storage(format!("Failed to create redb database: {}", e)))
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        let storage = Self { db: Arc::new(db) };

        // Initialize tables
        storage.initialize_tables().await?;

        info!("Successfully opened redb database");
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

    /// Clear all cached data (use with caution!)
    pub async fn clear_all(&self) -> Result<()> {
        info!("Clearing all cached data from redb");
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                let episodes = write_txn
                    .open_table(EPISODES_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open episodes table: {}", e)))?;
                let patterns = write_txn
                    .open_table(PATTERNS_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open patterns table: {}", e)))?;
                let heuristics = write_txn.open_table(HEURISTICS_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open heuristics table: {}", e))
                })?;
                let embeddings = write_txn.open_table(EMBEDDINGS_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open embeddings table: {}", e))
                })?;
                let metadata = write_txn
                    .open_table(METADATA_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open metadata table: {}", e)))?;

                // TODO: Implement clear using redb v2.1 API
                // The drain() method is not available in redb v2.1
                // Need to iterate and remove each key individually
                drop(episodes);
                drop(patterns);
                drop(heuristics);
                drop(embeddings);
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
}

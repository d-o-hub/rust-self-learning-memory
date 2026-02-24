//! Storage operations for RedbStorage
//!
//! Contains the implementation methods for RedbStorage.

use super::{
    CacheMetrics, EMBEDDINGS_TABLE, EPISODES_TABLE, HEURISTICS_TABLE, METADATA_TABLE,
    PATTERNS_TABLE, RELATIONSHIPS_TABLE, SUMMARIES_TABLE, with_db_timeout,
};
use crate::{RedbStorage, StorageStatistics};
use memory_core::{Error, Result};
use redb::{ReadableTable, ReadableTableMetadata};
use std::sync::Arc;
use tracing::info;

impl RedbStorage {
    /// Initialize database tables
    pub(super) async fn initialize_tables(&self) -> Result<()> {
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

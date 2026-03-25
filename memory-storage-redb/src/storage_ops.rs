//! Storage operations for RedbStorage
//!
//! Contains the implementation methods for RedbStorage.

use super::{
    CacheMetrics, EMBEDDINGS_TABLE, EPISODES_TABLE, HEURISTICS_TABLE, METADATA_TABLE,
    PATTERNS_TABLE, RECOMMENDATION_EPISODE_INDEX_TABLE, RECOMMENDATION_FEEDBACK_TABLE,
    RECOMMENDATION_SESSIONS_TABLE, RELATIONSHIPS_TABLE, SUMMARIES_TABLE, with_db_timeout,
    SCHEMA_VERSION, SCHEMA_VERSION_TABLE,
};
use crate::{RedbStorage, StorageStatistics};
use memory_core::{Error, Result};
use redb::{ReadableDatabase, ReadableTable, ReadableTableMetadata};
use std::sync::Arc;
use tracing::info;

impl RedbStorage {
    /// Initialize database tables with schema version check
    ///
    /// This method:
    /// 1. Opens all tables to ensure they exist
    /// 2. Checks the stored schema version against the current version
    /// 3. If versions differ, clears all cached data to prevent deserialization errors
    /// 4. Stores the new schema version
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
                let _rec_sessions = write_txn
                    .open_table(RECOMMENDATION_SESSIONS_TABLE)
                    .map_err(|e| {
                        Error::Storage(format!(
                            "Failed to open recommendation sessions table: {}",
                            e
                        ))
                    })?;
                let _rec_feedback = write_txn
                    .open_table(RECOMMENDATION_FEEDBACK_TABLE)
                    .map_err(|e| {
                        Error::Storage(format!(
                            "Failed to open recommendation feedback table: {}",
                            e
                        ))
                    })?;
                let _rec_episode = write_txn
                    .open_table(RECOMMENDATION_EPISODE_INDEX_TABLE)
                    .map_err(|e| {
                        Error::Storage(format!(
                            "Failed to open recommendation episode index: {}",
                            e
                        ))
                    })?;
                let _schema_version = write_txn
                    .open_table(SCHEMA_VERSION_TABLE)
                    .map_err(|e| {
                        Error::Storage(format!("Failed to open schema version table: {}", e))
                    })?;
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await?;

        // Check schema version and invalidate cache if needed
        self.check_and_update_schema_version().await?;

        info!("Initialized redb tables");
        Ok(())
    }

    /// Check schema version and clear cache if version mismatch
    ///
    /// This prevents deserialization errors when the Episode or other cached
    /// structs have been modified and the cached data is stale.
    async fn check_and_update_schema_version(&self) -> Result<()> {
        let db = Arc::clone(&self.db);
        let current_version = SCHEMA_VERSION;

        let needs_clear = with_db_timeout(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let version_table = read_txn
                .open_table(SCHEMA_VERSION_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open schema version table: {}", e)))?;

            let stored_version = version_table
                .get("version")
                .map_err(|e| Error::Storage(format!("Failed to read schema version: {}", e)))?
                .map(|guard| guard.value());

            match stored_version {
                Some(v) if v == current_version => {
                    info!("Schema version {} matches, cache is valid", current_version);
                    Ok(false)
                }
                Some(old_version) => {
                    info!(
                        "Schema version mismatch: stored={}, current={}. Clearing cache.",
                        old_version, current_version
                    );
                    Ok(true)
                }
                None => {
                    info!("No schema version found, storing version {}", current_version);
                    Ok(true)
                }
            }
        })
        .await?;

        if needs_clear {
            self.clear_all_tables().await?;
            self.store_schema_version().await?;
        }

        Ok(())
    }

    /// Store the current schema version
    async fn store_schema_version(&self) -> Result<()> {
        let db = Arc::clone(&self.db);
        let version = SCHEMA_VERSION;

        with_db_timeout(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                let mut version_table = write_txn
                    .open_table(SCHEMA_VERSION_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open schema version table: {}", e)))?;
                version_table
                    .insert("version", version)
                    .map_err(|e| Error::Storage(format!("Failed to store schema version: {}", e)))?;
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            info!("Stored schema version {}", version);
            Ok::<(), Error>(())
        })
        .await
    }

    /// Clear all tables (internal helper for schema version changes)
    async fn clear_all_tables(&self) -> Result<()> {
        info!("Clearing all tables due to schema version change");

        let db = Arc::clone(&self.db);

        with_db_timeout(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                // Clear each table by removing all entries (inline to avoid generic type issues)
                // Episodes
                let mut table = write_txn
                    .open_table(EPISODES_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open episodes table: {}", e)))?;
                let keys: Vec<String> = table
                    .iter()
                    .map_err(|e| Error::Storage(format!("Failed to iterate episodes: {}", e)))?
                    .filter_map(|item| item.ok())
                    .map(|(k, _v)| k.value().to_string())
                    .collect();
                for key in keys {
                    table.remove(key.as_str()).map_err(|e| {
                        Error::Storage(format!("Failed to remove episodes key: {}", e))
                    })?;
                }
                drop(table);

                // Patterns
                let mut table = write_txn
                    .open_table(PATTERNS_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open patterns table: {}", e)))?;
                let keys: Vec<String> = table
                    .iter()
                    .map_err(|e| Error::Storage(format!("Failed to iterate patterns: {}", e)))?
                    .filter_map(|item| item.ok())
                    .map(|(k, _v)| k.value().to_string())
                    .collect();
                for key in keys {
                    table.remove(key.as_str()).map_err(|e| {
                        Error::Storage(format!("Failed to remove patterns key: {}", e))
                    })?;
                }
                drop(table);

                // Heuristics
                let mut table = write_txn
                    .open_table(HEURISTICS_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open heuristics table: {}", e)))?;
                let keys: Vec<String> = table
                    .iter()
                    .map_err(|e| Error::Storage(format!("Failed to iterate heuristics: {}", e)))?
                    .filter_map(|item| item.ok())
                    .map(|(k, _v)| k.value().to_string())
                    .collect();
                for key in keys {
                    table.remove(key.as_str()).map_err(|e| {
                        Error::Storage(format!("Failed to remove heuristics key: {}", e))
                    })?;
                }
                drop(table);

                // Embeddings
                let mut table = write_txn
                    .open_table(EMBEDDINGS_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open embeddings table: {}", e)))?;
                let keys: Vec<String> = table
                    .iter()
                    .map_err(|e| Error::Storage(format!("Failed to iterate embeddings: {}", e)))?
                    .filter_map(|item| item.ok())
                    .map(|(k, _v)| k.value().to_string())
                    .collect();
                for key in keys {
                    table.remove(key.as_str()).map_err(|e| {
                        Error::Storage(format!("Failed to remove embeddings key: {}", e))
                    })?;
                }
                drop(table);

                // Metadata
                let mut table = write_txn
                    .open_table(METADATA_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open metadata table: {}", e)))?;
                let keys: Vec<String> = table
                    .iter()
                    .map_err(|e| Error::Storage(format!("Failed to iterate metadata: {}", e)))?
                    .filter_map(|item| item.ok())
                    .map(|(k, _v)| k.value().to_string())
                    .collect();
                for key in keys {
                    table.remove(key.as_str()).map_err(|e| {
                        Error::Storage(format!("Failed to remove metadata key: {}", e))
                    })?;
                }
                drop(table);

                // Summaries
                let mut table = write_txn
                    .open_table(SUMMARIES_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open summaries table: {}", e)))?;
                let keys: Vec<String> = table
                    .iter()
                    .map_err(|e| Error::Storage(format!("Failed to iterate summaries: {}", e)))?
                    .filter_map(|item| item.ok())
                    .map(|(k, _v)| k.value().to_string())
                    .collect();
                for key in keys {
                    table.remove(key.as_str()).map_err(|e| {
                        Error::Storage(format!("Failed to remove summaries key: {}", e))
                    })?;
                }
                drop(table);

                // Relationships
                let mut table = write_txn
                    .open_table(RELATIONSHIPS_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open relationships table: {}", e)))?;
                let keys: Vec<String> = table
                    .iter()
                    .map_err(|e| Error::Storage(format!("Failed to iterate relationships: {}", e)))?
                    .filter_map(|item| item.ok())
                    .map(|(k, _v)| k.value().to_string())
                    .collect();
                for key in keys {
                    table.remove(key.as_str()).map_err(|e| {
                        Error::Storage(format!("Failed to remove relationships key: {}", e))
                    })?;
                }
                drop(table);

                // Recommendation sessions
                let mut table = write_txn
                    .open_table(RECOMMENDATION_SESSIONS_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open recommendation_sessions table: {}", e)))?;
                let keys: Vec<String> = table
                    .iter()
                    .map_err(|e| Error::Storage(format!("Failed to iterate recommendation_sessions: {}", e)))?
                    .filter_map(|item| item.ok())
                    .map(|(k, _v)| k.value().to_string())
                    .collect();
                for key in keys {
                    table.remove(key.as_str()).map_err(|e| {
                        Error::Storage(format!("Failed to remove recommendation_sessions key: {}", e))
                    })?;
                }
                drop(table);

                // Recommendation feedback
                let mut table = write_txn
                    .open_table(RECOMMENDATION_FEEDBACK_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open recommendation_feedback table: {}", e)))?;
                let keys: Vec<String> = table
                    .iter()
                    .map_err(|e| Error::Storage(format!("Failed to iterate recommendation_feedback: {}", e)))?
                    .filter_map(|item| item.ok())
                    .map(|(k, _v)| k.value().to_string())
                    .collect();
                for key in keys {
                    table.remove(key.as_str()).map_err(|e| {
                        Error::Storage(format!("Failed to remove recommendation_feedback key: {}", e))
                    })?;
                }
                drop(table);

                // Recommendation episode index (different key type: &str -> &str)
                let mut table = write_txn
                    .open_table(RECOMMENDATION_EPISODE_INDEX_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open recommendation_episode_index table: {}", e)))?;
                let keys: Vec<String> = table
                    .iter()
                    .map_err(|e| Error::Storage(format!("Failed to iterate recommendation_episode_index: {}", e)))?
                    .filter_map(|item| item.ok())
                    .map(|(k, _v)| k.value().to_string())
                    .collect();
                for key in keys {
                    table.remove(key.as_str()).map_err(|e| {
                        Error::Storage(format!("Failed to remove recommendation_episode_index key: {}", e))
                    })?;
                }
                drop(table);
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await?;

        // Also clear the in-memory cache
        self.cache.clear().await;

        info!("Successfully cleared all tables");
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

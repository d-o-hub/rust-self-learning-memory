//! Episode query operations for redb cache

use crate::{RedbStorage, EPISODES_TABLE};
use memory_core::{Episode, Error, Result};
use redb::ReadableTable;
use std::sync::Arc;
use tracing::{debug, info};

impl RedbStorage {
    /// Query episodes modified since a given timestamp
    ///
    /// Returns all episodes where start_time >= the given timestamp.
    /// This is used for incremental synchronization.
    ///
    /// Note: This scans all episodes in the cache and filters by timestamp,
    /// which may be slow for large datasets. Consider using Turso for
    /// efficient timestamp-based queries.
    pub async fn query_episodes_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Episode>> {
        debug!("Querying episodes since {} from cache", since);
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let table = read_txn
                .open_table(EPISODES_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open episodes table: {}", e)))?;

            let mut episodes = Vec::new();
            let iter = table
                .iter()
                .map_err(|e| Error::Storage(format!("Failed to iterate episodes: {}", e)))?;

            for result in iter {
                let (_, bytes_guard) = result
                    .map_err(|e| Error::Storage(format!("Failed to read episode entry: {}", e)))?;

                let episode: Episode = postcard::from_bytes(bytes_guard.value())
                    .map_err(|e| Error::Storage(format!("Failed to deserialize episode: {}", e)))?;

                // Filter by timestamp
                if episode.start_time >= since {
                    episodes.push(episode);
                }
            }

            // Sort by start_time descending (most recent first)
            episodes.sort_by(|a, b| b.start_time.cmp(&a.start_time));

            info!("Found {} episodes since {} in cache", episodes.len(), since);
            Ok(episodes)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

    /// Query episodes by metadata key-value pair
    ///
    /// This method searches through all episodes and returns those whose metadata
    /// contains the specified key-value pair. This is less efficient than
    /// timestamp-based queries but necessary for metadata-based searches.
    ///
    /// # Arguments
    ///
    /// * `key` - Metadata key to search for
    /// * `value` - Metadata value to match
    ///
    /// # Returns
    ///
    /// Vector of episodes matching the metadata criteria
    pub async fn query_episodes_by_metadata(&self, key: &str, value: &str) -> Result<Vec<Episode>> {
        debug!("Querying episodes by metadata: {} = {}", key, value);
        let db = Arc::clone(&self.db);
        let key_str = key.to_string();
        let value_str = value.to_string();

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let table = read_txn
                .open_table(EPISODES_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open episodes table: {}", e)))?;

            let mut episodes = Vec::new();
            let iter = table
                .iter()
                .map_err(|e| Error::Storage(format!("Failed to iterate episodes: {}", e)))?;

            for result in iter {
                let (_, bytes_guard) = result
                    .map_err(|e| Error::Storage(format!("Failed to read episode entry: {}", e)))?;

                let episode: Episode = postcard::from_bytes(bytes_guard.value())
                    .map_err(|e| Error::Storage(format!("Failed to deserialize episode: {}", e)))?;

                // Check if metadata contains the key-value pair
                if let Some(metadata_value) = episode.metadata.get(key_str.as_str()) {
                    if metadata_value == value_str.as_str() {
                        episodes.push(episode);
                    }
                }
            }

            // Sort by start_time descending (most recent first)
            episodes.sort_by(|a, b| b.start_time.cmp(&a.start_time));

            info!(
                "Found {} episodes with metadata {} = {} in cache",
                episodes.len(),
                key_str,
                value_str
            );
            Ok(episodes)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }
}

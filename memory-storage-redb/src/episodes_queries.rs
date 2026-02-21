//! Episode query operations for redb cache

use crate::{EPISODES_TABLE, RedbStorage};
use memory_core::{Episode, Error, Result, apply_query_limit};
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
    ///
    /// # Arguments
    ///
    /// * `since` - Timestamp to query from
    /// * `limit` - Maximum number of episodes to return (default: 100, max: 1000)
    pub async fn query_episodes_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
        limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        // Apply limit with defaults and bounds
        let effective_limit = apply_query_limit(limit);
        debug!(
            "Querying episodes since {} from cache (limit: {})",
            since, effective_limit
        );
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
                // Check if we've hit the limit
                if episodes.len() >= effective_limit {
                    break;
                }

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

            // Apply limit after sorting (in case we collected more than limit during filtering)
            episodes.truncate(effective_limit);

            info!(
                "Found {} episodes since {} in cache (limit: {})",
                episodes.len(),
                since,
                effective_limit
            );
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
    /// * `limit` - Maximum number of episodes to return (default: 100, max: 1000)
    ///
    /// # Returns
    ///
    /// Vector of episodes matching the metadata criteria
    pub async fn query_episodes_by_metadata(
        &self,
        key: &str,
        value: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        // Apply limit with defaults and bounds
        let effective_limit = apply_query_limit(limit);
        debug!(
            "Querying episodes by metadata: {} = {} (limit: {})",
            key, value, effective_limit
        );
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
                // Check if we've hit the limit
                if episodes.len() >= effective_limit {
                    break;
                }

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

            // Apply limit after sorting
            episodes.truncate(effective_limit);

            info!(
                "Found {} episodes with metadata {} = {} in cache (limit: {})",
                episodes.len(),
                key_str,
                value_str,
                effective_limit
            );
            Ok(episodes)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }
}

//! Episode query operations for redb cache

use crate::{EPISODE_REVISIONS_TABLE, EPISODES_TABLE, RedbStorage};
use chrono::{DateTime, Utc};
use do_memory_core::{Episode, Error, Result, apply_query_limit};
use redb::{ReadableDatabase, ReadableTable, TableError};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

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
            episodes.sort_by_key(|b| std::cmp::Reverse(b.start_time));

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

    /// Query episodes modified at or after a watermark using keyset pagination.
    ///
    /// Orders by `(modified_at, episode_id)` ascending. `modified_at` comes
    /// from the `episode_revisions` side table; rows cached before that table
    /// existed fall back to `start_time`. When `cursor` is supplied the result
    /// resumes strictly after that tuple, so episodes sharing a timestamp are
    /// neither skipped nor duplicated.
    ///
    /// # Arguments
    ///
    /// * `since` - Inclusive modification watermark
    /// * `cursor` - Last `(modified_at, episode_id)` of the previous page
    /// * `limit` - Maximum number of episodes to return (default: 100, max: 1000)
    pub async fn query_episodes_modified_since(
        &self,
        since: DateTime<Utc>,
        cursor: Option<(DateTime<Utc>, Uuid)>,
        limit: Option<usize>,
    ) -> Result<Vec<(Episode, DateTime<Utc>)>> {
        // Apply limit with defaults and bounds
        let effective_limit = apply_query_limit(limit);
        debug!(
            "Querying episodes modified since {} from cache (limit: {}, cursor: {:?})",
            since, effective_limit, cursor
        );
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            // The revisions table may be absent in a cache written by an older
            // binary; treat that as "no watermark recorded" and fall back to
            // start_time below, which keeps the read additive.
            let mut revisions: HashMap<String, i64> = HashMap::new();
            match read_txn.open_table(EPISODE_REVISIONS_TABLE) {
                Ok(table) => {
                    for entry in table.iter().map_err(|e| {
                        Error::Storage(format!("Failed to iterate episode revisions: {}", e))
                    })? {
                        let (key, value) = entry.map_err(|e| {
                            Error::Storage(format!("Failed to read episode revision: {}", e))
                        })?;
                        revisions.insert(key.value().to_string(), value.value());
                    }
                }
                Err(TableError::TableDoesNotExist(_)) => {}
                Err(e) => {
                    return Err(Error::Storage(format!(
                        "Failed to open episode revisions table: {}",
                        e
                    )));
                }
            }

            let episodes_table = read_txn
                .open_table(EPISODES_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open episodes table: {}", e)))?;

            let mut episodes: Vec<(Episode, DateTime<Utc>)> = Vec::new();
            for result in episodes_table
                .iter()
                .map_err(|e| Error::Storage(format!("Failed to iterate episodes: {}", e)))?
            {
                let (key, bytes_guard) = result
                    .map_err(|e| Error::Storage(format!("Failed to read episode entry: {}", e)))?;

                let episode: Episode = postcard::from_bytes(bytes_guard.value())
                    .map_err(|e| Error::Storage(format!("Failed to deserialize episode: {}", e)))?;

                let modified_at = revisions
                    .get(key.value())
                    .and_then(|ms| DateTime::from_timestamp_millis(*ms))
                    .unwrap_or(episode.start_time);

                if modified_at < since {
                    continue;
                }
                if let Some((cursor_at, cursor_id)) = cursor {
                    if (modified_at, episode.episode_id) <= (cursor_at, cursor_id) {
                        continue;
                    }
                }

                episodes.push((episode, modified_at));
            }

            episodes.sort_by_key(|a| (a.1, a.0.episode_id));
            episodes.truncate(effective_limit);

            info!(
                "Found {} episodes modified since {} in cache (limit: {})",
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
            episodes.sort_by_key(|b| std::cmp::Reverse(b.start_time));

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

#[cfg(test)]
mod tests {
    use super::*;
    use do_memory_core::{Episode, TaskContext, TaskType};
    use tempfile::tempdir;

    async fn create_test_storage() -> Result<RedbStorage> {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.redb");
        RedbStorage::new(&db_path).await
    }

    #[tokio::test]
    async fn test_modified_since_sees_old_start_time() {
        let storage = create_test_storage().await.unwrap();

        // Given: an episode whose start_time predates the watermark window
        let mut episode = Episode::new(
            "old task".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        );
        episode.start_time = chrono::Utc::now() - chrono::Duration::hours(2);
        storage.store_episode(&episode).await.unwrap();

        let since = chrono::Utc::now() - chrono::Duration::hours(1);

        // Then: the start_time query misses it, the modification query finds it
        let by_start = storage.query_episodes_since(since, None).await.unwrap();
        assert_eq!(by_start.len(), 0);

        let by_modified = storage
            .query_episodes_modified_since(since, None, None)
            .await
            .unwrap();
        assert_eq!(by_modified.len(), 1);
        assert_eq!(by_modified[0].0.episode_id, episode.episode_id);
        assert!(by_modified[0].1 >= since);
    }

    #[tokio::test]
    async fn test_query_episodes_modified_since_pages_without_duplicates() {
        let storage = create_test_storage().await.unwrap();

        let mut ids = Vec::new();
        for i in 0..5 {
            let episode = Episode::new(
                format!("task-{i}"),
                TaskContext::default(),
                TaskType::CodeGeneration,
            );
            ids.push(episode.episode_id);
            storage.store_episode(&episode).await.unwrap();
        }

        let since = chrono::Utc::now() - chrono::Duration::hours(1);
        let mut seen = Vec::new();
        let mut cursor = None;
        for _ in 0..10 {
            let page = storage
                .query_episodes_modified_since(since, cursor, Some(2))
                .await
                .unwrap();
            if page.is_empty() {
                break;
            }
            let (last_episode, last_modified_at) = page.last().unwrap();
            cursor = Some((*last_modified_at, last_episode.episode_id));
            seen.extend(page.into_iter().map(|(episode, _)| episode.episode_id));
        }

        assert_eq!(seen.len(), 5, "no duplicates or omissions across pages");
        let unique: std::collections::HashSet<_> = seen.iter().copied().collect();
        assert_eq!(unique.len(), 5);
        for id in ids {
            assert!(
                unique.contains(&id),
                "episode {id} must appear exactly once"
            );
        }
    }

    #[tokio::test]
    async fn test_query_episodes_by_metadata_sorting() {
        let storage = create_test_storage().await.unwrap();
        let now = chrono::Utc::now();
        for i in 0..5 {
            let mut episode = Episode::new(
                format!("task-{}", i),
                TaskContext::default(),
                TaskType::CodeGeneration,
            );
            episode.start_time = now + chrono::Duration::minutes(i as i64);
            episode
                .metadata
                .insert("category".to_string(), "test".to_string());
            storage.store_episode(&episode).await.unwrap();
        }
        let results = storage
            .query_episodes_by_metadata("category", "test", None)
            .await
            .unwrap();
        assert_eq!(results.len(), 5);
        for i in 0..4 {
            assert!(results[i].start_time >= results[i + 1].start_time);
        }
    }
}

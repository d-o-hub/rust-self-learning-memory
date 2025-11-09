//! Storage operations for redb

use crate::{
    RedbStorage, EMBEDDINGS_TABLE, EPISODES_TABLE, HEURISTICS_TABLE, METADATA_TABLE, PATTERNS_TABLE,
};
use memory_core::{episode::PatternId, Episode, Error, Heuristic, Pattern, Result};
use redb::ReadableTable;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

/// Query options for redb (simpler than SQL, mainly for limiting results)
#[derive(Debug, Clone, Default)]
pub struct RedbQuery {
    pub limit: Option<usize>,
}

impl RedbStorage {
    /// Store an episode in cache
    pub async fn store_episode(&self, episode: &Episode) -> Result<()> {
        debug!("Storing episode in cache: {}", episode.episode_id);
        let db = Arc::clone(&self.db);
        let episode_id = episode.episode_id.to_string();
        let episode_bytes = bincode::serialize(episode)
            .map_err(|e| Error::Storage(format!("Failed to serialize episode: {}", e)))?;

        let byte_size = episode_bytes.len();

        tokio::task::spawn_blocking(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                let mut table = write_txn
                    .open_table(EPISODES_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open episodes table: {}", e)))?;

                table
                    .insert(episode_id.as_str(), episode_bytes.as_slice())
                    .map_err(|e| Error::Storage(format!("Failed to insert episode: {}", e)))?;
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        // Record cache miss (new item being added)
        self.cache
            .record_access(episode.episode_id, false, Some(byte_size))
            .await;

        info!("Successfully cached episode: {}", episode.episode_id);
        Ok(())
    }

    /// Retrieve an episode from cache
    pub async fn get_episode(&self, episode_id: Uuid) -> Result<Option<Episode>> {
        debug!("Retrieving episode from cache: {}", episode_id);
        let db = Arc::clone(&self.db);
        let episode_id_str = episode_id.to_string();

        let result = tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let table = read_txn
                .open_table(EPISODES_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open episodes table: {}", e)))?;

            match table
                .get(episode_id_str.as_str())
                .map_err(|e| Error::Storage(format!("Failed to get episode: {}", e)))?
            {
                Some(bytes_guard) => {
                    let bytes = bytes_guard.value();
                    let episode: Episode = bincode::deserialize(bytes).map_err(|e| {
                        Error::Storage(format!("Failed to deserialize episode: {}", e))
                    })?;
                    Ok::<Option<Episode>, Error>(Some(episode))
                }
                None => Ok::<Option<Episode>, Error>(None),
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        // Record cache access (hit if found, miss if not)
        let is_hit = result.is_some();
        self.cache.record_access(episode_id, is_hit, None).await;

        Ok(result)
    }

    /// Get all episodes from cache (with optional limit)
    pub async fn get_all_episodes(&self, query: &RedbQuery) -> Result<Vec<Episode>> {
        debug!("Retrieving all episodes from cache");
        let db = Arc::clone(&self.db);
        let limit = query.limit;

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

            for (count, result) in iter.enumerate() {
                if let Some(max) = limit {
                    if count >= max {
                        break;
                    }
                }

                let (_, bytes_guard) = result
                    .map_err(|e| Error::Storage(format!("Failed to read episode entry: {}", e)))?;

                let episode: Episode = bincode::deserialize(bytes_guard.value())
                    .map_err(|e| Error::Storage(format!("Failed to deserialize episode: {}", e)))?;

                episodes.push(episode);
            }

            Ok(episodes)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

    /// Delete an episode from cache
    pub async fn delete_episode(&self, episode_id: Uuid) -> Result<()> {
        debug!("Deleting episode from cache: {}", episode_id);
        let db = Arc::clone(&self.db);
        let episode_id_str = episode_id.to_string();

        tokio::task::spawn_blocking(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                let mut table = write_txn
                    .open_table(EPISODES_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open episodes table: {}", e)))?;

                table
                    .remove(episode_id_str.as_str())
                    .map_err(|e| Error::Storage(format!("Failed to delete episode: {}", e)))?;
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        // Remove from cache tracking
        self.cache.remove(episode_id).await;

        info!("Deleted episode from cache: {}", episode_id);
        Ok(())
    }

    /// Store a pattern in cache
    pub async fn store_pattern(&self, pattern: &Pattern) -> Result<()> {
        debug!("Storing pattern in cache: {}", pattern.id());
        let db = Arc::clone(&self.db);
        let pattern_id = pattern.id().to_string();
        let pattern_bytes = bincode::serialize(pattern)
            .map_err(|e| Error::Storage(format!("Failed to serialize pattern: {}", e)))?;

        tokio::task::spawn_blocking(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                let mut table = write_txn
                    .open_table(PATTERNS_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open patterns table: {}", e)))?;

                table
                    .insert(pattern_id.as_str(), pattern_bytes.as_slice())
                    .map_err(|e| Error::Storage(format!("Failed to insert pattern: {}", e)))?;
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        info!("Successfully cached pattern: {}", pattern.id());
        Ok(())
    }

    /// Retrieve a pattern from cache
    pub async fn get_pattern(&self, pattern_id: PatternId) -> Result<Option<Pattern>> {
        debug!("Retrieving pattern from cache: {}", pattern_id);
        let db = Arc::clone(&self.db);
        let pattern_id_str = pattern_id.to_string();

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let table = read_txn
                .open_table(PATTERNS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open patterns table: {}", e)))?;

            match table
                .get(pattern_id_str.as_str())
                .map_err(|e| Error::Storage(format!("Failed to get pattern: {}", e)))?
            {
                Some(bytes_guard) => {
                    let bytes = bytes_guard.value();
                    let pattern: Pattern = bincode::deserialize(bytes).map_err(|e| {
                        Error::Storage(format!("Failed to deserialize pattern: {}", e))
                    })?;
                    Ok(Some(pattern))
                }
                None => Ok(None),
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

    /// Get all patterns from cache (with optional limit)
    pub async fn get_all_patterns(&self, query: &RedbQuery) -> Result<Vec<Pattern>> {
        debug!("Retrieving all patterns from cache");
        let db = Arc::clone(&self.db);
        let limit = query.limit;

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let table = read_txn
                .open_table(PATTERNS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open patterns table: {}", e)))?;

            let mut patterns = Vec::new();
            let iter = table
                .iter()
                .map_err(|e| Error::Storage(format!("Failed to iterate patterns: {}", e)))?;

            for (count, result) in iter.enumerate() {
                if let Some(max) = limit {
                    if count >= max {
                        break;
                    }
                }

                let (_, bytes_guard) = result
                    .map_err(|e| Error::Storage(format!("Failed to read pattern entry: {}", e)))?;

                let pattern: Pattern = bincode::deserialize(bytes_guard.value())
                    .map_err(|e| Error::Storage(format!("Failed to deserialize pattern: {}", e)))?;

                patterns.push(pattern);
            }

            Ok(patterns)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

    /// Store a heuristic in cache
    pub async fn store_heuristic(&self, heuristic: &Heuristic) -> Result<()> {
        debug!("Storing heuristic in cache: {}", heuristic.heuristic_id);
        let db = Arc::clone(&self.db);
        let heuristic_id = heuristic.heuristic_id.to_string();
        let heuristic_bytes = bincode::serialize(heuristic)
            .map_err(|e| Error::Storage(format!("Failed to serialize heuristic: {}", e)))?;

        tokio::task::spawn_blocking(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                let mut table = write_txn.open_table(HEURISTICS_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open heuristics table: {}", e))
                })?;

                table
                    .insert(heuristic_id.as_str(), heuristic_bytes.as_slice())
                    .map_err(|e| Error::Storage(format!("Failed to insert heuristic: {}", e)))?;
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        info!("Successfully cached heuristic: {}", heuristic.heuristic_id);
        Ok(())
    }

    /// Retrieve a heuristic from cache
    pub async fn get_heuristic(&self, heuristic_id: Uuid) -> Result<Option<Heuristic>> {
        debug!("Retrieving heuristic from cache: {}", heuristic_id);
        let db = Arc::clone(&self.db);
        let heuristic_id_str = heuristic_id.to_string();

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let table = read_txn
                .open_table(HEURISTICS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open heuristics table: {}", e)))?;

            match table
                .get(heuristic_id_str.as_str())
                .map_err(|e| Error::Storage(format!("Failed to get heuristic: {}", e)))?
            {
                Some(bytes_guard) => {
                    let bytes = bytes_guard.value();
                    let heuristic: Heuristic = bincode::deserialize(bytes).map_err(|e| {
                        Error::Storage(format!("Failed to deserialize heuristic: {}", e))
                    })?;
                    Ok(Some(heuristic))
                }
                None => Ok(None),
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

    /// Get all heuristics from cache (with optional limit)
    pub async fn get_all_heuristics(&self, query: &RedbQuery) -> Result<Vec<Heuristic>> {
        debug!("Retrieving all heuristics from cache");
        let db = Arc::clone(&self.db);
        let limit = query.limit;

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let table = read_txn
                .open_table(HEURISTICS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open heuristics table: {}", e)))?;

            let mut heuristics = Vec::new();
            let iter = table
                .iter()
                .map_err(|e| Error::Storage(format!("Failed to iterate heuristics: {}", e)))?;

            for (count, result) in iter.enumerate() {
                if let Some(max) = limit {
                    if count >= max {
                        break;
                    }
                }

                let (_, bytes_guard) = result.map_err(|e| {
                    Error::Storage(format!("Failed to read heuristic entry: {}", e))
                })?;

                let heuristic: Heuristic =
                    bincode::deserialize(bytes_guard.value()).map_err(|e| {
                        Error::Storage(format!("Failed to deserialize heuristic: {}", e))
                    })?;

                heuristics.push(heuristic);
            }

            Ok(heuristics)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

    /// Store an embedding vector
    pub async fn store_embedding(&self, id: &str, embedding: &[f32]) -> Result<()> {
        debug!("Storing embedding: {}", id);
        let db = Arc::clone(&self.db);
        let id_str = id.to_string();
        let embedding_bytes = bincode::serialize(embedding)
            .map_err(|e| Error::Storage(format!("Failed to serialize embedding: {}", e)))?;

        tokio::task::spawn_blocking(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                let mut table = write_txn.open_table(EMBEDDINGS_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open embeddings table: {}", e))
                })?;

                table
                    .insert(id_str.as_str(), embedding_bytes.as_slice())
                    .map_err(|e| Error::Storage(format!("Failed to insert embedding: {}", e)))?;
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        Ok(())
    }

    /// Retrieve an embedding vector
    pub async fn get_embedding(&self, id: &str) -> Result<Option<Vec<f32>>> {
        debug!("Retrieving embedding: {}", id);
        let db = Arc::clone(&self.db);
        let id_str = id.to_string();

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let table = read_txn
                .open_table(EMBEDDINGS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open embeddings table: {}", e)))?;

            match table
                .get(id_str.as_str())
                .map_err(|e| Error::Storage(format!("Failed to get embedding: {}", e)))?
            {
                Some(bytes_guard) => {
                    let bytes = bytes_guard.value();
                    let embedding: Vec<f32> = bincode::deserialize(bytes).map_err(|e| {
                        Error::Storage(format!("Failed to deserialize embedding: {}", e))
                    })?;
                    Ok(Some(embedding))
                }
                None => Ok(None),
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

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

                let episode: Episode = bincode::deserialize(bytes_guard.value())
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

    /// Store metadata value
    pub async fn store_metadata(&self, key: &str, value: &str) -> Result<()> {
        debug!("Storing metadata: {} = {}", key, value);
        let db = Arc::clone(&self.db);
        let key_str = key.to_string();
        let value_bytes = value.as_bytes().to_vec();

        tokio::task::spawn_blocking(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                let mut table = write_txn
                    .open_table(METADATA_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open metadata table: {}", e)))?;

                table
                    .insert(key_str.as_str(), value_bytes.as_slice())
                    .map_err(|e| Error::Storage(format!("Failed to insert metadata: {}", e)))?;
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        Ok(())
    }

    /// Retrieve metadata value
    pub async fn get_metadata(&self, key: &str) -> Result<Option<String>> {
        debug!("Retrieving metadata: {}", key);
        let db = Arc::clone(&self.db);
        let key_str = key.to_string();

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let table = read_txn
                .open_table(METADATA_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open metadata table: {}", e)))?;

            match table
                .get(key_str.as_str())
                .map_err(|e| Error::Storage(format!("Failed to get metadata: {}", e)))?
            {
                Some(bytes_guard) => {
                    let bytes = bytes_guard.value();
                    let value = String::from_utf8(bytes.to_vec())
                        .map_err(|e| Error::Storage(format!("Failed to decode metadata: {}", e)))?;
                    Ok(Some(value))
                }
                None => Ok(None),
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }
}

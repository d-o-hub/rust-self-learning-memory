//! Storage operations for redb

use crate::{
    RedbStorage, EMBEDDINGS_TABLE, EPISODES_TABLE, HEURISTICS_TABLE, METADATA_TABLE,
    PATTERNS_TABLE, SUMMARIES_TABLE,
};
use async_trait::async_trait;
use memory_core::embeddings::{
    cosine_similarity, EmbeddingStorageBackend, SimilarityMetadata, SimilaritySearchResult,
};
use memory_core::episodic::CapacityManager;
use memory_core::semantic::EpisodeSummary;
use memory_core::{episode::PatternId, Episode, Error, Heuristic, Pattern, Result};
use redb::{ReadableTable, ReadableTableMetadata};
use std::sync::Arc;
use tracing::{debug, info, warn};
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
        let episode_bytes = postcard::to_allocvec(episode)
            .map_err(|e| Error::Storage(format!("Failed to serialize episode: {}", e)))?;

        let byte_size = episode_bytes.len();

        if byte_size > 100_000 {
            warn!(
                "Large cache payload detected: episode {}, size: {} bytes ({:.2} KB)",
                episode.episode_id,
                byte_size,
                byte_size as f64 / 1024.0
            );
        }

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
                    let _bytes = bytes_guard.value();
                    let payload_size = bytes_guard.value().len();
                    
                    if payload_size > 100_000 {
                        warn!(
                            "Large cache payload detected: episode {}, deserializing size: {} bytes ({:.2} KB)",
                            episode_id,
                            payload_size,
                            payload_size as f64 / 1024.0
                        );
                    }
                    
                    let episode: Episode =
                        postcard::from_bytes(bytes_guard.value()).map_err(|e| {
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

                let episode: Episode = postcard::from_bytes(bytes_guard.value())
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
        let pattern_bytes = postcard::to_allocvec(pattern)
            .map_err(|e| Error::Storage(format!("Failed to serialize pattern: {}", e)))?;

        if pattern_bytes.len() > 100_000 {
            warn!(
                "Large cache payload detected: pattern {}, size: {} bytes ({:.2} KB)",
                pattern.id(),
                pattern_bytes.len(),
                pattern_bytes.len() as f64 / 1024.0
            );
        }

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
                    let payload_size = bytes_guard.value().len();
                    
                    if payload_size > 100_000 {
                        warn!(
                            "Large cache payload detected: pattern {}, deserializing size: {} bytes ({:.2} KB)",
                            pattern_id,
                            payload_size,
                            payload_size as f64 / 1024.0
                        );
                    }
                    
                    let pattern: Pattern =
                        postcard::from_bytes(bytes_guard.value()).map_err(|e| {
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

                let pattern: Pattern = postcard::from_bytes(bytes_guard.value())
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
        let heuristic_bytes = postcard::to_allocvec(heuristic)
            .map_err(|e| Error::Storage(format!("Failed to serialize heuristic: {}", e)))?;

        if heuristic_bytes.len() > 100_000 {
            warn!(
                "Large cache payload detected: heuristic {}, size: {} bytes ({:.2} KB)",
                heuristic.heuristic_id,
                heuristic_bytes.len(),
                heuristic_bytes.len() as f64 / 1024.0
            );
        }

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
                    let payload_size = bytes_guard.value().len();
                    
                    if payload_size > 100_000 {
                        warn!(
                            "Large cache payload detected: heuristic {}, deserializing size: {} bytes ({:.2} KB)",
                            heuristic_id,
                            payload_size,
                            payload_size as f64 / 1024.0
                        );
                    }
                    
                    let heuristic: Heuristic =
                        postcard::from_bytes(bytes_guard.value()).map_err(|e| {
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
                    postcard::from_bytes(bytes_guard.value()).map_err(|e| {
                        Error::Storage(format!("Failed to deserialize heuristic: {}", e))
                    })?;

                heuristics.push(heuristic);
            }

            Ok(heuristics)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

    /// Store an embedding vector (internal method)
    pub async fn store_embedding_raw(&self, id: &str, embedding: &[f32]) -> Result<()> {
        debug!("Storing embedding: {}", id);
        let db = Arc::clone(&self.db);
        let id_str = id.to_string();
        let embedding_bytes = postcard::to_allocvec(embedding)
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

    /// Retrieve an embedding vector (internal method)
    pub async fn get_embedding_raw(&self, id: &str) -> Result<Option<Vec<f32>>> {
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
                    let _bytes = bytes_guard.value();
                    let embedding: Vec<f32> =
                        postcard::from_bytes(bytes_guard.value()).map_err(|e| {
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
                    let _bytes = bytes_guard.value();
                    let value = String::from_utf8(_bytes.to_vec())
                        .map_err(|e| Error::Storage(format!("Failed to decode metadata: {}", e)))?;
                    Ok(Some(value))
                }
                None => Ok(None),
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

    /// Store an episode summary.
    ///
    /// Stores a semantic summary of an episode for efficient retrieval.
    /// Uses postcard serialization for compact storage.
    ///
    /// # Arguments
    ///
    /// * `summary` - The episode summary to store
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, error otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use memory_storage_redb::RedbStorage;
    /// # use memory_core::semantic::EpisodeSummary;
    /// # use std::path::Path;
    /// # use uuid::Uuid;
    /// # use chrono::Utc;
    /// # async fn example() -> anyhow::Result<()> {
    /// # let storage = RedbStorage::new(Path::new("./test.redb")).await?;
    /// let summary = EpisodeSummary {
    ///     episode_id: Uuid::new_v4(),
    ///     summary_text: "Task completed successfully".to_string(),
    ///     key_concepts: vec!["rust".to_string(), "testing".to_string()],
    ///     key_steps: vec!["Step 1: Initialize".to_string()],
    ///     summary_embedding: None,
    ///     created_at: Utc::now(),
    /// };
    ///
    /// storage.store_episode_summary(&summary).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn store_episode_summary(&self, summary: &EpisodeSummary) -> Result<()> {
        debug!("Storing episode summary: {}", summary.episode_id);
        let db = Arc::clone(&self.db);
        let summary_id = summary.episode_id.to_string();
        let summary_bytes = postcard::to_allocvec(summary)
            .map_err(|e| Error::Storage(format!("Failed to serialize episode summary: {}", e)))?;

        tokio::task::spawn_blocking(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                let mut table = write_txn.open_table(SUMMARIES_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open summaries table: {}", e))
                })?;

                table
                    .insert(summary_id.as_str(), summary_bytes.as_slice())
                    .map_err(|e| {
                        Error::Storage(format!("Failed to insert episode summary: {}", e))
                    })?;
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        info!(
            "Successfully stored episode summary: {}",
            summary.episode_id
        );
        Ok(())
    }

    /// Retrieve an episode summary.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - ID of the episode whose summary to retrieve
    ///
    /// # Returns
    ///
    /// The episode summary if found, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use memory_storage_redb::RedbStorage;
    /// # use std::path::Path;
    /// # use uuid::Uuid;
    /// # async fn example() -> anyhow::Result<()> {
    /// # let storage = RedbStorage::new(Path::new("./test.redb")).await?;
    /// let episode_id = Uuid::new_v4();
    /// if let Some(summary) = storage.get_episode_summary(episode_id).await? {
    ///     println!("Summary: {}", summary.summary_text);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_episode_summary(&self, episode_id: Uuid) -> Result<Option<EpisodeSummary>> {
        debug!("Retrieving episode summary: {}", episode_id);
        let db = Arc::clone(&self.db);
        let episode_id_str = episode_id.to_string();

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let table = read_txn
                .open_table(SUMMARIES_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open summaries table: {}", e)))?;

            match table
                .get(episode_id_str.as_str())
                .map_err(|e| Error::Storage(format!("Failed to get episode summary: {}", e)))?
            {
                Some(bytes_guard) => {
                    let summary: EpisodeSummary = postcard::from_bytes(bytes_guard.value())
                        .map_err(|e| {
                            Error::Storage(format!("Failed to deserialize episode summary: {}", e))
                        })?;
                    Ok(Some(summary))
                }
                None => Ok(None),
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

    /// Store an episode with capacity enforcement.
    ///
    /// This method enforces capacity limits by evicting low-relevance episodes
    /// when storage is full. The eviction and insertion happen atomically in a
    /// single write transaction to ensure consistency.
    ///
    /// # Arguments
    ///
    /// * `episode` - The episode to store
    /// * `summary` - Optional episode summary to store alongside
    /// * `capacity_manager` - Manager that determines eviction policy
    ///
    /// # Returns
    ///
    /// `Ok(Some(evicted_ids))` if episodes were evicted, `Ok(None)` if no eviction needed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use memory_storage_redb::RedbStorage;
    /// # use memory_core::{Episode, TaskContext, TaskType};
    /// # use memory_core::episodic::{CapacityManager, EvictionPolicy};
    /// # use std::path::Path;
    /// # async fn example() -> anyhow::Result<()> {
    /// # let storage = RedbStorage::new(Path::new("./test.redb")).await?;
    /// let capacity_mgr = CapacityManager::new(100, EvictionPolicy::RelevanceWeighted);
    /// let episode = Episode::new(
    ///     "Test task".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::Testing,
    /// );
    ///
    /// let evicted = storage.store_episode_with_capacity(&episode, None, &capacity_mgr).await?;
    /// if let Some(evicted_ids) = evicted {
    ///     println!("Evicted {} episodes", evicted_ids.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn store_episode_with_capacity(
        &self,
        episode: &Episode,
        summary: Option<&EpisodeSummary>,
        capacity_manager: &CapacityManager,
    ) -> Result<Option<Vec<Uuid>>> {
        debug!(
            "Storing episode with capacity enforcement: {}",
            episode.episode_id
        );

        let db = Arc::clone(&self.db);
        let episode_clone = episode.clone();
        let summary_clone = summary.cloned();
        let capacity_manager_clone = capacity_manager.clone();

        let result = tokio::task::spawn_blocking(move || {
            // Begin a single write transaction for atomic evict-then-insert
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            let evicted_ids: Vec<Uuid>;

            {
                // 1. Get current episode count
                let episodes_table = write_txn
                    .open_table(EPISODES_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open episodes table: {}", e)))?;

                let current_count = episodes_table
                    .len()
                    .map_err(|e| Error::Storage(format!("Failed to get episode count: {}", e)))?
                    as usize;

                info!(
                    "Current episode count: {} (checking capacity)",
                    current_count
                );

                // 2. Check if we need to evict
                let need_eviction = !capacity_manager_clone.can_store(current_count);

                if need_eviction {
                    info!("Capacity limit reached, selecting episodes for eviction");

                    // Load all episodes to determine which to evict
                    let mut all_episodes = Vec::new();
                    let iter = episodes_table.iter().map_err(|e| {
                        Error::Storage(format!("Failed to iterate episodes: {}", e))
                    })?;

                    for result in iter {
                        let (_, bytes_guard) = result.map_err(|e| {
                            Error::Storage(format!("Failed to read episode entry: {}", e))
                        })?;

                        let ep: Episode =
                            postcard::from_bytes(bytes_guard.value()).map_err(|e| {
                                Error::Storage(format!("Failed to deserialize episode: {}", e))
                            })?;

                        all_episodes.push(ep);
                    }

                    // Determine which episodes to evict
                    evicted_ids = capacity_manager_clone.evict_if_needed(&all_episodes);

                    info!("Selected {} episodes for eviction", evicted_ids.len());
                } else {
                    evicted_ids = Vec::new();
                }

                drop(episodes_table); // Release the read-only table handle
            }

            // 3. Perform eviction if needed (in the same transaction)
            if !evicted_ids.is_empty() {
                let mut episodes_table = write_txn
                    .open_table(EPISODES_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open episodes table: {}", e)))?;

                let mut summaries_table = write_txn.open_table(SUMMARIES_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open summaries table: {}", e))
                })?;

                for evicted_id in &evicted_ids {
                    let evicted_id_str = evicted_id.to_string();

                    // Delete episode
                    episodes_table
                        .remove(evicted_id_str.as_str())
                        .map_err(|e| Error::Storage(format!("Failed to delete episode: {}", e)))?;

                    // Delete summary (if exists - no error if not found)
                    let _ = summaries_table.remove(evicted_id_str.as_str());
                }

                warn!("Evicted {} episodes to make room", evicted_ids.len());
            }

            // 4. Insert new episode
            {
                let episode_id = episode_clone.episode_id.to_string();
                let episode_bytes = postcard::to_allocvec(&episode_clone)
                    .map_err(|e| Error::Storage(format!("Failed to serialize episode: {}", e)))?;

                let mut episodes_table = write_txn
                    .open_table(EPISODES_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open episodes table: {}", e)))?;

                episodes_table
                    .insert(episode_id.as_str(), episode_bytes.as_slice())
                    .map_err(|e| Error::Storage(format!("Failed to insert episode: {}", e)))?;
            }

            // 5. Insert summary if provided
            if let Some(summary) = summary_clone {
                let summary_id = summary.episode_id.to_string();
                let summary_bytes = postcard::to_allocvec(&summary).map_err(|e| {
                    Error::Storage(format!("Failed to serialize episode summary: {}", e))
                })?;

                let mut summaries_table = write_txn.open_table(SUMMARIES_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open summaries table: {}", e))
                })?;

                summaries_table
                    .insert(summary_id.as_str(), summary_bytes.as_slice())
                    .map_err(|e| {
                        Error::Storage(format!("Failed to insert episode summary: {}", e))
                    })?;
            }

            // 6. Update episode count metadata
            {
                let episodes_table = write_txn
                    .open_table(EPISODES_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open episodes table: {}", e)))?;

                let new_count = episodes_table
                    .len()
                    .map_err(|e| Error::Storage(format!("Failed to get episode count: {}", e)))?
                    as usize;

                let mut metadata_table = write_txn
                    .open_table(METADATA_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open metadata table: {}", e)))?;

                metadata_table
                    .insert("episode_count", new_count.to_string().as_bytes())
                    .map_err(|e| {
                        Error::Storage(format!("Failed to update episode count: {}", e))
                    })?;

                info!("Updated episode count metadata: {} episodes", new_count);
            }

            // 7. Commit the transaction
            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            info!(
                "Successfully stored episode {} with capacity enforcement",
                episode_clone.episode_id
            );

            Ok::<Option<Vec<Uuid>>, Error>(if evicted_ids.is_empty() {
                None
            } else {
                Some(evicted_ids)
            })
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        // Record cache access for the new episode
        if let Ok(episode_bytes) = postcard::to_allocvec(episode) {
            self.cache
                .record_access(episode.episode_id, false, Some(episode_bytes.len()))
                .await;
        }

        Ok(result)
    }
}

#[async_trait]
impl EmbeddingStorageBackend for RedbStorage {
    async fn store_episode_embedding(&self, episode_id: Uuid, embedding: Vec<f32>) -> Result<()> {
        debug!("Storing episode embedding: {}", episode_id);
        let key = format!("episode_{}", episode_id);
        self.store_embedding_raw(&key, &embedding).await
    }

    async fn store_pattern_embedding(
        &self,
        pattern_id: PatternId,
        embedding: Vec<f32>,
    ) -> Result<()> {
        debug!("Storing pattern embedding: {}", pattern_id);
        let key = format!("pattern_{}", pattern_id);
        self.store_embedding_raw(&key, &embedding).await
    }

    async fn get_episode_embedding(&self, episode_id: Uuid) -> Result<Option<Vec<f32>>> {
        debug!("Retrieving episode embedding: {}", episode_id);
        let key = format!("episode_{}", episode_id);
        self.get_embedding_raw(&key).await
    }

    async fn get_pattern_embedding(&self, pattern_id: PatternId) -> Result<Option<Vec<f32>>> {
        debug!("Retrieving pattern embedding: {}", pattern_id);
        let key = format!("pattern_{}", pattern_id);
        self.get_embedding_raw(&key).await
    }

    async fn find_similar_episodes(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Episode>>> {
        debug!(
            "Finding similar episodes (limit: {}, threshold: {})",
            limit, threshold
        );

        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let embeddings_table = read_txn
                .open_table(EMBEDDINGS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open embeddings table: {}", e)))?;

            let episodes_table = read_txn
                .open_table(EPISODES_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open episodes table: {}", e)))?;

            let mut results = Vec::new();
            let iter = embeddings_table
                .iter()
                .map_err(|e| Error::Storage(format!("Failed to iterate embeddings: {}", e)))?;

            for result in iter {
                let (key_bytes, embedding_bytes_guard) = result.map_err(|e| {
                    Error::Storage(format!("Failed to read embedding entry: {}", e))
                })?;

                let key = key_bytes.value();

                // Only process episode embeddings
                if !key.starts_with("episode_") {
                    continue;
                }

                let embedding: Vec<f32> = postcard::from_bytes(embedding_bytes_guard.value())
                    .map_err(|e| {
                        Error::Storage(format!("Failed to deserialize embedding: {}", e))
                    })?;

                let similarity = cosine_similarity(&query_embedding, &embedding);

                if similarity >= threshold {
                    // Extract episode ID from key
                    let episode_id_str = &key[8..]; // Remove "episode_" prefix
                    if let Ok(_episode_id) = Uuid::parse_str(episode_id_str) {
                        // Try to get the episode
                        if let Some(episode_bytes) = episodes_table
                            .get(episode_id_str)
                            .map_err(|e| Error::Storage(format!("Failed to get episode: {}", e)))?
                        {
                            let episode: Episode = postcard::from_bytes(episode_bytes.value())
                                .map_err(|e| {
                                    Error::Storage(format!("Failed to deserialize episode: {}", e))
                                })?;

                            results.push(SimilaritySearchResult {
                                item: episode,
                                similarity,
                                metadata: SimilarityMetadata {
                                    embedding_model: "unknown".to_string(),
                                    embedding_timestamp: None,
                                    context: serde_json::json!({}),
                                },
                            });
                        }
                    }
                }
            }

            // Sort by similarity (highest first)
            results.sort_by(|a, b| {
                b.similarity
                    .partial_cmp(&a.similarity)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // Limit results
            results.truncate(limit);

            Ok(results)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

    async fn find_similar_patterns(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Pattern>>> {
        debug!(
            "Finding similar patterns (limit: {}, threshold: {})",
            limit, threshold
        );

        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let embeddings_table = read_txn
                .open_table(EMBEDDINGS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open embeddings table: {}", e)))?;

            let patterns_table = read_txn
                .open_table(PATTERNS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open patterns table: {}", e)))?;

            let mut results = Vec::new();
            let iter = embeddings_table
                .iter()
                .map_err(|e| Error::Storage(format!("Failed to iterate embeddings: {}", e)))?;

            for result in iter {
                let (key_bytes, embedding_bytes_guard) = result.map_err(|e| {
                    Error::Storage(format!("Failed to read embedding entry: {}", e))
                })?;

                let key = key_bytes.value();

                // Only process pattern embeddings
                if !key.starts_with("pattern_") {
                    continue;
                }

                let embedding: Vec<f32> = postcard::from_bytes(embedding_bytes_guard.value())
                    .map_err(|e| {
                        Error::Storage(format!("Failed to deserialize embedding: {}", e))
                    })?;

                let similarity = cosine_similarity(&query_embedding, &embedding);

                if similarity >= threshold {
                    // Extract pattern ID from key
                    let pattern_id_str = &key[8..]; // Remove "pattern_" prefix
                    if let Ok(_pattern_id) = PatternId::parse_str(pattern_id_str) {
                        // Try to get the pattern
                        if let Some(pattern_bytes) = patterns_table
                            .get(pattern_id_str)
                            .map_err(|e| Error::Storage(format!("Failed to get pattern: {}", e)))?
                        {
                            let pattern: Pattern = postcard::from_bytes(pattern_bytes.value())
                                .map_err(|e| {
                                    Error::Storage(format!("Failed to deserialize pattern: {}", e))
                                })?;

                            results.push(SimilaritySearchResult {
                                item: pattern,
                                similarity,
                                metadata: SimilarityMetadata {
                                    embedding_model: "unknown".to_string(),
                                    embedding_timestamp: None,
                                    context: serde_json::json!({}),
                                },
                            });
                        }
                    }
                }
            }

            // Sort by similarity (highest first)
            results.sort_by(|a, b| {
                b.similarity
                    .partial_cmp(&a.similarity)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // Limit results
            results.truncate(limit);

            Ok(results)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }
}

// ========== Helper Implementation Methods for StorageBackend ==========

impl RedbStorage {
    /// Store embedding implementation
    pub async fn store_embedding_impl(&self, id: &str, embedding: Vec<f32>) -> Result<()> {
        debug!("Storing embedding via StorageBackend: {}", id);

        // Validate embedding size
        let embedding_bytes = postcard::to_allocvec(&embedding)
            .map_err(|e| Error::Storage(format!("Failed to serialize embedding: {}", e)))?;

        if embedding_bytes.len() as u64 > crate::MAX_EMBEDDING_SIZE {
            return Err(Error::Storage(format!(
                "Embedding size {} exceeds maximum of {}",
                embedding_bytes.len(),
                crate::MAX_EMBEDDING_SIZE
            )));
        }

        let db = Arc::clone(&self.db);
        let id_str = id.to_string();

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

        info!("Successfully stored embedding: {}", id);
        Ok(())
    }

    /// Retrieve embedding implementation
    pub async fn get_embedding_impl(&self, id: &str) -> Result<Option<Vec<f32>>> {
        debug!("Retrieving embedding via StorageBackend: {}", id);

        let db = Arc::clone(&self.db);
        let id_str = id.to_string();

        let result = tokio::task::spawn_blocking(move || {
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
                    let _bytes = bytes_guard.value();

                    // Validate size before deserializing
                    if _bytes.len() as u64 > crate::MAX_EMBEDDING_SIZE {
                        return Err(Error::Storage(format!(
                            "Embedding size {} exceeds maximum of {}",
                            _bytes.len(),
                            crate::MAX_EMBEDDING_SIZE
                        )));
                    }

                    let embedding: Vec<f32> =
                        postcard::from_bytes(bytes_guard.value()).map_err(|e| {
                            Error::Storage(format!("Failed to deserialize embedding: {}", e))
                        })?;
                    Ok::<Option<Vec<f32>>, Error>(Some(embedding))
                }
                None => Ok::<Option<Vec<f32>>, Error>(None),
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        Ok(result)
    }

    /// Delete embedding implementation
    pub async fn delete_embedding_impl(&self, id: &str) -> Result<bool> {
        debug!("Deleting embedding via StorageBackend: {}", id);

        let db = Arc::clone(&self.db);
        let id_str = id.to_string();

        let result = tokio::task::spawn_blocking(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            let existed = {
                let mut table = write_txn.open_table(EMBEDDINGS_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open embeddings table: {}", e))
                })?;

                let existed = table
                    .get(id_str.as_str())
                    .map_err(|e| Error::Storage(format!("Failed to check embedding: {}", e)))?
                    .is_some();

                if existed {
                    table.remove(id_str.as_str()).map_err(|e| {
                        Error::Storage(format!("Failed to delete embedding: {}", e))
                    })?;
                }

                existed
            };

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<bool, Error>(existed)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        if result {
            info!("Deleted embedding: {}", id);
        } else {
            debug!("Embedding not found for deletion: {}", id);
        }

        Ok(result)
    }

    /// Store multiple embeddings in batch implementation
    pub async fn store_embeddings_batch_impl(
        &self,
        embeddings: Vec<(String, Vec<f32>)>,
    ) -> Result<()> {
        debug!("Storing {} embeddings in batch", embeddings.len());

        if embeddings.is_empty() {
            return Ok(());
        }

        let db = Arc::clone(&self.db);
        let count = embeddings.len();

        tokio::task::spawn_blocking(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                let mut table = write_txn.open_table(EMBEDDINGS_TABLE).map_err(|e| {
                    Error::Storage(format!("Failed to open embeddings table: {}", e))
                })?;

                for (id, embedding) in embeddings {
                    let embedding_bytes = postcard::to_allocvec(&embedding).map_err(|e| {
                        Error::Storage(format!("Failed to serialize embedding: {}", e))
                    })?;

                    // Validate size
                    if embedding_bytes.len() as u64 > crate::MAX_EMBEDDING_SIZE {
                        return Err(Error::Storage(format!(
                            "Embedding size {} exceeds maximum of {}",
                            embedding_bytes.len(),
                            crate::MAX_EMBEDDING_SIZE
                        )));
                    }

                    table
                        .insert(id.as_str(), embedding_bytes.as_slice())
                        .map_err(|e| {
                            Error::Storage(format!("Failed to insert embedding: {}", e))
                        })?;
                }
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        info!("Successfully stored {} embeddings in batch", count);
        Ok(())
    }

    /// Get multiple embeddings in batch implementation
    pub async fn get_embeddings_batch_impl(&self, ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        debug!("Retrieving {} embeddings in batch", ids.len());

        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let db = Arc::clone(&self.db);
        let ids_clone = ids.to_vec();

        let results_map = tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let table = read_txn
                .open_table(EMBEDDINGS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open embeddings table: {}", e)))?;

            let mut results_map = std::collections::HashMap::new();

            for id in &ids_clone {
                match table
                    .get(id.as_str())
                    .map_err(|e| Error::Storage(format!("Failed to get embedding: {}", e)))?
                {
                    Some(bytes_guard) => {
                        let _bytes = bytes_guard.value();

                        // Validate size before deserializing
                        if _bytes.len() as u64 <= crate::MAX_EMBEDDING_SIZE {
                            let embedding: Vec<f32> = postcard::from_bytes(bytes_guard.value())
                                .map_err(|e| {
                                    Error::Storage(format!(
                                        "Failed to deserialize embedding: {}",
                                        e
                                    ))
                                })?;
                            results_map.insert(id.clone(), Some(embedding));
                        } else {
                            results_map.insert(id.clone(), None);
                        }
                    }
                    None => {
                        results_map.insert(id.clone(), None);
                    }
                }
            }

            Ok::<std::collections::HashMap<String, Option<Vec<f32>>>, Error>(results_map)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        // Map results to maintain original order
        let results: Vec<Option<Vec<f32>>> = ids
            .iter()
            .map(|id| results_map.get(id).and_then(|o| o.clone()))
            .collect();

        info!("Retrieved {} embeddings from batch request", results.len());
        Ok(results)
    }
}

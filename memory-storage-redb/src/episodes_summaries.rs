//! Episode summaries and capacity operations for redb cache

use crate::{EPISODES_TABLE, METADATA_TABLE, RedbStorage, SUMMARIES_TABLE};
use memory_core::episodic::CapacityManager;
use memory_core::semantic::EpisodeSummary;
use memory_core::{Episode, Error, Result};
use redb::{ReadableTable, ReadableTableMetadata};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

impl RedbStorage {
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

//! Episode storage operations for redb cache

use crate::{RedbStorage, EPISODES_TABLE};
use memory_core::{Episode, Error, Result};
use redb::ReadableTable;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

/// Query options for episodes
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
        // Only track hits in the cache - misses should not add entries
        if let Some(episode) = &result {
            let episode_bytes = postcard::to_allocvec(episode)
                .map_err(|e| Error::Storage(format!("Failed to serialize episode: {}", e)))?;
            self.cache
                .record_access(episode_id, true, Some(episode_bytes.len()))
                .await;
        }

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
}

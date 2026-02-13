//! Storage synchronizer for coordinating Turso and redb

use crate::{Error, Result, MAX_QUERY_LIMIT};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tracing::{debug, error, info};
use uuid::Uuid;

use super::types::{SyncState, SyncStats};

// ============================================================================
// Timeout Constants
// ============================================================================

/// Timeout for sync episode operations (30 seconds)
const SYNC_EPISODE_TIMEOUT: Duration = Duration::from_secs(30);

/// Timeout for sync all episodes operations (60 seconds)
const SYNC_ALL_TIMEOUT: Duration = Duration::from_secs(60);

/// Storage synchronizer for coordinating Turso and redb
pub struct StorageSynchronizer<T, R> {
    /// Source storage (typically Turso - durable)
    pub turso: Arc<T>,
    /// Cache storage (typically redb - fast)
    pub redb: Arc<R>,
    sync_state: Arc<RwLock<SyncState>>,
}

impl<T, R> StorageSynchronizer<T, R> {
    /// Create a new storage synchronizer
    pub fn new(turso: Arc<T>, redb: Arc<R>) -> Self {
        Self {
            turso,
            redb,
            sync_state: Arc::new(RwLock::new(SyncState::default())),
        }
    }

    /// Get the current synchronization state
    pub async fn get_sync_state(&self) -> SyncState {
        self.sync_state.read().await.clone()
    }

    /// Update sync state after a successful sync
    async fn update_sync_state(&self, episodes_synced: usize, errors: usize) {
        let mut state = self.sync_state.write().await;
        state.last_sync = Some(chrono::Utc::now());
        state.sync_count += 1;
        if errors > 0 {
            state.last_error = Some(format!(
                "Synced {episodes_synced} episodes with {errors} errors"
            ));
        } else {
            state.last_error = None;
        }
    }
}

// Concrete implementations using the StorageBackend trait

impl<T, R> StorageSynchronizer<T, R>
where
    T: crate::storage::StorageBackend + 'static,
    R: crate::storage::StorageBackend + 'static,
{
    /// Sync a single episode from Turso (source) to redb (cache)
    ///
    /// Fetches the episode from the source storage and stores it in the cache storage.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - UUID of the episode to sync
    ///
    /// # Errors
    ///
    /// Returns error if episode not found or storage operation fails
    pub async fn sync_episode_to_cache(&self, episode_id: Uuid) -> Result<()> {
        let correlation_id = Uuid::new_v4();

        info!(correlation_id = %correlation_id, "Syncing episode {} to cache", episode_id);

        // Fetch from Turso (source of truth) with timeout
        // timeout returns Result<Result<Option<Episode>, Error>, Elapsed>
        let episode = match timeout(SYNC_EPISODE_TIMEOUT, self.turso.get_episode(episode_id)).await
        {
            Ok(Ok(Some(episode))) => episode,
            Ok(Ok(None)) => {
                return Err(Error::Storage(format!(
                    "Episode {episode_id} not found in source storage"
                )))
            }
            Ok(Err(e)) => return Err(Error::Storage(format!("Error fetching episode: {e}"))),
            Err(_) => {
                return Err(Error::Storage(format!(
                    "Timeout fetching episode {episode_id} after {SYNC_EPISODE_TIMEOUT:?}"
                )))
            }
        };

        // Store in redb cache with timeout
        match timeout(SYNC_EPISODE_TIMEOUT, self.redb.store_episode(&episode)).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(Error::Storage(format!("Error storing episode: {e}"))),
            Err(_) => {
                return Err(Error::Storage(format!(
                    "Timeout storing episode {episode_id} after {SYNC_EPISODE_TIMEOUT:?}"
                )))
            }
        }

        info!(correlation_id = %correlation_id, "Successfully synced episode {} to cache", episode_id);
        Ok(())
    }

    /// Sync all episodes modified since a given timestamp
    ///
    /// Queries the source storage for recent episodes and syncs them to the cache.
    ///
    /// # Arguments
    ///
    /// * `since` - Only sync episodes with `start_time` >= this timestamp
    ///
    /// # Returns
    ///
    /// Statistics about the sync operation (episodes synced, errors)
    ///
    /// # Errors
    ///
    /// Returns error if query fails, but continues syncing other episodes if individual stores fail
    pub async fn sync_all_recent_episodes(&self, since: DateTime<Utc>) -> Result<SyncStats> {
        let correlation_id = Uuid::new_v4();

        info!(correlation_id = %correlation_id, "Syncing all episodes since {}", since);

        // Query source storage for recent episodes with high limit for sync operations and with timeout
        // timeout returns Result<Result<Vec<Episode>, Error>, Elapsed>
        let episodes = match timeout(
            SYNC_ALL_TIMEOUT,
            self.turso
                .query_episodes_since(since, Some(MAX_QUERY_LIMIT)),
        )
        .await
        {
            Ok(Ok(episodes)) => episodes,
            Ok(Err(e)) => return Err(Error::Storage(format!("Error querying episodes: {e}"))),
            Err(_) => {
                return Err(Error::Storage(format!(
                    "Timeout querying episodes after {SYNC_ALL_TIMEOUT:?}"
                )))
            }
        };

        let total = episodes.len();

        let mut stats = SyncStats::default();

        // Batch update cache with individual timeouts for each store operation
        for episode in episodes {
            let episode_id = episode.episode_id;
            match timeout(SYNC_EPISODE_TIMEOUT, self.redb.store_episode(&episode)).await {
                Ok(Ok(())) => {
                    stats.episodes_synced += 1;
                }
                Ok(Err(e)) => {
                    error!(correlation_id = %correlation_id, "Failed to sync episode {}: {}", episode_id, e);
                    stats.errors += 1;
                }
                Err(_) => {
                    error!(
                        correlation_id = %correlation_id,
                        "Timeout syncing episode {} after {:?}",
                        episode_id, SYNC_EPISODE_TIMEOUT
                    );
                    stats.errors += 1;
                }
            }
        }

        // Update sync state
        self.update_sync_state(stats.episodes_synced, stats.errors)
            .await;

        info!(
            correlation_id = %correlation_id,
            "Sync complete: {}/{} episodes synced, {} errors",
            stats.episodes_synced, total, stats.errors
        );

        Ok(stats)
    }

    /// Start a periodic background sync task
    ///
    /// Spawns a background task that syncs recent episodes at the specified interval.
    /// The task will continue running until the returned `JoinHandle` is dropped or aborted.
    ///
    /// # Arguments
    ///
    /// * `interval` - How often to run the sync
    ///
    /// # Returns
    ///
    /// `JoinHandle` that can be used to cancel the background task
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::time::Duration;
    /// use std::sync::Arc;
    ///
    /// let sync = Arc::new(StorageSynchronizer::new(turso, redb));
    /// let handle = sync.start_periodic_sync(Duration::from_secs(300));
    ///
    /// // Later, to stop the sync:
    /// handle.abort();
    /// ```
    pub fn start_periodic_sync(self: Arc<Self>, interval: Duration) -> tokio::task::JoinHandle<()> {
        info!("Starting periodic sync with interval {:?}", interval);

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;

                let since = Utc::now() - chrono::Duration::hours(1);
                let correlation_id = Uuid::new_v4();

                match self.sync_all_recent_episodes(since).await {
                    Ok(stats) => {
                        debug!(
                            correlation_id = %correlation_id,
                            "Periodic sync successful: {} episodes synced",
                            stats.episodes_synced
                        );
                    }
                    Err(e) => {
                        error!(correlation_id = %correlation_id, "Periodic sync failed: {}", e);
                    }
                }
            }
        })
    }
}

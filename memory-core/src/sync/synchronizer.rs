//! Storage synchronizer for coordinating Turso and redb

use crate::{Error, MAX_QUERY_LIMIT, Result};
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
    /// Maximum number of episodes fetched per incremental-sync page.
    ///
    /// Clamped to [`MAX_QUERY_LIMIT`] so a single page can never exceed the
    /// storage-layer bound.
    page_size: usize,
}

impl<T, R> StorageSynchronizer<T, R> {
    /// Create a new storage synchronizer
    pub fn new(turso: Arc<T>, redb: Arc<R>) -> Self {
        Self {
            turso,
            redb,
            sync_state: Arc::new(RwLock::new(SyncState::default())),
            page_size: MAX_QUERY_LIMIT,
        }
    }

    /// Set the per-page bound used by incremental syncs.
    ///
    /// The value is clamped to `1..=`[`MAX_QUERY_LIMIT`]; the default is
    /// `MAX_QUERY_LIMIT`. Smaller pages are useful for tests and for limiting
    /// the working set on memory-constrained hosts.
    #[must_use]
    pub fn with_page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size.clamp(1, MAX_QUERY_LIMIT);
        self
    }

    /// Get the current synchronization state
    pub async fn get_sync_state(&self) -> SyncState {
        self.sync_state.read().await.clone()
    }

    /// Record a fully-synced page so the next run resumes from its end.
    async fn advance_watermark(&self, watermark: DateTime<Utc>) {
        self.sync_state.write().await.modified_watermark = Some(watermark);
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
    T: crate::storage::StorageBackend + crate::storage::SyncWatermarkBackend + 'static,
    R: crate::storage::StorageBackend + 'static,
{
    /// Resolve the watermark to start from: in-memory state first, then the
    /// durable watermark (survives restarts), else the caller's `since`.
    async fn resolve_start_watermark(&self, since: DateTime<Utc>) -> Result<DateTime<Utc>> {
        if let Some(watermark) = self.sync_state.read().await.modified_watermark {
            return Ok(watermark);
        }

        match timeout(SYNC_ALL_TIMEOUT, self.turso.load_sync_watermark()).await {
            Ok(Ok(Some(watermark))) => Ok(watermark),
            Ok(Ok(None)) => Ok(since),
            Ok(Err(e)) => Err(Error::Storage(format!("Error loading sync watermark: {e}"))),
            Err(_) => Err(Error::Storage(format!(
                "Timeout loading sync watermark after {SYNC_ALL_TIMEOUT:?}"
            ))),
        }
    }

    /// Durably persist a fully-synced page, then advance the in-memory
    /// watermark. Persisting first means a storage failure leaves the prior
    /// watermark intact and the page is retried on the next run.
    async fn persist_watermark(&self, watermark: DateTime<Utc>) -> Result<()> {
        match timeout(SYNC_ALL_TIMEOUT, self.turso.save_sync_watermark(watermark)).await {
            Ok(Ok(())) => {
                self.advance_watermark(watermark).await;
                Ok(())
            }
            Ok(Err(e)) => Err(Error::Storage(format!(
                "Error persisting sync watermark: {e}"
            ))),
            Err(_) => Err(Error::Storage(format!(
                "Timeout persisting sync watermark after {SYNC_ALL_TIMEOUT:?}"
            ))),
        }
    }

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
                )));
            }
            Ok(Err(e)) => return Err(Error::Storage(format!("Error fetching episode: {e}"))),
            Err(_) => {
                return Err(Error::Storage(format!(
                    "Timeout fetching episode {episode_id} after {SYNC_EPISODE_TIMEOUT:?}"
                )));
            }
        };

        // Store in redb cache with timeout
        match timeout(SYNC_EPISODE_TIMEOUT, self.redb.store_episode(&episode)).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(Error::Storage(format!("Error storing episode: {e}"))),
            Err(_) => {
                return Err(Error::Storage(format!(
                    "Timeout storing episode {episode_id} after {SYNC_EPISODE_TIMEOUT:?}"
                )));
            }
        }

        info!(correlation_id = %correlation_id, "Successfully synced episode {} to cache", episode_id);
        Ok(())
    }

    /// Sync all episodes modified since the recorded watermark.
    ///
    /// Pages through the source with a bounded keyset scan ordered by
    /// `(modified_at, episode_id)`. The watermark on [`SyncState`] advances
    /// only after a whole page is written successfully: a query error returns
    /// with the prior watermark intact, and a per-episode store error stops the
    /// loop before the failed page so a later run retries it.
    ///
    /// # Arguments
    ///
    /// * `since` - Initial watermark, used only when no previous sync has been
    ///   recorded. Once a watermark exists, syncing resumes from it so no
    ///   modification is skipped.
    ///
    /// # Returns
    ///
    /// Statistics about the sync operation (episodes synced, errors)
    ///
    /// # Errors
    ///
    /// Returns error if a page query fails or times out.
    pub async fn sync_all_recent_episodes(&self, since: DateTime<Utc>) -> Result<SyncStats> {
        let correlation_id = Uuid::new_v4();
        let page_size = self.page_size;

        // Resume from the durable watermark when one exists; `since` is only
        // the entry point for the very first incremental sync.
        let mut watermark = self.resolve_start_watermark(since).await?;
        let mut cursor: Option<(DateTime<Utc>, Uuid)> = None;
        let mut stats = SyncStats::default();

        info!(correlation_id = %correlation_id, "Syncing episodes modified since {}", watermark);

        loop {
            let mut page = match timeout(
                SYNC_ALL_TIMEOUT,
                self.turso
                    .query_episodes_modified_since(watermark, cursor, Some(page_size)),
            )
            .await
            {
                Ok(Ok(page)) => page,
                Ok(Err(e)) => {
                    return Err(Error::Storage(format!("Error querying episodes: {e}")));
                }
                Err(_) => {
                    return Err(Error::Storage(format!(
                        "Timeout querying episodes after {SYNC_ALL_TIMEOUT:?}"
                    )));
                }
            };

            // Defensive normalisation: drop anything at or before the cursor and
            // order by the keyset so the page bound and the new watermark are
            // computed on the same, unambiguous order.
            if let Some((cursor_at, cursor_id)) = cursor {
                page.retain(|(episode, modified_at)| {
                    (*modified_at, episode.episode_id) > (cursor_at, cursor_id)
                });
            }
            page.sort_by_key(|a| (a.1, a.0.episode_id));

            let Some(next_cursor) = page
                .last()
                .map(|(episode, modified_at)| (*modified_at, episode.episode_id))
            else {
                break;
            };

            let page_len = page.len();
            let mut page_errors = 0usize;

            for (episode, _modified_at) in &page {
                let episode_id = episode.episode_id;
                match timeout(SYNC_EPISODE_TIMEOUT, self.redb.store_episode(episode)).await {
                    Ok(Ok(())) => stats.episodes_synced += 1,
                    Ok(Err(e)) => {
                        error!(correlation_id = %correlation_id, "Failed to sync episode {}: {}", episode_id, e);
                        stats.errors += 1;
                        page_errors += 1;
                    }
                    Err(_) => {
                        error!(
                            correlation_id = %correlation_id,
                            "Timeout syncing episode {} after {:?}",
                            episode_id, SYNC_EPISODE_TIMEOUT
                        );
                        stats.errors += 1;
                        page_errors += 1;
                    }
                }
            }

            if page_errors > 0 {
                // Do not advance past a page that was not fully written: the
                // next run resumes from the previous watermark and retries it.
                error!(
                    correlation_id = %correlation_id,
                    "Page from {} had {} errors; watermark stays at {}",
                    watermark, page_errors, watermark
                );
                break;
            }

            watermark = next_cursor.0;
            cursor = Some(next_cursor);
            self.persist_watermark(next_cursor.0).await?;

            if page_len < page_size {
                break;
            }
        }

        // Update sync state
        self.update_sync_state(stats.episodes_synced, stats.errors)
            .await;

        info!(
            correlation_id = %correlation_id,
            "Sync complete: {} episodes synced, {} errors, watermark at {}",
            stats.episodes_synced, stats.errors, watermark
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

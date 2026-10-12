//! Incremental-sync watermark capabilities for storage backends.
//!
//! Split out of the main [`StorageBackend`](crate::storage::StorageBackend)
//! trait to keep `backend/mod.rs` within the per-file LOC budget. These
//! methods are only needed by the storage synchronizer, which bounds its source
//! backend on this trait in addition to `StorageBackend`.

use crate::{Episode, Error, Result};
use async_trait::async_trait;
use uuid::Uuid;

/// Modification-watermark queries and durable watermark persistence.
#[async_trait]
pub trait SyncWatermarkBackend: Send + Sync {
    /// Query episodes modified at or after a watermark, using keyset pagination.
    ///
    /// Unlike [`StorageBackend::query_episodes_since`](crate::storage::StorageBackend::query_episodes_since),
    /// which filters on `start_time`, this returns the episodes whose *storage
    /// watermark* changed at or after `since`, ordered ascending by
    /// `(modified_at, episode_id)`. Each item is returned together with its
    /// `modified_at` timestamp.
    ///
    /// `cursor` is the last `(modified_at, episode_id)` of the previous page.
    /// When supplied, only tuples strictly greater than the cursor are
    /// returned, so rows that share a `modified_at` are neither skipped nor
    /// duplicated. `limit` is bounded by
    /// [`MAX_QUERY_LIMIT`](crate::MAX_QUERY_LIMIT).
    ///
    /// # Default implementation
    ///
    /// There is no correct generic fallback: `query_episodes_since` filters on
    /// `start_time` and returns the *newest* `limit` rows, so deriving a
    /// watermark from it silently drops every older episode between `since` and
    /// the page maximum. Backends used for incremental synchronization must
    /// override this method; the default fails loudly rather than under-syncing.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Storage`] when the backend does
    /// not implement modification watermarks, or when the storage operation
    /// fails.
    async fn query_episodes_modified_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
        cursor: Option<(chrono::DateTime<chrono::Utc>, Uuid)>,
        limit: Option<usize>,
    ) -> Result<Vec<(Episode, chrono::DateTime<chrono::Utc>)>> {
        let _ = (since, cursor, limit);
        Err(Error::Storage(
            "backend does not implement modification watermarks; overriding \
             query_episodes_modified_since is required for incremental sync"
                .to_string(),
        ))
    }

    /// Load the durably persisted incremental-sync watermark, if any.
    ///
    /// The synchronizer resumes from this value across process restarts so a
    /// lookback shorter than the downtime cannot re-base the window forward and
    /// skip modifications. The default returns `None` (no durability); backends
    /// with durable metadata (Turso, redb) override it.
    ///
    /// # Errors
    ///
    /// Returns error if the storage operation fails.
    async fn load_sync_watermark(&self) -> Result<Option<chrono::DateTime<chrono::Utc>>> {
        Ok(None)
    }

    /// Durably persist the incremental-sync watermark.
    ///
    /// Called only after a whole page has been written to the cache. The default
    /// is a no-op for backends without durable metadata; the in-memory watermark
    /// still governs the current process.
    ///
    /// # Errors
    ///
    /// Returns error if the storage operation fails.
    async fn save_sync_watermark(&self, watermark: chrono::DateTime<chrono::Utc>) -> Result<()> {
        let _ = watermark;
        Ok(())
    }
}

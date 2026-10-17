//! Partial-failure cursor retryability tests (issue #1089, criterion 3).
//!
//! Pins the acceptance criterion "partial failure leaves a retryable cursor and
//! accurate stats": when one batch chunk of a page fails to write, the whole
//! page is retried by the next run because neither the in-memory watermark on
//! [`SyncState`](do_memory_core::sync::SyncState) nor the durable watermark on
//! the source is allowed to advance past it. Split out of `sync_watermark.rs`
//! so each integration test file stays reviewable.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use do_memory_core::storage::{StorageBackend, StorageBackendCapabilities, SyncWatermarkBackend};
use do_memory_core::sync::StorageSynchronizer;
use do_memory_core::{Episode, Result, TaskContext, TaskType};
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use uuid::Uuid;

// ============================================================================
// Test doubles
// ============================================================================

/// Source (durable) double that also implements the watermark persistence half
/// of [`SyncWatermarkBackend`], so the *durable* cursor is observable without
/// Turso or the network.
///
/// The production no-op defaults for `load_sync_watermark`/`save_sync_watermark`
/// would make a persisted cursor indistinguishable from never writing one, so
/// these overrides record the value in memory and the test reads it back.
struct DurableWatermarkSource {
    /// Scripted `(episode, modified_at)` rows, newest scan order applied per query.
    items: Vec<(Episode, DateTime<Utc>)>,
    /// Value written by the last `save_sync_watermark` call (the durable cursor).
    persisted: Mutex<Option<DateTime<Utc>>>,
    /// Number of `save_sync_watermark` calls that reached the backend.
    saves: AtomicUsize,
}

impl DurableWatermarkSource {
    fn new(items: Vec<(Episode, DateTime<Utc>)>) -> Self {
        Self {
            items,
            persisted: Mutex::new(None),
            saves: AtomicUsize::new(0),
        }
    }

    /// Read back the durable watermark exactly as a restarted process would.
    fn persisted_watermark(&self) -> Option<DateTime<Utc>> {
        *self.persisted.lock()
    }

    fn save_calls(&self) -> usize {
        self.saves.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl SyncWatermarkBackend for DurableWatermarkSource {
    async fn query_episodes_modified_since(
        &self,
        since: DateTime<Utc>,
        cursor: Option<(DateTime<Utc>, Uuid)>,
        limit: Option<usize>,
    ) -> Result<Vec<(Episode, DateTime<Utc>)>> {
        let limit = do_memory_core::apply_query_limit(limit);
        let mut page: Vec<(Episode, DateTime<Utc>)> = self
            .items
            .iter()
            .filter(|(_, modified_at)| *modified_at >= since)
            .filter(|(episode, modified_at)| match cursor {
                Some((cursor_at, cursor_id)) => {
                    (*modified_at, episode.episode_id) > (cursor_at, cursor_id)
                }
                None => true,
            })
            .cloned()
            .collect();
        page.sort_by_key(|a| (a.1, a.0.episode_id));
        page.truncate(limit);
        Ok(page)
    }

    async fn load_sync_watermark(&self) -> Result<Option<DateTime<Utc>>> {
        Ok(*self.persisted.lock())
    }

    async fn save_sync_watermark(&self, watermark: DateTime<Utc>) -> Result<()> {
        self.saves.fetch_add(1, Ordering::SeqCst);
        *self.persisted.lock() = Some(watermark);
        Ok(())
    }
}

/// The durable source advertises no optional capability (it is a test double).
impl StorageBackendCapabilities for DurableWatermarkSource {}

#[async_trait]
impl StorageBackend for DurableWatermarkSource {
    async fn store_episode(&self, _episode: &Episode) -> Result<()> {
        unimplemented!("scripted source is read-only")
    }

    async fn get_episode(&self, _id: Uuid) -> Result<Option<Episode>> {
        unimplemented!("scripted source is read-only")
    }

    async fn delete_episode(&self, _id: Uuid) -> Result<()> {
        unimplemented!("scripted source is read-only")
    }

    async fn store_pattern(&self, _pattern: &do_memory_core::Pattern) -> Result<()> {
        unimplemented!("scripted source is read-only")
    }

    async fn get_pattern(
        &self,
        _id: do_memory_core::episode::PatternId,
    ) -> Result<Option<do_memory_core::Pattern>> {
        unimplemented!("scripted source is read-only")
    }

    async fn store_heuristic(&self, _heuristic: &do_memory_core::Heuristic) -> Result<()> {
        unimplemented!("scripted source is read-only")
    }

    async fn get_heuristic(&self, _id: Uuid) -> Result<Option<do_memory_core::Heuristic>> {
        unimplemented!("scripted source is read-only")
    }

    async fn query_episodes_since(
        &self,
        _since: DateTime<Utc>,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        unimplemented!("scripted source only supports watermark queries")
    }

    async fn query_episodes_by_metadata(
        &self,
        _key: &str,
        _value: &str,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        unimplemented!("scripted source is read-only")
    }

    async fn store_embedding(&self, _id: &str, _embedding: Vec<f32>) -> Result<()> {
        unimplemented!("scripted source is read-only")
    }

    async fn get_embedding(&self, _id: &str) -> Result<Option<Vec<f32>>> {
        unimplemented!("scripted source is read-only")
    }

    async fn delete_embedding(&self, _id: &str) -> Result<bool> {
        unimplemented!("scripted source is read-only")
    }

    async fn store_embeddings_batch(&self, _embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        unimplemented!("scripted source is read-only")
    }

    async fn get_embeddings_batch(&self, _ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        unimplemented!("scripted source is read-only")
    }
}

/// Cache double with redb's all-or-nothing batch write semantics.
///
/// `store_episodes_batch` rejects the **whole slice** when any id in it is in
/// `fail_ids`, mirroring `memory-storage-redb`'s single write transaction that
/// never commits on a per-item error (`episodes.rs`). `store_episode` fails only
/// for the bad id itself, so the synchronizer's per-item fallback can still
/// recover the good episodes of the rejected chunk.
struct BatchFailingCache {
    stored: Mutex<HashMap<Uuid, Episode>>,
    fail_ids: Mutex<HashSet<Uuid>>,
    batch_calls: AtomicUsize,
    item_calls: AtomicUsize,
}

impl BatchFailingCache {
    fn new() -> Self {
        Self {
            stored: Mutex::new(HashMap::new()),
            fail_ids: Mutex::new(HashSet::new()),
            batch_calls: AtomicUsize::new(0),
            item_calls: AtomicUsize::new(0),
        }
    }

    /// Reject batch and per-item writes for `id` until [`Self::heal`].
    fn fail_for(&self, id: Uuid) {
        self.fail_ids.lock().insert(id);
    }

    fn heal(&self) {
        self.fail_ids.lock().clear();
    }

    fn batch_calls(&self) -> usize {
        self.batch_calls.load(Ordering::SeqCst)
    }

    /// Per-item `store_episode` calls, i.e. the synchronizer's fallback path.
    fn item_calls(&self) -> usize {
        self.item_calls.load(Ordering::SeqCst)
    }

    /// Zero the call counters so the retry run can be asserted on its own.
    fn reset_counters(&self) {
        self.batch_calls.store(0, Ordering::SeqCst);
        self.item_calls.store(0, Ordering::SeqCst);
    }

    fn contains(&self, id: Uuid) -> bool {
        self.stored.lock().contains_key(&id)
    }
}

impl StorageBackendCapabilities for BatchFailingCache {}

#[async_trait]
impl StorageBackend for BatchFailingCache {
    async fn store_episode(&self, episode: &Episode) -> Result<()> {
        self.item_calls.fetch_add(1, Ordering::SeqCst);
        if self.fail_ids.lock().contains(&episode.episode_id) {
            return Err(do_memory_core::Error::Storage(format!(
                "scripted cache rejected episode {}",
                episode.episode_id
            )));
        }
        self.stored
            .lock()
            .insert(episode.episode_id, episode.clone());
        Ok(())
    }

    async fn store_episodes_batch(&self, episodes: &[Episode]) -> Result<()> {
        self.batch_calls.fetch_add(1, Ordering::SeqCst);
        // All-or-nothing: one poisoned id aborts the chunk and writes nothing,
        // exactly like a rolled-back redb write transaction.
        let rejected: Vec<Uuid> = {
            let fail_ids = self.fail_ids.lock();
            episodes
                .iter()
                .filter(|episode| fail_ids.contains(&episode.episode_id))
                .map(|episode| episode.episode_id)
                .collect()
        };
        if !rejected.is_empty() {
            return Err(do_memory_core::Error::Storage(
                "scripted cache rejected the whole batch transaction".to_string(),
            ));
        }
        let mut stored = self.stored.lock();
        for episode in episodes {
            stored.insert(episode.episode_id, episode.clone());
        }
        Ok(())
    }

    async fn get_episode(&self, id: Uuid) -> Result<Option<Episode>> {
        Ok(self.stored.lock().get(&id).cloned())
    }

    async fn delete_episode(&self, id: Uuid) -> Result<()> {
        self.stored.lock().remove(&id);
        Ok(())
    }

    async fn store_pattern(&self, _pattern: &do_memory_core::Pattern) -> Result<()> {
        unimplemented!("cache double only backs episodes")
    }

    async fn get_pattern(
        &self,
        _id: do_memory_core::episode::PatternId,
    ) -> Result<Option<do_memory_core::Pattern>> {
        unimplemented!("cache double only backs episodes")
    }

    async fn store_heuristic(&self, _heuristic: &do_memory_core::Heuristic) -> Result<()> {
        unimplemented!("cache double only backs episodes")
    }

    async fn get_heuristic(&self, _id: Uuid) -> Result<Option<do_memory_core::Heuristic>> {
        unimplemented!("cache double only backs episodes")
    }

    async fn query_episodes_since(
        &self,
        _since: DateTime<Utc>,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        unimplemented!("cache double only backs episodes")
    }

    async fn query_episodes_by_metadata(
        &self,
        _key: &str,
        _value: &str,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        unimplemented!("cache double only backs episodes")
    }

    async fn store_embedding(&self, _id: &str, _embedding: Vec<f32>) -> Result<()> {
        unimplemented!("cache double only backs episodes")
    }

    async fn get_embedding(&self, _id: &str) -> Result<Option<Vec<f32>>> {
        unimplemented!("cache double only backs episodes")
    }

    async fn delete_embedding(&self, _id: &str) -> Result<bool> {
        unimplemented!("cache double only backs episodes")
    }

    async fn store_embeddings_batch(&self, _embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        unimplemented!("cache double only backs episodes")
    }

    async fn get_embeddings_batch(&self, _ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        unimplemented!("cache double only backs episodes")
    }
}

// ============================================================================
// Partial-failure cursor retryability (issue #1089 criterion 3)
// ============================================================================

#[tokio::test]
async fn should_leave_retryable_cursor_when_a_batch_chunk_fails() {
    // Given: six episodes on distinct increasing watermarks, a page holding four
    // of them and a batch chunk of two, so page 1 splits into `[0,1]`/`[2,3]`
    let now = Utc::now();
    let items: Vec<(Episode, DateTime<Utc>)> = (0..6)
        .map(|i| {
            let episode = Episode::new(
                format!("Task {i}"),
                TaskContext::default(),
                TaskType::Testing,
            );
            (episode, now + chrono::Duration::milliseconds(i + 1))
        })
        .collect();
    let ids: Vec<Uuid> = items
        .iter()
        .map(|(episode, _)| episode.episode_id)
        .collect();

    let source = Arc::new(DurableWatermarkSource::new(items.clone()));
    let cache = Arc::new(BatchFailingCache::new());
    // The second chunk of page 1 carries the poisoned id
    cache.fail_for(ids[3]);

    let sync = StorageSynchronizer::new(source.clone(), cache.clone())
        .with_page_size(4)
        .with_batch_size(2);
    let since = now - chrono::Duration::milliseconds(1);

    // When: the first sync hits a batch chunk that fails as a whole
    let stats = sync.sync_all_recent_episodes(since).await.unwrap();

    // Then: stats stay accurate - the good chunk and the recovered item land,
    // the poisoned item is the single error
    assert_eq!(
        stats.errors, 1,
        "issue #1089 criterion 3: the one rejected episode must be counted exactly once"
    );
    assert_eq!(
        stats.episodes_synced, 3,
        "issue #1089 criterion 3: two batched writes plus one fallback write are the only progress"
    );

    // Then: both chunks were attempted as batches and exactly the failing one
    // fell back to per-item writes
    assert_eq!(
        cache.batch_calls(),
        2,
        "each chunk of page 1 must be attempted as a batch write"
    );
    assert_eq!(
        cache.item_calls(),
        2,
        "only the rejected chunk triggers one per-item fallback pass"
    );

    // Then: the failing chunk wrote nothing durably, while the good episodes did
    assert!(
        !cache.contains(ids[3]),
        "issue #1089 criterion 3: the rejected episode must stay out of the cache"
    );
    assert!(cache.contains(ids[0]) && cache.contains(ids[1]) && cache.contains(ids[2]));

    // Then - the load-bearing pair: neither the in-memory cursor nor the durable
    // cursor moved past the page that failed to be written
    assert_eq!(
        sync.get_sync_state().await.modified_watermark,
        None,
        "issue #1089 criterion 3: a page with a failed chunk must not advance the sync cursor"
    );
    assert_eq!(
        source.persisted_watermark(),
        None,
        "issue #1089 criterion 3: the persisted watermark must not advance past the failed page"
    );
    assert_eq!(source.save_calls(), 0);

    // When: the cache heals and the same sync is retried
    cache.heal();
    cache.reset_counters();
    let retry = sync.sync_all_recent_episodes(since).await.unwrap();

    // Then: the retry leaves no errors and re-delivers everything the failed run
    // skipped, resuming from the cursor it never advanced
    assert_eq!(
        retry.errors, 0,
        "issue #1089 criterion 3: the retry of the un-advanced cursor must be clean"
    );
    assert_eq!(retry.episodes_synced, 6);
    assert_eq!(
        cache.batch_calls(),
        3,
        "page 1 (2 chunks) and page 2 (1 chunk)"
    );
    assert_eq!(cache.item_calls(), 0, "no fallback needed once healed");
    for id in &ids {
        assert!(
            cache.contains(*id),
            "episode {id} must be cached after the retry"
        );
    }

    // Then: both cursors now agree and point at the last episode's watermark
    let final_watermark = items[5].1;
    assert_eq!(
        sync.get_sync_state().await.modified_watermark,
        Some(final_watermark),
        "issue #1089 criterion 3: the recovered cursor advances to the final page"
    );
    assert_eq!(
        source.persisted_watermark(),
        Some(final_watermark),
        "issue #1089 criterion 3: the durable watermark must match the in-memory cursor"
    );
}

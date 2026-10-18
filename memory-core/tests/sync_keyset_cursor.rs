//! Keyset cursor ordering tests (issue #1089, criterion 4).
//!
//! Pins the acceptance criterion "the incremental sync cursor is a keyset, not a
//! bare timestamp": paging must be stable across pages and must break ties
//! deterministically when several episodes share a `modified_at`. A timestamp-only
//! page bound drops every tied row past the boundary, which under-syncs the cache
//! silently, so the ordering contract is asserted here rather than left to review.
//!
//! Split out of `sync_partial_failure_cursor.rs` (criterion 3) so each integration
//! test file stays reviewable, and out of `sync_watermark.rs` for the same reason.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use do_memory_core::storage::{StorageBackend, StorageBackendCapabilities, SyncWatermarkBackend};
use do_memory_core::sync::StorageSynchronizer;
use do_memory_core::{Episode, Result, TaskContext, TaskType};
use parking_lot::Mutex;
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// Test doubles
// ============================================================================

/// Scripted source answering the keyset query the way the backend contract
/// requires: keep rows at or after `since`, then strictly after the
/// `(modified_at, episode_id)` cursor, order by that keyset, truncate to the page.
///
/// `>= since` is deliberate — it is what makes the durable (timestamp-granular)
/// watermark re-read a tie group instead of losing it.
struct KeysetSource {
    items: Vec<(Episode, DateTime<Utc>)>,
    /// The cursor each page query was called with, in call order.
    cursors: Mutex<Vec<Option<(DateTime<Utc>, Uuid)>>>,
    /// Value written by the last `save_sync_watermark` call.
    persisted: Mutex<Option<DateTime<Utc>>>,
}

impl KeysetSource {
    fn new(items: Vec<(Episode, DateTime<Utc>)>) -> Self {
        Self {
            items,
            cursors: Mutex::new(Vec::new()),
            persisted: Mutex::new(None),
        }
    }

    fn persisted_watermark(&self) -> Option<DateTime<Utc>> {
        *self.persisted.lock()
    }

    /// Cursor seen by each page query of the current run.
    fn page_cursors(&self) -> Vec<Option<(DateTime<Utc>, Uuid)>> {
        self.cursors.lock().clone()
    }

    /// Forget the per-run query log so the next run can be asserted alone.
    fn reset_queries(&self) {
        self.cursors.lock().clear();
    }
}

#[async_trait]
impl SyncWatermarkBackend for KeysetSource {
    async fn query_episodes_modified_since(
        &self,
        since: DateTime<Utc>,
        cursor: Option<(DateTime<Utc>, Uuid)>,
        limit: Option<usize>,
    ) -> Result<Vec<(Episode, DateTime<Utc>)>> {
        self.cursors.lock().push(cursor);
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
        *self.persisted.lock() = Some(watermark);
        Ok(())
    }
}

impl StorageBackendCapabilities for KeysetSource {}

#[async_trait]
impl StorageBackend for KeysetSource {
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
        unimplemented!("scripted source only supports keyset queries")
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

/// Cache double that records the order in which episodes arrive.
///
/// Batches and single writes both append, so the test can distinguish "synced
/// twice" from "synced once" and read the delivery sequence directly.
struct RecordingCache {
    written: Mutex<Vec<Uuid>>,
}

impl RecordingCache {
    fn new() -> Self {
        Self {
            written: Mutex::new(Vec::new()),
        }
    }

    /// Every id handed to a write, in arrival order, duplicates included.
    fn deliveries(&self) -> Vec<Uuid> {
        self.written.lock().clone()
    }

    fn contains(&self, id: Uuid) -> bool {
        self.written.lock().contains(&id)
    }
}

impl StorageBackendCapabilities for RecordingCache {}

#[async_trait]
impl StorageBackend for RecordingCache {
    async fn store_episode(&self, episode: &Episode) -> Result<()> {
        self.written.lock().push(episode.episode_id);
        Ok(())
    }

    async fn store_episodes_batch(&self, episodes: &[Episode]) -> Result<()> {
        self.written
            .lock()
            .extend(episodes.iter().map(|episode| episode.episode_id));
        Ok(())
    }

    async fn get_episode(&self, _id: Uuid) -> Result<Option<Episode>> {
        Ok(None)
    }

    async fn delete_episode(&self, _id: Uuid) -> Result<()> {
        unimplemented!("cache double only records writes")
    }

    async fn store_pattern(&self, _pattern: &do_memory_core::Pattern) -> Result<()> {
        unimplemented!("cache double only records writes")
    }

    async fn get_pattern(
        &self,
        _id: do_memory_core::episode::PatternId,
    ) -> Result<Option<do_memory_core::Pattern>> {
        unimplemented!("cache double only records writes")
    }

    async fn store_heuristic(&self, _heuristic: &do_memory_core::Heuristic) -> Result<()> {
        unimplemented!("cache double only records writes")
    }

    async fn get_heuristic(&self, _id: Uuid) -> Result<Option<do_memory_core::Heuristic>> {
        unimplemented!("cache double only records writes")
    }

    async fn query_episodes_since(
        &self,
        _since: DateTime<Utc>,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        unimplemented!("cache double only records writes")
    }

    async fn query_episodes_by_metadata(
        &self,
        _key: &str,
        _value: &str,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        unimplemented!("cache double only records writes")
    }

    async fn store_embedding(&self, _id: &str, _embedding: Vec<f32>) -> Result<()> {
        unimplemented!("cache double only records writes")
    }

    async fn get_embedding(&self, _id: &str) -> Result<Option<Vec<f32>>> {
        unimplemented!("cache double only records writes")
    }

    async fn delete_embedding(&self, _id: &str) -> Result<bool> {
        unimplemented!("cache double only records writes")
    }

    async fn store_embeddings_batch(&self, _embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        unimplemented!("cache double only records writes")
    }

    async fn get_embeddings_batch(&self, _ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        unimplemented!("cache double only records writes")
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Build an episode carrying only the fields the sync path reads.
fn episode(label: &str) -> Episode {
    Episode::new(
        format!("Task {label}"),
        TaskContext::default(),
        TaskType::Testing,
    )
}

/// Keyset order the synchronizer must reproduce across pages.
fn keyset_order(items: &[(Episode, DateTime<Utc>)]) -> Vec<Uuid> {
    let mut sorted = items.to_vec();
    sorted.sort_by_key(|a| (a.1, a.0.episode_id));
    sorted
        .iter()
        .map(|(episode, _)| episode.episode_id)
        .collect()
}

// ============================================================================
// Keyset ordering (issue #1089 criterion 4)
// ============================================================================

/// Ties straddle a page boundary: three episodes share one `modified_at` while
/// the page holds two. A timestamp-only bound would stop after the second tied
/// row and never revisit the third, so all five must arrive exactly once and in
/// keyset order, and the second page must be requested with a `(timestamp, id)`
/// cursor rather than a bare timestamp.
#[tokio::test]
async fn should_sync_every_tied_episode_once_across_pages() {
    // Given: a three-way tie at `t0`, then two later rows; page 2, batch 3
    let t0 = Utc::now();
    let items: Vec<(Episode, DateTime<Utc>)> = vec![
        (episode("a"), t0),
        (episode("b"), t0),
        (episode("c"), t0),
        (episode("d"), t0 + chrono::Duration::milliseconds(1)),
        (episode("e"), t0 + chrono::Duration::milliseconds(2)),
    ];
    let expected = keyset_order(&items);

    let source = Arc::new(KeysetSource::new(items.clone()));
    let cache = Arc::new(RecordingCache::new());
    let sync = StorageSynchronizer::new(source.clone(), cache.clone())
        .with_page_size(2)
        .with_batch_size(3);

    // When: one sync run walks the whole keyset
    let stats = sync
        .sync_all_recent_episodes(t0 - chrono::Duration::milliseconds(1))
        .await
        .unwrap();

    // Then: every episode is delivered, none twice, none reordered
    assert_eq!(
        stats.episodes_synced, 5,
        "issue #1089 criterion 4: a tie group split across pages must still sync all rows"
    );
    let deliveries = cache.deliveries();
    assert_eq!(
        deliveries.len(),
        deliveries
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        "issue #1089 criterion 4: paging must not re-deliver an episode inside one run"
    );
    assert_eq!(
        deliveries, expected,
        "issue #1089 criterion 4: delivery order must equal the (modified_at, episode_id) keyset order"
    );

    // Then: the page bound is the full keyset pair, not just the timestamp
    let cursors = source.page_cursors();
    let second_page = cursors
        .get(1)
        .expect("a second page query must be issued for five rows at page size two");
    let tail_at = items
        .iter()
        .find(|(episode, _)| episode.episode_id == expected[1])
        .map(|(_, at)| *at)
        .expect("the page bound must reference a scripted episode");
    assert_eq!(
        *second_page,
        Some((tail_at, expected[1])),
        "issue #1089 criterion 4: page 2 must resume from the last (modified_at, episode_id) of page 1"
    );
}

/// A resumed run starts from the durably persisted watermark, which has
/// timestamp granularity only. When that watermark lands inside a tie group the
/// next run must re-read the tied rows (at-least-once) rather than skip them
/// (lost updates), so coverage has to stay complete across runs.
#[tokio::test]
async fn should_not_lose_tied_episodes_when_resuming_from_the_watermark() {
    // Given: four episodes all sharing one `modified_at`, page size 2
    let t0 = Utc::now();
    let items: Vec<(Episode, DateTime<Utc>)> = vec![
        (episode("a"), t0),
        (episode("b"), t0),
        (episode("c"), t0),
        (episode("d"), t0),
    ];
    let ids: Vec<Uuid> = items
        .iter()
        .map(|(episode, _)| episode.episode_id)
        .collect();

    let source = Arc::new(KeysetSource::new(items));
    let cache = Arc::new(RecordingCache::new());

    // When: the first run drains the tie group across two pages
    let first = StorageSynchronizer::new(source.clone(), cache.clone())
        .with_page_size(2)
        .with_batch_size(2)
        .sync_all_recent_episodes(t0 - chrono::Duration::milliseconds(1))
        .await
        .unwrap();

    // Then: all four arrive in one run, and the watermark stops at the shared
    // timestamp — the reason a resuming run cannot trust a `>` comparison
    assert_eq!(
        first.episodes_synced, 4,
        "issue #1089 criterion 4: a tie group spanning page boundaries must not lose rows"
    );
    assert_eq!(
        source.persisted_watermark(),
        Some(t0),
        "the durable watermark keeps timestamp granularity, so ties are not separable across runs"
    );

    // When: a fresh synchronizer resumes from that watermark (new process)
    source.reset_queries();
    let before = cache.deliveries().len();
    let second = StorageSynchronizer::new(source.clone(), cache.clone())
        .with_page_size(2)
        .with_batch_size(2)
        .sync_all_recent_episodes(t0 - chrono::Duration::milliseconds(1))
        .await
        .unwrap();

    // Then: the resumed run re-reads the tie group instead of skipping it, and
    // no episode is ever missing from the cache
    assert!(
        source.page_cursors().first().copied().flatten().is_none(),
        "a resuming run must start with an open keyset bound, filtered only by the watermark"
    );
    assert!(
        second.episodes_synced > 0,
        "issue #1089 criterion 4: rows sharing the watermark timestamp must be re-read, not lost"
    );
    assert_eq!(
        cache.deliveries().len(),
        before + second.episodes_synced,
        "the resumed run must account for every delivery it made"
    );
    for id in &ids {
        assert!(cache.contains(*id), "episode {id} never reached the cache");
    }
    assert_eq!(
        ids.iter().filter(|id| cache.contains(**id)).count(),
        ids.len(),
        "issue #1089 criterion 4: coverage across runs must stay complete for tied timestamps"
    );
}

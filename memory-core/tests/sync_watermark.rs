//! Incremental modification-watermark synchronization tests (issue #1067).
//!
//! These tests drive `query_episodes_modified_since` through a scripted source
//! and assert the watermark contract: bounded keyset pages, no duplicates or
//! omissions across pages, a prior watermark kept when a page fails, and a
//! resume from the durable watermark after a restart. Split out of
//! `storage_sync.rs` so each integration test file stays reviewable.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use do_memory_core::storage::{StorageBackend, StorageBackendCapabilities, SyncWatermarkBackend};
use do_memory_core::sync::StorageSynchronizer;
use do_memory_core::{Episode, Result, TaskContext, TaskType};
use do_memory_storage_redb::RedbStorage;
use do_memory_storage_turso::TursoStorage;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tempfile::TempDir;
use uuid::Uuid;

/// Create a test Turso storage with local file database
async fn create_test_turso() -> anyhow::Result<(TursoStorage, TempDir)> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("test_turso.db");

    // Use Builder::new_local for file-based test databases
    let db = libsql::Builder::new_local(&db_path)
        .build()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create test database: {e}"))?;

    let storage = TursoStorage::from_database(db)?;
    storage.initialize_schema().await?;
    Ok((storage, dir))
}

async fn create_test_redb() -> anyhow::Result<(RedbStorage, TempDir)> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("test.redb");
    let storage = RedbStorage::new(&db_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create redb storage: {e}"))?;
    Ok((storage, dir))
}
// ============================================================================
// Modification-watermark incremental sync (issue #1067)
// ============================================================================

/// Source/cache backend that serves scripted keyset pages and can fail on
/// demand.
///
/// `query_episodes_modified_since` is the meaningful read path and
/// `store_episode`/`get_episode` back the cache role; the remaining required
/// methods are never reached by these tests.
struct ScriptedSource {
    items: Vec<(Episode, DateTime<Utc>)>,
    fail_after_calls: AtomicUsize,
    calls: AtomicUsize,
    fail_store_for: Mutex<Option<Uuid>>,
    stored: Mutex<HashMap<Uuid, Episode>>,
}

impl ScriptedSource {
    fn new(items: Vec<(Episode, DateTime<Utc>)>) -> Self {
        Self {
            items,
            fail_after_calls: AtomicUsize::new(usize::MAX),
            calls: AtomicUsize::new(0),
            fail_store_for: Mutex::new(None),
            stored: Mutex::new(HashMap::new()),
        }
    }

    /// Let `successes` pages succeed, then fail every subsequent query.
    fn fail_after(&self, successes: usize) {
        self.fail_after_calls.store(successes, Ordering::SeqCst);
    }

    fn heal(&self) {
        self.fail_after_calls.store(usize::MAX, Ordering::SeqCst);
    }

    /// Make `store_episode` fail for the given episode until healed.
    fn fail_store_for(&self, episode_id: Uuid) {
        *self.fail_store_for.lock() = Some(episode_id);
    }

    fn heal_stores(&self) {
        *self.fail_store_for.lock() = None;
    }
}

#[async_trait]
impl SyncWatermarkBackend for ScriptedSource {
    async fn query_episodes_modified_since(
        &self,
        since: DateTime<Utc>,
        cursor: Option<(DateTime<Utc>, Uuid)>,
        limit: Option<usize>,
    ) -> Result<Vec<(Episode, DateTime<Utc>)>> {
        let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        if call > self.fail_after_calls.load(Ordering::SeqCst) {
            return Err(do_memory_core::Error::Storage(
                "scripted page failure".to_string(),
            ));
        }

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
}

/// The scripted source advertises no optional capability (it is a test double).
impl StorageBackendCapabilities for ScriptedSource {}

#[async_trait]
impl StorageBackend for ScriptedSource {
    async fn store_episode(&self, episode: &Episode) -> Result<()> {
        if *self.fail_store_for.lock() == Some(episode.episode_id) {
            return Err(do_memory_core::Error::Storage(
                "scripted store failure".to_string(),
            ));
        }
        self.stored
            .lock()
            .insert(episode.episode_id, episode.clone());
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

#[tokio::test]
async fn should_sync_updated_old_episode_on_next_incremental_sync() {
    // Given: a synchronizer and an episode whose start_time predates the window
    let (turso, _turso_dir) = create_test_turso().await.unwrap();
    let (redb, _redb_dir) = create_test_redb().await.unwrap();
    let sync = StorageSynchronizer::new(Arc::new(turso), Arc::new(redb));

    let mut episode = Episode::new(
        "Original task".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );
    episode.start_time = Utc::now() - chrono::Duration::hours(2);
    sync.turso.store_episode(&episode).await.unwrap();

    let since = Utc::now() - chrono::Duration::hours(1);

    // When: syncing since one hour ago
    let stats = sync.sync_all_recent_episodes(since).await.unwrap();

    // Then: the watermark query sees the old episode even though start_time is old
    assert_eq!(stats.episodes_synced, 1, "old episode must be observed");
    let first_watermark = sync
        .get_sync_state()
        .await
        .modified_watermark
        .expect("watermark must be recorded");

    // When: the old episode is updated and synced again (guarantee a later ms)
    tokio::time::sleep(Duration::from_millis(5)).await;
    let mut updated = episode.clone();
    updated.task_description = "Updated task".to_string();
    sync.turso.store_episode(&updated).await.unwrap();

    let stats = sync.sync_all_recent_episodes(since).await.unwrap();

    // Then: the update is picked up by the next incremental sync
    assert_eq!(stats.episodes_synced, 1, "updated old episode must re-sync");
    let second_watermark = sync
        .get_sync_state()
        .await
        .modified_watermark
        .expect("watermark must advance");
    assert!(second_watermark > first_watermark);

    let cached = sync
        .redb
        .get_episode(episode.episode_id)
        .await
        .unwrap()
        .expect("episode must be cached");
    assert_eq!(cached.task_description, "Updated task");
}

#[tokio::test]
async fn should_page_dataset_larger_than_one_page_without_duplicates_or_omissions() {
    // Given: a dataset larger than the configured page size
    let (turso, _turso_dir) = create_test_turso().await.unwrap();
    let (redb, _redb_dir) = create_test_redb().await.unwrap();
    let sync = StorageSynchronizer::new(Arc::new(turso), Arc::new(redb)).with_page_size(2);

    let mut ids = Vec::new();
    for i in 0..7 {
        let episode = Episode::new(
            format!("Task {i}"),
            TaskContext::default(),
            TaskType::Testing,
        );
        ids.push(episode.episode_id);
        sync.turso.store_episode(&episode).await.unwrap();
    }

    let since = Utc::now() - chrono::Duration::hours(1);

    // When: syncing in bounded pages
    let stats = sync.sync_all_recent_episodes(since).await.unwrap();

    // Then: every episode is synced exactly once, with no errors
    assert_eq!(stats.episodes_synced, 7, "no duplicates or omissions");
    assert_eq!(stats.errors, 0);
    for id in ids {
        assert!(
            sync.redb.get_episode(id).await.unwrap().is_some(),
            "episode {id} must be cached"
        );
    }
    assert!(
        sync.get_sync_state().await.modified_watermark.is_some(),
        "watermark must advance to the final page"
    );
}

#[tokio::test]
async fn should_keep_prior_watermark_when_a_page_fails_and_retry_safely() {
    // Given: four episodes and a source that fails after the first page
    let (redb, _redb_dir) = create_test_redb().await.unwrap();
    let now = Utc::now();
    let items: Vec<(Episode, DateTime<Utc>)> = (0..4)
        .map(|i| {
            let episode = Episode::new(
                format!("Task {i}"),
                TaskContext::default(),
                TaskType::Testing,
            );
            (episode, now + chrono::Duration::milliseconds(i))
        })
        .collect();

    let source = Arc::new(ScriptedSource::new(items.clone()));
    source.fail_after(1);
    let sync = StorageSynchronizer::new(source.clone(), Arc::new(redb)).with_page_size(2);
    let since = now - chrono::Duration::milliseconds(1);

    // When: the second page fails
    let result = sync.sync_all_recent_episodes(since).await;

    // Then: the error surfaces and the watermark stays at the last complete page
    assert!(result.is_err(), "a failed page must surface as an error");
    let failed_state = sync.get_sync_state().await;
    assert_eq!(
        failed_state.modified_watermark,
        Some(items[1].1),
        "watermark must not advance past the failed page"
    );
    assert!(
        sync.redb
            .get_episode(items[0].0.episode_id)
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        sync.redb
            .get_episode(items[1].0.episode_id)
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        sync.redb
            .get_episode(items[2].0.episode_id)
            .await
            .unwrap()
            .is_none()
    );

    // When: the source recovers and the sync is retried
    source.heal();
    let retry_stats = sync.sync_all_recent_episodes(since).await.unwrap();

    // Then: it resumes from the stored watermark and completes without omission
    assert_eq!(retry_stats.errors, 0);
    for (episode, _) in &items {
        assert!(
            sync.redb
                .get_episode(episode.episode_id)
                .await
                .unwrap()
                .is_some(),
            "episode {} must be cached after retry",
            episode.episode_id
        );
    }
    assert_eq!(
        sync.get_sync_state().await.modified_watermark,
        Some(items[3].1)
    );
}

#[tokio::test]
async fn should_not_advance_watermark_past_a_failed_store_page() {
    // Given: a source of four episodes and a cache that rejects one store
    let now = Utc::now();
    let items: Vec<(Episode, DateTime<Utc>)> = (0..4)
        .map(|i| {
            let episode = Episode::new(
                format!("Task {i}"),
                TaskContext::default(),
                TaskType::Testing,
            );
            (episode, now + chrono::Duration::milliseconds(i))
        })
        .collect();

    let source = Arc::new(ScriptedSource::new(items.clone()));
    let cache = Arc::new(ScriptedSource::new(Vec::new()));
    cache.fail_store_for(items[1].0.episode_id);

    let sync = StorageSynchronizer::new(source, cache.clone()).with_page_size(2);
    let since = now - chrono::Duration::milliseconds(1);

    // When: the first page contains a store failure
    let stats = sync.sync_all_recent_episodes(since).await.unwrap();

    // Then: the failure is counted and the watermark does not advance
    assert_eq!(stats.errors, 1, "the failed store must be counted");
    assert_eq!(stats.episodes_synced, 1);
    assert_eq!(
        sync.get_sync_state().await.modified_watermark,
        None,
        "a page with a store failure must not advance the watermark"
    );
    assert!(
        cache
            .get_episode(items[0].0.episode_id)
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        cache
            .get_episode(items[1].0.episode_id)
            .await
            .unwrap()
            .is_none()
    );

    // When: the cache recovers and the sync is retried
    cache.heal_stores();
    let retry = sync.sync_all_recent_episodes(since).await.unwrap();

    // Then: the failed page is re-delivered and the dataset completes
    assert_eq!(retry.errors, 0);
    for (episode, _) in &items {
        assert!(
            cache
                .get_episode(episode.episode_id)
                .await
                .unwrap()
                .is_some(),
            "episode {} must be cached after retry",
            episode.episode_id
        );
    }
    assert!(sync.get_sync_state().await.modified_watermark.is_some());
}

#[tokio::test]
async fn should_resume_from_durable_watermark_after_restart() {
    // Given: a synchronizer backed by real Turso + redb
    let (turso, _turso_dir) = create_test_turso().await.unwrap();
    let (redb, _redb_dir) = create_test_redb().await.unwrap();
    let turso = Arc::new(turso);
    let redb = Arc::new(redb);

    let first = Episode::new(
        "First".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );
    turso.store_episode(&first).await.unwrap();

    // When: a first run persists the watermark into the durable source
    let initial_sync = StorageSynchronizer::new(turso.clone(), redb.clone());
    initial_sync
        .sync_all_recent_episodes(Utc::now() - chrono::Duration::hours(1))
        .await
        .unwrap();
    // Simulated process restart: only the durable watermark survives.
    drop(initial_sync);

    tokio::time::sleep(Duration::from_millis(5)).await;
    let second = Episode::new(
        "Second".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );
    turso.store_episode(&second).await.unwrap();

    // When: a fresh synchronizer runs with a lookback that *excludes* both writes
    tokio::time::sleep(Duration::from_millis(5)).await;
    let restarted_sync = StorageSynchronizer::new(turso.clone(), redb.clone());
    restarted_sync
        .sync_all_recent_episodes(Utc::now())
        .await
        .unwrap();

    // Then: it resumed from the durable watermark instead of re-basing the window
    assert!(
        redb.get_episode(second.episode_id).await.unwrap().is_some(),
        "restart must resume from the durable watermark, not the caller's since"
    );
}

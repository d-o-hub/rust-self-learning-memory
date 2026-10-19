//! BDD-style integration tests for storage synchronization between Turso and redb
//!
//! Tests verify that the memory system correctly synchronizes episodes, patterns,
//! and heuristics between durable storage (Turso) and cache (redb).
//!
//! All tests follow the Given-When-Then pattern for clarity.

use do_memory_core::StorageBackend;
use do_memory_core::sync::{ConflictResolution, StorageSynchronizer, SyncConfig};
use do_memory_core::{Episode, TaskContext, TaskType};
use do_memory_storage_redb::RedbStorage;
use do_memory_storage_turso::TursoStorage;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

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

/// Create a test Redb storage with temp directory
async fn create_test_redb() -> anyhow::Result<(RedbStorage, TempDir)> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("test_redb.db");

    let storage = RedbStorage::new(&db_path).await?;
    Ok((storage, dir))
}

#[tokio::test]
async fn should_sync_single_episode_from_turso_to_redb_cache() {
    // Given: A synchronizer with Turso and redb storage
    let (turso, _turso_dir) = create_test_turso().await.unwrap();
    let (redb, _redb_dir) = create_test_redb().await.unwrap();
    let sync = StorageSynchronizer::new(Arc::new(turso), Arc::new(redb));

    // Given: An episode stored in Turso
    let context = TaskContext::default();
    let episode = Episode::new("Test task".to_string(), context, TaskType::Testing);
    let episode_id = episode.episode_id;
    sync.turso.store_episode(&episode).await.unwrap();

    // When: Syncing the episode to cache
    let result = sync.sync_episode_to_cache(episode_id).await;

    // Then: Sync should succeed
    assert!(result.is_ok(), "Sync should succeed");

    // Then: Episode should be available in redb cache
    let cached = sync.redb.get_episode(episode_id).await.unwrap();
    assert!(cached.is_some(), "Episode should be in cache");
    assert_eq!(cached.unwrap().episode_id, episode_id);
}

#[tokio::test]
async fn should_sync_all_recent_episodes_in_batch() {
    // Given: A synchronizer with Turso and redb storage
    let (turso, _turso_dir) = create_test_turso().await.unwrap();
    let (redb, _redb_dir) = create_test_redb().await.unwrap();
    let sync = StorageSynchronizer::new(Arc::new(turso), Arc::new(redb));

    // Given: Multiple episodes stored in Turso
    let context = TaskContext::default();
    let mut episode_ids = Vec::new();

    for i in 0..5 {
        let episode = Episode::new(format!("Test task {i}"), context.clone(), TaskType::Testing);
        episode_ids.push(episode.episode_id);
        sync.turso.store_episode(&episode).await.unwrap();
    }

    // When: Syncing all recent episodes (since 1 hour ago)
    let since = chrono::Utc::now() - chrono::Duration::hours(1);
    let stats = sync.sync_all_recent_episodes(since).await.unwrap();

    // Then: All episodes should be synced successfully
    assert_eq!(stats.episodes_synced, 5, "Should sync all 5 episodes");
    assert_eq!(stats.errors, 0, "Should have no errors");

    // Then: All episodes should be available in cache
    for episode_id in episode_ids {
        let cached = sync.redb.get_episode(episode_id).await.unwrap();
        assert!(cached.is_some(), "Episode {episode_id} should be in cache");
    }
}

#[tokio::test]
async fn should_track_sync_state_and_statistics() {
    // Given: A synchronizer with Turso and redb storage
    let (turso, _turso_dir) = create_test_turso().await.unwrap();
    let (redb, _redb_dir) = create_test_redb().await.unwrap();
    let sync = StorageSynchronizer::new(Arc::new(turso), Arc::new(redb));

    // Then: Initial state should have zero syncs
    let state = sync.get_sync_state().await;
    assert_eq!(state.sync_count, 0);
    assert!(state.last_sync.is_none());

    // Given: An episode stored in Turso
    let context = TaskContext::default();
    let episode = Episode::new("Test task".to_string(), context, TaskType::Testing);
    sync.turso.store_episode(&episode).await.unwrap();

    // When: Syncing all recent episodes
    let since = chrono::Utc::now() - chrono::Duration::hours(1);
    sync.sync_all_recent_episodes(since).await.unwrap();

    // Then: Sync state should be updated with accurate statistics
    let state = sync.get_sync_state().await;
    assert_eq!(state.sync_count, 1);
    assert!(state.last_sync.is_some());
    assert!(state.last_error.is_none());
}

#[tokio::test]
async fn should_run_periodic_background_sync_automatically() {
    // Given: A synchronizer with Turso and redb storage
    let (turso, _turso_dir) = create_test_turso().await.unwrap();
    let (redb, _redb_dir) = create_test_redb().await.unwrap();
    let sync = Arc::new(StorageSynchronizer::new(Arc::new(turso), Arc::new(redb)));

    // Given: An episode stored in Turso
    let context = TaskContext::default();
    let episode = Episode::new("Test task".to_string(), context, TaskType::Testing);
    let episode_id = episode.episode_id;
    sync.turso.store_episode(&episode).await.unwrap();

    // When: Starting periodic sync with short interval (100ms for testing)
    let handle = sync.clone().start_periodic_sync(Duration::from_millis(100));

    // When: Polling for sync completion (with timeout for CI reliability)
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(10); // Generous timeout for slow CI (especially Windows)
    let mut synced = false;

    while start.elapsed() < timeout {
        if let Ok(Some(_)) = sync.redb.get_episode(episode_id).await {
            synced = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await; // Poll every 50ms
    }

    handle.abort();

    // Then: The episode should be automatically synced to cache
    assert!(
        synced,
        "Episode should be synced to cache within {}s (took {:?})",
        timeout.as_secs(),
        start.elapsed()
    );

    // Then: Sync state should reflect multiple syncs
    let state = sync.get_sync_state().await;
    assert!(
        state.sync_count > 0,
        "Should have performed at least one sync"
    );
}

#[tokio::test]
async fn should_handle_missing_episode_gracefully() {
    // Given: A synchronizer with Turso and redb storage
    let (turso, _turso_dir) = create_test_turso().await.unwrap();
    let (redb, _redb_dir) = create_test_redb().await.unwrap();
    let sync = StorageSynchronizer::new(Arc::new(turso), Arc::new(redb));

    // When: Attempting to sync a non-existent episode
    let fake_id = uuid::Uuid::new_v4();
    let result = sync.sync_episode_to_cache(fake_id).await;

    // Then: Should return an error for missing episode
    assert!(result.is_err(), "Should fail for missing episode");
}

#[test]
fn should_resolve_conflicts_with_turso_wins_strategy() {
    // Given: Two episodes with same ID but different content
    let context = TaskContext::default();
    let episode1 = Arc::new(Episode::new(
        "Task from Turso".to_string(),
        context.clone(),
        TaskType::Testing,
    ));
    let mut episode2 = Episode::new("Task from redb".to_string(), context, TaskType::Testing);
    episode2.episode_id = episode1.episode_id;
    let episode2 = Arc::new(episode2);

    // When: Resolving conflict with TursoWins strategy
    let resolved = do_memory_core::sync::resolve_episode_conflict(
        &episode1,
        &episode2,
        ConflictResolution::TursoWins,
    );

    // Then: Turso version should be selected
    assert_eq!(resolved.task_description, "Task from Turso");
}

#[test]
fn should_resolve_conflicts_with_redb_wins_strategy() {
    // Given: Two episodes with same ID but different content
    let context = TaskContext::default();
    let episode1 = Arc::new(Episode::new(
        "Task from Turso".to_string(),
        context.clone(),
        TaskType::Testing,
    ));
    let mut episode2 = Episode::new("Task from redb".to_string(), context, TaskType::Testing);
    episode2.episode_id = episode1.episode_id;
    let episode2 = Arc::new(episode2);

    // When: Resolving conflict with RedbWins strategy
    let resolved = do_memory_core::sync::resolve_episode_conflict(
        &episode1,
        &episode2,
        ConflictResolution::RedbWins,
    );

    // Then: Redb version should be selected
    assert_eq!(resolved.task_description, "Task from redb");
}

#[test]
fn should_resolve_conflicts_with_most_recent_strategy() {
    // Given: Two episodes with same ID but different timestamps
    let context = TaskContext::default();
    let episode1 = Arc::new(Episode::new(
        "Older task".to_string(),
        context.clone(),
        TaskType::Testing,
    ));
    let mut episode2 = Episode::new("Newer task".to_string(), context, TaskType::Testing);
    episode2.episode_id = episode1.episode_id;
    episode2.end_time = Some(chrono::Utc::now());
    let episode2 = Arc::new(episode2);

    // When: Resolving conflict with MostRecent strategy
    let resolved = do_memory_core::sync::resolve_episode_conflict(
        &episode1,
        &episode2,
        ConflictResolution::MostRecent,
    );

    // Then: The newer episode should be selected
    assert_eq!(
        resolved.task_description, "Newer task",
        "Should choose the newer episode"
    );
}

#[test]
fn should_provide_sensible_default_sync_configuration() {
    // Given: Default sync configuration
    let config = SyncConfig::default();

    // Then: Should have reasonable default values
    assert_eq!(config.sync_interval, Duration::from_secs(300));
    assert_eq!(config.batch_size, 100);
    assert!(config.sync_patterns);
    assert!(config.sync_heuristics);
}

#[tokio::test]
async fn should_fallback_to_per_item_store_when_batch_fails_and_isolate_errors() {
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FailingBatchStorage {
        real_redb: RedbStorage,
        batch_call_count: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl do_memory_core::StorageBackendCapabilities for FailingBatchStorage {}

    #[async_trait]
    impl do_memory_core::StorageBackend for FailingBatchStorage {
        async fn store_episode(&self, episode: &Episode) -> do_memory_core::Result<()> {
            // Induce failure for a specific bad episode ID
            if episode.task_description == "BAD_EPISODE" {
                return Err(do_memory_core::Error::Storage(
                    "Simulated episode store failure".to_string(),
                ));
            }
            self.real_redb.store_episode(episode).await
        }

        async fn store_episodes_batch(&self, _episodes: &[Episode]) -> do_memory_core::Result<()> {
            self.batch_call_count.fetch_add(1, Ordering::SeqCst);
            // Always fail batch write to trigger fallback
            Err(do_memory_core::Error::Storage(
                "Simulated batch failure".to_string(),
            ))
        }

        async fn get_episode(&self, id: uuid::Uuid) -> do_memory_core::Result<Option<Episode>> {
            self.real_redb.get_episode(id).await
        }

        async fn delete_episode(&self, id: uuid::Uuid) -> do_memory_core::Result<()> {
            self.real_redb.delete_episode(id).await
        }

        async fn store_pattern(
            &self,
            pattern: &do_memory_core::Pattern,
        ) -> do_memory_core::Result<()> {
            self.real_redb.store_pattern(pattern).await
        }

        async fn get_pattern(
            &self,
            id: do_memory_core::episode::PatternId,
        ) -> do_memory_core::Result<Option<do_memory_core::Pattern>> {
            self.real_redb.get_pattern(id).await
        }

        async fn store_heuristic(
            &self,
            heuristic: &do_memory_core::Heuristic,
        ) -> do_memory_core::Result<()> {
            self.real_redb.store_heuristic(heuristic).await
        }

        async fn get_heuristic(
            &self,
            id: uuid::Uuid,
        ) -> do_memory_core::Result<Option<do_memory_core::Heuristic>> {
            self.real_redb.get_heuristic(id).await
        }

        async fn query_episodes_since(
            &self,
            since: chrono::DateTime<chrono::Utc>,
            limit: Option<usize>,
        ) -> do_memory_core::Result<Vec<Episode>> {
            self.real_redb.query_episodes_since(since, limit).await
        }

        async fn query_episodes_by_metadata(
            &self,
            key: &str,
            value: &str,
            limit: Option<usize>,
        ) -> do_memory_core::Result<Vec<Episode>> {
            self.real_redb
                .query_episodes_by_metadata(key, value, limit)
                .await
        }

        async fn store_embedding(
            &self,
            id: &str,
            embedding: Vec<f32>,
        ) -> do_memory_core::Result<()> {
            self.real_redb.store_embedding(id, embedding).await
        }

        async fn get_embedding(&self, id: &str) -> do_memory_core::Result<Option<Vec<f32>>> {
            self.real_redb.get_embedding(id).await
        }

        async fn delete_embedding(&self, id: &str) -> do_memory_core::Result<bool> {
            self.real_redb.delete_embedding(id).await
        }

        async fn store_embeddings_batch(
            &self,
            embeddings: Vec<(String, Vec<f32>)>,
        ) -> do_memory_core::Result<()> {
            self.real_redb.store_embeddings_batch(embeddings).await
        }

        async fn get_embeddings_batch(
            &self,
            ids: &[String],
        ) -> do_memory_core::Result<Vec<Option<Vec<f32>>>> {
            self.real_redb.get_embeddings_batch(ids).await
        }
    }

    let (turso, _turso_dir) = create_test_turso().await.unwrap();
    let (redb, _redb_dir) = create_test_redb().await.unwrap();
    let batch_call_count = Arc::new(AtomicUsize::new(0));

    let failing_cache = FailingBatchStorage {
        real_redb: redb,
        batch_call_count: Arc::clone(&batch_call_count),
    };

    let sync =
        StorageSynchronizer::new(Arc::new(turso), Arc::new(failing_cache)).with_batch_size(10);

    let context = TaskContext::default();
    let mut episodes = Vec::new();
    for i in 0..5 {
        let desc = if i == 2 {
            "BAD_EPISODE".to_string()
        } else {
            format!("Good task {i}")
        };
        let ep = Episode::new(desc, context.clone(), TaskType::Testing);
        episodes.push(ep);
    }

    for chunk in episodes.chunks(10) {
        sync.turso
            .store_episodes_batch(chunk.to_vec())
            .await
            .unwrap();
    }

    let since = chrono::Utc::now() - chrono::Duration::hours(1);
    let stats = sync.sync_all_recent_episodes(since).await.unwrap();

    // Verify batch was attempted
    assert!(
        batch_call_count.load(Ordering::SeqCst) > 0,
        "Batch store should have been attempted"
    );

    // 4 good episodes synced, 1 error
    assert_eq!(stats.episodes_synced, 4, "4 good episodes should be synced");
    assert_eq!(stats.errors, 1, "1 bad episode should be recorded as error");

    // Verify good episodes are in cache and bad episode is not
    for (i, ep) in episodes.iter().enumerate() {
        let cached = sync.redb.get_episode(ep.episode_id).await.unwrap();
        if i == 2 {
            assert!(cached.is_none(), "BAD_EPISODE should not be cached");
        } else {
            assert!(cached.is_some(), "Good episode {i} should be cached");
        }
    }
}

#[tokio::test]
async fn should_handle_per_item_timeout_in_fallback_loop() {
    use async_trait::async_trait;

    struct TimingOutItemStorage {
        real_redb: RedbStorage,
    }

    #[async_trait]
    impl do_memory_core::StorageBackendCapabilities for TimingOutItemStorage {}

    #[async_trait]
    impl do_memory_core::StorageBackend for TimingOutItemStorage {
        async fn store_episode(&self, episode: &Episode) -> do_memory_core::Result<()> {
            if episode.task_description == "TIMEOUT_EPISODE" {
                // Sleep longer than SYNC_EPISODE_TIMEOUT in test if needed, or trigger fallback path
                tokio::time::sleep(std::time::Duration::from_secs(31)).await;
            }
            self.real_redb.store_episode(episode).await
        }

        async fn store_episodes_batch(&self, _episodes: &[Episode]) -> do_memory_core::Result<()> {
            // Always fail batch write to trigger fallback
            Err(do_memory_core::Error::Storage(
                "Simulated batch failure".to_string(),
            ))
        }

        async fn get_episode(&self, id: uuid::Uuid) -> do_memory_core::Result<Option<Episode>> {
            self.real_redb.get_episode(id).await
        }

        async fn delete_episode(&self, id: uuid::Uuid) -> do_memory_core::Result<()> {
            self.real_redb.delete_episode(id).await
        }

        async fn store_pattern(
            &self,
            pattern: &do_memory_core::Pattern,
        ) -> do_memory_core::Result<()> {
            self.real_redb.store_pattern(pattern).await
        }

        async fn get_pattern(
            &self,
            id: do_memory_core::episode::PatternId,
        ) -> do_memory_core::Result<Option<do_memory_core::Pattern>> {
            self.real_redb.get_pattern(id).await
        }

        async fn store_heuristic(
            &self,
            heuristic: &do_memory_core::Heuristic,
        ) -> do_memory_core::Result<()> {
            self.real_redb.store_heuristic(heuristic).await
        }

        async fn get_heuristic(
            &self,
            id: uuid::Uuid,
        ) -> do_memory_core::Result<Option<do_memory_core::Heuristic>> {
            self.real_redb.get_heuristic(id).await
        }

        async fn query_episodes_since(
            &self,
            since: chrono::DateTime<chrono::Utc>,
            limit: Option<usize>,
        ) -> do_memory_core::Result<Vec<Episode>> {
            self.real_redb.query_episodes_since(since, limit).await
        }

        async fn query_episodes_by_metadata(
            &self,
            key: &str,
            value: &str,
            limit: Option<usize>,
        ) -> do_memory_core::Result<Vec<Episode>> {
            self.real_redb
                .query_episodes_by_metadata(key, value, limit)
                .await
        }

        async fn store_embedding(
            &self,
            id: &str,
            embedding: Vec<f32>,
        ) -> do_memory_core::Result<()> {
            self.real_redb.store_embedding(id, embedding).await
        }

        async fn get_embedding(&self, id: &str) -> do_memory_core::Result<Option<Vec<f32>>> {
            self.real_redb.get_embedding(id).await
        }

        async fn delete_embedding(&self, id: &str) -> do_memory_core::Result<bool> {
            self.real_redb.delete_embedding(id).await
        }

        async fn store_embeddings_batch(
            &self,
            embeddings: Vec<(String, Vec<f32>)>,
        ) -> do_memory_core::Result<()> {
            self.real_redb.store_embeddings_batch(embeddings).await
        }

        async fn get_embeddings_batch(
            &self,
            ids: &[String],
        ) -> do_memory_core::Result<Vec<Option<Vec<f32>>>> {
            self.real_redb.get_embeddings_batch(ids).await
        }
    }

    // Pause time or use tokio time control
    tokio::time::pause();

    let (turso, _turso_dir) = create_test_turso().await.unwrap();
    let (redb, _redb_dir) = create_test_redb().await.unwrap();

    let timeout_cache = TimingOutItemStorage { real_redb: redb };
    let sync =
        StorageSynchronizer::new(Arc::new(turso), Arc::new(timeout_cache)).with_batch_size(10);

    let context = TaskContext::default();
    let ep1 = Episode::new(
        "Good task 1".to_string(),
        context.clone(),
        TaskType::Testing,
    );
    let ep2 = Episode::new(
        "TIMEOUT_EPISODE".to_string(),
        context.clone(),
        TaskType::Testing,
    );

    sync.turso
        .store_episodes_batch(vec![ep1, ep2])
        .await
        .unwrap();

    let since = chrono::Utc::now() - chrono::Duration::hours(1);

    // Spawn task to advance virtual time if blocked
    let sync_task = tokio::spawn(async move { sync.sync_all_recent_episodes(since).await });

    tokio::time::advance(std::time::Duration::from_secs(35)).await;
    let stats = sync_task.await.unwrap().unwrap();

    assert_eq!(stats.episodes_synced, 1, "1 good episode should be synced");
    assert_eq!(
        stats.errors, 1,
        "1 timeout episode should be recorded as error"
    );
}

#[tokio::test]
async fn should_fall_back_to_per_item_store_when_batch_write_times_out() {
    use async_trait::async_trait;

    struct BatchTimingOutStorage {
        real_redb: RedbStorage,
        batch_attempts: std::sync::atomic::AtomicUsize,
        item_stores: std::sync::atomic::AtomicUsize,
    }

    #[async_trait]
    impl do_memory_core::StorageBackendCapabilities for BatchTimingOutStorage {}

    #[async_trait]
    impl do_memory_core::StorageBackend for BatchTimingOutStorage {
        async fn store_episodes_batch(&self, _episodes: &[Episode]) -> do_memory_core::Result<()> {
            // Block past SYNC_EPISODE_TIMEOUT so the batch future is cancelled and
            // the per-item fallback takes over.
            self.batch_attempts
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            tokio::time::sleep(Duration::from_secs(31)).await;
            Ok(())
        }

        async fn store_episode(&self, episode: &Episode) -> do_memory_core::Result<()> {
            self.item_stores
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.real_redb.store_episode(episode).await
        }

        async fn get_episode(&self, id: uuid::Uuid) -> do_memory_core::Result<Option<Episode>> {
            self.real_redb.get_episode(id).await
        }

        async fn delete_episode(&self, id: uuid::Uuid) -> do_memory_core::Result<()> {
            self.real_redb.delete_episode(id).await
        }

        async fn store_pattern(
            &self,
            pattern: &do_memory_core::Pattern,
        ) -> do_memory_core::Result<()> {
            self.real_redb.store_pattern(pattern).await
        }

        async fn get_pattern(
            &self,
            id: do_memory_core::episode::PatternId,
        ) -> do_memory_core::Result<Option<do_memory_core::Pattern>> {
            self.real_redb.get_pattern(id).await
        }

        async fn store_heuristic(
            &self,
            heuristic: &do_memory_core::Heuristic,
        ) -> do_memory_core::Result<()> {
            self.real_redb.store_heuristic(heuristic).await
        }

        async fn get_heuristic(
            &self,
            id: uuid::Uuid,
        ) -> do_memory_core::Result<Option<do_memory_core::Heuristic>> {
            self.real_redb.get_heuristic(id).await
        }

        async fn query_episodes_since(
            &self,
            since: chrono::DateTime<chrono::Utc>,
            limit: Option<usize>,
        ) -> do_memory_core::Result<Vec<Episode>> {
            self.real_redb.query_episodes_since(since, limit).await
        }

        async fn query_episodes_by_metadata(
            &self,
            key: &str,
            value: &str,
            limit: Option<usize>,
        ) -> do_memory_core::Result<Vec<Episode>> {
            self.real_redb
                .query_episodes_by_metadata(key, value, limit)
                .await
        }

        async fn store_embedding(
            &self,
            id: &str,
            embedding: Vec<f32>,
        ) -> do_memory_core::Result<()> {
            self.real_redb.store_embedding(id, embedding).await
        }

        async fn get_embedding(&self, id: &str) -> do_memory_core::Result<Option<Vec<f32>>> {
            self.real_redb.get_embedding(id).await
        }

        async fn delete_embedding(&self, id: &str) -> do_memory_core::Result<bool> {
            self.real_redb.delete_embedding(id).await
        }

        async fn store_embeddings_batch(
            &self,
            embeddings: Vec<(String, Vec<f32>)>,
        ) -> do_memory_core::Result<()> {
            self.real_redb.store_embeddings_batch(embeddings).await
        }

        async fn get_embeddings_batch(
            &self,
            ids: &[String],
        ) -> do_memory_core::Result<Vec<Option<Vec<f32>>>> {
            self.real_redb.get_embeddings_batch(ids).await
        }
    }

    tokio::time::pause();

    let (turso, _turso_dir) = create_test_turso().await.unwrap();
    let (redb, _redb_dir) = create_test_redb().await.unwrap();

    let cache = Arc::new(BatchTimingOutStorage {
        real_redb: redb,
        batch_attempts: std::sync::atomic::AtomicUsize::new(0),
        item_stores: std::sync::atomic::AtomicUsize::new(0),
    });
    let sync = StorageSynchronizer::new(Arc::new(turso), Arc::clone(&cache)).with_batch_size(10);

    let context = TaskContext::default();
    let ep1 = Episode::new(
        "Batch timeout 1".to_string(),
        context.clone(),
        TaskType::Testing,
    );
    let ep2 = Episode::new(
        "Batch timeout 2".to_string(),
        context.clone(),
        TaskType::Testing,
    );
    let ids = [ep1.episode_id, ep2.episode_id];

    sync.turso
        .store_episodes_batch(vec![ep1, ep2])
        .await
        .unwrap();

    let since = chrono::Utc::now() - chrono::Duration::hours(1);

    let sync_task = tokio::spawn(async move { sync.sync_all_recent_episodes(since).await });
    tokio::time::advance(Duration::from_secs(35)).await;
    let stats = sync_task.await.unwrap().unwrap();

    assert_eq!(
        stats.episodes_synced, 2,
        "per-item fallback must recover every episode the batch lost"
    );
    assert_eq!(
        stats.errors, 0,
        "a batch timeout alone is not an error once fallback succeeds"
    );

    // Proves the timeout arm (not a fast batch success) drove the sync: exactly one
    // batch attempt was abandoned and every episode went through per-item storage.
    assert_eq!(
        cache
            .batch_attempts
            .load(std::sync::atomic::Ordering::SeqCst),
        1,
        "the batch write should be attempted once and then abandoned"
    );
    assert_eq!(
        cache.item_stores.load(std::sync::atomic::Ordering::SeqCst),
        2,
        "both episodes must be stored by the per-item fallback"
    );

    for id in ids {
        let cached = cache.real_redb.get_episode(id).await.unwrap();
        assert!(
            cached.is_some(),
            "episode {id} missing from cache after fallback"
        );
    }
}

#[tokio::test]
async fn should_replace_stale_redb_episode_with_newer_turso_episode() {
    use do_memory_core::{MemoryConfig, SelfLearningMemory};

    let (turso, _turso_dir) = create_test_turso().await.unwrap();
    let (redb, _redb_dir) = create_test_redb().await.unwrap();

    let turso_arc: Arc<dyn StorageBackend> = Arc::new(turso);
    let redb_arc: Arc<dyn StorageBackend> = Arc::new(redb);

    let context = TaskContext::default();
    let t1 = chrono::Utc::now();
    let t2 = t1 + chrono::Duration::seconds(10);

    let mut stale_redb = Episode::new(
        "Stale redb description".to_string(),
        context.clone(),
        TaskType::Testing,
    );
    stale_redb.start_time = t1;
    stale_redb
        .metadata
        .insert("modified_at".to_string(), t1.to_rfc3339());
    redb_arc.store_episode(&stale_redb).await.unwrap();

    let mut newer_turso = Episode::new(
        "Newer Turso description".to_string(),
        context,
        TaskType::Testing,
    );
    newer_turso.episode_id = stale_redb.episode_id;
    newer_turso.start_time = t2;
    newer_turso
        .metadata
        .insert("modified_at".to_string(), t2.to_rfc3339());
    turso_arc.store_episode(&newer_turso).await.unwrap();

    let memory = SelfLearningMemory::with_storage(MemoryConfig::default(), turso_arc, redb_arc);

    let episodes = memory.get_all_episodes().await.unwrap();

    assert_eq!(episodes.len(), 1);
    assert_eq!(episodes[0].task_description, "Newer Turso description");
}

#[tokio::test]
async fn should_preserve_newer_redb_episode_over_older_turso_episode() {
    use do_memory_core::{MemoryConfig, SelfLearningMemory};

    let (turso, _turso_dir) = create_test_turso().await.unwrap();
    let (redb, _redb_dir) = create_test_redb().await.unwrap();

    let turso_arc: Arc<dyn StorageBackend> = Arc::new(turso);
    let redb_arc: Arc<dyn StorageBackend> = Arc::new(redb);

    let context = TaskContext::default();
    let t1 = chrono::Utc::now();
    let t2 = t1 + chrono::Duration::seconds(10);

    let mut newer_redb = Episode::new(
        "Newer redb description".to_string(),
        context.clone(),
        TaskType::Testing,
    );
    newer_redb.start_time = t2;
    newer_redb
        .metadata
        .insert("modified_at".to_string(), t2.to_rfc3339());
    redb_arc.store_episode(&newer_redb).await.unwrap();

    let mut older_turso = Episode::new(
        "Older Turso description".to_string(),
        context,
        TaskType::Testing,
    );
    older_turso.episode_id = newer_redb.episode_id;
    older_turso.start_time = t1;
    older_turso
        .metadata
        .insert("modified_at".to_string(), t1.to_rfc3339());
    turso_arc.store_episode(&older_turso).await.unwrap();

    let memory = SelfLearningMemory::with_storage(MemoryConfig::default(), turso_arc, redb_arc);

    let episodes = memory.get_all_episodes().await.unwrap();

    assert_eq!(episodes.len(), 1);
    assert_eq!(episodes[0].task_description, "Newer redb description");
}

#[tokio::test]
async fn should_follow_conservative_policy_for_missing_revision_metadata() {
    use do_memory_core::{MemoryConfig, SelfLearningMemory};

    let (turso, _turso_dir) = create_test_turso().await.unwrap();
    let (redb, _redb_dir) = create_test_redb().await.unwrap();

    let turso_arc: Arc<dyn StorageBackend> = Arc::new(turso);
    let redb_arc: Arc<dyn StorageBackend> = Arc::new(redb);

    let context = TaskContext::default();
    let now = chrono::Utc::now();
    let t1 = chrono::DateTime::from_timestamp(now.timestamp(), 0).unwrap();

    let mut redb_legacy = Episode::new(
        "Legacy redb payload".to_string(),
        context.clone(),
        TaskType::Testing,
    );
    redb_legacy.start_time = t1;
    redb_arc.store_episode(&redb_legacy).await.unwrap();

    let mut turso_legacy = Episode::new(
        "Legacy Turso payload".to_string(),
        context,
        TaskType::Testing,
    );
    turso_legacy.episode_id = redb_legacy.episode_id;
    turso_legacy.start_time = t1;
    turso_arc.store_episode(&turso_legacy).await.unwrap();

    let memory = SelfLearningMemory::with_storage(MemoryConfig::default(), turso_arc, redb_arc);

    let episodes = memory.get_all_episodes().await.unwrap();

    assert_eq!(episodes.len(), 1);
    assert_eq!(episodes[0].task_description, "Legacy Turso payload");
}

#[tokio::test]
async fn should_select_same_winner_in_get_all_episodes_and_context_backfill() {
    use do_memory_core::{MemoryConfig, SelfLearningMemory, TaskOutcome};

    let (turso1, _t1) = create_test_turso().await.unwrap();
    let (redb1, _r1) = create_test_redb().await.unwrap();
    let (turso2, _t2) = create_test_turso().await.unwrap();
    let (redb2, _r2) = create_test_redb().await.unwrap();

    let context = TaskContext::default();
    let ts1 = chrono::Utc::now();
    let ts2 = ts1 + chrono::Duration::seconds(10);

    let mut ep_redb1 = Episode::new(
        "Context task stale redb".to_string(),
        context.clone(),
        TaskType::Testing,
    );
    ep_redb1.complete(TaskOutcome::Success {
        verdict: "ok".to_string(),
        artifacts: vec![],
    });
    ep_redb1.start_time = ts1;
    ep_redb1
        .metadata
        .insert("modified_at".to_string(), ts1.to_rfc3339());

    let mut ep_turso1 = Episode::new(
        "Context task newer turso".to_string(),
        context.clone(),
        TaskType::Testing,
    );
    ep_turso1.episode_id = ep_redb1.episode_id;
    ep_turso1.complete(TaskOutcome::Success {
        verdict: "ok".to_string(),
        artifacts: vec![],
    });
    ep_turso1.start_time = ts2;
    ep_turso1
        .metadata
        .insert("modified_at".to_string(), ts2.to_rfc3339());

    let turso1_arc: Arc<dyn StorageBackend> = Arc::new(turso1);
    let redb1_arc: Arc<dyn StorageBackend> = Arc::new(redb1);
    let turso2_arc: Arc<dyn StorageBackend> = Arc::new(turso2);
    let redb2_arc: Arc<dyn StorageBackend> = Arc::new(redb2);

    redb1_arc.store_episode(&ep_redb1).await.unwrap();
    turso1_arc.store_episode(&ep_turso1).await.unwrap();

    redb2_arc.store_episode(&ep_redb1).await.unwrap();
    turso2_arc.store_episode(&ep_turso1).await.unwrap();

    // 1) Winner from get_all_episodes
    let config1 = MemoryConfig {
        quality_threshold: 0.0,
        pattern_extraction_threshold: 0.0,
        ..MemoryConfig::default()
    };
    let mem1 = SelfLearningMemory::with_storage(config1, turso1_arc, redb1_arc);
    let get_all_result = mem1.get_all_episodes().await.unwrap();

    // 2) Winner from context backfill
    let config2 = MemoryConfig {
        quality_threshold: 0.0,
        pattern_extraction_threshold: 0.0,
        ..MemoryConfig::default()
    };
    let mem2 = SelfLearningMemory::with_storage(config2, turso2_arc, redb2_arc);
    let backfilled = mem2
        .retrieve_relevant_context("Context task".to_string(), context, 1)
        .await;

    assert_eq!(get_all_result.len(), 1);
    assert_eq!(backfilled.len(), 1);
    assert_eq!(
        get_all_result[0].task_description,
        "Context task newer turso"
    );
    assert_eq!(backfilled[0].task_description, "Context task newer turso");
}

#[tokio::test]
#[allow(clippy::cast_precision_loss)]
async fn should_sync_large_volumes_using_batch_writes_100_and_1000() {
    // Test 100 and 1,000 episode scale sync
    let batch_size = 100;
    for count in [100, 1000] {
        let (turso, _turso_dir) = create_test_turso().await.unwrap();
        let (redb, _redb_dir) = create_test_redb().await.unwrap();
        let sync =
            StorageSynchronizer::new(Arc::new(turso), Arc::new(redb)).with_batch_size(batch_size);

        let context = TaskContext::default();
        let mut episodes = Vec::with_capacity(count);
        for i in 0..count {
            episodes.push(Episode::new(
                format!("Batch task {i}"),
                context.clone(),
                TaskType::Testing,
            ));
        }

        // Store episodes into source Turso storage
        for chunk in episodes.chunks(100) {
            sync.turso
                .store_episodes_batch(chunk.to_vec())
                .await
                .unwrap();
        }

        let since = chrono::Utc::now() - chrono::Duration::hours(1);
        let start_time = std::time::Instant::now();
        let stats = sync.sync_all_recent_episodes(since).await.unwrap();
        let elapsed = start_time.elapsed();

        assert_eq!(
            stats.episodes_synced, count,
            "Should sync all {count} episodes"
        );
        assert_eq!(stats.errors, 0, "Should have no errors for {count} sync");

        // Print measured throughput for inspection during benchmark runs
        let rate = (count as f64) / elapsed.as_secs_f64();
        println!("Synced {count} episodes in {elapsed:?} ({rate:.2} episodes/sec)");

        // Acceptance criteria 2 and 4: the cache must end up holding every
        // synced episode exactly once. Reading the whole page back instead of
        // sampling a few ids is what makes a silently dropped or duplicated
        // item detectable at this volume.
        let cached = sync
            .redb
            .query_episodes_modified_since(since, None, Some(count))
            .await
            .unwrap();
        assert_eq!(
            cached.len(),
            count,
            "Every episode must land in the cache exactly once, saw {} for {count}",
            cached.len()
        );
    }
}

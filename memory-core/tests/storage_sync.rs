//! BDD-style integration tests for storage synchronization between Turso and redb
//!
//! Tests verify that the memory system correctly synchronizes episodes, patterns,
//! and heuristics between durable storage (Turso) and cache (redb).
//!
//! All tests follow the Given-When-Then pattern for clarity.

use memory_core::sync::{ConflictResolution, StorageSynchronizer, SyncConfig};
use memory_core::{Episode, TaskContext, TaskType};
use memory_storage_redb::RedbStorage;
use memory_storage_turso::TursoStorage;
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
    let resolved = memory_core::sync::resolve_episode_conflict(
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
    let resolved = memory_core::sync::resolve_episode_conflict(
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
    let resolved = memory_core::sync::resolve_episode_conflict(
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

//! Integration tests for storage synchronization

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
        .map_err(|e| anyhow::anyhow!("Failed to create test database: {}", e))?;

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
async fn test_sync_episode_to_cache() {
    let (turso, _turso_dir) = create_test_turso().await.unwrap();
    let (redb, _redb_dir) = create_test_redb().await.unwrap();

    // Create synchronizer
    let sync = StorageSynchronizer::new(Arc::new(turso), Arc::new(redb));

    // Create and store an episode in Turso
    let context = TaskContext::default();
    let episode = Episode::new("Test task".to_string(), context, TaskType::Testing);
    let episode_id = episode.episode_id;

    sync.turso.store_episode(&episode).await.unwrap();

    // Sync to cache
    let result = sync.sync_episode_to_cache(episode_id).await;
    assert!(result.is_ok(), "Sync should succeed");

    // Verify it's in cache
    let cached = sync.redb.get_episode(episode_id).await.unwrap();
    assert!(cached.is_some(), "Episode should be in cache");
    assert_eq!(cached.unwrap().episode_id, episode_id);
}

#[tokio::test]
async fn test_sync_all_recent_episodes() {
    let (turso, _turso_dir) = create_test_turso().await.unwrap();
    let (redb, _redb_dir) = create_test_redb().await.unwrap();

    let sync = StorageSynchronizer::new(Arc::new(turso), Arc::new(redb));

    // Create multiple episodes in Turso
    let context = TaskContext::default();
    let mut episode_ids = Vec::new();

    for i in 0..5 {
        let episode = Episode::new(
            format!("Test task {}", i),
            context.clone(),
            TaskType::Testing,
        );
        episode_ids.push(episode.episode_id);
        sync.turso.store_episode(&episode).await.unwrap();
    }

    // Sync all recent episodes (since 1 hour ago)
    let since = chrono::Utc::now() - chrono::Duration::hours(1);
    let stats = sync.sync_all_recent_episodes(since).await.unwrap();

    // Verify all episodes were synced
    assert_eq!(stats.episodes_synced, 5, "Should sync all 5 episodes");
    assert_eq!(stats.errors, 0, "Should have no errors");

    // Verify they're all in cache
    for episode_id in episode_ids {
        let cached = sync.redb.get_episode(episode_id).await.unwrap();
        assert!(
            cached.is_some(),
            "Episode {} should be in cache",
            episode_id
        );
    }
}

#[tokio::test]
async fn test_sync_state_tracking() {
    let (turso, _turso_dir) = create_test_turso().await.unwrap();
    let (redb, _redb_dir) = create_test_redb().await.unwrap();

    let sync = StorageSynchronizer::new(Arc::new(turso), Arc::new(redb));

    // Initial state
    let state = sync.get_sync_state().await;
    assert_eq!(state.sync_count, 0);
    assert!(state.last_sync.is_none());

    // Create and sync an episode
    let context = TaskContext::default();
    let episode = Episode::new("Test task".to_string(), context, TaskType::Testing);
    sync.turso.store_episode(&episode).await.unwrap();

    let since = chrono::Utc::now() - chrono::Duration::hours(1);
    sync.sync_all_recent_episodes(since).await.unwrap();

    // Check state was updated
    let state = sync.get_sync_state().await;
    assert_eq!(state.sync_count, 1);
    assert!(state.last_sync.is_some());
    assert!(state.last_error.is_none());
}

#[tokio::test]
async fn test_periodic_sync() {
    let (turso, _turso_dir) = create_test_turso().await.unwrap();
    let (redb, _redb_dir) = create_test_redb().await.unwrap();

    let sync = Arc::new(StorageSynchronizer::new(Arc::new(turso), Arc::new(redb)));

    // Create an episode
    let context = TaskContext::default();
    let episode = Episode::new("Test task".to_string(), context, TaskType::Testing);
    let episode_id = episode.episode_id;
    sync.turso.store_episode(&episode).await.unwrap();

    // Start periodic sync with short interval (100ms for testing)
    let handle = sync.clone().start_periodic_sync(Duration::from_millis(100));

    // Wait for a couple of sync cycles
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Abort the background task
    handle.abort();

    // Verify the episode was synced
    let cached = sync.redb.get_episode(episode_id).await.unwrap();
    assert!(cached.is_some(), "Episode should be synced to cache");

    // Verify sync state
    let state = sync.get_sync_state().await;
    assert!(
        state.sync_count > 0,
        "Should have performed at least one sync"
    );
}

#[tokio::test]
async fn test_sync_with_missing_episode() {
    let (turso, _turso_dir) = create_test_turso().await.unwrap();
    let (redb, _redb_dir) = create_test_redb().await.unwrap();

    let sync = StorageSynchronizer::new(Arc::new(turso), Arc::new(redb));

    // Try to sync a non-existent episode
    let fake_id = uuid::Uuid::new_v4();
    let result = sync.sync_episode_to_cache(fake_id).await;

    assert!(result.is_err(), "Should fail for missing episode");
}

#[test]
fn test_conflict_resolution_turso_wins() {
    let context = TaskContext::default();
    let episode1 = Episode::new(
        "Task from Turso".to_string(),
        context.clone(),
        TaskType::Testing,
    );
    let mut episode2 = Episode::new("Task from redb".to_string(), context, TaskType::Testing);
    episode2.episode_id = episode1.episode_id; // Same ID, different content

    let resolved = memory_core::sync::resolve_episode_conflict(
        &episode1,
        &episode2,
        ConflictResolution::TursoWins,
    );
    assert_eq!(resolved.task_description, "Task from Turso");
}

#[test]
fn test_conflict_resolution_redb_wins() {
    let context = TaskContext::default();
    let episode1 = Episode::new(
        "Task from Turso".to_string(),
        context.clone(),
        TaskType::Testing,
    );
    let mut episode2 = Episode::new("Task from redb".to_string(), context, TaskType::Testing);
    episode2.episode_id = episode1.episode_id;

    let resolved = memory_core::sync::resolve_episode_conflict(
        &episode1,
        &episode2,
        ConflictResolution::RedbWins,
    );
    assert_eq!(resolved.task_description, "Task from redb");
}

#[test]
fn test_conflict_resolution_most_recent() {
    let context = TaskContext::default();

    // Create two episodes with different timestamps
    let episode1 = Episode::new("Older task".to_string(), context.clone(), TaskType::Testing);
    let mut episode2 = Episode::new("Newer task".to_string(), context, TaskType::Testing);
    episode2.episode_id = episode1.episode_id;

    // Make episode2 newer by setting end_time
    episode2.end_time = Some(chrono::Utc::now());

    let resolved = memory_core::sync::resolve_episode_conflict(
        &episode1,
        &episode2,
        ConflictResolution::MostRecent,
    );
    assert_eq!(
        resolved.task_description, "Newer task",
        "Should choose the newer episode"
    );
}

#[test]
fn test_sync_config_default() {
    let config = SyncConfig::default();
    assert_eq!(config.sync_interval, Duration::from_secs(300));
    assert_eq!(config.batch_size, 100);
    assert!(config.sync_patterns);
    assert!(config.sync_heuristics);
}

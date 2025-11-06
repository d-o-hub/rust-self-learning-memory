//! Integration tests for Turso storage

use memory_core::{Episode, TaskContext, TaskType};
use memory_storage_turso::{EpisodeQuery, TursoStorage};
use tempfile::TempDir;

async fn create_test_storage() -> anyhow::Result<(TursoStorage, TempDir)> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("test.db");

    // Create Turso storage with local file database
    let url = format!("file://{}", db_path.to_str().unwrap());
    let storage = TursoStorage::new(&url, "").await?;
    storage.initialize_schema().await?;
    Ok((storage, dir))
}

#[tokio::test]
#[ignore] // Requires proper Turso database setup
async fn test_store_and_retrieve_episode() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let context = TaskContext::default();
    let episode = Episode::new("Test task".to_string(), context, TaskType::Testing);

    // Store episode
    storage.store_episode(&episode).await.unwrap();

    // Retrieve episode
    let retrieved = storage.get_episode(episode.episode_id).await.unwrap();
    assert!(retrieved.is_some());

    let retrieved_episode = retrieved.unwrap();
    assert_eq!(retrieved_episode.episode_id, episode.episode_id);
    assert_eq!(retrieved_episode.task_description, "Test task");
}

#[tokio::test]
#[ignore] // Requires proper Turso database setup
async fn test_query_episodes() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    // Store multiple episodes
    for i in 0..3 {
        let context = TaskContext::default();
        let episode = Episode::new(format!("Task {}", i), context, TaskType::Testing);
        storage.store_episode(&episode).await.unwrap();
    }

    // Query episodes
    let query = EpisodeQuery {
        task_type: Some(TaskType::Testing),
        ..Default::default()
    };

    let episodes = storage.query_episodes(&query).await.unwrap();
    assert_eq!(episodes.len(), 3);
}

#[tokio::test]
#[ignore] // Requires proper Turso database setup
async fn test_delete_episode() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let context = TaskContext::default();
    let episode = Episode::new("Test task".to_string(), context, TaskType::Testing);

    // Store and delete
    storage.store_episode(&episode).await.unwrap();
    storage.delete_episode(episode.episode_id).await.unwrap();

    // Verify it's deleted
    let retrieved = storage.get_episode(episode.episode_id).await.unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
#[ignore] // Requires proper Turso database setup
async fn test_storage_statistics() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let stats = storage.get_statistics().await.unwrap();
    assert_eq!(stats.episode_count, 0);
    assert_eq!(stats.pattern_count, 0);
    assert_eq!(stats.heuristic_count, 0);
}

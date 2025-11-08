//! Integration tests for redb storage

use memory_core::{Episode, TaskContext, TaskType};
use memory_storage_redb::{RedbQuery, RedbStorage};
use tempfile::TempDir;

async fn create_test_storage() -> anyhow::Result<(RedbStorage, TempDir)> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("test.redb");

    let storage = RedbStorage::new(&db_path).await?;
    Ok((storage, dir))
}

#[tokio::test]
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
async fn test_get_all_episodes() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    // Store multiple episodes
    for i in 0..3 {
        let context = TaskContext::default();
        let episode = Episode::new(format!("Task {}", i), context, TaskType::Testing);
        storage.store_episode(&episode).await.unwrap();
    }

    // Get all episodes
    let query = RedbQuery { limit: None };
    let episodes = storage.get_all_episodes(&query).await.unwrap();
    assert_eq!(episodes.len(), 3);
}

#[tokio::test]
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
async fn test_embeddings() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let id = "test_embedding";
    let embedding = vec![0.1, 0.2, 0.3, 0.4];

    // Store embedding
    storage.store_embedding(id, &embedding).await.unwrap();

    // Retrieve embedding
    let retrieved = storage.get_embedding(id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), embedding);
}

#[tokio::test]
async fn test_metadata() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let key = "test_key";
    let value = "test_value";

    // Store metadata
    storage.store_metadata(key, value).await.unwrap();

    // Retrieve metadata
    let retrieved = storage.get_metadata(key).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), value);
}

#[tokio::test]
async fn test_clear_all() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    // Add some data
    let context = TaskContext::default();
    let episode = Episode::new("Test".to_string(), context, TaskType::Testing);
    storage.store_episode(&episode).await.unwrap();

    // Clear all
    storage.clear_all().await.unwrap();

    // Verify cleared
    let stats = storage.get_statistics().await.unwrap();
    assert_eq!(stats.episode_count, 0);
}

#[tokio::test]
async fn test_storage_statistics() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let stats = storage.get_statistics().await.unwrap();
    assert_eq!(stats.episode_count, 0);
    assert_eq!(stats.pattern_count, 0);
    assert_eq!(stats.heuristic_count, 0);
}

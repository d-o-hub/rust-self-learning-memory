use super::crud::*;
use crate::TursoStorage;
use do_memory_core::{
    Episode, Error, Result, TaskContext, TaskType, memory::checkpoint::CheckpointMeta,
};
use tempfile::TempDir;
use uuid::Uuid;

async fn create_test_storage() -> Result<(TursoStorage, TempDir)> {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");

    let db = libsql::Builder::new_local(&db_path)
        .build()
        .await
        .map_err(|e| Error::Storage(format!("Failed to create test database: {}", e)))?;

    let storage = TursoStorage::from_database(db)?;
    storage.initialize_schema().await?;

    Ok((storage, dir))
}

#[tokio::test]
async fn test_store_and_get_episode() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let episode = Episode::new(
        "Test episode".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    );

    let episode_id = episode.episode_id;
    storage.store_episode(&episode).await.unwrap();

    let retrieved = storage.get_episode(episode_id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().task_description, "Test episode");
}

#[tokio::test]
async fn test_delete_episode() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let episode = Episode::new(
        "To delete".to_string(),
        TaskContext::default(),
        TaskType::Debugging,
    );

    let episode_id = episode.episode_id;
    storage.store_episode(&episode).await.unwrap();

    storage.delete_episode(episode_id).await.unwrap();

    let retrieved = storage.get_episode(episode_id).await.unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_get_nonexistent_episode() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let nonexistent_id = Uuid::new_v4();
    let result = storage.get_episode(nonexistent_id).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_store_and_get_episode_persists_checkpoints() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let mut episode = Episode::new(
        "Checkpoint test".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    );
    episode.checkpoints.push(CheckpointMeta::new(
        "handoff".to_string(),
        2,
        Some("persist me".to_string()),
    ));

    let episode_id = episode.episode_id;
    storage.store_episode(&episode).await.unwrap();

    let retrieved = storage.get_episode(episode_id).await.unwrap().unwrap();
    assert_eq!(retrieved.checkpoints.len(), 1);
    assert_eq!(retrieved.checkpoints[0].reason, "handoff");
    assert_eq!(retrieved.checkpoints[0].step_number, 2);
    assert_eq!(retrieved.checkpoints[0].note.as_deref(), Some("persist me"));
}

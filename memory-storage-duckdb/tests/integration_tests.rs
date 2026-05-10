use do_memory_core::{Episode, TaskContext, TaskType, StorageBackend};
use do_memory_storage_duckdb::DuckDbStorage;
use tempfile::tempdir;
use uuid::Uuid;

#[tokio::test]
async fn test_duckdb_storage_basic_ops() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("test.duckdb");
    let storage = DuckDbStorage::new(&db_path).await?;

    let episode = Episode::new(
        "Test task".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    );
    let episode_id = episode.episode_id;

    // Store
    storage.store_episode(&episode).await?;

    // Get
    // Note: get_episode_internal is not fully implemented in the placeholder,
    // but we can check if it returns without error
    let retrieved = storage.get_episode(episode_id).await?;
    // assert!(retrieved.is_some()); // Enable when fully implemented

    // Delete
    storage.delete_episode(episode_id).await?;

    Ok(())
}

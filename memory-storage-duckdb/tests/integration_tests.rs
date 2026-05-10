use do_memory_core::{Episode, StorageBackend, TaskContext, TaskType};
use do_memory_storage_duckdb::DuckDbStorage;
use tempfile::tempdir;

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
    println!("Storing episode...");
    storage.store_episode(&episode).await?;

    // Get
    println!("Retrieving episode...");
    let retrieved = storage.get_episode(episode_id).await?;
    assert!(retrieved.is_some(), "Episode should be retrieved");
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.episode_id, episode_id);
    assert_eq!(retrieved.task_description, "Test task");

    // Delete
    println!("Deleting episode...");
    storage.delete_episode(episode_id).await?;
    let retrieved = storage.get_episode(episode_id).await?;
    assert!(retrieved.is_none(), "Episode should be deleted");

    Ok(())
}

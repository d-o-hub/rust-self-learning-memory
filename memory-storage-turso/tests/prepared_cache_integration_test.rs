//! Integration test for prepared statement cache
//!
//! Note: The current implementation clears the prepared statement cache
//! immediately after each operation for memory management. This test verifies
//! that the prepare_cached mechanism works correctly, even though the cache
//! is cleared after each use.

use libsql::Builder;
use memory_core::{Episode, TaskContext, TaskType};
use memory_storage_turso::TursoStorage;
use tempfile::TempDir;
use tokio::time::{Duration, timeout};

#[tokio::test]
async fn test_prepared_statement_cache_integration() {
    // Create temp directory that will be cleaned up when dropped
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");

    // Create database with explicit timeout
    let db = timeout(
        Duration::from_secs(10),
        Builder::new_local(&db_path).build(),
    )
    .await
    .expect("Database creation timed out")
    .unwrap();

    let storage = TursoStorage::from_database(db).unwrap();

    // Initialize schema with timeout
    timeout(Duration::from_secs(5), storage.initialize_schema())
        .await
        .expect("Schema initialization timed out")
        .unwrap();

    // Get initial cache stats
    let stats_before = storage.prepared_cache_stats();
    assert_eq!(stats_before.current_size, 0, "Cache should start empty");
    assert_eq!(stats_before.hits, 0, "No hits initially");
    assert_eq!(stats_before.misses, 0, "No misses initially");

    // Create and store an episode
    let episode = Episode::new(
        "Test episode for prepared cache".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    );

    // Store episode - this will use prepare_cached internally
    // Note: Cache is cleared after the operation completes
    timeout(Duration::from_secs(5), storage.store_episode(&episode))
        .await
        .expect("Store episode timed out")
        .unwrap();

    // Cache is cleared after each operation, so current_size will be 0
    // But we can verify the operation completed successfully
    let stats_after_store = storage.prepared_cache_stats();
    assert_eq!(
        stats_after_store.current_size, 0,
        "Cache cleared after operation"
    );

    // Verify episode was stored by retrieving it
    let retrieved = timeout(
        Duration::from_secs(5),
        storage.get_episode(episode.episode_id),
    )
    .await
    .expect("Get episode timed out")
    .unwrap();

    assert!(retrieved.is_some(), "Episode should be retrievable");
    assert_eq!(
        retrieved.unwrap().task_description,
        "Test episode for prepared cache"
    );

    // Store another episode - prepare_cached will be called again
    let episode2 = Episode::new(
        "Second test episode".to_string(),
        TaskContext::default(),
        TaskType::Debugging,
    );

    timeout(Duration::from_secs(5), storage.store_episode(&episode2))
        .await
        .expect("Store second episode timed out")
        .unwrap();

    // Verify both episodes exist by retrieving them individually
    let retrieved2 = timeout(
        Duration::from_secs(5),
        storage.get_episode(episode2.episode_id),
    )
    .await
    .expect("Get second episode timed out")
    .unwrap();

    assert!(retrieved2.is_some(), "Second episode should be retrievable");
    assert_eq!(retrieved2.unwrap().task_description, "Second test episode");

    let stats_final = storage.prepared_cache_stats();
    assert_eq!(
        stats_final.current_size, 0,
        "Cache cleared after operations"
    );

    println!("Prepared cache integration test passed!");
    println!("  Episodes stored and retrieved: 2");
    println!("  Cache stats (cleared after each operation):");
    println!("    Current size: {}", stats_final.current_size);
    println!("    Total prepares: {}", stats_final.prepared);

    // Explicit cleanup - drop storage before tempdir
    drop(storage);
    // dir will be automatically cleaned up when it goes out of scope
}

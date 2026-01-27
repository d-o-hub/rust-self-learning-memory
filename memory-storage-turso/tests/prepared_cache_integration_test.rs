//! Integration test for prepared statement cache

use libsql::Builder;
use memory_core::{Episode, TaskContext, TaskType};
use memory_storage_turso::TursoStorage;
use tempfile::TempDir;

#[tokio::test]
async fn test_prepared_statement_cache_integration() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");

    let db = Builder::new_local(&db_path).build().await.unwrap();
    let storage = TursoStorage::from_database(db).unwrap();
    storage.initialize_schema().await.unwrap();

    // Create and store an episode
    let episode = Episode::new(
        "Test episode for prepared cache".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    );

    // Get initial cache stats
    let stats_before = storage.prepared_cache_stats();
    assert_eq!(stats_before.current_size, 0);
    assert_eq!(stats_before.hits, 0);
    assert_eq!(stats_before.misses, 0);

    // Store episode - should prepare statement (miss)
    storage.store_episode(&episode).await.unwrap();
    let stats_after_store = storage.prepared_cache_stats();
    assert!(stats_after_store.current_size > 0);
    assert_eq!(stats_after_store.misses, 1);

    // Get episode - should use cached statement (hit)
    let retrieved = storage.get_episode(episode.episode_id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(
        retrieved.unwrap().task_description,
        "Test episode for prepared cache"
    );

    let stats_after_get = storage.prepared_cache_stats();
    assert!(stats_after_get.hits >= 1);

    // Store again - should be a cache hit now
    let episode2 = Episode::new(
        "Second test episode".to_string(),
        TaskContext::default(),
        TaskType::Debugging,
    );
    storage.store_episode(&episode2).await.unwrap();

    let stats_final = storage.prepared_cache_stats();
    assert!(stats_final.hits > 0);
    assert_eq!(stats_final.misses, 1); // First statement prepare
    assert!(stats_final.current_size > 0);

    println!("Prepared cache stats:");
    println!("  Hits: {}", stats_final.hits);
    println!("  Misses: {}", stats_final.misses);
    println!("  Prepared: {}", stats_final.prepared);
    println!("  Current size: {}", stats_final.current_size);
    println!("  Hit rate: {:.2}%", stats_final.hit_rate() * 100.0);
}

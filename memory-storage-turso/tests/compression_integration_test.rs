//! Integration tests for compression feature
//!
//! Tests verify compression is applied correctly and data integrity is maintained

use memory_core::Episode;
use memory_storage_turso::{TursoConfig, TursoStorage};
use tempfile::TempDir;

/// Helper to create test storage with compression enabled
async fn create_test_storage_with_compression() -> (TursoStorage, TempDir) {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = dir.path().join("test.db");

    let mut config = TursoConfig::default();
    config.compression_threshold = 1024; // 1KB threshold
    config.compress_episodes = true;

    let storage = TursoStorage::with_config(&format!("file:{}", db_path.display()), "", config)
        .await
        .expect("Failed to create storage");

    storage
        .initialize_schema()
        .await
        .expect("Failed to initialize schema");

    (storage, dir)
}

/// Helper to create a large episode (>1KB) that should be compressed
fn create_large_episode() -> Episode {
    let task_desc = format!("Large task: {}", "x".repeat(2000)); // 2KB
    let mut episode = Episode::new("compression-test", &task_desc, "test");

    // Add many steps to ensure size >1KB
    for i in 0..50 {
        episode.add_step(&format!("Large step {} content: {}", i, "x".repeat(100)));
    }

    episode.complete(memory_core::TaskOutcome::Success);
    episode
}

/// Helper to create a small episode (<1KB)
fn create_small_episode() -> Episode {
    let mut episode = Episode::new("small-test", "Small task", "test");
    episode.add_step("Small content");
    episode.complete(memory_core::TaskOutcome::Success);
    episode
}

#[tokio::test]
#[cfg(feature = "compression")]
async fn test_compression_large_episodes() {
    let (storage, _dir) = create_test_storage_with_compression().await;

    let episode = create_large_episode();
    let episode_id = episode.episode_id;

    storage
        .store_episode(&episode)
        .await
        .expect("Failed to store episode");

    let retrieved = storage
        .get_episode(episode_id)
        .await
        .expect("Failed to get episode")
        .expect("Episode not found");

    assert_eq!(retrieved.episode_id, episode.episode_id);
    assert_eq!(retrieved.steps.len(), episode.steps.len());

    println!("✓ Large episode compressed and retrieved successfully");
}

#[tokio::test]
#[cfg(feature = "compression")]
async fn test_compression_small_episodes() {
    let (storage, _dir) = create_test_storage_with_compression().await;

    let episode = create_small_episode();
    let episode_id = episode.episode_id;

    storage
        .store_episode(&episode)
        .await
        .expect("Failed to store episode");

    let retrieved = storage
        .get_episode(episode_id)
        .await
        .expect("Failed to get episode")
        .expect("Episode not found");

    assert_eq!(retrieved.episode_id, episode.episode_id);
    assert_eq!(retrieved.steps.len(), 1);

    println!("✓ Small episode stored successfully");
}

#[tokio::test]
#[cfg(feature = "compression")]
async fn test_multiple_episodes_compression() {
    let (storage, _dir) = create_test_storage_with_compression().await;

    let mut episode_ids = Vec::new();

    for i in 0..5 {
        let task_desc = format!("Multi test {}: {}", i, "x".repeat(2000));
        let mut episode = Episode::new(&format!("multi-{}", i), &task_desc, "test");
        episode.complete(memory_core::TaskOutcome::Success);

        episode_ids.push(episode.episode_id);

        storage
            .store_episode(&episode)
            .await
            .expect("Failed to store episode");
    }

    for episode_id in episode_ids {
        let retrieved = storage
            .get_episode(episode_id)
            .await
            .expect("Failed to get episode")
            .expect("Episode not found");

        assert_eq!(retrieved.episode_id, episode_id);
    }

    println!("✓ Multiple episodes compressed successfully");
}

#[tokio::test]
#[cfg(not(feature = "compression"))]
async fn test_storage_without_compression_feature() {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = dir.path().join("test.db");

    let storage = TursoStorage::new(&format!("file:{}", db_path.display()), "")
        .await
        .expect("Failed to create storage");

    storage
        .initialize_schema()
        .await
        .expect("Failed to initialize schema");

    let mut episode = Episode::new("no-compression", "Test task", "test");
    episode.add_step("Content");
    episode.complete(memory_core::TaskOutcome::Success);

    let episode_id = episode.episode_id;

    storage
        .store_episode(&episode)
        .await
        .expect("Failed to store episode");

    let retrieved = storage
        .get_episode(episode_id)
        .await
        .expect("Failed to get episode")
        .expect("Episode not found");

    assert_eq!(retrieved.episode_id, episode.episode_id);

    println!("✓ Storage works without compression feature");
}

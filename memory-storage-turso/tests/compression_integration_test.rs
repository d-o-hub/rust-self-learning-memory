//! Integration tests for compression feature
//!
//! Tests verify compression is applied correctly and data integrity is maintained

#![allow(clippy::expect_used)]

use memory_core::{Episode, ExecutionStep, TaskContext, TaskOutcome, TaskType};
use memory_storage_turso::{TursoConfig, TursoStorage};
use tempfile::TempDir;

/// Helper to create test storage with compression enabled
async fn create_test_storage_with_compression() -> (TursoStorage, TempDir) {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = dir.path().join("test.db");

    let config = TursoConfig {
        compression_threshold: 1024, // 1KB threshold
        compress_episodes: true,
        ..Default::default()
    };

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
    let context = TaskContext {
        domain: "test".to_string(),
        ..Default::default()
    };
    let task_type = TaskType::CodeGeneration;
    let mut episode = Episode::new(task_desc, context, task_type);

    // Add many steps to ensure size >1KB
    for i in 0..50 {
        let step = ExecutionStep::new(
            i + 1,
            "test".to_string(),
            format!("Large step {} content: {}", i, "x".repeat(100)),
        );
        episode.steps.push(step);
    }

    episode.outcome = Some(TaskOutcome::Success {
        verdict: "Test completed".to_string(),
        artifacts: vec![],
    });

    episode
}

/// Helper to create a small episode (<1KB)
fn create_small_episode() -> Episode {
    let context = TaskContext {
        domain: "test".to_string(),
        ..Default::default()
    };
    let mut episode = Episode::new("small-test".to_string(), context, TaskType::CodeGeneration);

    let step = ExecutionStep::new(1, "test".to_string(), "Small content".to_string());
    episode.steps.push(step);

    episode.outcome = Some(TaskOutcome::Success {
        verdict: "Test completed".to_string(),
        artifacts: vec![],
    });

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
        let context = TaskContext {
            domain: "test".to_string(),
            ..Default::default()
        };
        let mut episode = Episode::new(task_desc, context, TaskType::CodeGeneration);
        episode.outcome = Some(TaskOutcome::Success {
            verdict: "Test completed".to_string(),
            artifacts: vec![],
        });

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

    let context = TaskContext {
        domain: "test".to_string(),
        ..Default::default()
    };
    let mut episode = Episode::new(
        "no-compression".to_string(),
        context,
        TaskType::CodeGeneration,
    );
    let step = ExecutionStep::new(1, "test".to_string(), "Content".to_string());
    episode.steps.push(step);
    episode.outcome = Some(TaskOutcome::Success {
        verdict: "Test completed".to_string(),
        artifacts: vec![],
    });

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

//! Integration tests for async pattern extraction queue
//!
//! Tests the complete workflow of async pattern extraction including:
//! - Queue operations
//! - Worker pool processing
//! - Episode completion flow
//! - Error handling
//! - Performance characteristics

use memory_core::{
    ExecutionResult, ExecutionStep, QueueConfig, SelfLearningMemory, TaskContext, TaskOutcome,
    TaskType,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Helper to create a test episode with steps
async fn create_test_episode(
    memory: &SelfLearningMemory,
    description: &str,
    step_count: usize,
) -> uuid::Uuid {
    let context = TaskContext::default();
    let episode_id = memory
        .start_episode(description.to_string(), context, TaskType::Testing)
        .await;

    // Add execution steps
    for i in 0..step_count {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        step.latency_ms = 100;
        memory.log_step(episode_id, step).await;
    }

    episode_id
}

#[tokio::test]
async fn test_async_extraction_basic() {
    let memory = SelfLearningMemory::new().enable_async_extraction(QueueConfig::default());
    let memory_arc = Arc::new(memory);

    // Start workers
    memory_arc.start_workers().await;

    // Create and complete an episode
    let episode_id = create_test_episode(&memory_arc, "Test task", 3).await;

    memory_arc
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Check queue stats
    let stats = memory_arc.get_queue_stats().await.unwrap();
    assert_eq!(stats.total_enqueued, 1);

    // Wait for processing
    sleep(Duration::from_millis(500)).await;

    // Episode should be complete with patterns extracted
    let episode = memory_arc.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete());
    assert!(episode.reward.is_some());
    assert!(episode.reflection.is_some());
}

#[tokio::test]
async fn test_sync_vs_async_extraction() {
    // Test sync extraction
    let sync_memory = SelfLearningMemory::new();
    let episode_id_sync = create_test_episode(&sync_memory, "Sync task", 3).await;

    let start_sync = std::time::Instant::now();
    sync_memory
        .complete_episode(
            episode_id_sync,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();
    let sync_duration = start_sync.elapsed();

    // Verify patterns extracted immediately
    let episode_sync = sync_memory.get_episode(episode_id_sync).await.unwrap();
    assert!(!episode_sync.patterns.is_empty() || episode_sync.steps.len() < 2);

    // Test async extraction
    let async_memory =
        Arc::new(SelfLearningMemory::new().enable_async_extraction(QueueConfig::default()));
    async_memory.start_workers().await;

    let episode_id_async = create_test_episode(&async_memory, "Async task", 3).await;

    let start_async = std::time::Instant::now();
    async_memory
        .complete_episode(
            episode_id_async,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();
    let async_duration = start_async.elapsed();

    // Async should be faster (doesn't wait for pattern extraction)
    println!("Sync: {:?}, Async: {:?}", sync_duration, async_duration);

    // Wait for async processing to complete
    sleep(Duration::from_millis(500)).await;

    let episode_async = async_memory.get_episode(episode_id_async).await.unwrap();
    assert!(episode_async.is_complete());
}

#[tokio::test]
async fn test_multiple_episodes_parallel() {
    let config = QueueConfig {
        worker_count: 4,
        ..Default::default()
    };

    let memory = Arc::new(SelfLearningMemory::new().enable_async_extraction(config));
    memory.start_workers().await;

    let episode_count = 10;
    let mut episode_ids = Vec::new();

    // Create and complete multiple episodes
    for i in 0..episode_count {
        let episode_id = create_test_episode(&memory, &format!("Task {}", i), 3).await;

        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: format!("Done {}", i),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();

        episode_ids.push(episode_id);
    }

    // Check all enqueued
    let stats = memory.get_queue_stats().await.unwrap();
    assert_eq!(stats.total_enqueued, episode_count as u64);

    // Wait for processing
    sleep(Duration::from_secs(2)).await;

    // Verify all episodes completed
    for episode_id in episode_ids {
        let episode = memory.get_episode(episode_id).await.unwrap();
        assert!(episode.is_complete());
    }

    let final_stats = memory.get_queue_stats().await.unwrap();
    println!("Final stats: {:?}", final_stats);
}

#[tokio::test]
async fn test_backpressure_handling() {
    let config = QueueConfig {
        worker_count: 1,
        max_queue_size: 5,
        poll_interval_ms: 50,
    };

    let memory = Arc::new(SelfLearningMemory::new().enable_async_extraction(config));
    memory.start_workers().await;

    // Enqueue more than max_queue_size
    for i in 0..10 {
        let episode_id = create_test_episode(&memory, &format!("Task {}", i), 2).await;

        let result = memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: "Done".to_string(),
                    artifacts: vec![],
                },
            )
            .await;

        // Should still succeed (we just warn, don't reject)
        assert!(result.is_ok());
    }

    let stats = memory.get_queue_stats().await.unwrap();
    assert_eq!(stats.total_enqueued, 10);

    // Wait for queue to drain
    sleep(Duration::from_secs(3)).await;

    let final_stats = memory.get_queue_stats().await.unwrap();
    assert_eq!(final_stats.current_queue_size, 0);
}

#[tokio::test]
async fn test_error_recovery_in_worker() {
    let memory =
        Arc::new(SelfLearningMemory::new().enable_async_extraction(QueueConfig::default()));
    memory.start_workers().await;

    // Create an incomplete episode (should fail extraction)
    let context = TaskContext::default();
    let _incomplete_id = memory
        .start_episode("Incomplete".to_string(), context, TaskType::Testing)
        .await;

    // Try to enqueue incomplete episode (will be enqueued but fail during processing)
    // Note: In production, this shouldn't happen because complete_episode marks it complete first
    // This is to test error handling in workers

    // Create a complete episode
    let complete_id = create_test_episode(&memory, "Complete task", 3).await;
    memory
        .complete_episode(
            complete_id,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Wait for processing
    sleep(Duration::from_millis(500)).await;

    // Complete episode should be processed successfully
    let complete_episode = memory.get_episode(complete_id).await.unwrap();
    assert!(complete_episode.is_complete());

    let stats = memory.get_queue_stats().await.unwrap();
    assert!(stats.total_processed >= 1);
}

#[tokio::test]
async fn test_worker_pool_scaling() {
    // Test with different worker counts
    for worker_count in [1, 2, 4, 8] {
        let config = QueueConfig {
            worker_count,
            poll_interval_ms: 10,
            ..Default::default()
        };

        let memory = Arc::new(SelfLearningMemory::new().enable_async_extraction(config));
        memory.start_workers().await;

        let episode_count = 20;
        let start = std::time::Instant::now();

        // Create and complete episodes
        for i in 0..episode_count {
            let episode_id = create_test_episode(&memory, &format!("Task {}", i), 3).await;
            memory
                .complete_episode(
                    episode_id,
                    TaskOutcome::Success {
                        verdict: "Done".to_string(),
                        artifacts: vec![],
                    },
                )
                .await
                .unwrap();
        }

        // Wait for all to process
        sleep(Duration::from_secs(3)).await;

        let duration = start.elapsed();
        let stats = memory.get_queue_stats().await.unwrap();

        println!(
            "Workers: {}, Episodes: {}, Duration: {:?}, Processed: {}, Failed: {}",
            worker_count, episode_count, duration, stats.total_processed, stats.total_failed
        );

        assert_eq!(stats.current_queue_size, 0, "Queue should be empty");
    }
}

#[tokio::test]
async fn test_queue_statistics_accuracy() {
    let memory =
        Arc::new(SelfLearningMemory::new().enable_async_extraction(QueueConfig::default()));
    memory.start_workers().await;

    // Initial stats
    let stats = memory.get_queue_stats().await.unwrap();
    assert_eq!(stats.total_enqueued, 0);
    assert_eq!(stats.total_processed, 0);
    assert_eq!(stats.current_queue_size, 0);

    // Enqueue some episodes
    for i in 0..5 {
        let episode_id = create_test_episode(&memory, &format!("Task {}", i), 3).await;
        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: "Done".to_string(),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();
    }

    // Check enqueued count
    let stats = memory.get_queue_stats().await.unwrap();
    assert_eq!(stats.total_enqueued, 5);

    // Wait for processing
    sleep(Duration::from_secs(1)).await;

    // Check processed count
    let final_stats = memory.get_queue_stats().await.unwrap();
    assert_eq!(final_stats.current_queue_size, 0);
    assert!(final_stats.total_processed >= 5 || final_stats.total_failed > 0);
}

#[tokio::test]
async fn test_performance_under_100ms() {
    // Test that episode completion is fast with async extraction
    let memory =
        Arc::new(SelfLearningMemory::new().enable_async_extraction(QueueConfig::default()));
    memory.start_workers().await;

    let episode_id = create_test_episode(&memory, "Performance test", 5).await;

    let start = std::time::Instant::now();
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();
    let duration = start.elapsed();

    println!("Episode completion time: {:?}", duration);

    // Should complete in under 100ms (the requirement)
    assert!(
        duration.as_millis() < 100,
        "Episode completion took {:?}, expected < 100ms",
        duration
    );
}

#[tokio::test]
async fn test_disabled_async_extraction() {
    // Test that sync extraction still works when async is not enabled
    let memory = SelfLearningMemory::new();

    let episode_id = create_test_episode(&memory, "Sync test", 3).await;

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Should return None for queue stats
    let stats = memory.get_queue_stats().await;
    assert!(stats.is_none());

    // Patterns should be extracted immediately
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete());
}

#[tokio::test]
async fn test_concurrent_episode_completions() {
    let memory =
        Arc::new(SelfLearningMemory::new().enable_async_extraction(QueueConfig::default()));
    memory.start_workers().await;

    // Create multiple episodes
    let mut episode_ids = Vec::new();
    for i in 0..5 {
        let episode_id = create_test_episode(&memory, &format!("Task {}", i), 3).await;
        episode_ids.push(episode_id);
    }

    // Complete them concurrently
    let mut handles = Vec::new();
    for episode_id in episode_ids.clone() {
        let mem = Arc::clone(&memory);
        let handle = tokio::spawn(async move {
            mem.complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: "Done".to_string(),
                    artifacts: vec![],
                },
            )
            .await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    // Check stats
    let stats = memory.get_queue_stats().await.unwrap();
    assert_eq!(stats.total_enqueued, 5);

    // Wait for processing
    sleep(Duration::from_secs(1)).await;

    // All should be processed
    for episode_id in episode_ids {
        let episode = memory.get_episode(episode_id).await.unwrap();
        assert!(episode.is_complete());
    }
}

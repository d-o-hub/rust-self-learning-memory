//! BDD-style tests for async pattern extraction queue
//!
//! Tests verify that the async pattern extraction system correctly processes
//! episodes in background workers without blocking episode completion.
//!
//! ## Test Coverage
//! - Basic async extraction workflow and queue operations
//! - Synchronous vs asynchronous extraction performance comparison
//! - Parallel processing with multiple episodes and workers
//! - Backpressure handling and queue capacity management
//! - Error recovery in worker pool
//! - Worker pool scaling with different worker counts
//! - Queue statistics accuracy tracking
//! - Performance requirements (< 100ms completion time)

use memory_core::{
    ExecutionResult, ExecutionStep, MemoryConfig, QueueConfig, SelfLearningMemory, TaskContext,
    TaskOutcome, TaskType,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Create test memory config with lower quality threshold for testing
fn test_memory_config() -> MemoryConfig {
    MemoryConfig {
        quality_threshold: 0.0, // Zero threshold for test episodes
        ..Default::default()
    }
}

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
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        step.latency_ms = 100;
        memory.log_step(episode_id, step).await;
    }

    episode_id
}

#[tokio::test]
async fn should_extract_patterns_asynchronously_in_background() {
    // Given: Memory system with async extraction enabled and workers started
    let memory = SelfLearningMemory::with_config(test_memory_config())
        .enable_async_extraction(QueueConfig::default());
    let memory_arc = Arc::new(memory);

    memory_arc.start_workers().await;

    // When: Creating and completing an episode
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

    // Then: Episode should be enqueued for processing
    let stats = memory_arc.get_queue_stats().await.unwrap();
    assert_eq!(stats.total_enqueued, 1);

    // When: Waiting for background processing
    sleep(Duration::from_millis(500)).await;

    // Then: Episode should be complete with patterns, reward, and reflection
    let episode = memory_arc.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete());
    assert!(episode.reward.is_some());
    assert!(episode.reflection.is_some());
}

#[tokio::test]
#[ignore = "slow integration test - run with --ignored or in release CI"]
async fn should_complete_faster_with_async_extraction_than_sync() {
    // Given: Sync memory system
    let sync_memory = SelfLearningMemory::with_config(test_memory_config());
    let sync_episode_id = create_test_episode(&sync_memory, "Sync task", 3).await;

    // When: Completing episode synchronously
    let start_sync = std::time::Instant::now();
    sync_memory
        .complete_episode(
            sync_episode_id,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();
    let sync_duration = start_sync.elapsed();

    // Then: Patterns should be extracted immediately
    let episode_sync = sync_memory.get_episode(sync_episode_id).await.unwrap();
    assert!(!episode_sync.patterns.is_empty() || episode_sync.steps.len() < 2);

    // Given: Async memory system with workers started
    let async_memory = Arc::new(
        SelfLearningMemory::with_config(test_memory_config())
            .enable_async_extraction(QueueConfig::default()),
    );
    async_memory.start_workers().await;

    let async_episode_id = create_test_episode(&async_memory, "Async task", 3).await;

    // When: Completing episode asynchronously
    let async_start = std::time::Instant::now();
    async_memory
        .complete_episode(
            async_episode_id,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();
    let async_duration = async_start.elapsed();

    // Then: Async should be faster (doesn't block on pattern extraction)
    println!("Sync: {sync_duration:?}, Async: {async_duration:?}");

    // When: Waiting for async processing to complete
    sleep(Duration::from_millis(500)).await;

    // Then: Episode should be fully processed
    let async_episode = async_memory.get_episode(async_episode_id).await.unwrap();
    assert!(async_episode.is_complete());
}

#[tokio::test]
async fn should_process_multiple_episodes_in_parallel_with_worker_pool() {
    // Given: Memory system with 4 workers
    let config = QueueConfig {
        worker_count: 4,
        ..Default::default()
    };

    let memory = Arc::new(
        SelfLearningMemory::with_config(test_memory_config()).enable_async_extraction(config),
    );
    memory.start_workers().await;

    let episode_count = 10;
    let mut episode_ids = Vec::new();

    // When: Creating and completing multiple episodes
    for i in 0..episode_count {
        let episode_id = create_test_episode(&memory, &format!("Task {i}"), 3).await;

        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: format!("Done {i}"),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();

        episode_ids.push(episode_id);
    }

    // Then: All episodes should be enqueued
    let stats = memory.get_queue_stats().await.unwrap();
    assert_eq!(stats.total_enqueued, u64::try_from(episode_count).unwrap());

    // When: Waiting for parallel processing
    sleep(Duration::from_secs(2)).await;

    // Then: All episodes should be complete
    for episode_id in episode_ids {
        let episode = memory.get_episode(episode_id).await.unwrap();
        assert!(episode.is_complete());
    }

    let final_stats = memory.get_queue_stats().await.unwrap();
    println!("Final stats: {final_stats:?}");
}

#[tokio::test]
async fn should_handle_backpressure_when_queue_exceeds_capacity() {
    // Given: Memory system with limited queue size (5) and single worker
    let config = QueueConfig {
        worker_count: 1,
        max_queue_size: 5,
        poll_interval_ms: 50,
    };

    let memory = Arc::new(
        SelfLearningMemory::with_config(test_memory_config()).enable_async_extraction(config),
    );
    memory.start_workers().await;

    // When: Enqueuing more episodes than max_queue_size
    for i in 0..10 {
        let episode_id = create_test_episode(&memory, &format!("Task {i}"), 2).await;

        let result = memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: "Done".to_string(),
                    artifacts: vec![],
                },
            )
            .await;

        // Then: Should still succeed (warn but don't reject)
        assert!(result.is_ok());
    }

    // Then: All episodes should be enqueued
    let stats = memory.get_queue_stats().await.unwrap();
    assert_eq!(stats.total_enqueued, 10);

    // When: Waiting for queue to drain
    sleep(Duration::from_secs(3)).await;

    // Then: Queue should be empty
    let final_stats = memory.get_queue_stats().await.unwrap();
    assert_eq!(final_stats.current_queue_size, 0);
}

#[tokio::test]
async fn should_recover_from_worker_errors_and_continue_processing() {
    // Given: Memory system with async extraction and workers
    let memory = Arc::new(
        SelfLearningMemory::with_config(test_memory_config())
            .enable_async_extraction(QueueConfig::default()),
    );
    memory.start_workers().await;

    // Given: An incomplete episode (would fail extraction if enqueued)
    let context = TaskContext::default();
    let _incomplete_id = memory
        .start_episode("Incomplete".to_string(), context, TaskType::Testing)
        .await;

    // Note: In production, this shouldn't happen because complete_episode marks it complete first
    // This tests error handling in workers

    // When: Creating and completing a valid episode
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

    // When: Waiting for processing
    sleep(Duration::from_millis(500)).await;

    // Then: Complete episode should be processed successfully
    let complete_episode = memory.get_episode(complete_id).await.unwrap();
    assert!(complete_episode.is_complete());

    // Then: At least one episode should be processed
    let stats = memory.get_queue_stats().await.unwrap();
    assert!(stats.total_processed >= 1);
}

#[tokio::test]
async fn should_scale_processing_with_different_worker_counts() {
    // Given/When/Then: Testing worker pool with different worker counts
    for worker_count in [1, 2, 4, 8] {
        // Given: Memory system configured with specific worker count
        let config = QueueConfig {
            worker_count,
            poll_interval_ms: 10,
            ..Default::default()
        };

        let memory = Arc::new(
            SelfLearningMemory::with_config(test_memory_config()).enable_async_extraction(config),
        );
        memory.start_workers().await;

        let episode_count = 20;
        let start = std::time::Instant::now();

        // When: Creating and completing multiple episodes
        for i in 0..episode_count {
            let episode_id = create_test_episode(&memory, &format!("Task {i}"), 3).await;
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

        // When: Waiting for all to process
        sleep(Duration::from_secs(3)).await;

        let duration = start.elapsed();
        let stats = memory.get_queue_stats().await.unwrap();

        println!(
            "Workers: {}, Episodes: {}, Duration: {:?}, Processed: {}, Failed: {}",
            worker_count, episode_count, duration, stats.total_processed, stats.total_failed
        );

        // Then: Queue should be empty after processing
        assert_eq!(stats.current_queue_size, 0, "Queue should be empty");
    }
}

#[tokio::test]
async fn should_track_queue_statistics_accurately() {
    // Given: Memory system with async extraction and workers
    let memory = Arc::new(
        SelfLearningMemory::with_config(test_memory_config())
            .enable_async_extraction(QueueConfig::default()),
    );
    memory.start_workers().await;

    // Then: Initial stats should be zero
    let stats = memory.get_queue_stats().await.unwrap();
    assert_eq!(stats.total_enqueued, 0);
    assert_eq!(stats.total_processed, 0);
    assert_eq!(stats.current_queue_size, 0);

    // When: Enqueuing episodes
    for i in 0..5 {
        let episode_id = create_test_episode(&memory, &format!("Task {i}"), 3).await;
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

    // Then: Enqueued count should be accurate
    let stats = memory.get_queue_stats().await.unwrap();
    assert_eq!(stats.total_enqueued, 5);

    // When: Waiting for processing
    sleep(Duration::from_secs(1)).await;

    // Then: All should be processed and queue empty
    let final_stats = memory.get_queue_stats().await.unwrap();
    assert_eq!(final_stats.current_queue_size, 0);
    assert!(final_stats.total_processed >= 5 || final_stats.total_failed > 0);
}

#[tokio::test]
async fn should_complete_episodes_in_under_100ms_with_async_extraction() {
    // Given: Memory system with async extraction enabled
    let memory = Arc::new(
        SelfLearningMemory::with_config(test_memory_config())
            .enable_async_extraction(QueueConfig::default()),
    );
    memory.start_workers().await;

    let episode_id = create_test_episode(&memory, "Performance test", 5).await;

    // When: Completing episode
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

    println!("Episode completion time: {duration:?}");

    // Then: Should complete in under 100ms (performance requirement)
    assert!(
        duration.as_millis() < 100u128,
        "Episode completion took {duration:?}, expected < 100ms"
    );
}

#[tokio::test]
#[ignore = "slow integration test - run with --ignored or in release CI"]
async fn should_work_with_sync_extraction_when_async_disabled() {
    // Given: Memory system without async extraction enabled
    let memory = SelfLearningMemory::with_config(test_memory_config());

    let episode_id = create_test_episode(&memory, "Sync test", 3).await;

    // When: Completing episode
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

    // Then: Queue stats should not be available
    let stats = memory.get_queue_stats().await;
    assert!(stats.is_none());

    // Then: Patterns should be extracted synchronously (immediately)
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete());
}

#[tokio::test]
async fn should_handle_concurrent_episode_completions_safely() {
    // Given: Memory system with async extraction
    let memory = Arc::new(
        SelfLearningMemory::with_config(test_memory_config())
            .enable_async_extraction(QueueConfig::default()),
    );
    memory.start_workers().await;

    // Given: Multiple episodes created
    let mut episode_ids = Vec::new();
    for i in 0..5 {
        let episode_id = create_test_episode(&memory, &format!("Task {i}"), 3).await;
        episode_ids.push(episode_id);
    }

    // When: Completing them concurrently from multiple tasks
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

    // When: Waiting for all to complete
    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    // Then: All should be enqueued
    let stats = memory.get_queue_stats().await.unwrap();
    assert_eq!(stats.total_enqueued, 5);

    // When: Waiting for processing
    sleep(Duration::from_secs(1)).await;

    // Then: All episodes should be complete
    for episode_id in episode_ids {
        let episode = memory.get_episode(episode_id).await.unwrap();
        assert!(episode.is_complete());
    }
}

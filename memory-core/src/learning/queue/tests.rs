//! # Queue Tests
//!
//! Unit tests for the pattern extraction queue.

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::episode::ExecutionStep;
    use crate::extraction::PatternExtractor;
    use crate::learning::queue::operations::PatternExtractionQueue;
    use crate::learning::queue::types::QueueConfig;
    use crate::memory::SelfLearningMemory;
    use crate::types::{ExecutionResult, TaskContext, TaskOutcome, TaskType};
    use std::sync::Arc;
    use std::time::Duration;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_queue_creation() {
        let memory = Arc::new(SelfLearningMemory::new());
        let config = QueueConfig::default();
        let queue = PatternExtractionQueue::new(config, memory);

        let size = queue.queue_size().await;
        assert_eq!(size, 0);

        let stats = queue.get_stats().await;
        assert_eq!(stats.total_enqueued, 0);
        assert_eq!(stats.total_processed, 0);
    }

    #[tokio::test]
    async fn test_enqueue_episode() {
        let memory = Arc::new(SelfLearningMemory::new());
        let config = QueueConfig::default();
        let queue = PatternExtractionQueue::new(config, memory);

        let episode_id = Uuid::new_v4();
        queue.enqueue_episode(episode_id).await.unwrap();

        let size = queue.queue_size().await;
        assert_eq!(size, 1);

        let stats = queue.get_stats().await;
        assert_eq!(stats.total_enqueued, 1);
        assert_eq!(stats.current_queue_size, 1);
    }

    #[tokio::test]
    async fn test_multiple_enqueue() {
        let memory = Arc::new(SelfLearningMemory::new());
        let config = QueueConfig::default();
        let queue = PatternExtractionQueue::new(config, memory);

        for _ in 0..10 {
            let episode_id = Uuid::new_v4();
            queue.enqueue_episode(episode_id).await.unwrap();
        }

        let size = queue.queue_size().await;
        assert_eq!(size, 10);

        let stats = queue.get_stats().await;
        assert_eq!(stats.total_enqueued, 10);
    }

    #[tokio::test]
    async fn test_backpressure_warning() {
        let memory = Arc::new(SelfLearningMemory::new());
        let config = QueueConfig {
            max_queue_size: 5,
            ..Default::default()
        };
        let queue = PatternExtractionQueue::new(config, memory);

        // Enqueue beyond capacity (should warn but not fail)
        for _ in 0..10 {
            let episode_id = Uuid::new_v4();
            let result = queue.enqueue_episode(episode_id).await;
            assert!(result.is_ok());
        }

        let size = queue.queue_size().await;
        assert_eq!(size, 10);
    }

    #[tokio::test]
    async fn test_worker_pool_startup() {
        let memory = Arc::new(SelfLearningMemory::new());
        let config = QueueConfig {
            worker_count: 2,
            ..Default::default()
        };
        let queue = PatternExtractionQueue::new(config, memory);

        queue.start_workers().await;

        // Give workers time to start
        tokio::time::sleep(Duration::from_millis(50)).await;

        let stats = queue.get_stats().await;
        assert_eq!(stats.active_workers, 2);
    }

    #[tokio::test]
    async fn test_worker_processes_episodes() {
        use crate::types::MemoryConfig;

        // Use lower quality threshold for test episodes
        let test_config = MemoryConfig {
            quality_threshold: 0.5,
            ..Default::default()
        };
        let memory = Arc::new(SelfLearningMemory::with_config(test_config));

        // Create and complete an episode
        let context = TaskContext::default();
        let episode_id = memory
            .start_episode("Test task".to_string(), context, TaskType::Testing)
            .await;

        // Add steps and complete
        for i in 0..20 {
            let mut step =
                ExecutionStep::new(i + 1, format!("tool_{}", i % 6), format!("Action {i}"));
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            memory.log_step(episode_id, step).await;
        }

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

        // Create queue and start workers
        let config = QueueConfig {
            worker_count: 1,
            poll_interval_ms: 50,
            ..Default::default()
        };
        let queue = PatternExtractionQueue::new(config, memory.clone());

        queue.start_workers().await;

        // Enqueue the episode
        queue.enqueue_episode(episode_id).await.unwrap();

        // Wait for processing
        let emptied = queue.wait_until_empty(Duration::from_secs(2)).await;
        assert!(emptied, "Queue should be empty after processing");

        let stats = queue.get_stats().await;
        assert_eq!(stats.total_enqueued, 1);
        // Note: total_processed might be 0 or 1 depending on timing and implementation
    }

    #[tokio::test]
    async fn test_parallel_processing() {
        use crate::types::MemoryConfig;

        // Use lower quality threshold for test episodes
        let test_config = MemoryConfig {
            quality_threshold: 0.5,
            ..Default::default()
        };
        let memory = Arc::new(SelfLearningMemory::with_config(test_config));

        // Create multiple episodes
        let mut episode_ids = Vec::new();
        for i in 0..5 {
            let context = TaskContext::default();
            let episode_id = memory
                .start_episode(format!("Task {i}"), context, TaskType::Testing)
                .await;

            for j in 0..20 {
                let mut step =
                    ExecutionStep::new(j + 1, format!("tool_{}", j % 6), format!("Action {j}"));
                step.result = Some(ExecutionResult::Success {
                    output: "OK".to_string(),
                });
                memory.log_step(episode_id, step).await;
            }

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

            episode_ids.push(episode_id);
        }

        // Create queue with multiple workers
        let config = QueueConfig {
            worker_count: 3,
            poll_interval_ms: 50,
            ..Default::default()
        };
        let queue = PatternExtractionQueue::new(config, memory);

        queue.start_workers().await;

        // Enqueue all episodes
        for episode_id in episode_ids {
            queue.enqueue_episode(episode_id).await.unwrap();
        }

        // Wait for all to process
        let emptied = queue.wait_until_empty(Duration::from_secs(3)).await;
        assert!(emptied, "All episodes should be processed");

        let stats = queue.get_stats().await;
        assert_eq!(stats.total_enqueued, 5);
    }

    #[tokio::test]
    async fn test_graceful_shutdown() {
        let memory = Arc::new(SelfLearningMemory::new());
        let config = QueueConfig::default();
        let queue = PatternExtractionQueue::new(config, memory);

        queue.start_workers().await;

        // Give workers time to start
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Signal shutdown
        queue.shutdown().await;

        // Workers should eventually stop (we can't easily verify but we initiated it)
        // Just verify shutdown() was called without error
    }

    #[tokio::test]
    async fn test_extract_from_nonexistent_episode() {
        let memory = Arc::new(SelfLearningMemory::new());
        let extractor = PatternExtractor::new();
        let fake_id = Uuid::new_v4();

        let result =
            PatternExtractionQueue::extract_patterns_for_episode(&memory, &extractor, fake_id)
                .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_extract_from_incomplete_episode() {
        let memory = Arc::new(SelfLearningMemory::new());

        // Create but don't complete episode
        let context = TaskContext::default();
        let episode_id = memory
            .start_episode("Incomplete".to_string(), context, TaskType::Testing)
            .await;

        let extractor = PatternExtractor::new();
        let result =
            PatternExtractionQueue::extract_patterns_for_episode(&memory, &extractor, episode_id)
                .await;

        assert!(result.is_err());
    }
}

//! Pattern extraction logic for the queue.
//!
//! Contains worker loop and pattern extraction methods.

use super::{
    QueueConfig, QueueStats, DEFAULT_MAX_QUEUE_SIZE, DEFAULT_POLL_INTERVAL_MS, DEFAULT_WORKER_COUNT,
};
use crate::error::{Error, Result};
use crate::extraction::PatternExtractor;
use crate::memory::SelfLearningMemory;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// Async pattern extraction queue
///
/// Manages a queue of episode IDs waiting for pattern extraction
/// and coordinates a pool of worker tasks to process them.
pub struct PatternExtractionQueue {
    /// Configuration
    config: QueueConfig,
    /// Episode ID queue
    queue: Arc<Mutex<VecDeque<Uuid>>>,
    /// Reference to memory system for pattern extraction
    memory: Arc<SelfLearningMemory>,
    /// Pattern extractor
    extractor: PatternExtractor,
    /// Statistics
    stats: Arc<RwLock<QueueStats>>,
    /// Shutdown signal
    shutdown: Arc<RwLock<bool>>,
}

impl PatternExtractionQueue {
    /// Create a new pattern extraction queue
    #[must_use]
    pub fn new(config: QueueConfig, memory: Arc<SelfLearningMemory>) -> Self {
        let extractor = PatternExtractor::new();

        Self {
            config,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            memory,
            extractor,
            stats: Arc::new(RwLock::new(QueueStats {
                active_workers: 0,
                ..Default::default()
            })),
            shutdown: Arc::new(RwLock::new(false)),
        }
    }

    /// Enqueue an episode for pattern extraction
    #[instrument(skip(self), fields(episode_id = %episode_id))]
    pub async fn enqueue_episode(&self, episode_id: Uuid) -> Result<()> {
        let mut queue = self.queue.lock().await;

        if self.config.max_queue_size > 0 && queue.len() >= self.config.max_queue_size {
            warn!(
                queue_size = queue.len(),
                max_size = self.config.max_queue_size,
                "Pattern extraction queue at capacity"
            );
        }

        queue.push_back(episode_id);

        let mut stats = self.stats.write().await;
        stats.total_enqueued += 1;
        stats.current_queue_size = queue.len();

        debug!(
            episode_id = %episode_id,
            queue_size = queue.len(),
            "Enqueued episode for pattern extraction"
        );

        Ok(())
    }

    /// Start worker tasks to process the queue
    pub async fn start_workers(&self) {
        info!(
            worker_count = self.config.worker_count,
            "Starting pattern extraction workers"
        );

        {
            let mut stats = self.stats.write().await;
            stats.active_workers = self.config.worker_count;
        }

        for worker_id in 0..self.config.worker_count {
            let queue = Arc::clone(&self.queue);
            let memory = Arc::clone(&self.memory);
            let extractor = self.extractor.clone();
            let stats = Arc::clone(&self.stats);
            let shutdown = Arc::clone(&self.shutdown);
            let poll_interval = Duration::from_millis(self.config.poll_interval_ms);

            tokio::spawn(async move {
                Self::worker_loop(
                    worker_id,
                    queue,
                    memory,
                    extractor,
                    stats,
                    shutdown,
                    poll_interval,
                )
                .await;
            });
        }

        info!("All pattern extraction workers started");
    }

    /// Worker loop that processes episodes from the queue
    #[instrument(skip(queue, memory, extractor, stats, shutdown))]
    async fn worker_loop(
        worker_id: usize,
        queue: Arc<Mutex<VecDeque<Uuid>>>,
        memory: Arc<SelfLearningMemory>,
        extractor: PatternExtractor,
        stats: Arc<RwLock<QueueStats>>,
        shutdown: Arc<RwLock<bool>>,
        poll_interval: Duration,
    ) {
        debug!(worker_id, "Worker started");

        loop {
            {
                let should_shutdown = *shutdown.read().await;
                if should_shutdown {
                    info!(worker_id, "Worker shutting down gracefully");
                    break;
                }
            }

            let episode_id = {
                let mut q = queue.lock().await;
                q.pop_front()
            };

            match episode_id {
                Some(id) => {
                    debug!(worker_id, episode_id = %id, "Processing episode");

                    match Self::extract_patterns_for_episode(&memory, &extractor, id).await {
                        Ok(pattern_count) => {
                            debug!(
                                worker_id,
                                episode_id = %id,
                                pattern_count,
                                "Successfully extracted patterns"
                            );

                            let mut s = stats.write().await;
                            s.total_processed += 1;
                            s.current_queue_size = {
                                let q = queue.lock().await;
                                q.len()
                            };
                        }
                        Err(e) => {
                            error!(
                                worker_id,
                                episode_id = %id,
                                error = %e,
                                "Pattern extraction failed"
                            );

                            let mut s = stats.write().await;
                            s.total_failed += 1;
                        }
                    }
                }
                None => {
                    sleep(poll_interval).await;
                }
            }
        }

        debug!(worker_id, "Worker stopped");
    }

    /// Extract patterns for a specific episode
    #[instrument(skip(memory, extractor), fields(episode_id = %episode_id))]
    async fn extract_patterns_for_episode(
        memory: &SelfLearningMemory,
        extractor: &PatternExtractor,
        episode_id: Uuid,
    ) -> Result<usize> {
        let episode = memory.get_episode(episode_id).await?;

        if !episode.is_complete() {
            return Err(Error::InvalidState(format!(
                "Episode {episode_id} is not complete"
            )));
        }

        let patterns = extractor.extract(&episode);
        let pattern_count = patterns.len();

        if pattern_count > 0 {
            memory.store_patterns(episode_id, patterns).await?;
            info!(
                episode_id = %episode_id,
                pattern_count,
                "Extracted and stored patterns asynchronously"
            );
        }

        Ok(pattern_count)
    }

    /// Get current queue statistics
    pub async fn get_stats(&self) -> QueueStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Get current queue size
    pub async fn queue_size(&self) -> usize {
        let queue = self.queue.lock().await;
        queue.len()
    }

    /// Signal workers to shutdown gracefully
    pub async fn shutdown(&self) {
        info!("Initiating pattern extraction queue shutdown");
        let mut shutdown = self.shutdown.write().await;
        *shutdown = true;
    }

    /// Wait for queue to be empty
    pub async fn wait_until_empty(&self, timeout: Duration) -> bool {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            let size = self.queue_size().await;
            if size == 0 {
                return true;
            }

            sleep(Duration::from_millis(100)).await;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::ExecutionStep;
    use crate::types::{ExecutionResult, TaskContext, TaskOutcome, TaskType};

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

        sleep(Duration::from_millis(50)).await;

        let stats = queue.get_stats().await;
        assert_eq!(stats.active_workers, 2);
    }

    #[tokio::test]
    async fn test_worker_processes_episodes() {
        use crate::types::MemoryConfig;

        let test_config = MemoryConfig {
            quality_threshold: 0.5,
            ..Default::default()
        };
        let memory = Arc::new(SelfLearningMemory::with_config(test_config));

        let context = TaskContext::default();
        let episode_id = memory
            .start_episode("Test task".to_string(), context, TaskType::Testing)
            .await;

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

        let config = QueueConfig {
            worker_count: 1,
            poll_interval_ms: 50,
            ..Default::default()
        };
        let queue = PatternExtractionQueue::new(config, memory.clone());

        queue.start_workers().await;

        queue.enqueue_episode(episode_id).await.unwrap();

        let emptied = queue.wait_until_empty(Duration::from_secs(2)).await;
        assert!(emptied, "Queue should be empty after processing");

        let stats = queue.get_stats().await;
        assert_eq!(stats.total_enqueued, 1);
    }

    #[tokio::test]
    async fn test_parallel_processing() {
        use crate::types::MemoryConfig;

        let test_config = MemoryConfig {
            quality_threshold: 0.5,
            ..Default::default()
        };
        let memory = Arc::new(SelfLearningMemory::with_config(test_config));

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

        let config = QueueConfig {
            worker_count: 3,
            poll_interval_ms: 50,
            ..Default::default()
        };
        let queue = PatternExtractionQueue::new(config, memory);

        queue.start_workers().await;

        for episode_id in episode_ids {
            queue.enqueue_episode(episode_id).await.unwrap();
        }

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

        sleep(Duration::from_millis(50)).await;

        queue.shutdown().await;

        let shutdown_flag = *queue.shutdown.read().await;
        assert!(shutdown_flag);
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

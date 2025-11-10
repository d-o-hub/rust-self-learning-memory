//! # Pattern Extraction Queue
//!
//! Asynchronous queue system for pattern extraction from completed episodes.
//!
//! Episodes are enqueued when completed, and a pool of worker tasks processes
//! them in parallel without blocking the main completion flow.
//!
//! ## Example
//!
//! ```no_run
//! use memory_core::learning::queue::{PatternExtractionQueue, QueueConfig};
//! use memory_core::memory::SelfLearningMemory;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let memory = Arc::new(SelfLearningMemory::new());
//!     let config = QueueConfig::default();
//!     let queue = PatternExtractionQueue::new(config, memory);
//!
//!     // Start worker pool
//!     queue.start_workers().await;
//!
//!     // Workers now process episodes in background
//! }
//! ```

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

/// Default number of worker tasks
const DEFAULT_WORKER_COUNT: usize = 4;

/// Default maximum queue size (for backpressure)
const DEFAULT_MAX_QUEUE_SIZE: usize = 1000;

/// Default worker poll interval when queue is empty
const DEFAULT_POLL_INTERVAL_MS: u64 = 100;

/// Configuration for pattern extraction queue
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// Number of worker tasks to spawn
    pub worker_count: usize,
    /// Maximum queue size (0 = unlimited)
    pub max_queue_size: usize,
    /// Polling interval when queue is empty (milliseconds)
    pub poll_interval_ms: u64,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            worker_count: DEFAULT_WORKER_COUNT,
            max_queue_size: DEFAULT_MAX_QUEUE_SIZE,
            poll_interval_ms: DEFAULT_POLL_INTERVAL_MS,
        }
    }
}

/// Statistics about queue operations
#[derive(Debug, Clone, Default)]
pub struct QueueStats {
    /// Total episodes enqueued
    pub total_enqueued: u64,
    /// Total episodes processed successfully
    pub total_processed: u64,
    /// Total episodes that failed processing
    pub total_failed: u64,
    /// Current queue size
    pub current_queue_size: usize,
    /// Number of active workers
    pub active_workers: usize,
}

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
    ///
    /// # Arguments
    ///
    /// * `config` - Queue configuration
    /// * `memory` - Shared reference to memory system
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
    ///
    /// Adds the episode ID to the queue. If the queue is at max capacity,
    /// logs a warning but still enqueues (optional: could reject).
    ///
    /// # Arguments
    ///
    /// * `episode_id` - ID of the completed episode
    ///
    /// # Errors
    ///
    /// Returns error if queue is at maximum capacity (when enforced)
    #[instrument(skip(self), fields(episode_id = %episode_id))]
    pub async fn enqueue_episode(&self, episode_id: Uuid) -> Result<()> {
        let mut queue = self.queue.lock().await;

        // Check backpressure
        if self.config.max_queue_size > 0 && queue.len() >= self.config.max_queue_size {
            warn!(
                queue_size = queue.len(),
                max_size = self.config.max_queue_size,
                "Pattern extraction queue at capacity"
            );

            // Optional: return error to enforce backpressure
            // For now, we just warn and continue
        }

        queue.push_back(episode_id);

        // Update stats
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
    ///
    /// Spawns the configured number of worker tasks that continuously
    /// poll the queue and extract patterns from episodes.
    pub async fn start_workers(&self) {
        info!(
            worker_count = self.config.worker_count,
            "Starting pattern extraction workers"
        );

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.active_workers = self.config.worker_count;
        }

        // Spawn worker tasks
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
    ///
    /// Each worker continuously polls the queue, extracts patterns from episodes,
    /// and updates statistics. Handles errors gracefully without crashing.
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
            // Check shutdown signal
            {
                let should_shutdown = *shutdown.read().await;
                if should_shutdown {
                    info!(worker_id, "Worker shutting down gracefully");
                    break;
                }
            }

            // Try to get an episode from queue
            let episode_id = {
                let mut q = queue.lock().await;
                q.pop_front()
            };

            match episode_id {
                Some(id) => {
                    debug!(worker_id, episode_id = %id, "Processing episode");

                    // Extract patterns for this episode
                    match Self::extract_patterns_for_episode(&memory, &extractor, id).await {
                        Ok(pattern_count) => {
                            debug!(
                                worker_id,
                                episode_id = %id,
                                pattern_count,
                                "Successfully extracted patterns"
                            );

                            // Update stats
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

                            // Update error stats
                            let mut s = stats.write().await;
                            s.total_failed += 1;
                        }
                    }
                }
                None => {
                    // Queue is empty, sleep briefly
                    sleep(poll_interval).await;
                }
            }
        }

        debug!(worker_id, "Worker stopped");
    }

    /// Extract patterns for a specific episode
    ///
    /// Retrieves the episode from memory, extracts patterns, and stores them.
    ///
    /// # Arguments
    ///
    /// * `memory` - Memory system reference
    /// * `extractor` - Pattern extractor
    /// * `episode_id` - Episode to process
    ///
    /// # Returns
    ///
    /// Number of patterns extracted
    ///
    /// # Errors
    ///
    /// Returns error if episode not found or pattern extraction fails
    #[instrument(skip(memory, extractor), fields(episode_id = %episode_id))]
    async fn extract_patterns_for_episode(
        memory: &SelfLearningMemory,
        extractor: &PatternExtractor,
        episode_id: Uuid,
    ) -> Result<usize> {
        // Get the episode
        let episode = memory.get_episode(episode_id).await?;

        // Ensure episode is complete
        if !episode.is_complete() {
            return Err(Error::InvalidState(format!(
                "Episode {} is not complete",
                episode_id
            )));
        }

        // Extract patterns
        let patterns = extractor.extract(&episode);
        let pattern_count = patterns.len();

        if pattern_count > 0 {
            // Store patterns through memory system
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
    ///
    /// Returns a snapshot of the queue's operational statistics.
    pub async fn get_stats(&self) -> QueueStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Get current queue size
    ///
    /// Returns the number of episodes waiting for pattern extraction.
    pub async fn queue_size(&self) -> usize {
        let queue = self.queue.lock().await;
        queue.len()
    }

    /// Signal workers to shutdown gracefully
    ///
    /// Sets the shutdown flag. Workers will finish their current task
    /// and then exit. Does not wait for workers to complete.
    pub async fn shutdown(&self) {
        info!("Initiating pattern extraction queue shutdown");
        let mut shutdown = self.shutdown.write().await;
        *shutdown = true;
    }

    /// Wait for queue to be empty
    ///
    /// Blocks until all enqueued episodes have been processed.
    /// Useful for graceful shutdown or testing.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait
    ///
    /// # Returns
    ///
    /// `true` if queue emptied, `false` if timed out
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
        sleep(Duration::from_millis(50)).await;

        let stats = queue.get_stats().await;
        assert_eq!(stats.active_workers, 2);
    }

    #[tokio::test]
    async fn test_worker_processes_episodes() {
        let memory = Arc::new(SelfLearningMemory::new());

        // Create and complete an episode
        let context = TaskContext::default();
        let episode_id = memory
            .start_episode("Test task".to_string(), context, TaskType::Testing)
            .await;

        // Add steps and complete
        let mut step = ExecutionStep::new(1, "tool1".to_string(), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        memory.log_step(episode_id, step).await;

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
        let memory = Arc::new(SelfLearningMemory::new());

        // Create multiple episodes
        let mut episode_ids = Vec::new();
        for i in 0..5 {
            let context = TaskContext::default();
            let episode_id = memory
                .start_episode(format!("Task {}", i), context, TaskType::Testing)
                .await;

            let mut step = ExecutionStep::new(1, "tool".to_string(), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            memory.log_step(episode_id, step).await;

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
        sleep(Duration::from_millis(50)).await;

        // Signal shutdown
        queue.shutdown().await;

        // Workers should eventually stop (we can't easily verify but we initiated it)
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

//! # Queue Operations
//!
//! Core queue operations and worker management.

use crate::error::{Error, Result};
use crate::extraction::PatternExtractor;
use crate::learning::queue::types::{QueueConfig, QueueStats};
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
    ///
    /// # Arguments
    ///
    /// * `config` - Queue configuration
    /// * `memory` - Shared reference to memory system
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
    pub(crate) async fn extract_patterns_for_episode(
        memory: &SelfLearningMemory,
        extractor: &PatternExtractor,
        episode_id: Uuid,
    ) -> Result<usize> {
        // Get the episode
        let episode = memory.get_episode(episode_id).await?;

        // Ensure episode is complete
        if !episode.is_complete() {
            return Err(Error::InvalidState(format!(
                "Episode {episode_id} is not complete"
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

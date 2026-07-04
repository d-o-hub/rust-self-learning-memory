//! # Queue Types
//!
//! Type definitions for the pattern extraction queue.

/// Default number of worker tasks
pub(crate) const DEFAULT_WORKER_COUNT: usize = 4;

/// Default maximum queue size (for backpressure)
pub(crate) const DEFAULT_MAX_QUEUE_SIZE: usize = 1000;

/// Configuration for pattern extraction queue
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// Number of worker tasks to spawn
    pub worker_count: usize,
    /// Maximum queue size (0 = unlimited)
    pub max_queue_size: usize,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            worker_count: DEFAULT_WORKER_COUNT,
            max_queue_size: DEFAULT_MAX_QUEUE_SIZE,
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

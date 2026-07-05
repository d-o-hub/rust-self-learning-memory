//! # Queue Configuration
//!
//! Configuration constants and settings for the pattern extraction queue.

/// Default number of worker tasks
pub const DEFAULT_WORKER_COUNT: usize = 4;

/// Default maximum queue size (for backpressure)
pub const DEFAULT_MAX_QUEUE_SIZE: usize = 1000;

/// Default worker poll interval when queue is empty
pub const DEFAULT_POLL_INTERVAL_MS: u64 = 100;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = QueueConfig::default();
        assert_eq!(config.worker_count, DEFAULT_WORKER_COUNT);
        assert_eq!(config.max_queue_size, DEFAULT_MAX_QUEUE_SIZE);
        assert_eq!(config.poll_interval_ms, DEFAULT_POLL_INTERVAL_MS);
    }

    #[test]
    fn test_custom_config() {
        let config = QueueConfig {
            worker_count: 8,
            max_queue_size: 500,
            poll_interval_ms: 50,
        };
        assert_eq!(config.worker_count, 8);
        assert_eq!(config.max_queue_size, 500);
        assert_eq!(config.poll_interval_ms, 50);
    }
}

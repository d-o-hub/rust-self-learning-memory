//! # Queue Statistics
//!
//! Statistics tracking for pattern extraction queue operations.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_stats() {
        let stats = QueueStats::default();
        assert_eq!(stats.total_enqueued, 0);
        assert_eq!(stats.total_processed, 0);
        assert_eq!(stats.total_failed, 0);
        assert_eq!(stats.current_queue_size, 0);
        assert_eq!(stats.active_workers, 0);
    }

    #[test]
    fn test_stats_clone() {
        let stats = QueueStats {
            total_enqueued: 10,
            total_processed: 8,
            total_failed: 2,
            current_queue_size: 5,
            active_workers: 4,
        };
        let mut stats = stats;

        let cloned = stats.clone();
        assert_eq!(cloned.total_enqueued, 10);
        assert_eq!(cloned.total_processed, 8);
        assert_eq!(cloned.total_failed, 2);
        assert_eq!(cloned.current_queue_size, 5);
        assert_eq!(cloned.active_workers, 4);
    }
}

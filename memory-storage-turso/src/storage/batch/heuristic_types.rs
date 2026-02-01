//! # Batch Heuristic Operations - Types
//!
//! Types for batch heuristic operations.

/// Progress tracking for batch operations
#[derive(Debug, Clone)]
pub struct HeuristicBatchProgress {
    /// Total items to process
    pub total: usize,
    /// Items processed so far
    pub processed: usize,
    /// Items successfully stored
    pub succeeded: usize,
    /// Items that failed
    pub failed: usize,
    /// Current batch number
    pub current_batch: usize,
    /// Total batches
    pub total_batches: usize,
}

impl HeuristicBatchProgress {
    /// Create new progress tracker
    #[must_use]
    pub fn new(total: usize, batch_size: usize) -> Self {
        let total_batches = (total + batch_size - 1) / batch_size.max(1);
        Self {
            total,
            processed: 0,
            succeeded: 0,
            failed: 0,
            current_batch: 0,
            total_batches,
        }
    }

    /// Update progress after processing items
    pub fn update(&mut self, processed: usize, succeeded: usize, failed: usize) {
        self.processed += processed;
        self.succeeded += succeeded;
        self.failed += failed;
        self.current_batch += 1;
    }

    /// Get completion percentage
    #[must_use]
    pub fn percent_complete(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            (self.processed as f64 / self.total as f64) * 100.0
        }
    }

    /// Check if batch is complete
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.processed >= self.total
    }
}

/// Result of a batch operation
#[derive(Debug, Clone)]
pub struct HeuristicBatchResult {
    /// Total items processed
    pub total_processed: usize,
    /// Items successfully stored
    pub succeeded: usize,
    /// Items that failed
    pub failed: usize,
    /// Whether the entire batch succeeded
    pub all_succeeded: bool,
    /// Error messages for failed items (if any)
    pub errors: Vec<String>,
}

impl HeuristicBatchResult {
    /// Create a successful batch result
    #[must_use]
    pub fn success(count: usize) -> Self {
        Self {
            total_processed: count,
            succeeded: count,
            failed: 0,
            all_succeeded: true,
            errors: Vec::new(),
        }
    }

    /// Create a failed batch result
    #[must_use]
    pub fn failure(error: String) -> Self {
        Self {
            total_processed: 0,
            succeeded: 0,
            failed: 0,
            all_succeeded: false,
            errors: vec![error],
        }
    }
}

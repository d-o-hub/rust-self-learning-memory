//! # Batch Operations Module
//!
//! Optimized batch operations for episodes and patterns using transactions
//! and prepared statements for 4-6x throughput improvement.
//!
//! ## Modules
//!
//! - `episode_batch` - Episode batch operations
//! - `pattern_batch` - Pattern batch operations
//! - `combined_batch` - Combined episode + pattern batch
//! - `query_batch` - Batch query operations

pub mod combined_batch;
pub mod episode_batch;
pub mod pattern_batch;
pub mod query_batch;

/// Configuration for batch operations
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum number of items per batch (default: 100)
    pub batch_size: usize,
    /// Maximum retry attempts for failed operations
    pub max_retries: u32,
    /// Base delay for exponential backoff (milliseconds)
    pub retry_base_delay_ms: u64,
    /// Maximum delay for exponential backoff (milliseconds)
    pub retry_max_delay_ms: u64,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            max_retries: 3,
            retry_base_delay_ms: 100,
            retry_max_delay_ms: 5000,
        }
    }
}

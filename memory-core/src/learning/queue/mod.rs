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
//!

pub mod operations;
#[cfg(test)]
pub mod tests;
pub mod types;

pub use operations::PatternExtractionQueue;
pub use types::QueueConfig;
pub use types::QueueStats;

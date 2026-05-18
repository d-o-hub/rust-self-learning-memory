//! # Learning Module
//!
//! Asynchronous learning components for pattern extraction and analysis.
//!
//! This module provides non-blocking pattern extraction through a queue-based
//! worker pool system, allowing episode completion to return quickly while
//! pattern extraction happens in the background.

pub mod queue;

mod config;
mod stats;

pub use config::QueueConfig;
pub use queue::PatternExtractionQueue;
pub use stats::QueueStats;

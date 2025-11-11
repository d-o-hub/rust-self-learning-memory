//! # Pattern Extractor
//!
//! Extracts reusable patterns from completed episodes:
//! - Tool sequences that worked well
//! - Decision points with outcomes
//! - Error recovery strategies
//! - Context-based patterns
//!
//! ## Example
//!
//! ```
//! use memory_core::extraction::PatternExtractor;
//! use memory_core::{Episode, TaskContext, TaskType, TaskOutcome, ExecutionStep};
//!
//! let context = TaskContext::default();
//! let mut episode = Episode::new("Test task".to_string(), context, TaskType::Testing);
//!
//! // Add some execution steps
//! for i in 0..3 {
//!     let step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
//!     episode.add_step(step);
//! }
//!
//! episode.complete(TaskOutcome::Success {
//!     verdict: "Done".to_string(),
//!     artifacts: vec![],
//! });
//!
//! let extractor = PatternExtractor::new();
//! let patterns = extractor.extract(&episode);
//!
//! // Patterns may be extracted based on episode content
//! ```

mod extractor;
mod extractors;
mod utils;

#[cfg(test)]
mod tests;

// Re-export main types
pub use extractor::PatternExtractor;
pub use utils::{deduplicate_patterns, rank_patterns};

/// Minimum success rate to extract a pattern
pub const MIN_PATTERN_SUCCESS_RATE: f32 = 0.7;

/// Minimum sequence length for tool sequence patterns
pub const MIN_SEQUENCE_LENGTH: usize = 2;

/// Maximum sequence length for tool sequence patterns
pub const MAX_SEQUENCE_LENGTH: usize = 5;

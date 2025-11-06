//! # Memory Core
//!
//! Core data structures and types for the self-learning memory system.
//!
//! This crate provides the fundamental building blocks for episodic learning:
//! - Episodes: Complete task execution records
//! - Patterns: Reusable patterns extracted from episodes
//! - Heuristics: Learned condition-action rules
//! - Types: Common types and enums
//!
//! ## Example
//!
//! ```
//! use memory_core::{Episode, TaskContext, TaskType};
//!
//! let context = TaskContext::default();
//! let episode = Episode::new(
//!     "Implement authentication".to_string(),
//!     context,
//!     TaskType::CodeGeneration,
//! );
//!
//! assert!(!episode.is_complete());
//! ```

pub mod episode;
pub mod error;
pub mod pattern;
pub mod types;

// Re-export commonly used types
pub use episode::{Episode, ExecutionStep, PatternId};
pub use error::{Error, Result};
pub use pattern::{Heuristic, Pattern};
pub use types::{
    ComplexityLevel, Evidence, ExecutionResult, MemoryConfig, OutcomeStats, Reflection,
    RewardScore, StorageConfig, TaskContext, TaskOutcome, TaskType,
};

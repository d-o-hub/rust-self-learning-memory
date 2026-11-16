//! # Learning Coordination
//!
//! Coordinates the post-episode learning cycle:
//! - [`LearningOrchestrator`] owns reward/reflection analysis and dispatches pattern
//!   extraction to the async queue or the caller's synchronous path.
//! - [`queue`] manages the async pattern extraction pipeline.
//!
//! The underlying stages remain in their focused modules:
//! - [`crate::reward`] — reward signal computation
//! - [`crate::reflection`] — memory consolidation and reflection
//! - [`crate::episode`] — episodic memory storage
//!
//! This module provides non-blocking pattern extraction through a queue-based
//! worker pool system, allowing episode completion to return quickly while
//! pattern extraction happens in the background.

pub mod queue;

mod config;
mod orchestrator;
mod stats;

pub use config::QueueConfig;
pub use orchestrator::{LearningAnalysis, LearningOrchestrator, PatternDispatch};
pub use queue::PatternExtractionQueue;
pub use stats::QueueStats;

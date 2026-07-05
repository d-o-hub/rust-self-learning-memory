//! # Learning Coordination
//!
//! This module is intentionally scoped to **queue coordination only**.
//! It manages the async pattern extraction pipeline that fires after
//! episode completion.
//!
//! The broader learning cycle (reward signals, reflection, consolidation)
//! is handled by the following sibling modules:
//! - [`crate::reward`] — reward signal computation
//! - [`crate::reflection`] — memory consolidation and reflection
//! - [`crate::episode`] — episodic memory storage
//!
//! A future `LearningOrchestrator` may be introduced here to coordinate
//! the full cycle. See: `plans/adr/ADR-028-Feature-Enhancement-Roadmap.md`
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

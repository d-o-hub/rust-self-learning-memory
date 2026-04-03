//! Episode Retention Policy and GC Configuration
//!
//! This module provides configuration for episode lifecycle management,
//! including TTL-based cleanup, retention policies, and garbage collection.
//!
//! ## Design Philosophy
//!
//! Episodes accumulate over time and can consume significant storage.
//! This module provides policies to automatically clean up:
//! - Old, unused episodes that haven't been referenced
//! - Failed episodes with low reward scores
//! - Episodes exceeding storage quotas
//!
//! ## Usage
//!
//! ```ignore
//! use do_memory_core::episode::retention::{EpisodeRetentionPolicy, RetentionTrigger};
//!
//! let policy = EpisodeRetentionPolicy::default()
//!     .with_max_age_days(90)
//!     .with_min_reward_threshold(0.3)
//!     .with_max_episodes(10_000);
//!
//! // Trigger cleanup when storage exceeds threshold
//! let trigger = RetentionTrigger::StorageExceeded { current: 15000, max: 10000 };
//! ```

mod policy;
#[cfg(test)]
mod tests;
mod types;

use std::time::Duration as StdDuration;

pub use self::policy::EpisodeRetentionPolicy;
pub use self::types::{CleanupResult, RetentionCriterion, RetentionPolicyError, RetentionTrigger};

/// Default maximum age for episodes (90 days)
pub const DEFAULT_MAX_AGE_DAYS: i64 = 90;

/// Default minimum reward threshold for keeping episodes
pub const DEFAULT_MIN_REWARD_THRESHOLD: f64 = 0.3;

/// Default maximum number of episodes
pub const DEFAULT_MAX_EPISODES: usize = 50_000;

/// Default cleanup interval (24 hours)
pub const DEFAULT_CLEANUP_INTERVAL: StdDuration = StdDuration::from_secs(24 * 60 * 60);

/// Default batch size for cleanup operations
pub const DEFAULT_CLEANUP_BATCH_SIZE: usize = 100;

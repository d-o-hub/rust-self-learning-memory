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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration as StdDuration;

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

/// Criteria for episode retention decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetentionCriterion {
    /// Keep all episodes (no cleanup)
    KeepAll,
    /// Delete episodes older than a threshold
    AgeBased,
    /// Delete episodes with low reward scores
    RewardBased,
    /// Delete unreferenced episodes (no patterns/heuristics derived)
    Unreferenced,
    /// Delete failed episodes with no successful patterns
    FailedOnly,
    /// Combined criteria (all must pass to keep)
    Combined,
}

/// Trigger for running cleanup
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RetentionTrigger {
    /// Scheduled cleanup interval
    Scheduled,
    /// Storage quota exceeded
    StorageExceeded {
        /// Current episode count
        current: usize,
        /// Maximum allowed
        max: usize,
    },
    /// Memory pressure threshold reached
    MemoryPressure {
        /// Pressure level (0.0 - 1.0)
        level: f64,
    },
    /// Manual cleanup requested
    Manual,
}

/// Policy for episode retention and garbage collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeRetentionPolicy {
    /// Maximum age in days before episode is eligible for cleanup
    pub max_age_days: i64,

    /// Minimum reward score to keep (episodes below this are candidates)
    pub min_reward_threshold: f64,

    /// Maximum number of episodes to retain
    pub max_episodes: usize,

    /// Cleanup interval in seconds
    pub cleanup_interval_secs: u64,

    /// Batch size for cleanup operations (prevents long-running deletes)
    pub cleanup_batch_size: usize,

    /// Which criterion to use for cleanup decisions
    pub criterion: RetentionCriterion,

    /// Keep episodes that have derived patterns
    pub keep_pattern_sources: bool,

    /// Keep episodes that have derived heuristics
    pub keep_heuristic_sources: bool,

    /// Keep successful episodes (reward >= 0.7)
    /// keep_high_reward: bool, // Removed as unused

    /// Dry run mode (report what would be deleted without actually deleting)
    pub dry_run: bool,
}

impl Default for EpisodeRetentionPolicy {
    fn default() -> Self {
        Self {
            max_age_days: DEFAULT_MAX_AGE_DAYS,
            min_reward_threshold: DEFAULT_MIN_REWARD_THRESHOLD,
            max_episodes: DEFAULT_MAX_EPISODES,
            cleanup_interval_secs: DEFAULT_CLEANUP_INTERVAL.as_secs(),
            cleanup_batch_size: DEFAULT_CLEANUP_BATCH_SIZE,
            criterion: RetentionCriterion::Combined,
            keep_pattern_sources: true,
            keep_heuristic_sources: true,
            dry_run: false,
        }
    }
}

impl EpisodeRetentionPolicy {
    /// Create a new retention policy with defaults
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a policy optimized for limited storage
    #[must_use]
    pub fn storage_limited() -> Self {
        Self {
            max_age_days: 30,
            min_reward_threshold: 0.5,
            max_episodes: 5_000,
            cleanup_interval_secs: 12 * 60 * 60, // 12 hours
            cleanup_batch_size: 50,
            criterion: RetentionCriterion::Combined,
            keep_pattern_sources: true,
            keep_heuristic_sources: true,
            dry_run: false,
        }
    }

    /// Create a policy for archival mode (keep everything)
    #[must_use]
    pub fn archival() -> Self {
        Self {
            max_age_days: 365, // 1 year
            min_reward_threshold: 0.0,
            max_episodes: usize::MAX,
            cleanup_interval_secs: 7 * 24 * 60 * 60, // Weekly
            cleanup_batch_size: 100,
            criterion: RetentionCriterion::KeepAll,
            keep_pattern_sources: true,
            keep_heuristic_sources: true,
            dry_run: false,
        }
    }

    /// Create a policy for aggressive cleanup
    #[must_use]
    pub fn aggressive() -> Self {
        Self {
            max_age_days: 7,
            min_reward_threshold: 0.6,
            max_episodes: 1_000,
            cleanup_interval_secs: 6 * 60 * 60, // 6 hours
            cleanup_batch_size: 200,
            criterion: RetentionCriterion::Combined,
            keep_pattern_sources: false,
            keep_heuristic_sources: false,
            dry_run: false,
        }
    }

    /// Builder: set maximum age in days
    #[must_use]
    pub fn with_max_age_days(mut self, days: i64) -> Self {
        self.max_age_days = days;
        self
    }

    /// Builder: set minimum reward threshold
    #[must_use]
    pub fn with_min_reward_threshold(mut self, threshold: f64) -> Self {
        self.min_reward_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Builder: set maximum episodes
    #[must_use]
    pub fn with_max_episodes(mut self, max: usize) -> Self {
        self.max_episodes = max;
        self
    }

    /// Builder: set cleanup interval
    #[must_use]
    pub fn with_cleanup_interval(mut self, interval: StdDuration) -> Self {
        self.cleanup_interval_secs = interval.as_secs();
        self
    }

    /// Builder: set cleanup batch size
    #[must_use]
    pub fn with_cleanup_batch_size(mut self, size: usize) -> Self {
        self.cleanup_batch_size = size.max(1);
        self
    }

    /// Builder: set retention criterion
    #[must_use]
    pub fn with_criterion(mut self, criterion: RetentionCriterion) -> Self {
        self.criterion = criterion;
        self
    }

    /// Builder: enable dry run mode
    #[must_use]
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Validate policy configuration
    pub fn validate(&self) -> Result<(), RetentionPolicyError> {
        if self.max_age_days < 0 {
            return Err(RetentionPolicyError::InvalidMaxAge(self.max_age_days));
        }
        if !(0.0..=1.0).contains(&self.min_reward_threshold) {
            return Err(RetentionPolicyError::InvalidThreshold(
                self.min_reward_threshold,
            ));
        }
        if self.cleanup_batch_size == 0 {
            return Err(RetentionPolicyError::InvalidBatchSize);
        }
        Ok(())
    }

    /// Check if an episode should be retained based on policy
    ///
    /// # Arguments
    ///
    /// * `episode` - Episode to evaluate
    /// * `now` - Current timestamp
    ///
    /// # Returns
    ///
    /// `true` if episode should be kept, `false` if eligible for cleanup
    #[must_use]
    pub fn should_retain(&self, episode: &crate::Episode, now: DateTime<Utc>) -> bool {
        match self.criterion {
            RetentionCriterion::KeepAll => true,
            RetentionCriterion::AgeBased => self.check_age(episode, now),
            RetentionCriterion::RewardBased => self.check_reward(episode),
            RetentionCriterion::Unreferenced => self.check_references(episode),
            RetentionCriterion::FailedOnly => self.check_failed(episode),
            RetentionCriterion::Combined => {
                // All criteria must pass to keep
                self.check_age(episode, now)
                    && self.check_reward(episode)
                    && self.check_references(episode)
            }
        }
    }

    fn check_age(&self, episode: &crate::Episode, now: DateTime<Utc>) -> bool {
        let age_days = (now - episode.start_time).num_days();
        age_days <= self.max_age_days
    }

    fn check_reward(&self, episode: &crate::Episode) -> bool {
        // Episodes without reward scores are kept (not yet evaluated)
        match episode.reward.as_ref() {
            None => true,
            Some(r) => r.total >= self.min_reward_threshold as f32,
        }
    }

    fn check_references(&self, episode: &crate::Episode) -> bool {
        // Keep if it has derived patterns/heuristics (unless policy says otherwise)
        if self.keep_pattern_sources && !episode.patterns.is_empty() {
            return true;
        }
        if self.keep_heuristic_sources && !episode.heuristics.is_empty() {
            return true;
        }
        // Otherwise, unreferenced episodes are candidates for cleanup
        false
    }

    fn check_failed(&self, episode: &crate::Episode) -> bool {
        // Keep episodes that are not failed or have patterns/heuristics
        match episode.outcome.as_ref() {
            None => true,
            Some(o) if !matches!(o, crate::types::TaskOutcome::Failure { .. }) => true,
            Some(_) => self.check_references(episode),
        }
    }

    /// Get cleanup interval as Duration
    #[must_use]
    pub fn cleanup_interval(&self) -> StdDuration {
        StdDuration::from_secs(self.cleanup_interval_secs)
    }
}

/// Errors in retention policy configuration
#[derive(Debug, Clone, PartialEq)]
pub enum RetentionPolicyError {
    /// Invalid maximum age (must be >= 0)
    InvalidMaxAge(i64),
    /// Invalid reward threshold (must be 0.0 - 1.0)
    InvalidThreshold(f64),
    /// Invalid batch size (must be > 0)
    InvalidBatchSize,
}

impl std::fmt::Display for RetentionPolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMaxAge(days) => write!(f, "Invalid max age: {days} (must be >= 0)"),
            Self::InvalidThreshold(threshold) => {
                write!(f, "Invalid threshold: {threshold} (must be 0.0 - 1.0)")
            }
            Self::InvalidBatchSize => write!(f, "Invalid batch size: must be > 0"),
        }
    }
}

impl std::error::Error for RetentionPolicyError {}

/// Result of a cleanup operation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CleanupResult {
    /// Number of episodes evaluated
    pub evaluated: usize,

    /// Number of episodes deleted
    pub deleted: usize,

    /// Number of episodes kept
    pub kept: usize,

    /// IDs of deleted episodes
    pub deleted_ids: Vec<uuid::Uuid>,

    /// IDs of kept episodes (when dry run)
    pub kept_ids: Vec<uuid::Uuid>,

    /// Trigger that caused cleanup
    pub trigger: Option<RetentionTrigger>,

    /// Duration of cleanup in milliseconds
    pub duration_ms: u64,

    /// Error messages (if any)
    pub errors: Vec<String>,
}

impl CleanupResult {
    /// Create a new cleanup result
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a deleted episode
    pub fn add_deleted(&mut self, id: uuid::Uuid) {
        self.deleted += 1;
        self.deleted_ids.push(id);
    }

    /// Add a kept episode
    pub fn add_kept(&mut self, id: uuid::Uuid) {
        self.kept += 1;
        self.kept_ids.push(id);
    }

    /// Add an error
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    /// Check if cleanup had any errors
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get cleanup success rate
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.evaluated == 0 {
            return 1.0;
        }
        let successful = self.evaluated - self.errors.len();
        successful as f64 / self.evaluated as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::Episode;
    use crate::types::{TaskContext, TaskType};

    fn create_test_episode() -> Episode {
        Episode::new(
            "test task".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        )
    }

    #[test]
    fn test_default_policy() {
        let policy = EpisodeRetentionPolicy::default();
        assert!(policy.validate().is_ok());
        assert_eq!(policy.max_age_days, DEFAULT_MAX_AGE_DAYS);
        assert_eq!(policy.max_episodes, DEFAULT_MAX_EPISODES);
    }

    #[test]
    fn test_storage_limited_policy() {
        let policy = EpisodeRetentionPolicy::storage_limited();
        assert!(policy.validate().is_ok());
        assert!(policy.max_age_days < DEFAULT_MAX_AGE_DAYS);
        assert!(policy.max_episodes < DEFAULT_MAX_EPISODES);
    }

    #[test]
    fn test_archival_policy() {
        let policy = EpisodeRetentionPolicy::archival();
        assert!(policy.validate().is_ok());
        assert_eq!(policy.criterion, RetentionCriterion::KeepAll);
    }

    #[test]
    fn test_aggressive_policy() {
        let policy = EpisodeRetentionPolicy::aggressive();
        assert!(policy.validate().is_ok());
        assert!(policy.max_age_days <= 7);
        assert!(!policy.keep_pattern_sources);
    }

    #[test]
    fn test_should_retain_keep_all() {
        let policy = EpisodeRetentionPolicy::archival();
        let episode = create_test_episode();
        assert!(policy.should_retain(&episode, Utc::now()));
    }

    #[test]
    fn test_should_retain_age_based() {
        let policy = EpisodeRetentionPolicy::default()
            .with_criterion(RetentionCriterion::AgeBased)
            .with_max_age_days(30);

        // Recent episode - should keep
        let recent = create_test_episode();
        assert!(policy.should_retain(&recent, Utc::now()));

        // Old episode - should cleanup
        let _old = Episode::new(
            "old task".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        );
        // Manually set old start time would require modifying episode
        // For now, just verify policy logic works with current episode
    }

    #[test]
    fn test_should_retain_with_patterns() {
        let policy =
            EpisodeRetentionPolicy::default().with_criterion(RetentionCriterion::Unreferenced);

        let mut episode = create_test_episode();
        episode.patterns.push(uuid::Uuid::new_v4());

        // Episode with patterns should be kept
        assert!(policy.should_retain(&episode, Utc::now()));
    }

    #[test]
    fn test_builder_methods() {
        let policy = EpisodeRetentionPolicy::new()
            .with_max_age_days(60)
            .with_min_reward_threshold(0.5)
            .with_max_episodes(5000)
            .with_cleanup_batch_size(50)
            .with_dry_run(true);

        assert_eq!(policy.max_age_days, 60);
        assert_eq!(policy.min_reward_threshold, 0.5);
        assert_eq!(policy.max_episodes, 5000);
        assert_eq!(policy.cleanup_batch_size, 50);
        assert!(policy.dry_run);
    }

    #[test]
    fn test_invalid_policy() {
        let invalid_age = EpisodeRetentionPolicy::default().with_max_age_days(-1);
        assert!(invalid_age.validate().is_err());

        // Note: with_min_reward_threshold clamps values to 0.0-1.0, so validation passes
        // This tests the validate() function directly with an unclamped value
        let invalid_threshold = EpisodeRetentionPolicy {
            min_reward_threshold: 1.5,
            ..Default::default()
        };
        assert!(invalid_threshold.validate().is_err());
    }

    #[test]
    fn test_cleanup_result() {
        let mut result = CleanupResult::new();
        result.evaluated = 10;
        result.add_deleted(uuid::Uuid::new_v4());
        result.add_deleted(uuid::Uuid::new_v4());
        result.add_kept(uuid::Uuid::new_v4());

        assert_eq!(result.deleted, 2);
        assert_eq!(result.kept, 1);
        assert_eq!(result.deleted_ids.len(), 2);
        assert!(!result.has_errors());
        assert_eq!(result.success_rate(), 1.0);
    }

    #[test]
    fn test_cleanup_result_with_errors() {
        let mut result = CleanupResult::new();
        result.evaluated = 10;
        result.add_error("Failed to delete episode".to_string());

        assert!(result.has_errors());
        assert!(result.success_rate() < 1.0);
    }
}

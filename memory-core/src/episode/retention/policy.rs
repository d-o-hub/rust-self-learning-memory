//! Episode retention policy and evaluation logic.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration as StdDuration;

use super::types::{RetentionCriterion, RetentionPolicyError};
use super::{
    DEFAULT_CLEANUP_BATCH_SIZE, DEFAULT_CLEANUP_INTERVAL, DEFAULT_MAX_AGE_DAYS,
    DEFAULT_MAX_EPISODES, DEFAULT_MIN_REWARD_THRESHOLD,
};

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
        match episode.reward.as_ref() {
            None => true,
            Some(r) => r.total >= self.min_reward_threshold as f32,
        }
    }

    fn check_references(&self, episode: &crate::Episode) -> bool {
        if self.keep_pattern_sources && !episode.patterns.is_empty() {
            return true;
        }
        if self.keep_heuristic_sources && !episode.heuristics.is_empty() {
            return true;
        }
        false
    }

    fn check_failed(&self, episode: &crate::Episode) -> bool {
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

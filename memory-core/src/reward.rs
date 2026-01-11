//! # Reward Calculator
//!
//! Calculates reward scores for episodes based on outcome, efficiency, and quality.
//! Supports both fixed thresholds and adaptive domain-based calibration.
//!
//! ## Modules
//!
//! - `domain_stats` - Domain-specific statistics for adaptive calibration
//! - `adaptive` - Adaptive reward calculator using domain baselines
//!

#![allow(clippy::if_not_else)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::doc_markdown)]
//! Calculates reward scores for completed episodes based on:
//! - Task outcome (success/partial/failure)
//! - Efficiency (duration and step count)
//! - Complexity bonus
//!
//! ## Example
//!
//! ```
//! use memory_core::reward::RewardCalculator;
//! use memory_core::{Episode, TaskContext, TaskType, TaskOutcome};
//!
//! let context = TaskContext::default();
//! let mut episode = Episode::new("Test task".to_string(), context, TaskType::Testing);
//! episode.complete(TaskOutcome::Success {
//!     verdict: "All tests passed".to_string(),
//!     artifacts: vec![],
//! });
//!
//! let calculator = RewardCalculator::new();
//! let reward = calculator.calculate(&episode);
//!
//! assert!(reward.total > 0.0);
//! assert_eq!(reward.base, 1.0); // Success gives base 1.0
//! ```

use crate::episode::Episode;
use crate::types::{ComplexityLevel, RewardScore, TaskOutcome};
use tracing::{debug, instrument};

// Public modules
pub mod adaptive;
pub mod domain_stats;

#[cfg(test)]
pub mod tests;

// Re-export for convenience
pub use adaptive::AdaptiveRewardCalculator;
pub use domain_stats::{DomainStatistics, DomainStatisticsCache};

/// Threshold for "efficient" episode duration (in seconds)
const EFFICIENT_DURATION_SECS: f32 = 60.0;

/// Threshold for "efficient" step count
const EFFICIENT_STEP_COUNT: usize = 10;

/// Maximum efficiency multiplier
const MAX_EFFICIENCY_MULTIPLIER: f32 = 1.5;

/// Minimum efficiency multiplier
const MIN_EFFICIENCY_MULTIPLIER: f32 = 0.5;

/// Calculator for episode reward scores
#[derive(Clone)]
pub struct RewardCalculator {
    /// Weight for duration in efficiency calculation
    duration_weight: f32,
    /// Weight for step count in efficiency calculation
    step_count_weight: f32,
}

impl Default for RewardCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl RewardCalculator {
    /// Create a new reward calculator with default weights
    #[must_use]
    pub fn new() -> Self {
        Self {
            duration_weight: 0.5,
            step_count_weight: 0.5,
        }
    }

    /// Create a calculator with custom weights
    #[must_use]
    pub fn with_weights(duration_weight: f32, step_count_weight: f32) -> Self {
        Self {
            duration_weight,
            step_count_weight,
        }
    }

    /// Calculate reward score for an episode
    #[instrument(skip(self, episode), fields(episode_id = %episode.episode_id))]
    pub fn calculate(&self, episode: &Episode) -> RewardScore {
        let base = self.calculate_base_reward(episode);
        let efficiency = self.calculate_efficiency_multiplier(episode);
        let complexity_bonus = self.calculate_complexity_bonus(episode);
        let quality_multiplier = self.calculate_quality_multiplier(episode);
        let learning_bonus = self.calculate_learning_bonus(episode);

        // Calculate total: base reward * multipliers + bonuses
        let total = (base * efficiency * complexity_bonus * quality_multiplier) + learning_bonus;

        debug!(
            base = base,
            efficiency = efficiency,
            complexity_bonus = complexity_bonus,
            quality_multiplier = quality_multiplier,
            learning_bonus = learning_bonus,
            total = total,
            "Calculated reward score"
        );

        RewardScore {
            total,
            base,
            efficiency,
            complexity_bonus,
            quality_multiplier,
            learning_bonus,
        }
    }

    /// Calculate base reward from outcome
    fn calculate_base_reward(&self, episode: &Episode) -> f32 {
        match &episode.outcome {
            Some(TaskOutcome::Success { .. }) => 1.0,
            Some(TaskOutcome::PartialSuccess {
                completed, failed, ..
            }) => {
                // Proportional reward based on completion ratio
                let total = completed.len() + failed.len();
                if total == 0 {
                    0.5 // Default for partial success with no specifics
                } else {
                    completed.len() as f32 / total as f32
                }
            }
            Some(TaskOutcome::Failure { .. }) => 0.0,
            None => 0.0, // Not completed
        }
    }

    /// Calculate efficiency multiplier based on duration and step count
    fn calculate_efficiency_multiplier(&self, episode: &Episode) -> f32 {
        let duration_score = self.calculate_duration_efficiency(episode);
        let step_count_score = self.calculate_step_count_efficiency(episode);

        let combined =
            (duration_score * self.duration_weight) + (step_count_score * self.step_count_weight);

        // Clamp to reasonable bounds
        combined.clamp(MIN_EFFICIENCY_MULTIPLIER, MAX_EFFICIENCY_MULTIPLIER)
    }

    /// Calculate duration efficiency score
    fn calculate_duration_efficiency(&self, episode: &Episode) -> f32 {
        if let Some(duration) = episode.duration() {
            let duration_secs = duration.num_seconds() as f32;

            if duration_secs <= 0.0 {
                return MAX_EFFICIENCY_MULTIPLIER;
            }

            // Efficiency decreases as duration increases
            // Exponential decay: e^(-x/threshold)
            let ratio = duration_secs / EFFICIENT_DURATION_SECS;
            let score = (-ratio / 2.0).exp();

            // Map to multiplier range
            MIN_EFFICIENCY_MULTIPLIER
                + (score * (MAX_EFFICIENCY_MULTIPLIER - MIN_EFFICIENCY_MULTIPLIER))
        } else {
            1.0 // Default if no duration
        }
    }

    /// Calculate step count efficiency score
    fn calculate_step_count_efficiency(&self, episode: &Episode) -> f32 {
        let step_count = episode.steps.len();

        if step_count == 0 {
            return MIN_EFFICIENCY_MULTIPLIER;
        }

        // Efficiency decreases as step count increases
        let ratio = step_count as f32 / EFFICIENT_STEP_COUNT as f32;
        let score = (-ratio / 2.0).exp();

        // Map to multiplier range
        MIN_EFFICIENCY_MULTIPLIER
            + (score * (MAX_EFFICIENCY_MULTIPLIER - MIN_EFFICIENCY_MULTIPLIER))
    }

    /// Calculate complexity bonus multiplier
    fn calculate_complexity_bonus(&self, episode: &Episode) -> f32 {
        match episode.context.complexity {
            ComplexityLevel::Simple => 1.0,
            ComplexityLevel::Moderate => 1.1,
            ComplexityLevel::Complex => 1.2,
        }
    }

    /// Calculate quality multiplier based on code quality metrics
    ///
    /// Analyzes artifacts and execution quality to determine a multiplier.
    /// Factors include:
    /// - Test coverage (detected from artifacts)
    /// - Code quality indicators (linting, formatting)
    /// - Error handling quality (low error rate)
    fn calculate_quality_multiplier(&self, episode: &Episode) -> f32 {
        let mut quality: f32 = 1.0;

        // Analyze artifacts for quality indicators
        if let Some(TaskOutcome::Success { artifacts, .. }) = &episode.outcome {
            // Bonus for test coverage artifacts
            let has_test_coverage = artifacts
                .iter()
                .any(|a| a.contains("coverage") || a.contains("test"));
            if has_test_coverage {
                quality += 0.1;
            }

            // Bonus for multiple quality artifacts (docs, tests, etc.)
            if artifacts.len() >= 3 {
                quality += 0.05;
            }

            // Check for quality-related metadata
            if let Some(coverage_str) = episode.metadata.get("test_coverage") {
                if let Ok(coverage) = coverage_str.parse::<f32>() {
                    // Bonus for high test coverage (>80%)
                    #[allow(clippy::excessive_nesting)]
                    if coverage > 80.0 {
                        quality += 0.15;
                    } else if coverage > 60.0 {
                        quality += 0.1;
                    }
                }
            }
        }

        // Quality based on error handling
        let total_steps = episode.steps.len();
        if total_steps > 0 {
            let error_rate = episode.failed_steps_count() as f32 / total_steps as f32;

            // Penalize high error rates
            if error_rate > 0.3 {
                quality -= 0.2;
            } else if error_rate > 0.1 {
                quality -= 0.1;
            } else if error_rate == 0.0 {
                // Bonus for zero errors
                quality += 0.1;
            }
        }

        // Check for linting/formatting indicators
        if episode.metadata.contains_key("clippy_warnings") {
            if let Some(warnings) = episode.metadata.get("clippy_warnings") {
                if warnings == "0" {
                    quality += 0.05;
                }
            }
        }

        // Clamp to reasonable bounds (0.5 to 1.5)
        quality.clamp(0.5, 1.5)
    }

    /// Calculate learning bonus for discovering patterns and improvements
    ///
    /// Awards bonus points for:
    /// - Discovering new patterns (novel approaches)
    /// - Improving on past attempts (learning from history)
    /// - Efficient problem-solving (first-time success)
    fn calculate_learning_bonus(&self, episode: &Episode) -> f32 {
        let mut bonus = 0.0;

        // Bonus for discovering new patterns
        let pattern_count = episode.patterns.len();
        if pattern_count > 0 {
            // More patterns = more learning
            bonus += (pattern_count as f32 * 0.1).min(0.3);
        }

        // Bonus for novel tool sequences
        if let Some(novelty) = self.calculate_novelty_bonus(episode) {
            bonus += novelty;
        }

        // Bonus for efficient problem solving (high success rate)
        let total_steps = episode.steps.len();
        if total_steps > 0 {
            let success_rate = episode.successful_steps_count() as f32 / total_steps as f32;

            if success_rate > 0.9 && total_steps >= 5 {
                // High reliability with meaningful complexity
                bonus += 0.2;
            } else if success_rate == 1.0 && total_steps >= 3 {
                // Perfect execution
                bonus += 0.15;
            }
        }

        // Bonus for error recovery (learning from failures)
        if self.detect_error_recovery(episode) {
            bonus += 0.15;
        }

        // Bonus for optimization (completing quickly with few steps)
        if let Some(duration) = episode.duration() {
            let duration_secs = duration.num_seconds() as f32;
            if duration_secs < 30.0 && total_steps > 0 && total_steps < 10 {
                bonus += 0.1;
            }
        }

        // Cap learning bonus
        bonus.min(0.5)
    }

    /// Calculate novelty bonus for unique tool combinations
    fn calculate_novelty_bonus(&self, episode: &Episode) -> Option<f32> {
        if episode.steps.len() < 3 {
            return None;
        }

        // Count unique tools used
        let unique_tools: std::collections::HashSet<_> =
            episode.steps.iter().map(|s| &s.tool).collect();

        // Bonus for diverse tool usage
        if unique_tools.len() >= 5 {
            Some(0.15)
        } else if unique_tools.len() >= 3 {
            Some(0.1)
        } else {
            None
        }
    }

    /// Detect if the episode shows error recovery
    fn detect_error_recovery(&self, episode: &Episode) -> bool {
        for i in 0..episode.steps.len().saturating_sub(1) {
            let current = &episode.steps[i];
            let next = &episode.steps[i + 1];

            // Error followed by success = recovery
            if !current.is_success() && next.is_success() {
                return true;
            }
        }
        false
    }
}

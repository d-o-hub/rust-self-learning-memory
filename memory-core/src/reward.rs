//! # Reward Calculator
//!
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
    pub fn new() -> Self {
        Self {
            duration_weight: 0.5,
            step_count_weight: 0.5,
        }
    }

    /// Create a calculator with custom weights
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

        let total = base * efficiency * complexity_bonus;

        debug!(
            base = base,
            efficiency = efficiency,
            complexity_bonus = complexity_bonus,
            total = total,
            "Calculated reward score"
        );

        RewardScore {
            total,
            base,
            efficiency,
            complexity_bonus,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::ExecutionStep;
    use crate::types::{ComplexityLevel, ExecutionResult, TaskContext, TaskType};

    fn create_test_episode(complexity: ComplexityLevel) -> Episode {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity,
            domain: "testing".to_string(),
            tags: vec![],
        };

        Episode::new("Test task".to_string(), context, TaskType::Testing)
    }

    #[test]
    fn test_successful_episode_reward() {
        let calculator = RewardCalculator::new();
        let mut episode = create_test_episode(ComplexityLevel::Moderate);

        episode.complete(TaskOutcome::Success {
            verdict: "All tests passed".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        assert_eq!(reward.base, 1.0);
        assert!(reward.efficiency > 0.0);
        assert_eq!(reward.complexity_bonus, 1.1); // Moderate complexity
        assert!(reward.total > 0.0);
    }

    #[test]
    fn test_failed_episode_reward() {
        let calculator = RewardCalculator::new();
        let mut episode = create_test_episode(ComplexityLevel::Simple);

        episode.complete(TaskOutcome::Failure {
            reason: "Tests failed".to_string(),
            error_details: None,
        });

        let reward = calculator.calculate(&episode);

        assert_eq!(reward.base, 0.0);
        assert_eq!(reward.total, 0.0); // Base is 0, so total is 0
    }

    #[test]
    fn test_partial_success_reward() {
        let calculator = RewardCalculator::new();
        let mut episode = create_test_episode(ComplexityLevel::Moderate);

        episode.complete(TaskOutcome::PartialSuccess {
            verdict: "Some tests passed".to_string(),
            completed: vec!["test1".to_string(), "test2".to_string()],
            failed: vec!["test3".to_string()],
        });

        let reward = calculator.calculate(&episode);

        // 2 out of 3 succeeded = 0.667
        assert!((reward.base - 0.667).abs() < 0.01);
        assert!(reward.total > 0.0);
    }

    #[test]
    fn test_efficiency_fast_execution() {
        let calculator = RewardCalculator::new();
        let mut episode = create_test_episode(ComplexityLevel::Simple);

        // Add just a few steps
        for i in 0..3 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Quick completion".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        // Should have high efficiency (few steps, fast completion)
        assert!(reward.efficiency > 1.0);
    }

    #[test]
    fn test_efficiency_slow_execution() {
        let calculator = RewardCalculator::new();
        let mut episode = create_test_episode(ComplexityLevel::Simple);

        // Simulate episode that started 5 minutes ago
        episode.start_time = chrono::Utc::now() - chrono::Duration::minutes(5);

        // Add many steps (more than efficient threshold)
        for i in 0..50 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Slow completion".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        // Should have lower efficiency (long duration + many steps)
        assert!(
            reward.efficiency < 1.0,
            "Expected efficiency < 1.0, got {}",
            reward.efficiency
        );
    }

    #[test]
    fn test_complexity_bonus() {
        let calculator = RewardCalculator::new();

        let mut simple = create_test_episode(ComplexityLevel::Simple);
        simple.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let mut moderate = create_test_episode(ComplexityLevel::Moderate);
        moderate.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let mut complex = create_test_episode(ComplexityLevel::Complex);
        complex.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let simple_reward = calculator.calculate(&simple);
        let moderate_reward = calculator.calculate(&moderate);
        let complex_reward = calculator.calculate(&complex);

        assert_eq!(simple_reward.complexity_bonus, 1.0);
        assert_eq!(moderate_reward.complexity_bonus, 1.1);
        assert_eq!(complex_reward.complexity_bonus, 1.2);

        // More complex tasks should have higher total rewards (all else equal)
        assert!(complex_reward.total > moderate_reward.total);
        assert!(moderate_reward.total > simple_reward.total);
    }

    #[test]
    fn test_custom_weights() {
        // Heavily weight duration
        let calculator = RewardCalculator::with_weights(0.9, 0.1);
        let mut episode = create_test_episode(ComplexityLevel::Moderate);

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);
        assert!(reward.total > 0.0);
    }

    #[test]
    fn test_incomplete_episode() {
        let calculator = RewardCalculator::new();
        let episode = create_test_episode(ComplexityLevel::Moderate);

        // Episode not completed
        let reward = calculator.calculate(&episode);

        assert_eq!(reward.base, 0.0);
        assert_eq!(reward.total, 0.0);
    }
}

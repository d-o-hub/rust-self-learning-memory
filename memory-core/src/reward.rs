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
        assert!(reward.quality_multiplier > 0.0);
        assert!(reward.learning_bonus >= 0.0);
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
            let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
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
            let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
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

    #[test]
    fn test_quality_multiplier_with_test_coverage() {
        let calculator = RewardCalculator::new();
        let mut episode = create_test_episode(ComplexityLevel::Moderate);

        // Add test coverage metadata
        episode
            .metadata
            .insert("test_coverage".to_string(), "85.5".to_string());

        episode.complete(TaskOutcome::Success {
            verdict: "Tests passed with coverage".to_string(),
            artifacts: vec!["coverage_report.html".to_string()],
        });

        let reward = calculator.calculate(&episode);

        // Should have quality bonus for high coverage
        assert!(reward.quality_multiplier > 1.0);
    }

    #[test]
    fn test_quality_multiplier_with_zero_errors() {
        let calculator = RewardCalculator::new();
        let mut episode = create_test_episode(ComplexityLevel::Simple);

        // Add all successful steps
        for i in 0..5 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Perfect execution".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        // Zero errors should give quality bonus
        assert!(reward.quality_multiplier >= 1.0);
    }

    #[test]
    fn test_quality_multiplier_with_high_error_rate() {
        let calculator = RewardCalculator::new();
        let mut episode = create_test_episode(ComplexityLevel::Simple);

        // Add many failed steps
        for i in 0..10 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
            if i < 7 {
                step.result = Some(ExecutionResult::Error {
                    message: "Error".to_string(),
                });
            } else {
                step.result = Some(ExecutionResult::Success {
                    output: "OK".to_string(),
                });
            }
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Eventually succeeded".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        // High error rate should penalize quality
        assert!(reward.quality_multiplier < 1.0);
    }

    #[test]
    fn test_learning_bonus_with_patterns() {
        let calculator = RewardCalculator::new();
        let mut episode = create_test_episode(ComplexityLevel::Moderate);

        // Add some successful steps
        for i in 0..5 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        // Add pattern IDs to simulate pattern discovery
        use uuid::Uuid;
        episode.patterns.push(Uuid::new_v4());
        episode.patterns.push(Uuid::new_v4());

        episode.complete(TaskOutcome::Success {
            verdict: "Learned new patterns".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        // Should have learning bonus for pattern discovery
        assert!(reward.learning_bonus > 0.0);
    }

    #[test]
    fn test_learning_bonus_for_error_recovery() {
        let calculator = RewardCalculator::new();
        let mut episode = create_test_episode(ComplexityLevel::Moderate);

        // Add error followed by recovery
        let mut error_step =
            ExecutionStep::new(1, "failing_tool".to_string(), "Failed action".to_string());
        error_step.result = Some(ExecutionResult::Error {
            message: "Error".to_string(),
        });
        episode.add_step(error_step);

        let mut recovery_step = ExecutionStep::new(
            2,
            "recovery_tool".to_string(),
            "Recovery action".to_string(),
        );
        recovery_step.result = Some(ExecutionResult::Success {
            output: "Recovered".to_string(),
        });
        episode.add_step(recovery_step);

        episode.complete(TaskOutcome::Success {
            verdict: "Recovered from error".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        // Should have learning bonus for error recovery
        assert!(reward.learning_bonus > 0.0);
    }

    #[test]
    fn test_learning_bonus_for_diverse_tools() {
        let calculator = RewardCalculator::new();
        let mut episode = create_test_episode(ComplexityLevel::Complex);

        // Add many different tools (diverse approach)
        for i in 0..6 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Used diverse toolset".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        // Should have learning bonus for tool diversity
        assert!(reward.learning_bonus > 0.0);
    }

    #[test]
    fn test_learning_bonus_for_efficient_execution() {
        let calculator = RewardCalculator::new();
        let mut episode = create_test_episode(ComplexityLevel::Simple);

        // Add successful steps with perfect execution
        for i in 0..5 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Perfect execution".to_string(),
            artifacts: vec![],
        });

        let reward = calculator.calculate(&episode);

        // Should have learning bonus for high success rate
        assert!(reward.learning_bonus > 0.0);
    }

    #[test]
    fn test_combined_quality_and_learning_bonuses() {
        let calculator = RewardCalculator::new();
        let mut episode = create_test_episode(ComplexityLevel::Complex);

        // Add high-quality execution
        for i in 0..7 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        // Add quality indicators
        episode
            .metadata
            .insert("test_coverage".to_string(), "90.0".to_string());
        episode
            .metadata
            .insert("clippy_warnings".to_string(), "0".to_string());

        // Add patterns
        use uuid::Uuid;
        episode.patterns.push(Uuid::new_v4());

        episode.complete(TaskOutcome::Success {
            verdict: "High quality with learning".to_string(),
            artifacts: vec![
                "tests.rs".to_string(),
                "coverage.html".to_string(),
                "docs.md".to_string(),
            ],
        });

        let reward = calculator.calculate(&episode);

        // Should have both quality and learning bonuses
        assert!(reward.quality_multiplier > 1.0);
        assert!(reward.learning_bonus > 0.0);
        assert!(reward.total > reward.base);
    }
}

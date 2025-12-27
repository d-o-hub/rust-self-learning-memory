//! Episode quality assessment for pre-storage filtering.
//!
//! Implements quality scoring using multiple features to determine whether
//! an episode is worth storing in the memory system.

use crate::episode::Episode;
use std::collections::HashSet;

/// Configuration for quality assessment.
///
/// Controls the quality threshold and feature weights used to assess
/// episode quality before storage.
///
/// # Examples
///
/// ```
/// use memory_core::pre_storage::{QualityConfig, QualityFeature};
/// use std::collections::HashMap;
///
/// // Default configuration (quality threshold 0.7)
/// let config = QualityConfig::default();
///
/// // Custom configuration with higher threshold
/// let mut custom_config = QualityConfig::new(0.8);
/// custom_config.set_weight(QualityFeature::TaskComplexity, 0.3);
/// custom_config.set_weight(QualityFeature::StepDiversity, 0.2);
/// ```
#[derive(Debug, Clone)]
pub struct QualityConfig {
    /// Minimum quality score required for storage (0.0 to 1.0)
    pub quality_threshold: f32,
    /// Weights for each quality feature
    feature_weights: std::collections::HashMap<QualityFeature, f32>,
}

impl QualityConfig {
    /// Create a new quality configuration with the specified threshold.
    ///
    /// Uses default feature weights that sum to 1.0.
    ///
    /// # Arguments
    ///
    /// * `quality_threshold` - Minimum quality score (0.0 to 1.0)
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::pre_storage::QualityConfig;
    ///
    /// let config = QualityConfig::new(0.75);
    /// assert_eq!(config.quality_threshold, 0.75);
    /// ```
    #[must_use]
    pub fn new(quality_threshold: f32) -> Self {
        let mut feature_weights = std::collections::HashMap::new();
        feature_weights.insert(QualityFeature::TaskComplexity, 0.25);
        feature_weights.insert(QualityFeature::StepDiversity, 0.20);
        feature_weights.insert(QualityFeature::ErrorRate, 0.20);
        feature_weights.insert(QualityFeature::ReflectionDepth, 0.20);
        feature_weights.insert(QualityFeature::PatternNovelty, 0.15);

        Self {
            quality_threshold,
            feature_weights,
        }
    }

    /// Set the weight for a specific quality feature.
    ///
    /// # Arguments
    ///
    /// * `feature` - The quality feature to set weight for
    /// * `weight` - The weight value (should sum to 1.0 across all features)
    pub fn set_weight(&mut self, feature: QualityFeature, weight: f32) {
        self.feature_weights.insert(feature, weight);
    }

    /// Get the weight for a specific quality feature.
    ///
    /// # Arguments
    ///
    /// * `feature` - The quality feature to get weight for
    ///
    /// # Returns
    ///
    /// The weight value, or 0.0 if not set
    #[must_use]
    pub fn get_weight(&self, feature: QualityFeature) -> f32 {
        *self.feature_weights.get(&feature).unwrap_or(&0.0)
    }
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self::new(0.7)
    }
}

/// Quality features used to assess episode quality.
///
/// Each feature measures a different aspect of episode quality and
/// contributes to the overall quality score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QualityFeature {
    /// Task complexity based on number of steps and tool diversity
    TaskComplexity,
    /// Diversity of steps taken during execution
    StepDiversity,
    /// Error rate and recovery quality
    ErrorRate,
    /// Depth and quality of reflections
    ReflectionDepth,
    /// Novelty of patterns discovered
    PatternNovelty,
}

/// Assess episode quality before storage.
///
/// Implements the `PREMem` quality assessment algorithm using multiple
/// quality features to determine if an episode is worth storing.
///
/// # Examples
///
/// ```no_run
/// use memory_core::pre_storage::{QualityAssessor, QualityConfig};
/// use memory_core::{Episode, TaskContext, TaskType};
///
/// let assessor = QualityAssessor::new(QualityConfig::default());
///
/// let episode = Episode::new(
///     "Complex task".to_string(),
///     TaskContext::default(),
///     TaskType::CodeGeneration,
/// );
///
/// let score = assessor.assess_episode(&episode);
/// println!("Quality score: {:.2}", score);
/// ```
#[derive(Debug, Clone)]
pub struct QualityAssessor {
    config: QualityConfig,
}

impl QualityAssessor {
    /// Create a new quality assessor with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Quality configuration including threshold and weights
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::pre_storage::{QualityAssessor, QualityConfig};
    ///
    /// let assessor = QualityAssessor::new(QualityConfig::default());
    /// ```
    #[must_use]
    pub fn new(config: QualityConfig) -> Self {
        Self { config }
    }

    /// Assess the quality of an episode.
    ///
    /// Computes a weighted quality score from multiple features. The score
    /// ranges from 0.0 (lowest quality) to 1.0 (highest quality).
    ///
    /// # Arguments
    ///
    /// * `episode` - The episode to assess
    ///
    /// # Returns
    ///
    /// Quality score in range 0.0-1.0
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use memory_core::pre_storage::{QualityAssessor, QualityConfig};
    /// use memory_core::{Episode, TaskContext, TaskType};
    ///
    /// let assessor = QualityAssessor::new(QualityConfig::default());
    /// let episode = Episode::new(
    ///     "Test task".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::Testing,
    /// );
    ///
    /// let score = assessor.assess_episode(&episode);
    /// assert!(score >= 0.0 && score <= 1.0);
    /// ```
    #[must_use]
    pub fn assess_episode(&self, episode: &Episode) -> f32 {
        let task_complexity = self.assess_task_complexity(episode);
        let step_diversity = self.assess_step_diversity(episode);
        let error_rate = self.assess_error_rate(episode);
        let reflection_depth = self.assess_reflection_depth(episode);
        let pattern_novelty = self.assess_pattern_novelty(episode);

        // Weighted sum of all features
        let score = task_complexity * self.config.get_weight(QualityFeature::TaskComplexity)
            + step_diversity * self.config.get_weight(QualityFeature::StepDiversity)
            + error_rate * self.config.get_weight(QualityFeature::ErrorRate)
            + reflection_depth * self.config.get_weight(QualityFeature::ReflectionDepth)
            + pattern_novelty * self.config.get_weight(QualityFeature::PatternNovelty);

        // Clamp score to valid range
        score.clamp(0.0, 1.0)
    }

    /// Check if an episode meets the quality threshold.
    ///
    /// # Arguments
    ///
    /// * `episode` - The episode to check
    ///
    /// # Returns
    ///
    /// `true` if quality score >= threshold, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use memory_core::pre_storage::{QualityAssessor, QualityConfig};
    /// use memory_core::{Episode, TaskContext, TaskType};
    ///
    /// let assessor = QualityAssessor::new(QualityConfig::default());
    /// let episode = Episode::new(
    ///     "Test task".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::Testing,
    /// );
    ///
    /// if assessor.should_store(&episode) {
    ///     println!("Episode meets quality threshold");
    /// }
    /// ```
    #[must_use]
    pub fn should_store(&self, episode: &Episode) -> bool {
        self.assess_episode(episode) >= self.config.quality_threshold
    }

    /// Assess task complexity based on number of steps and tool diversity.
    ///
    /// Higher complexity scores indicate more sophisticated tasks with
    /// diverse tool usage and multiple execution steps.
    ///
    /// # Scoring
    ///
    /// - 0.0-0.3: Simple tasks (< 3 steps, single tool)
    /// - 0.3-0.7: Moderate tasks (3-10 steps, 2-3 tools)
    /// - 0.7-1.0: Complex tasks (> 10 steps, diverse tools)
    fn assess_task_complexity(&self, episode: &Episode) -> f32 {
        let step_count = episode.steps.len();
        let unique_tools: HashSet<_> = episode.steps.iter().map(|s| &s.tool).collect();
        let tool_diversity = unique_tools.len();

        // Score based on step count (0.0 to 0.5)
        let step_score = match step_count {
            0..=2 => 0.1,
            3..=5 => 0.25,
            6..=10 => 0.35,
            11..=20 => 0.45,
            _ => 0.5,
        };

        // Score based on tool diversity (0.0 to 0.5)
        let tool_score = match tool_diversity {
            0..=1 => 0.1,
            2..=3 => 0.25,
            4..=6 => 0.35,
            7..=10 => 0.45,
            _ => 0.5,
        };

        // Combined complexity score
        step_score + tool_score
    }

    /// Assess step diversity based on variety of actions and results.
    ///
    /// Higher diversity scores indicate episodes with varied execution
    /// patterns rather than repetitive actions.
    ///
    /// # Scoring
    ///
    /// - 0.0-0.3: Low diversity (repetitive actions)
    /// - 0.3-0.7: Moderate diversity (some variation)
    /// - 0.7-1.0: High diversity (varied actions and results)
    fn assess_step_diversity(&self, episode: &Episode) -> f32 {
        if episode.steps.is_empty() {
            return 0.0;
        }

        // Count unique actions
        let unique_actions: HashSet<_> = episode.steps.iter().map(|s| &s.action).collect();
        let action_diversity = unique_actions.len() as f32 / episode.steps.len() as f32;

        // Count unique result types
        let success_count = episode.successful_steps_count();
        let error_count = episode.failed_steps_count();
        let result_diversity = if success_count > 0 && error_count > 0 {
            0.5 // Mix of success and failure shows error recovery
        } else if success_count > 0 || error_count > 0 {
            0.3 // All same result type
        } else {
            0.0 // No results
        };

        // Combined diversity score
        (action_diversity * 0.6 + result_diversity * 0.4).clamp(0.0, 1.0)
    }

    /// Assess error handling quality based on error rate and recovery.
    ///
    /// Better scores for episodes with good error handling and recovery
    /// patterns rather than complete failure or no errors.
    ///
    /// # Scoring
    ///
    /// - 0.0-0.3: High error rate without recovery
    /// - 0.3-0.7: Some errors with recovery attempts
    /// - 0.7-1.0: Good error handling or very few errors
    fn assess_error_rate(&self, episode: &Episode) -> f32 {
        let total_steps = episode.steps.len();
        if total_steps == 0 {
            return 0.5; // Neutral score for no steps
        }

        let error_count = episode.failed_steps_count();
        let success_count = episode.successful_steps_count();
        let error_rate = error_count as f32 / total_steps as f32;

        // Score based on error handling patterns
        if error_rate == 0.0 {
            // Perfect execution - high score
            0.9
        } else if error_rate < 0.2 && success_count > error_count {
            // Few errors with successful recovery - highest score
            1.0
        } else if error_rate < 0.4 {
            // Moderate errors - medium score
            0.6
        } else if error_rate < 0.6 && success_count > 0 {
            // Many errors but some success - low-medium score
            0.4
        } else {
            // Very high error rate - low score
            0.2
        }
    }

    /// Assess reflection quality based on depth and insight content.
    ///
    /// Higher scores for episodes with rich, detailed reflections that
    /// capture meaningful insights.
    ///
    /// # Scoring
    ///
    /// - 0.0-0.3: No reflection or shallow
    /// - 0.3-0.7: Basic reflection with some insights
    /// - 0.7-1.0: Deep reflection with valuable insights
    fn assess_reflection_depth(&self, episode: &Episode) -> f32 {
        let Some(ref reflection) = episode.reflection else {
            return 0.0; // No reflection
        };

        let success_count = reflection.successes.len();
        let improvement_count = reflection.improvements.len();
        let insight_count = reflection.insights.len();
        let total_items = success_count + improvement_count + insight_count;

        // Score based on reflection content
        match total_items {
            0 => 0.0,      // Empty reflection
            1..=2 => 0.3,  // Minimal reflection
            3..=5 => 0.6,  // Basic reflection
            6..=10 => 0.8, // Good reflection
            _ => 1.0,      // Comprehensive reflection
        }
    }

    /// Assess pattern novelty based on number of patterns extracted.
    ///
    /// Higher scores for episodes that resulted in discovering new
    /// patterns or heuristics.
    ///
    /// # Scoring
    ///
    /// - 0.0-0.3: No new patterns discovered
    /// - 0.3-0.7: Some patterns extracted
    /// - 0.7-1.0: Multiple novel patterns discovered
    fn assess_pattern_novelty(&self, episode: &Episode) -> f32 {
        let pattern_count = episode.patterns.len();
        let heuristic_count = episode.heuristics.len();
        let total_patterns = pattern_count + heuristic_count;

        match total_patterns {
            0 => 0.2,      // No patterns - still has some learning value
            1..=2 => 0.5,  // Few patterns
            3..=5 => 0.75, // Moderate pattern discovery
            _ => 1.0,      // High pattern discovery
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::ExecutionStep;
    use crate::types::{ExecutionResult, Reflection};
    use crate::types::{TaskContext, TaskOutcome, TaskType};
    use chrono::Utc;

    fn create_test_episode() -> Episode {
        Episode::new(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
    }

    #[test]
    fn test_quality_config_default() {
        let config = QualityConfig::default();
        assert_eq!(config.quality_threshold, 0.7);
        assert_eq!(config.get_weight(QualityFeature::TaskComplexity), 0.25);
        assert_eq!(config.get_weight(QualityFeature::StepDiversity), 0.20);
    }

    #[test]
    fn test_quality_config_custom() {
        let mut config = QualityConfig::new(0.8);
        config.set_weight(QualityFeature::TaskComplexity, 0.5);
        assert_eq!(config.quality_threshold, 0.8);
        assert_eq!(config.get_weight(QualityFeature::TaskComplexity), 0.5);
    }

    #[test]
    fn test_quality_score_in_valid_range() {
        let assessor = QualityAssessor::new(QualityConfig::default());
        let episode = create_test_episode();
        let score = assessor.assess_episode(&episode);
        assert!((0.0..=1.0).contains(&score));
    }

    #[test]
    fn test_empty_episode_low_quality() {
        let assessor = QualityAssessor::new(QualityConfig::default());
        let episode = create_test_episode();
        let score = assessor.assess_episode(&episode);
        assert!(score < 0.3, "Empty episode should have low quality score");
    }

    #[test]
    fn test_complex_episode_high_quality() {
        let assessor = QualityAssessor::new(QualityConfig::default());
        let mut episode = create_test_episode();

        // Add many diverse steps
        for i in 0..15 {
            let mut step =
                ExecutionStep::new(i + 1, format!("tool_{}", i % 5), format!("action_{i}"));
            step.result = Some(ExecutionResult::Success {
                output: "Success".to_string(),
            });
            episode.add_step(step);
        }

        // Add reflection
        episode.reflection = Some(Reflection {
            successes: vec!["Good work".to_string(); 3],
            improvements: vec!["Could improve".to_string(); 2],
            insights: vec!["Key insight".to_string(); 2],
            generated_at: Utc::now(),
        });

        // Complete with success
        episode.complete(TaskOutcome::Success {
            verdict: "Success".to_string(),
            artifacts: vec!["artifact.rs".to_string()],
        });

        let score = assessor.assess_episode(&episode);
        assert!(
            score > 0.6,
            "Complex episode should have high quality score, got {score}"
        );
    }

    #[test]
    fn test_simple_episode_low_quality() {
        let assessor = QualityAssessor::new(QualityConfig::default());
        let mut episode = create_test_episode();

        // Add single simple step
        let mut step = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);

        let score = assessor.assess_episode(&episode);
        assert!(
            score < 0.5,
            "Simple episode should have low quality score, got {score}"
        );
    }

    #[test]
    fn test_high_error_rate_low_quality() {
        let assessor = QualityAssessor::new(QualityConfig::default());
        let mut episode = create_test_episode();

        // Add mostly failing steps
        for i in 0..10 {
            let mut step = ExecutionStep::new(i + 1, "tool".to_string(), format!("action_{i}"));
            step.result = Some(ExecutionResult::Error {
                message: "Error".to_string(),
            });
            episode.add_step(step);
        }

        let score = assessor.assess_episode(&episode);
        assert!(score < 0.4, "High error rate should result in low quality");
    }

    #[test]
    fn test_error_recovery_high_quality() {
        let assessor = QualityAssessor::new(QualityConfig::default());
        let mut episode = create_test_episode();

        // Add mix of errors and successes (error recovery pattern)
        for i in 0..10 {
            let mut step =
                ExecutionStep::new(i + 1, format!("tool_{}", i % 3), format!("action_{i}"));
            step.result = if i % 3 == 0 {
                Some(ExecutionResult::Error {
                    message: "Error".to_string(),
                })
            } else {
                Some(ExecutionResult::Success {
                    output: "Success".to_string(),
                })
            };
            episode.add_step(step);
        }

        let score = assessor.assess_episode(&episode);
        assert!(
            score > 0.4,
            "Error recovery should improve quality score, got {score}"
        );
    }

    #[test]
    fn test_reflection_improves_quality() {
        let assessor = QualityAssessor::new(QualityConfig::default());

        // Episode without reflection
        let mut episode1 = create_test_episode();
        for i in 0..5 {
            let mut step = ExecutionStep::new(i + 1, "tool".to_string(), "action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode1.add_step(step);
        }
        let score1 = assessor.assess_episode(&episode1);

        // Same episode with reflection
        let mut episode2 = episode1.clone();
        episode2.reflection = Some(Reflection {
            successes: vec!["Success 1".to_string(), "Success 2".to_string()],
            improvements: vec!["Improvement 1".to_string()],
            insights: vec!["Insight 1".to_string(), "Insight 2".to_string()],
            generated_at: Utc::now(),
        });
        let score2 = assessor.assess_episode(&episode2);

        assert!(score2 > score1, "Reflection should improve quality score");
    }

    #[test]
    fn test_should_store_threshold() {
        let assessor = QualityAssessor::new(QualityConfig::new(0.7));
        let mut episode = create_test_episode();

        // Add steps to get moderate quality
        for i in 0..5 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), format!("action_{i}"));
            step.result = Some(ExecutionResult::Success {
                output: "Success".to_string(),
            });
            episode.add_step(step);
        }

        episode.reflection = Some(Reflection {
            successes: vec!["Good".to_string(); 3],
            improvements: vec!["Better".to_string()],
            insights: vec!["Key".to_string(); 2],
            generated_at: Utc::now(),
        });

        let should_store = assessor.should_store(&episode);
        let score = assessor.assess_episode(&episode);

        assert_eq!(
            should_store,
            score >= 0.7,
            "should_store should match threshold comparison"
        );
    }

    #[test]
    fn test_task_complexity_scoring() {
        let assessor = QualityAssessor::new(QualityConfig::default());
        let mut episode = create_test_episode();

        // Low complexity: few steps, single tool
        for i in 0..2 {
            let step = ExecutionStep::new(i + 1, "tool".to_string(), "action".to_string());
            episode.add_step(step);
        }
        let low_complexity = assessor.assess_task_complexity(&episode);

        // High complexity: many steps, diverse tools
        let mut episode2 = create_test_episode();
        for i in 0..15 {
            let step = ExecutionStep::new(i + 1, format!("tool_{i}"), format!("action_{i}"));
            episode2.add_step(step);
        }
        let high_complexity = assessor.assess_task_complexity(&episode2);

        assert!(
            high_complexity > low_complexity,
            "More complex tasks should score higher"
        );
    }

    #[test]
    fn test_step_diversity_scoring() {
        let assessor = QualityAssessor::new(QualityConfig::default());

        // Low diversity: repetitive actions
        let mut episode1 = create_test_episode();
        for i in 0..5 {
            let step = ExecutionStep::new(i + 1, "tool".to_string(), "same_action".to_string());
            episode1.add_step(step);
        }
        let low_diversity = assessor.assess_step_diversity(&episode1);

        // High diversity: varied actions
        let mut episode2 = create_test_episode();
        for i in 0..5 {
            let step = ExecutionStep::new(i + 1, "tool".to_string(), format!("action_{i}"));
            episode2.add_step(step);
        }
        let high_diversity = assessor.assess_step_diversity(&episode2);

        assert!(
            high_diversity > low_diversity,
            "More diverse steps should score higher"
        );
    }
}

//! Quality assessor implementation.
//!
//! Contains the `QualityAssessor` struct and its assessment methods.

use super::types::{QualityConfig, QualityFeature};
use crate::episode::Episode;
use std::collections::HashSet;

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

        let score = task_complexity * self.config.get_weight(QualityFeature::TaskComplexity)
            + step_diversity * self.config.get_weight(QualityFeature::StepDiversity)
            + error_rate * self.config.get_weight(QualityFeature::ErrorRate)
            + reflection_depth * self.config.get_weight(QualityFeature::ReflectionDepth)
            + pattern_novelty * self.config.get_weight(QualityFeature::PatternNovelty);

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
    fn assess_task_complexity(&self, episode: &Episode) -> f32 {
        let step_count = episode.steps.len();
        let unique_tools: HashSet<_> = episode.steps.iter().map(|s| &s.tool).collect();
        let tool_diversity = unique_tools.len();

        let step_score = match step_count {
            0..=2 => 0.1,
            3..=5 => 0.25,
            6..=10 => 0.35,
            11..=20 => 0.45,
            _ => 0.5,
        };

        let tool_score = match tool_diversity {
            0..=1 => 0.1,
            2..=3 => 0.25,
            4..=6 => 0.35,
            7..=10 => 0.45,
            _ => 0.5,
        };

        step_score + tool_score
    }

    /// Assess step diversity based on variety of actions and results.
    fn assess_step_diversity(&self, episode: &Episode) -> f32 {
        if episode.steps.is_empty() {
            return 0.0;
        }

        let unique_actions: HashSet<_> = episode.steps.iter().map(|s| &s.action).collect();
        let action_diversity = unique_actions.len() as f32 / episode.steps.len() as f32;

        let success_count = episode.successful_steps_count();
        let error_count = episode.failed_steps_count();
        let result_diversity = if success_count > 0 && error_count > 0 {
            0.5
        } else if success_count > 0 || error_count > 0 {
            0.3
        } else {
            0.0
        };

        (action_diversity * 0.6 + result_diversity * 0.4).clamp(0.0, 1.0)
    }

    /// Assess error handling quality based on error rate and recovery.
    fn assess_error_rate(&self, episode: &Episode) -> f32 {
        let total_steps = episode.steps.len();
        if total_steps == 0 {
            return 0.5;
        }

        let error_count = episode.failed_steps_count();
        let success_count = episode.successful_steps_count();
        let error_rate = error_count as f32 / total_steps as f32;

        if error_rate == 0.0 {
            0.9
        } else if error_rate < 0.2 && success_count > error_count {
            1.0
        } else if error_rate < 0.4 {
            0.6
        } else if error_rate < 0.6 && success_count > 0 {
            0.4
        } else {
            0.2
        }
    }

    /// Assess reflection quality based on depth and insight content.
    fn assess_reflection_depth(&self, episode: &Episode) -> f32 {
        let Some(reflection) = &episode.reflection else {
            return 0.0;
        };

        let total_items =
            reflection.successes.len() + reflection.improvements.len() + reflection.insights.len();

        match total_items {
            0 => 0.0,
            1..=2 => 0.3,
            3..=5 => 0.6,
            6..=10 => 0.8,
            _ => 1.0,
        }
    }

    /// Assess pattern novelty based on number of patterns extracted.
    fn assess_pattern_novelty(&self, episode: &Episode) -> f32 {
        let total_patterns = episode.patterns.len() + episode.heuristics.len();

        match total_patterns {
            0 => 0.2,
            1..=2 => 0.5,
            3..=5 => 0.75,
            _ => 1.0,
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

        for i in 0..15 {
            let mut step =
                ExecutionStep::new(i + 1, format!("tool_{}", i % 5), format!("action_{i}"));
            step.result = Some(ExecutionResult::Success {
                output: "Success".to_string(),
            });
            episode.add_step(step);
        }

        episode.reflection = Some(Reflection {
            successes: vec!["Good work".to_string(); 3],
            improvements: vec!["Could improve".to_string(); 2],
            insights: vec!["Key insight".to_string(); 2],
            generated_at: Utc::now(),
        });

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

        let mut episode1 = create_test_episode();
        for i in 0..5 {
            let mut step = ExecutionStep::new(i + 1, "tool".to_string(), "action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode1.add_step(step);
        }
        let score1 = assessor.assess_episode(&episode1);

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

        for i in 0..2 {
            let step = ExecutionStep::new(i + 1, "tool".to_string(), "action".to_string());
            episode.add_step(step);
        }
        let low_complexity = assessor.assess_task_complexity(&episode);

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

        let mut episode1 = create_test_episode();
        for i in 0..5 {
            let step = ExecutionStep::new(i + 1, "tool".to_string(), "same_action".to_string());
            episode1.add_step(step);
        }
        let low_diversity = assessor.assess_step_diversity(&episode1);

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

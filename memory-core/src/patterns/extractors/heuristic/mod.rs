//! Heuristic rule extractor
//!
//! Extracts condition→action heuristic rules from successful decision points in episodes.
//! Heuristics represent learned guidelines that can improve future task execution.

mod extraction;

pub use extraction::{extract_action, extract_condition, is_decision_action};

use crate::episode::Episode;
use crate::pattern::Heuristic;
use crate::types::{Evidence, TaskOutcome};
use anyhow::{Context as AnyhowContext, Result};
use std::collections::HashMap;
use uuid::Uuid;

/// Configuration for heuristic extraction
#[derive(Debug, Clone)]
pub struct HeuristicExtractorConfig {
    /// Minimum confidence score to keep a heuristic (0.0 to 1.0)
    pub min_confidence: f32,
    /// Minimum number of occurrences needed to extract a heuristic
    pub min_sample_size: usize,
}

impl Default for HeuristicExtractorConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
            min_sample_size: 2,
        }
    }
}

/// Extracts heuristic rules from episodes
///
/// Analyzes decision points in successful episodes to learn condition→action
/// heuristics that can guide future task execution.
///
/// # Algorithm
///
/// 1. Identify decision points in episode steps (by keywords)
/// 2. For successful episodes, extract the decision context and resulting action
/// 3. Group similar condition→action pairs
/// 4. Calculate confidence = `success_rate` × √`sample_size`
/// 5. Filter by minimum confidence threshold
///
/// # Examples
///
/// ```no_run
/// use memory_core::patterns::extractors::heuristic::HeuristicExtractor;
/// use memory_core::Episode;
///
/// let extractor = HeuristicExtractor::new();
/// // let heuristics = extractor.extract(&episode).await?;
/// ```
#[derive(Clone)]
pub struct HeuristicExtractor {
    config: HeuristicExtractorConfig,
}

impl Default for HeuristicExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl HeuristicExtractor {
    /// Create a new heuristic extractor with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: HeuristicExtractorConfig::default(),
        }
    }

    /// Create with custom configuration
    #[must_use]
    pub fn with_config(config: HeuristicExtractorConfig) -> Self {
        Self { config }
    }

    /// Create with custom thresholds
    #[must_use]
    pub fn with_thresholds(min_confidence: f32, min_sample_size: usize) -> Self {
        Self {
            config: HeuristicExtractorConfig {
                min_confidence,
                min_sample_size,
            },
        }
    }

    /// Extract heuristics from an episode
    ///
    /// Analyzes the episode steps to find decision points and extracts
    /// condition→action rules that can be applied to future tasks.
    ///
    /// # Arguments
    ///
    /// * `episode` - The episode to analyze
    ///
    /// # Returns
    ///
    /// A vector of extracted heuristics that meet the confidence threshold
    ///
    /// # Errors
    ///
    /// Returns error if confidence calculation fails or data is invalid
    #[allow(clippy::unused_async)]
    pub async fn extract(&self, episode: &Episode) -> Result<Vec<Heuristic>> {
        // Only extract from complete episodes
        if !episode.is_complete() {
            return Ok(Vec::new());
        }

        // Only extract from successful episodes
        let is_successful = matches!(
            episode.outcome,
            Some(TaskOutcome::Success { .. } | TaskOutcome::PartialSuccess { .. })
        );

        if !is_successful {
            return Ok(Vec::new());
        }

        // Calculate success rate based on outcome
        let success_rate = match &episode.outcome {
            Some(TaskOutcome::Success { .. }) => 1.0,
            Some(TaskOutcome::PartialSuccess { .. }) => 0.5,
            _ => 0.0,
        };

        // Group decision points by condition→action pairs
        let mut decision_map: HashMap<(String, String), Vec<Uuid>> = HashMap::new();

        // Find decision points in steps
        for (idx, step) in episode.steps.iter().enumerate() {
            if !is_decision_action(&step.action) || !step.is_success() {
                continue;
            }

            // Extract condition (the decision context)
            let condition = extract_condition(episode, step, idx)?;

            // Extract action (what was done after the decision)
            let action = extract_action(episode, step, idx)?;

            // Group by condition→action pair
            decision_map
                .entry((condition, action))
                .or_default()
                .push(episode.episode_id);
        }

        // Generate heuristics from grouped decisions
        let mut heuristics = Vec::new();

        for ((condition, action), episode_ids) in decision_map {
            let sample_size = episode_ids.len();

            // Skip if below minimum sample size
            if sample_size < self.config.min_sample_size {
                continue;
            }

            // Calculate confidence = success_rate × √sample_size
            // Note: confidence can exceed 1.0 for high sample sizes
            let confidence = success_rate * (sample_size as f32).sqrt();

            // Skip if below minimum confidence
            if confidence < self.config.min_confidence {
                continue;
            }

            // Validate confidence is non-negative and finite
            if !confidence.is_finite() || confidence < 0.0 {
                return Err(anyhow::anyhow!(
                    "Invalid confidence score: {confidence}. Must be finite and non-negative"
                ))
                .context("Failed to calculate heuristic confidence");
            }

            // Create heuristic with evidence
            let mut heuristic = Heuristic::new(condition, action, confidence);
            heuristic.evidence = Evidence {
                episode_ids,
                success_rate,
                sample_size,
            };

            heuristics.push(heuristic);
        }

        Ok(heuristics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::{Episode, ExecutionStep};
    use crate::types::{ComplexityLevel, ExecutionResult, TaskContext, TaskType};

    fn create_test_episode() -> Episode {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity: ComplexityLevel::Moderate,
            domain: "testing".to_string(),
            tags: vec!["async".to_string()],
        };

        Episode::new("Test task".to_string(), context, TaskType::Testing)
    }

    #[tokio::test]
    async fn test_extract_from_complete_successful_episode() {
        // Use low threshold to ensure extraction happens
        let extractor = HeuristicExtractor::with_thresholds(0.5, 2);
        let mut episode = create_test_episode();

        // Add identical decision points so they group together
        for i in 0..3 {
            let mut step = ExecutionStep::new(
                i * 2 + 1,
                "validator".to_string(),
                "Check if input is valid".to_string(), // Same text for all
            );
            step.result = Some(ExecutionResult::Success {
                output: "Valid".to_string(),
            });
            episode.add_step(step);

            // Add follow-up action
            let mut action_step = ExecutionStep::new(
                i * 2 + 2,
                "processor".to_string(),
                "Process the data".to_string(),
            );
            action_step.result = Some(ExecutionResult::Success {
                output: "Processed".to_string(),
            });
            episode.add_step(action_step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "All validations passed".to_string(),
            artifacts: vec![],
        });

        let heuristics = extractor.extract(&episode).await.unwrap();

        eprintln!("DEBUG: Extracted {} heuristics", heuristics.len());
        for (i, h) in heuristics.iter().enumerate() {
            eprintln!(
                "  Heuristic {}: condition='{}', action='{}', confidence={}, sample_size={}",
                i, h.condition, h.action, h.confidence, h.evidence.sample_size
            );
        }

        // Should extract heuristics (3 identical decisions should group)
        assert!(
            !heuristics.is_empty(),
            "Should extract heuristics. Check debug output above."
        );

        // Check heuristic properties
        for heuristic in &heuristics {
            assert!(heuristic.confidence >= extractor.config.min_confidence);
            assert!(heuristic.evidence.sample_size >= extractor.config.min_sample_size);
            assert!(!heuristic.condition.is_empty());
            assert!(!heuristic.action.is_empty());
        }
    }

    #[tokio::test]
    async fn test_no_extraction_from_incomplete_episode() {
        let extractor = HeuristicExtractor::new();
        let mut episode = create_test_episode();

        let mut step = ExecutionStep::new(
            1,
            "validator".to_string(),
            "Check if input is valid".to_string(),
        );
        step.result = Some(ExecutionResult::Success {
            output: "Valid".to_string(),
        });
        episode.add_step(step);

        // Don't complete the episode

        let heuristics = extractor.extract(&episode).await.unwrap();
        assert!(
            heuristics.is_empty(),
            "Should not extract from incomplete episode"
        );
    }

    #[tokio::test]
    async fn test_no_extraction_from_failed_episode() {
        let extractor = HeuristicExtractor::new();
        let mut episode = create_test_episode();

        let mut step = ExecutionStep::new(
            1,
            "validator".to_string(),
            "Check if input is valid".to_string(),
        );
        step.result = Some(ExecutionResult::Success {
            output: "Valid".to_string(),
        });
        episode.add_step(step);

        episode.complete(TaskOutcome::Failure {
            reason: "Validation failed".to_string(),
            error_details: None,
        });

        let heuristics = extractor.extract(&episode).await.unwrap();
        assert!(
            heuristics.is_empty(),
            "Should not extract from failed episode"
        );
    }

    #[tokio::test]
    async fn test_confidence_calculation() {
        let extractor = HeuristicExtractor::with_thresholds(0.0, 1);
        let mut episode = create_test_episode();

        // Add 4 identical decision points so they group together
        for i in 0..4 {
            let mut step = ExecutionStep::new(
                i * 2 + 1,
                "validator".to_string(),
                "Verify input validity".to_string(), // Same text for all
            );
            step.result = Some(ExecutionResult::Success {
                output: "Valid".to_string(),
            });
            episode.add_step(step);

            // Add follow-up action (same for all)
            let mut action = ExecutionStep::new(
                i * 2 + 2,
                "processor".to_string(),
                "Process data".to_string(),
            );
            action.result = Some(ExecutionResult::Success {
                output: "Done".to_string(),
            });
            episode.add_step(action);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Success".to_string(),
            artifacts: vec![],
        });

        let heuristics = extractor.extract(&episode).await.unwrap();

        assert!(
            !heuristics.is_empty(),
            "Should extract at least one heuristic"
        );

        // For a success with 4 samples: confidence = 1.0 × √4 = 2.0
        // Actually looking at the code, confidence can exceed 1.0
        let heuristic = &heuristics[0];
        let expected_confidence = 1.0 * (4.0_f32).sqrt(); // = 2.0
        assert!(
            (heuristic.confidence - expected_confidence).abs() < 0.1,
            "Confidence should be ~{}, got {}",
            expected_confidence,
            heuristic.confidence
        );
    }

    #[tokio::test]
    async fn test_filtering_by_confidence_threshold() {
        // Set high confidence threshold
        let extractor = HeuristicExtractor::with_thresholds(2.0, 1);
        let mut episode = create_test_episode();

        // Add only 1 decision point (confidence = 1.0 × √1 = 1.0, below threshold of 2.0)
        let mut step = ExecutionStep::new(1, "validator".to_string(), "Check validity".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "Valid".to_string(),
        });
        episode.add_step(step);

        let mut action = ExecutionStep::new(2, "processor".to_string(), "Process".to_string());
        action.result = Some(ExecutionResult::Success {
            output: "Done".to_string(),
        });
        episode.add_step(action);

        episode.complete(TaskOutcome::Success {
            verdict: "Success".to_string(),
            artifacts: vec![],
        });

        let heuristics = extractor.extract(&episode).await.unwrap();

        // Should be filtered out due to low confidence
        assert!(
            heuristics.is_empty(),
            "Should filter out low confidence heuristics"
        );
    }

    #[tokio::test]
    async fn test_filtering_by_sample_size() {
        // Set minimum sample size to 3
        let extractor = HeuristicExtractor::with_thresholds(0.0, 3);
        let mut episode = create_test_episode();

        // Add only 2 similar decision points (below min sample size of 3)
        for i in 0..2 {
            let mut step =
                ExecutionStep::new(i + 1, "validator".to_string(), "Check validity".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "Valid".to_string(),
            });
            episode.add_step(step);

            let mut action =
                ExecutionStep::new(i + 3, "processor".to_string(), "Process".to_string());
            action.result = Some(ExecutionResult::Success {
                output: "Done".to_string(),
            });
            episode.add_step(action);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Success".to_string(),
            artifacts: vec![],
        });

        let heuristics = extractor.extract(&episode).await.unwrap();

        // Should be filtered out due to small sample size
        assert!(
            heuristics.is_empty(),
            "Should filter out heuristics with insufficient samples"
        );
    }

    #[tokio::test]
    async fn test_decision_action_detection() {
        assert!(is_decision_action("Check if valid"));
        assert!(is_decision_action("Verify the input"));
        assert!(is_decision_action("Validate parameters"));
        assert!(is_decision_action("When ready"));
        assert!(is_decision_action("Ensure safety"));
        assert!(is_decision_action("Decide on approach"));
        assert!(is_decision_action("Determine the path"));

        assert!(!is_decision_action("Read file"));
        assert!(!is_decision_action("Write data"));
        assert!(!is_decision_action("Process input"));
    }

    #[tokio::test]
    async fn test_debug_extraction() {
        let extractor = HeuristicExtractor::with_thresholds(0.0, 1);
        let mut episode = create_test_episode();

        // Add a simple decision point
        let mut step = ExecutionStep::new(
            1,
            "validator".to_string(),
            "Check if input is valid".to_string(),
        );
        step.result = Some(ExecutionResult::Success {
            output: "Valid".to_string(),
        });
        episode.add_step(step);

        // Add follow-up action
        let mut action = ExecutionStep::new(2, "processor".to_string(), "Process data".to_string());
        action.result = Some(ExecutionResult::Success {
            output: "Done".to_string(),
        });
        episode.add_step(action);

        episode.complete(TaskOutcome::Success {
            verdict: "Success".to_string(),
            artifacts: vec![],
        });

        let heuristics = extractor.extract(&episode).await.unwrap();

        eprintln!("Extracted {} heuristics", heuristics.len());
        for (i, h) in heuristics.iter().enumerate() {
            eprintln!(
                "Heuristic {}: condition='{}', action='{}', confidence={}, sample_size={}",
                i, h.condition, h.action, h.confidence, h.evidence.sample_size
            );
        }

        // With min_sample_size=1, should extract at least one
        assert!(
            !heuristics.is_empty(),
            "Should extract heuristic with min_sample_size=1"
        );
    }

    #[tokio::test]
    async fn test_partial_success_lower_confidence() {
        let extractor = HeuristicExtractor::with_thresholds(0.0, 1);
        let mut episode = create_test_episode();

        // Add 4 similar decision points
        for i in 0..4 {
            let mut step =
                ExecutionStep::new(i + 1, "validator".to_string(), "Check input".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "Valid".to_string(),
            });
            episode.add_step(step);

            let mut action =
                ExecutionStep::new(i + 5, "processor".to_string(), "Process".to_string());
            action.result = Some(ExecutionResult::Success {
                output: "Done".to_string(),
            });
            episode.add_step(action);
        }

        episode.complete(TaskOutcome::PartialSuccess {
            verdict: "Partial success".to_string(),
            completed: vec!["some".to_string()],
            failed: vec!["others".to_string()],
        });

        let heuristics = extractor.extract(&episode).await.unwrap();

        assert!(!heuristics.is_empty());

        // For partial success with 4 samples: confidence = 0.5 × √4 = 1.0
        let heuristic = &heuristics[0];
        let expected_confidence = 0.5 * (4.0_f32).sqrt();
        assert!(
            (heuristic.confidence - expected_confidence).abs() < 0.1,
            "Confidence should be ~{}, got {}",
            expected_confidence,
            heuristic.confidence
        );
    }
}

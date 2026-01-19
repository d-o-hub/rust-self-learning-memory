//! Heuristic extractor implementation

use crate::episode::Episode;
use crate::pattern::Heuristic;
use crate::types::{Evidence, TaskOutcome};
use anyhow::{Context as AnyhowContext, Result};
use std::collections::HashMap;
use uuid::Uuid;

use super::config::HeuristicExtractorConfig;
use super::extraction::{extract_action, extract_condition, is_decision_action};

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
    /// Configuration for the extractor
    pub config: HeuristicExtractorConfig,
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

//! Decision point pattern extractor
//!
//! Extracts conditional branching patterns where decisions affect outcomes.

use super::PatternExtractor as PatternExtractorTrait;
use crate::episode::Episode;
use crate::pattern::Pattern;
use crate::types::OutcomeStats;
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

/// Extracts decision point patterns from episodes
pub struct DecisionPointExtractor {
    confidence_threshold: f32,
}

impl Default for DecisionPointExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl DecisionPointExtractor {
    /// Create new decision point extractor
    pub fn new() -> Self {
        Self {
            confidence_threshold: 0.6,
        }
    }

    /// Create with custom confidence threshold
    pub fn with_threshold(threshold: f32) -> Self {
        Self {
            confidence_threshold: threshold,
        }
    }

    /// Check if action indicates a decision point
    fn is_decision_action(action: &str) -> bool {
        let action_lower = action.to_lowercase();
        action_lower.contains("if ")
            || action_lower.contains("when ")
            || action_lower.contains("check ")
            || action_lower.contains("verify ")
            || action_lower.contains("validate ")
            || action_lower.contains("ensure ")
            || action_lower.starts_with("decide ")
            || action_lower.starts_with("determine ")
    }
}

#[async_trait]
impl PatternExtractorTrait for DecisionPointExtractor {
    async fn extract(&self, episode: &Episode) -> Result<Vec<Pattern>> {
        let mut patterns = Vec::new();

        // Only extract from complete episodes
        if !episode.is_complete() {
            return Ok(patterns);
        }

        // Look for decision points in steps
        for step in &episode.steps {
            if Self::is_decision_action(&step.action) {
                let outcome_stats = OutcomeStats {
                    success_count: if step.is_success() { 1 } else { 0 },
                    failure_count: if step.is_success() { 0 } else { 1 },
                    total_count: 1,
                    avg_duration_secs: step.latency_ms as f32 / 1000.0,
                };

                patterns.push(Pattern::DecisionPoint {
                    id: Uuid::new_v4(),
                    condition: step.action.clone(),
                    action: step.tool.clone(),
                    outcome_stats,
                    context: episode.context.clone(),
                });
            }
        }

        Ok(patterns)
    }

    fn name(&self) -> &str {
        "DecisionPointExtractor"
    }

    fn confidence_threshold(&self) -> f32 {
        self.confidence_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::ExecutionStep;
    use crate::patterns::extractors::tests::{complete_episode_successfully, create_test_episode};
    use crate::types::ExecutionResult;

    #[tokio::test]
    async fn test_extract_decision_point() {
        let extractor = DecisionPointExtractor::new();
        let mut episode = create_test_episode();

        // Add a decision point step
        let mut step = ExecutionStep::new(
            1,
            "validator".to_string(),
            "Check if input is valid".to_string(),
        );
        step.result = Some(ExecutionResult::Success {
            output: "Valid".to_string(),
        });
        step.latency_ms = 50;
        episode.add_step(step);

        complete_episode_successfully(&mut episode);

        let patterns = extractor.extract(&episode).await.unwrap();

        assert_eq!(patterns.len(), 1);
        if let Pattern::DecisionPoint {
            condition, action, ..
        } = &patterns[0]
        {
            assert!(condition.contains("Check"));
            assert_eq!(action, "validator");
        } else {
            panic!("Expected DecisionPoint pattern");
        }
    }

    #[tokio::test]
    async fn test_multiple_decision_points() {
        let extractor = DecisionPointExtractor::new();
        let mut episode = create_test_episode();

        // Add multiple decision steps
        let decision_keywords = ["Check if", "Verify that", "Validate"];
        for (i, keyword) in decision_keywords.iter().enumerate() {
            let mut step = ExecutionStep::new(
                i + 1,
                format!("tool_{}", i),
                format!("{} something", keyword),
            );
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        complete_episode_successfully(&mut episode);

        let patterns = extractor.extract(&episode).await.unwrap();
        assert_eq!(patterns.len(), 3);
    }

    #[tokio::test]
    async fn test_no_decision_points() {
        let extractor = DecisionPointExtractor::new();
        let mut episode = create_test_episode();

        // Add regular steps without decision keywords
        let mut step = ExecutionStep::new(1, "reader".to_string(), "Read file".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);

        complete_episode_successfully(&mut episode);

        let patterns = extractor.extract(&episode).await.unwrap();
        assert!(patterns.is_empty());
    }
}

//! Context pattern extractor
//!
//! Extracts patterns based on task context and successful approaches.

use super::PatternExtractor as PatternExtractorTrait;
use crate::episode::Episode;
use crate::pattern::Pattern;
use crate::types::TaskOutcome;
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

/// Extracts context-based patterns from episodes
pub struct ContextPatternExtractor {
    confidence_threshold: f32,
}

impl Default for ContextPatternExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextPatternExtractor {
    /// Create new context pattern extractor
    #[must_use]
    pub fn new() -> Self {
        Self {
            confidence_threshold: 0.7,
        }
    }

    /// Create with custom confidence threshold
    #[must_use]
    pub fn with_threshold(threshold: f32) -> Self {
        Self {
            confidence_threshold: threshold,
        }
    }

    /// Calculate success rate from outcome
    fn calculate_success_rate(outcome: &Option<TaskOutcome>) -> f32 {
        match outcome {
            Some(TaskOutcome::Success { .. }) => 1.0,
            Some(TaskOutcome::PartialSuccess {
                completed, failed, ..
            }) => {
                let total = completed.len() + failed.len();
                if total > 0 {
                    completed.len() as f32 / total as f32
                } else {
                    0.5
                }
            }
            _ => 0.0,
        }
    }

    /// Build context features from episode
    fn build_context_features(episode: &Episode) -> Vec<String> {
        let mut features = Vec::new();

        if let Some(lang) = &episode.context.language {
            features.push(format!("language:{lang}"));
        }

        if let Some(framework) = &episode.context.framework {
            features.push(format!("framework:{framework}"));
        }

        features.push(format!("domain:{}", episode.context.domain));
        features.push(format!("complexity:{:?}", episode.context.complexity));

        for tag in &episode.context.tags {
            features.push(format!("tag:{tag}"));
        }

        features
    }

    /// Build recommended approach from successful steps
    fn build_recommended_approach(episode: &Episode) -> String {
        let successful_tools: Vec<&str> = episode
            .steps
            .iter()
            .filter(|s| s.is_success())
            .map(|s| s.tool.as_str())
            .collect();

        if successful_tools.is_empty() {
            "No clear approach identified".to_string()
        } else {
            format!("Use tools: {}", successful_tools.join(", "))
        }
    }
}

#[async_trait]
impl PatternExtractorTrait for ContextPatternExtractor {
    async fn extract(&self, episode: &Episode) -> Result<Vec<Pattern>> {
        let mut patterns = Vec::new();

        // Only extract from complete episodes
        if !episode.is_complete() {
            return Ok(patterns);
        }

        let success_rate = Self::calculate_success_rate(&episode.outcome);

        // Only extract if above threshold
        if success_rate < self.confidence_threshold {
            return Ok(patterns);
        }

        let context_features = Self::build_context_features(episode);
        let recommended_approach = Self::build_recommended_approach(episode);

        patterns.push(Pattern::ContextPattern {
            id: Uuid::new_v4(),
            context_features,
            recommended_approach,
            evidence: vec![episode.episode_id],
            success_rate,
        });

        Ok(patterns)
    }

    fn name(&self) -> &'static str {
        "ContextPatternExtractor"
    }

    fn confidence_threshold(&self) -> f32 {
        self.confidence_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::extractors::tests::{
        add_successful_steps, complete_episode_successfully, create_test_episode,
    };

    #[tokio::test]
    async fn test_extract_context_pattern() {
        let extractor = ContextPatternExtractor::new();
        let mut episode = create_test_episode();

        add_successful_steps(&mut episode, 3);
        complete_episode_successfully(&mut episode);

        let patterns = extractor.extract(&episode).await.unwrap();

        assert_eq!(patterns.len(), 1);
        if let Pattern::ContextPattern {
            context_features,
            success_rate,
            recommended_approach,
            ..
        } = &patterns[0]
        {
            assert!(context_features.iter().any(|f| f.contains("rust")));
            assert!(context_features.iter().any(|f| f.contains("testing")));
            assert_eq!(*success_rate, 1.0);
            assert!(recommended_approach.contains("tool_"));
        } else {
            panic!("Expected ContextPattern");
        }
    }

    #[tokio::test]
    async fn test_no_pattern_on_failure() {
        let extractor = ContextPatternExtractor::new();
        let mut episode = create_test_episode();

        add_successful_steps(&mut episode, 2);

        episode.complete(TaskOutcome::Failure {
            reason: "Failed".to_string(),
            error_details: None,
        });

        let patterns = extractor.extract(&episode).await.unwrap();
        assert!(patterns.is_empty());
    }

    #[tokio::test]
    async fn test_context_features_extraction() {
        let extractor = ContextPatternExtractor::new();
        let mut episode = create_test_episode();

        add_successful_steps(&mut episode, 2);
        complete_episode_successfully(&mut episode);

        let patterns = extractor.extract(&episode).await.unwrap();

        if let Pattern::ContextPattern {
            context_features, ..
        } = &patterns[0]
        {
            // Check for expected features
            assert!(context_features.iter().any(|f| f.starts_with("language:")));
            assert!(context_features.iter().any(|f| f.starts_with("framework:")));
            assert!(context_features.iter().any(|f| f.starts_with("domain:")));
            assert!(context_features
                .iter()
                .any(|f| f.starts_with("complexity:")));
            assert!(context_features.iter().any(|f| f.starts_with("tag:")));
        }
    }
}

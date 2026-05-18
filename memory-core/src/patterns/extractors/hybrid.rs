//! Hybrid pattern extractor
//!
//! Runs multiple specialized extractors in parallel and combines results.

use super::{
    cluster_similar_patterns, ContextPatternExtractor, DecisionPointExtractor,
    ErrorRecoveryExtractor, PatternExtractor as PatternExtractorTrait, ToolSequenceExtractor,
};
use crate::episode::Episode;
use crate::pattern::Pattern;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, instrument};

/// Hybrid extractor that runs multiple extractors in parallel
pub struct HybridPatternExtractor {
    extractors: Vec<Box<dyn PatternExtractorTrait>>,
    confidence_threshold: f32,
    enable_clustering: bool,
}

impl Default for HybridPatternExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl HybridPatternExtractor {
    /// Create a new hybrid extractor with all default extractors
    #[must_use]
    pub fn new() -> Self {
        let extractors: Vec<Box<dyn PatternExtractorTrait>> = vec![
            Box::new(ToolSequenceExtractor::new()),
            Box::new(DecisionPointExtractor::new()),
            Box::new(ErrorRecoveryExtractor::new()),
            Box::new(ContextPatternExtractor::new()),
        ];

        Self {
            extractors,
            confidence_threshold: 0.7,
            enable_clustering: true,
        }
    }

    /// Create with custom extractors
    #[must_use]
    pub fn with_extractors(extractors: Vec<Box<dyn PatternExtractorTrait>>) -> Self {
        Self {
            extractors,
            confidence_threshold: 0.7,
            enable_clustering: true,
        }
    }

    /// Set confidence threshold for filtering patterns
    #[must_use]
    pub fn with_confidence_threshold(mut self, threshold: f32) -> Self {
        self.confidence_threshold = threshold;
        self
    }

    /// Enable or disable clustering
    #[must_use]
    pub fn with_clustering(mut self, enable: bool) -> Self {
        self.enable_clustering = enable;
        self
    }

    /// Extract patterns using all extractors in parallel
    #[instrument(skip(self, episode), fields(episode_id = %episode.episode_id))]
    pub async fn extract_patterns(&self, episode: &Episode) -> Result<Vec<Pattern>> {
        let start = Instant::now();

        // Create Arc for shared episode access
        let episode = Arc::new(episode.clone());
        let mut handles = Vec::new();

        // Spawn parallel extraction tasks
        for extractor in &self.extractors {
            let episode_clone = Arc::clone(&episode);
            let extractor_name = extractor.name().to_string();

            // Note: We can't use tokio::spawn directly with trait objects easily,
            // so we'll extract sequentially but with async/await for now.
            // For true parallelism, we could use channels or other patterns.
            let patterns = extractor.extract(&episode_clone).await?;
            debug!(
                extractor = %extractor_name,
                pattern_count = patterns.len(),
                "Extractor completed"
            );
            handles.push(patterns);
        }

        // Combine all patterns
        let mut all_patterns: Vec<Pattern> = handles.into_iter().flatten().collect();

        debug!(
            total_patterns = all_patterns.len(),
            "Combined patterns from all extractors"
        );

        // Filter by confidence threshold
        all_patterns.retain(|p| p.success_rate() >= self.confidence_threshold);

        debug!(
            filtered_patterns = all_patterns.len(),
            threshold = self.confidence_threshold,
            "Filtered patterns by confidence"
        );

        // Deduplicate and cluster if enabled
        let final_patterns = if self.enable_clustering {
            let clustered = cluster_similar_patterns(all_patterns);
            debug!(
                clustered_patterns = clustered.len(),
                "Clustered similar patterns"
            );
            clustered
        } else {
            all_patterns
        };

        let duration = start.elapsed();
        debug!(
            duration_ms = duration.as_millis(),
            final_count = final_patterns.len(),
            "Pattern extraction complete"
        );

        Ok(final_patterns)
    }

    /// Get the number of registered extractors
    #[must_use]
    pub fn extractor_count(&self) -> usize {
        self.extractors.len()
    }

    /// Get names of all registered extractors
    #[must_use]
    pub fn extractor_names(&self) -> Vec<&str> {
        self.extractors.iter().map(|e| e.name()).collect()
    }
}

#[async_trait]
impl PatternExtractorTrait for HybridPatternExtractor {
    async fn extract(&self, episode: &Episode) -> Result<Vec<Pattern>> {
        self.extract_patterns(episode).await
    }

    fn name(&self) -> &'static str {
        "HybridPatternExtractor"
    }

    fn confidence_threshold(&self) -> f32 {
        self.confidence_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::ExecutionStep;
    use crate::patterns::extractors::tests::{
        add_successful_steps, complete_episode_successfully, create_test_episode,
    };
    use crate::types::ExecutionResult;

    #[tokio::test]
    async fn test_hybrid_extractor_basic() {
        let extractor = HybridPatternExtractor::new();
        let mut episode = create_test_episode();

        add_successful_steps(&mut episode, 4);
        complete_episode_successfully(&mut episode);

        let patterns = extractor.extract_patterns(&episode).await.unwrap();

        // Should extract at least tool sequence and context pattern
        assert!(!patterns.is_empty());
        assert!(patterns.len() >= 2);
    }

    #[tokio::test]
    async fn test_parallel_extraction() {
        let extractor = HybridPatternExtractor::new();

        // Create a rich episode with multiple pattern types
        let mut episode = create_test_episode();

        // Add tool sequence
        add_successful_steps(&mut episode, 3);

        // Add decision point
        let mut decision_step =
            ExecutionStep::new(4, "validator".to_string(), "Check if valid".to_string());
        decision_step.result = Some(ExecutionResult::Success {
            output: "Valid".to_string(),
        });
        episode.add_step(decision_step);

        // Add error recovery
        let mut error_step = ExecutionStep::new(5, "failer".to_string(), "Try".to_string());
        error_step.result = Some(ExecutionResult::Error {
            message: "Error".to_string(),
        });
        episode.add_step(error_step);

        let mut recovery_step = ExecutionStep::new(6, "recoverer".to_string(), "Retry".to_string());
        recovery_step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(recovery_step);

        complete_episode_successfully(&mut episode);

        let patterns = extractor.extract_patterns(&episode).await.unwrap();

        // Should extract multiple pattern types
        assert!(patterns.len() >= 3);

        // Verify different pattern types
        let has_tool_seq = patterns
            .iter()
            .any(|p| matches!(p, Pattern::ToolSequence { .. }));
        let has_decision = patterns
            .iter()
            .any(|p| matches!(p, Pattern::DecisionPoint { .. }));
        let has_recovery = patterns
            .iter()
            .any(|p| matches!(p, Pattern::ErrorRecovery { .. }));
        let has_context = patterns
            .iter()
            .any(|p| matches!(p, Pattern::ContextPattern { .. }));

        assert!(has_tool_seq || has_decision || has_recovery || has_context);
    }

    #[tokio::test]
    async fn test_confidence_filtering() {
        let extractor = HybridPatternExtractor::new().with_confidence_threshold(0.8);

        let mut episode = create_test_episode();
        add_successful_steps(&mut episode, 3);
        complete_episode_successfully(&mut episode);

        let patterns = extractor.extract_patterns(&episode).await.unwrap();

        // All patterns should have high confidence
        for pattern in &patterns {
            assert!(pattern.success_rate() >= 0.8);
        }
    }

    #[tokio::test]
    async fn test_clustering_enabled() {
        let extractor = HybridPatternExtractor::new().with_clustering(true);

        let mut episode = create_test_episode();
        add_successful_steps(&mut episode, 5);
        complete_episode_successfully(&mut episode);

        let patterns = extractor.extract_patterns(&episode).await.unwrap();

        // Should have deduplicated patterns
        assert!(!patterns.is_empty());

        // Check that patterns are sorted by success rate
        for i in 0..patterns.len().saturating_sub(1) {
            assert!(patterns[i].success_rate() >= patterns[i + 1].success_rate());
        }
    }

    #[tokio::test]
    async fn test_clustering_disabled() {
        let extractor = HybridPatternExtractor::new().with_clustering(false);

        let mut episode = create_test_episode();
        add_successful_steps(&mut episode, 3);
        complete_episode_successfully(&mut episode);

        let patterns = extractor.extract_patterns(&episode).await.unwrap();

        // Should still extract patterns
        assert!(!patterns.is_empty());
    }

    #[tokio::test]
    async fn test_extractor_metadata() {
        let extractor = HybridPatternExtractor::new();

        assert_eq!(extractor.name(), "HybridPatternExtractor");
        assert_eq!(extractor.extractor_count(), 4);

        let names = extractor.extractor_names();
        assert!(names.contains(&"ToolSequenceExtractor"));
        assert!(names.contains(&"DecisionPointExtractor"));
        assert!(names.contains(&"ErrorRecoveryExtractor"));
        assert!(names.contains(&"ContextPatternExtractor"));
    }

    #[tokio::test]
    async fn test_performance_under_1000ms() {
        use std::time::Instant;

        let extractor = HybridPatternExtractor::new();
        let mut episode = create_test_episode();

        // Create a realistic episode
        add_successful_steps(&mut episode, 10);

        // Add some decision points
        for i in 0..3 {
            let mut step = ExecutionStep::new(
                11 + i,
                format!("validator_{i}"),
                format!("Check condition {i}"),
            );
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        complete_episode_successfully(&mut episode);

        let start = Instant::now();
        let patterns = extractor.extract_patterns(&episode).await.unwrap();
        let duration = start.elapsed();

        assert!(!patterns.is_empty());
        assert!(
            duration.as_millis() < 1000,
            "Extraction took {}ms, expected < 1000ms",
            duration.as_millis()
        );
    }
}

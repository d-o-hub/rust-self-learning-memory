//! Tool sequence pattern extractor
//!
//! Extracts successful sequences of tools used together.

use super::PatternExtractor as PatternExtractorTrait;
use crate::episode::Episode;
use crate::pattern::Pattern;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Duration;
use uuid::Uuid;

/// Extracts tool sequence patterns from successful episodes
pub struct ToolSequenceExtractor {
    min_sequence_len: usize,
    max_sequence_len: usize,
    success_threshold: f32,
}

impl Default for ToolSequenceExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolSequenceExtractor {
    /// Create new tool sequence extractor with default settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            min_sequence_len: 2,
            max_sequence_len: 5,
            success_threshold: 0.7,
        }
    }

    /// Create with custom thresholds
    #[must_use]
    pub fn with_thresholds(min_len: usize, max_len: usize, threshold: f32) -> Self {
        Self {
            min_sequence_len: min_len,
            max_sequence_len: max_len,
            success_threshold: threshold,
        }
    }

    /// Calculate step success rate
    fn calculate_step_success_rate(&self, episode: &Episode) -> f32 {
        if episode.steps.is_empty() {
            return 0.0;
        }
        episode.successful_steps_count() as f32 / episode.steps.len() as f32
    }

    /// Calculate average latency
    fn calculate_average_latency(&self, episode: &Episode) -> Duration {
        if episode.steps.is_empty() {
            return Duration::zero();
        }

        let total_ms: u64 = episode.steps.iter().map(|s| s.latency_ms).sum();
        let avg_ms = total_ms / episode.steps.len() as u64;
        Duration::milliseconds(avg_ms as i64)
    }
}

#[async_trait]
impl PatternExtractorTrait for ToolSequenceExtractor {
    async fn extract(&self, episode: &Episode) -> Result<Vec<Pattern>> {
        let mut patterns = Vec::new();

        // Only extract from complete episodes
        if !episode.is_complete() {
            return Ok(patterns);
        }

        // Check minimum length
        if episode.steps.len() < self.min_sequence_len {
            return Ok(patterns);
        }

        // Check success rate threshold
        let success_rate = self.calculate_step_success_rate(episode);
        if success_rate < self.success_threshold {
            return Ok(patterns);
        }

        // Extract tool sequence (up to max length)
        let tools: Vec<String> = episode
            .steps
            .iter()
            .take(self.max_sequence_len)
            .map(|s| s.tool.clone())
            .collect();

        if tools.len() >= self.min_sequence_len {
            let avg_latency = self.calculate_average_latency(episode);

            patterns.push(Pattern::ToolSequence {
                id: Uuid::new_v4(),
                tools,
                context: episode.context.clone(),
                success_rate,
                avg_latency,
                occurrence_count: 1,
            });
        }

        Ok(patterns)
    }

    fn name(&self) -> &'static str {
        "ToolSequenceExtractor"
    }

    fn confidence_threshold(&self) -> f32 {
        self.success_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::extractors::tests::{
        add_successful_steps, complete_episode_successfully, create_test_episode,
    };

    #[tokio::test]
    async fn test_extract_tool_sequence() {
        let extractor = ToolSequenceExtractor::new();
        let mut episode = create_test_episode();

        add_successful_steps(&mut episode, 4);
        complete_episode_successfully(&mut episode);

        let patterns = extractor.extract(&episode).await.unwrap();

        assert_eq!(patterns.len(), 1);
        if let Pattern::ToolSequence {
            tools,
            success_rate,
            ..
        } = &patterns[0]
        {
            assert_eq!(tools.len(), 4);
            assert_eq!(*success_rate, 1.0);
        } else {
            panic!("Expected ToolSequence pattern");
        }
    }

    #[tokio::test]
    async fn test_no_pattern_below_threshold() {
        let extractor = ToolSequenceExtractor::new();
        let mut episode = create_test_episode();

        // Only 1 step - below minimum
        add_successful_steps(&mut episode, 1);
        complete_episode_successfully(&mut episode);

        let patterns = extractor.extract(&episode).await.unwrap();
        assert!(patterns.is_empty());
    }

    #[tokio::test]
    async fn test_custom_thresholds() {
        let extractor = ToolSequenceExtractor::with_thresholds(3, 6, 0.8);
        let mut episode = create_test_episode();

        add_successful_steps(&mut episode, 5);
        complete_episode_successfully(&mut episode);

        let patterns = extractor.extract(&episode).await.unwrap();

        assert_eq!(patterns.len(), 1);
        if let Pattern::ToolSequence { tools, .. } = &patterns[0] {
            assert_eq!(tools.len(), 5);
        }
    }
}

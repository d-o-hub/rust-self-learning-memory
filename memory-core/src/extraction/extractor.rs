//! Core pattern extractor implementation

use crate::episode::Episode;
use crate::pattern::Pattern;
use chrono::Duration;
use tracing::{debug, instrument};

use super::extractors::{
    extract_context_pattern, extract_decision_points, extract_error_recovery, extract_tool_sequence,
};
use super::{MAX_SEQUENCE_LENGTH, MIN_PATTERN_SUCCESS_RATE, MIN_SEQUENCE_LENGTH};

/// Pattern extractor
#[derive(Clone)]
pub struct PatternExtractor {
    /// Minimum success rate threshold
    pub(crate) success_threshold: f32,
    /// Minimum sequence length
    #[allow(dead_code)]
    pub(crate) min_sequence_len: usize,
    /// Maximum sequence length
    pub(crate) max_sequence_len: usize,
}

impl Default for PatternExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternExtractor {
    /// Create a new pattern extractor with default thresholds
    #[must_use]
    pub fn new() -> Self {
        Self {
            success_threshold: MIN_PATTERN_SUCCESS_RATE,
            min_sequence_len: MIN_SEQUENCE_LENGTH,
            max_sequence_len: MAX_SEQUENCE_LENGTH,
        }
    }

    /// Create an extractor with custom thresholds
    #[must_use]
    pub fn with_thresholds(
        success_threshold: f32,
        min_sequence_len: usize,
        max_sequence_len: usize,
    ) -> Self {
        Self {
            success_threshold,
            min_sequence_len,
            max_sequence_len,
        }
    }

    /// Extract patterns from a completed episode
    #[instrument(skip(self, episode), fields(episode_id = %episode.episode_id))]
    pub fn extract(&self, episode: &Episode) -> Vec<Pattern> {
        let mut patterns = Vec::new();

        // Only extract from completed episodes
        if !episode.is_complete() {
            return patterns;
        }

        // Extract tool sequence patterns
        if let Some(tool_seq) = extract_tool_sequence(self, episode) {
            patterns.push(tool_seq);
        }

        // Extract decision point patterns
        patterns.extend(extract_decision_points(self, episode));

        // Extract error recovery patterns
        if let Some(error_recovery) = extract_error_recovery(self, episode) {
            patterns.push(error_recovery);
        }

        // Extract context patterns
        if let Some(context_pattern) = extract_context_pattern(self, episode) {
            patterns.push(context_pattern);
        }

        debug!(
            pattern_count = patterns.len(),
            "Extracted patterns from episode"
        );

        patterns
    }

    /// Calculate step success rate
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn calculate_step_success_rate(episode: &Episode) -> f32 {
        if episode.steps.is_empty() {
            return 0.0;
        }

        let successful = episode.successful_steps_count();
        successful as f32 / episode.steps.len() as f32
    }

    /// Calculate average latency
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]
    pub(crate) fn calculate_average_latency(episode: &Episode) -> Duration {
        if episode.steps.is_empty() {
            return Duration::zero();
        }

        let total_ms: u64 = episode.steps.iter().map(|s| s.latency_ms).sum();
        let avg_ms = total_ms / episode.steps.len() as u64;

        Duration::milliseconds(avg_ms as i64)
    }
}

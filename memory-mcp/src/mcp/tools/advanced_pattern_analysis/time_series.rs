//! # Advanced Pattern Analysis Time Series
//!
//! Time series extraction from memory episodes for analysis.

use memory_core::{ComplexityLevel, TaskOutcome};

use super::tool::AdvancedPatternAnalysisTool;

const EXECUTION_TIME_MS: &str = "execution_time_ms";
const SUCCESS_RATE: &str = "success_rate";
const COMPLEXITY_SCORE: &str = "complexity_score";
const PATTERN_MATCH_SCORE: &str = "pattern_match_score";
const MEMORY_USAGE_MB: &str = "memory_usage_mb";

const ALL_METRICS: [&str; 5] = [
    EXECUTION_TIME_MS,
    SUCCESS_RATE,
    COMPLEXITY_SCORE,
    PATTERN_MATCH_SCORE,
    MEMORY_USAGE_MB,
];

/// Extractor for time series data from memory episodes
pub struct TimeSeriesExtractor;

impl TimeSeriesExtractor {
    /// Create a new time series extractor
    pub fn new() -> Self {
        Self
    }

    /// Extract metrics from a single episode
    pub fn extract_metric(
        &self,
        metric: &str,
        episode: &memory_core::Episode,
        all_episodes: &[memory_core::Episode],
    ) -> Option<f64> {
        match metric {
            EXECUTION_TIME_MS => {
                // Try to extract from execution steps
                let total_time: u64 = episode.steps.iter().map(|step| step.latency_ms).sum();
                Some(total_time as f64)
            }
            SUCCESS_RATE => {
                // Calculate success rate from outcomes
                let success_count = all_episodes
                    .iter()
                    .filter(|e| matches!(e.outcome, Some(TaskOutcome::Success { .. })))
                    .count();
                let rate = success_count as f64 / all_episodes.len() as f64;
                Some(rate * 100.0) // Convert to percentage
            }
            COMPLEXITY_SCORE => {
                // Use complexity level as numeric score
                let score = match episode.context.complexity {
                    ComplexityLevel::Simple => 1.0,
                    ComplexityLevel::Moderate => 2.0,
                    ComplexityLevel::Complex => 3.0,
                };
                Some(score)
            }
            PATTERN_MATCH_SCORE => {
                // Simplified pattern matching score
                Some(0.8) // Placeholder
            }
            MEMORY_USAGE_MB => {
                // Estimate memory usage
                Some(50.0) // Placeholder
            }
            _ => None,
        }
    }

    /// Check if values meet minimum threshold for inclusion
    pub fn meets_threshold(&self, values: &[f64], min_points: usize) -> bool {
        !values.is_empty() && values.len() >= min_points
    }
}

impl Default for TimeSeriesExtractor {
    fn default() -> Self {
        Self::new()
    }
}

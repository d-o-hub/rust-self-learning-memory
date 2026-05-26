//! # Advanced Pattern Analysis Time Series
//!
//! Time series extraction from memory episodes for analysis.

use do_memory_core::{ComplexityLevel, TaskOutcome};

const _EXECUTION_TIME_MS: &str = "execution_time_ms";
const _SUCCESS_RATE: &str = "success_rate";
const _COMPLEXITY_SCORE: &str = "complexity_score";
const _PATTERN_MATCH_SCORE: &str = "pattern_match_score";
const _MEMORY_USAGE_MB: &str = "memory_usage_mb";

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
        episode: &do_memory_core::Episode,
        all_episodes: &[do_memory_core::Episode],
    ) -> Option<f64> {
        match metric {
            _EXECUTION_TIME_MS => {
                // Try to extract from execution steps
                let total_time: u64 = episode.steps.iter().map(|step| step.latency_ms).sum();
                Some(total_time as f64)
            }
            _SUCCESS_RATE => {
                // Calculate success rate from outcomes
                let success_count = all_episodes
                    .iter()
                    .filter(|e| matches!(e.outcome, Some(TaskOutcome::Success { .. })))
                    .count();
                let rate = success_count as f64 / all_episodes.len() as f64;
                Some(rate * 100.0) // Convert to percentage
            }
            _COMPLEXITY_SCORE => {
                // Use complexity level as numeric score
                let score = match episode.context.complexity {
                    ComplexityLevel::Simple => 1.0,
                    ComplexityLevel::Moderate => 2.0,
                    ComplexityLevel::Complex => 3.0,
                };
                Some(score)
            }
            _PATTERN_MATCH_SCORE => {
                // Compute from Episode's applied patterns or patterns list
                if !episode.applied_patterns.is_empty() {
                    let success = episode
                        .applied_patterns
                        .iter()
                        .filter(|p| p.outcome.is_success())
                        .count();
                    Some(success as f64 / episode.applied_patterns.len() as f64)
                } else if !all_episodes.is_empty() {
                    // Fallback to pattern density relative to other episodes
                    let max_patterns = all_episodes
                        .iter()
                        .map(|e| e.patterns.len())
                        .max()
                        .unwrap_or(0);
                    if max_patterns > 0 {
                        Some(episode.patterns.len() as f64 / max_patterns as f64)
                    } else {
                        Some(0.0)
                    }
                } else {
                    Some(0.0)
                }
            }
            _MEMORY_USAGE_MB => {
                // Estimate memory usage of the current process using sysinfo
                let mut system = sysinfo::System::new();
                if let Ok(pid) = sysinfo::get_current_pid() {
                    system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), false);
                    if let Some(process) = system.process(pid) {
                        Some(process.memory() as f64 / 1024.0 / 1024.0)
                    } else {
                        Some(50.0)
                    }
                } else {
                    Some(50.0)
                }
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

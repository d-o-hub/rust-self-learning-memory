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

#[cfg(test)]
mod tests {
    use super::*;
    use do_memory_core::{Episode, ExecutionStep, TaskContext, TaskType, TaskOutcome, ExecutionResult, ComplexityLevel};
    use do_memory_core::episode::structs::{PatternApplication, ApplicationOutcome};
    use uuid::Uuid;

    fn create_mock_episode(latency_ms: u64, success: bool, complexity: ComplexityLevel) -> Episode {
        let mut episode = Episode::new(
            "test task".to_string(),
            TaskContext {
                domain: "test".to_string(),
                complexity,
                ..Default::default()
            },
            TaskType::CodeGeneration,
        );

        let mut step = ExecutionStep::new(1, "test".to_string(), "test step".to_string());
        step.latency_ms = latency_ms;
        step.result = Some(ExecutionResult::Success { output: "ok".to_string() });
        episode.steps.push(step);

        if success {
            episode.outcome = Some(TaskOutcome::Success {
                verdict: "success".to_string(),
                artifacts: vec![],
            });
        } else {
            episode.outcome = Some(TaskOutcome::Failure {
                reason: "failed".to_string(),
                error_details: None,
            });
        }

        episode
    }

    #[test]
    fn test_extract_metric_execution_time() {
        let extractor = TimeSeriesExtractor::new();
        let episode = create_mock_episode(150, true, ComplexityLevel::Simple);
        let all = vec![episode.clone()];

        let value = extractor.extract_metric("execution_time_ms", &episode, &all);
        assert_eq!(value, Some(150.0));
    }

    #[test]
    fn test_extract_metric_success_rate() {
        let extractor = TimeSeriesExtractor::new();
        let ep1 = create_mock_episode(100, true, ComplexityLevel::Simple);
        let ep2 = create_mock_episode(100, false, ComplexityLevel::Simple);
        let all = vec![ep1.clone(), ep2.clone()];

        let value = extractor.extract_metric("success_rate", &ep1, &all);
        assert_eq!(value, Some(50.0));
    }

    #[test]
    fn test_extract_metric_complexity() {
        let extractor = TimeSeriesExtractor::new();

        let ep1 = create_mock_episode(100, true, ComplexityLevel::Simple);
        assert_eq!(extractor.extract_metric("complexity_score", &ep1, &[]), Some(1.0));

        let ep2 = create_mock_episode(100, true, ComplexityLevel::Moderate);
        assert_eq!(extractor.extract_metric("complexity_score", &ep2, &[]), Some(2.0));

        let ep3 = create_mock_episode(100, true, ComplexityLevel::Complex);
        assert_eq!(extractor.extract_metric("complexity_score", &ep3, &[]), Some(3.0));
    }

    #[test]
    fn test_extract_metric_pattern_match() {
        let extractor = TimeSeriesExtractor::new();
        let mut ep = create_mock_episode(100, true, ComplexityLevel::Simple);

        // No patterns
        assert_eq!(extractor.extract_metric("pattern_match_score", &ep, &[]), Some(0.0));

        // With applied patterns
        ep.applied_patterns.push(PatternApplication {
            pattern_id: Uuid::new_v4(),
            applied_at_step: 1,
            outcome: ApplicationOutcome::Helped,
            notes: None,
        });
        assert_eq!(extractor.extract_metric("pattern_match_score", &ep, &[]), Some(1.0));

        // Fallback to pattern density
        let mut ep_no_applied = create_mock_episode(100, true, ComplexityLevel::Simple);
        ep_no_applied.patterns.push(Uuid::new_v4());
        let mut ep_max = create_mock_episode(100, true, ComplexityLevel::Simple);
        ep_max.patterns.push(Uuid::new_v4());
        ep_max.patterns.push(Uuid::new_v4());
        let all = vec![ep_no_applied.clone(), ep_max.clone()];
        assert_eq!(extractor.extract_metric("pattern_match_score", &ep_no_applied, &all), Some(0.5));
    }

    #[test]
    fn test_extract_metric_memory_usage() {
        let extractor = TimeSeriesExtractor::new();
        let ep = create_mock_episode(100, true, ComplexityLevel::Simple);

        let value = extractor.extract_metric("memory_usage_mb", &ep, &[]);
        assert!(value.is_some());
        assert!(value.unwrap() > 0.0);
    }

    #[test]
    fn test_extract_metric_none() {
        let extractor = TimeSeriesExtractor::new();
        let ep = create_mock_episode(100, true, ComplexityLevel::Simple);

        assert_eq!(extractor.extract_metric("invalid_metric", &ep, &[]), None);
    }

    #[test]
    fn test_meets_threshold() {
        let extractor = TimeSeriesExtractor::new();

        assert!(!extractor.meets_threshold(&[], 3));
        assert!(!extractor.meets_threshold(&[1.0, 2.0], 3));
        assert!(extractor.meets_threshold(&[1.0, 2.0, 3.0], 3));
    }
}

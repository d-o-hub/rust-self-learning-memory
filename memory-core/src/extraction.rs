//! # Pattern Extractor
//!
//! Extracts reusable patterns from completed episodes:
//! - Tool sequences that worked well
//! - Decision points with outcomes
//! - Error recovery strategies
//! - Context-based patterns
//!
//! ## Example
//!
//! ```
//! use memory_core::extraction::PatternExtractor;
//! use memory_core::{Episode, TaskContext, TaskType, TaskOutcome, ExecutionStep};
//!
//! let context = TaskContext::default();
//! let mut episode = Episode::new("Test task".to_string(), context, TaskType::Testing);
//!
//! // Add some execution steps
//! for i in 0..3 {
//!     let step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
//!     episode.add_step(step);
//! }
//!
//! episode.complete(TaskOutcome::Success {
//!     verdict: "Done".to_string(),
//!     artifacts: vec![],
//! });
//!
//! let extractor = PatternExtractor::new();
//! let patterns = extractor.extract(&episode);
//!
//! // Patterns may be extracted based on episode content
//! ```

use crate::episode::Episode;
use crate::pattern::Pattern;
use crate::types::{ExecutionResult, OutcomeStats, TaskOutcome};
use chrono::Duration;
use std::collections::HashMap;
use tracing::{debug, instrument};
use uuid::Uuid;

/// Minimum success rate to extract a pattern
const MIN_PATTERN_SUCCESS_RATE: f32 = 0.7;

/// Minimum sequence length for tool sequence patterns
const MIN_SEQUENCE_LENGTH: usize = 2;

/// Maximum sequence length for tool sequence patterns
const MAX_SEQUENCE_LENGTH: usize = 5;

/// Pattern extractor
#[derive(Clone)]
pub struct PatternExtractor {
    /// Minimum success rate threshold
    success_threshold: f32,
    /// Minimum sequence length
    min_sequence_len: usize,
    /// Maximum sequence length
    max_sequence_len: usize,
}

impl Default for PatternExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternExtractor {
    /// Create a new pattern extractor with default thresholds
    pub fn new() -> Self {
        Self {
            success_threshold: MIN_PATTERN_SUCCESS_RATE,
            min_sequence_len: MIN_SEQUENCE_LENGTH,
            max_sequence_len: MAX_SEQUENCE_LENGTH,
        }
    }

    /// Create an extractor with custom thresholds
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
        if let Some(tool_seq) = self.extract_tool_sequence(episode) {
            patterns.push(tool_seq);
        }

        // Extract decision point patterns
        patterns.extend(self.extract_decision_points(episode));

        // Extract error recovery patterns
        if let Some(error_recovery) = self.extract_error_recovery(episode) {
            patterns.push(error_recovery);
        }

        // Extract context patterns
        if let Some(context_pattern) = self.extract_context_pattern(episode) {
            patterns.push(context_pattern);
        }

        debug!(
            pattern_count = patterns.len(),
            "Extracted patterns from episode"
        );

        patterns
    }

    /// Extract tool sequence pattern
    fn extract_tool_sequence(&self, episode: &Episode) -> Option<Pattern> {
        if episode.steps.len() < self.min_sequence_len {
            return None;
        }

        // Only extract from successful episodes
        let success_rate = self.calculate_step_success_rate(episode);
        if success_rate < self.success_threshold {
            return None;
        }

        // Extract the tool sequence (up to max length)
        let tools: Vec<String> = episode
            .steps
            .iter()
            .take(self.max_sequence_len)
            .map(|s| s.tool.clone())
            .collect();

        if tools.len() < self.min_sequence_len {
            return None;
        }

        // Calculate average latency
        let avg_latency = self.calculate_average_latency(episode);

        Some(Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools,
            context: episode.context.clone(),
            success_rate,
            avg_latency,
            occurrence_count: 1, // Initial occurrence
        })
    }

    /// Extract decision point patterns
    fn extract_decision_points(&self, episode: &Episode) -> Vec<Pattern> {
        let mut patterns = Vec::new();

        // Look for conditional branching in steps (identified by metadata or action patterns)
        for step in &episode.steps {
            // Check if this step represents a decision point
            // (e.g., contains "if", "when", "check", etc. in action)
            let action_lower = step.action.to_lowercase();
            if action_lower.contains("if ")
                || action_lower.contains("when ")
                || action_lower.contains("check ")
                || action_lower.contains("verify ")
            {
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

        patterns
    }

    /// Extract error recovery pattern
    fn extract_error_recovery(&self, episode: &Episode) -> Option<Pattern> {
        let mut recovery_sequences = Vec::new();

        // Look for error -> success patterns
        for i in 0..episode.steps.len().saturating_sub(1) {
            let current = &episode.steps[i];
            let next = &episode.steps[i + 1];

            if !current.is_success() && next.is_success() {
                // Found a recovery pattern
                let error_type = if let Some(ExecutionResult::Error { message }) = &current.result {
                    message.clone()
                } else {
                    "Unknown error".to_string()
                };

                let recovery_step = format!("{}: {}", next.tool, next.action);
                recovery_sequences.push((error_type, recovery_step));
            }
        }

        if recovery_sequences.is_empty() {
            return None;
        }

        // Group by error type and find most common recovery
        let mut error_recoveries: HashMap<String, Vec<String>> = HashMap::new();
        for (error_type, recovery) in recovery_sequences {
            error_recoveries
                .entry(error_type)
                .or_default()
                .push(recovery);
        }

        // Take the most common error type
        if let Some((error_type, recovery_steps)) = error_recoveries.into_iter().next() {
            // Calculate success rate (for error recovery, if episode succeeded, recovery worked)
            let success_rate = match &episode.outcome {
                Some(TaskOutcome::Success { .. }) => 1.0,
                Some(TaskOutcome::PartialSuccess { .. }) => 0.5,
                _ => 0.0,
            };

            return Some(Pattern::ErrorRecovery {
                id: Uuid::new_v4(),
                error_type,
                recovery_steps,
                success_rate,
                context: episode.context.clone(),
            });
        }

        None
    }

    /// Extract context-based pattern
    fn extract_context_pattern(&self, episode: &Episode) -> Option<Pattern> {
        // Only extract from successful episodes
        let success_rate = match &episode.outcome {
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
        };

        if success_rate < self.success_threshold {
            return None;
        }

        // Build context features
        let mut context_features = Vec::new();

        if let Some(lang) = &episode.context.language {
            context_features.push(format!("language:{}", lang));
        }

        if let Some(framework) = &episode.context.framework {
            context_features.push(format!("framework:{}", framework));
        }

        context_features.push(format!("domain:{}", episode.context.domain));
        context_features.push(format!("complexity:{:?}", episode.context.complexity));

        for tag in &episode.context.tags {
            context_features.push(format!("tag:{}", tag));
        }

        // Build recommended approach from successful steps
        let successful_tools: Vec<&str> = episode
            .steps
            .iter()
            .filter(|s| s.is_success())
            .map(|s| s.tool.as_str())
            .collect();

        let recommended_approach = if successful_tools.is_empty() {
            "No clear approach identified".to_string()
        } else {
            format!("Use tools: {}", successful_tools.join(", "))
        };

        Some(Pattern::ContextPattern {
            id: Uuid::new_v4(),
            context_features,
            recommended_approach,
            evidence: vec![episode.episode_id],
            success_rate,
        })
    }

    /// Calculate step success rate
    fn calculate_step_success_rate(&self, episode: &Episode) -> f32 {
        if episode.steps.is_empty() {
            return 0.0;
        }

        let successful = episode.successful_steps_count();
        successful as f32 / episode.steps.len() as f32
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

/// Deduplicate and rank patterns by relevance
pub fn deduplicate_patterns(patterns: Vec<Pattern>) -> Vec<Pattern> {
    let mut unique_patterns = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for pattern in patterns {
        let id = pattern.id();
        if !seen_ids.contains(&id) {
            seen_ids.insert(id);
            unique_patterns.push(pattern);
        }
    }

    // Sort by success rate (descending)
    unique_patterns.sort_by(|a, b| {
        b.success_rate()
            .partial_cmp(&a.success_rate())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    unique_patterns
}

/// Rank patterns by relevance to a context
pub fn rank_patterns(patterns: Vec<Pattern>, context: &crate::types::TaskContext) -> Vec<Pattern> {
    let mut ranked = patterns;

    // Sort by relevance and success rate
    ranked.sort_by(|a, b| {
        let a_relevant = a.is_relevant_to(context);
        let b_relevant = b.is_relevant_to(context);

        // First by relevance
        match (a_relevant, b_relevant) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                // Then by success rate
                b.success_rate()
                    .partial_cmp(&a.success_rate())
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        }
    });

    ranked
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::ExecutionStep;
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

    #[test]
    fn test_extract_tool_sequence() {
        let extractor = PatternExtractor::new();
        let mut episode = create_test_episode();

        // Add successful steps
        for i in 0..4 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            step.latency_ms = 100;
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let patterns = extractor.extract(&episode);

        // Should extract tool sequence
        assert!(patterns
            .iter()
            .any(|p| matches!(p, Pattern::ToolSequence { .. })));

        if let Some(Pattern::ToolSequence {
            tools,
            success_rate,
            ..
        }) = patterns
            .iter()
            .find(|p| matches!(p, Pattern::ToolSequence { .. }))
        {
            assert_eq!(tools.len(), 4);
            assert_eq!(*success_rate, 1.0);
        }
    }

    #[test]
    fn test_no_pattern_from_failed_episode() {
        let extractor = PatternExtractor::new();
        let mut episode = create_test_episode();

        // Add failed steps
        for i in 0..3 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
            step.result = Some(ExecutionResult::Error {
                message: "Error".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Failure {
            reason: "Failed".to_string(),
            error_details: None,
        });

        let patterns = extractor.extract(&episode);

        // Should not extract tool sequence from failed episode
        assert!(!patterns
            .iter()
            .any(|p| matches!(p, Pattern::ToolSequence { .. })));
    }

    #[test]
    fn test_extract_decision_point() {
        let extractor = PatternExtractor::new();
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

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let patterns = extractor.extract(&episode);

        // Should extract decision point
        assert!(patterns
            .iter()
            .any(|p| matches!(p, Pattern::DecisionPoint { .. })));
    }

    #[test]
    fn test_extract_error_recovery() {
        let extractor = PatternExtractor::new();
        let mut episode = create_test_episode();

        // Add error step
        let mut error_step =
            ExecutionStep::new(1, "failing_tool".to_string(), "Try action".to_string());
        error_step.result = Some(ExecutionResult::Error {
            message: "Connection timeout".to_string(),
        });
        episode.add_step(error_step);

        // Add recovery step
        let mut recovery_step = ExecutionStep::new(
            2,
            "retry_tool".to_string(),
            "Retry with backoff".to_string(),
        );
        recovery_step.result = Some(ExecutionResult::Success {
            output: "Success".to_string(),
        });
        episode.add_step(recovery_step);

        episode.complete(TaskOutcome::Success {
            verdict: "Recovered".to_string(),
            artifacts: vec![],
        });

        let patterns = extractor.extract(&episode);

        // Should extract error recovery
        assert!(patterns
            .iter()
            .any(|p| matches!(p, Pattern::ErrorRecovery { .. })));

        if let Some(Pattern::ErrorRecovery {
            error_type,
            recovery_steps,
            success_rate,
            ..
        }) = patterns
            .iter()
            .find(|p| matches!(p, Pattern::ErrorRecovery { .. }))
        {
            assert!(error_type.contains("Connection timeout"));
            assert!(!recovery_steps.is_empty());
            assert_eq!(*success_rate, 1.0);
        }
    }

    #[test]
    fn test_extract_context_pattern() {
        let extractor = PatternExtractor::new();
        let mut episode = create_test_episode();

        // Add successful steps
        for i in 0..3 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let patterns = extractor.extract(&episode);

        // Should extract context pattern
        assert!(patterns
            .iter()
            .any(|p| matches!(p, Pattern::ContextPattern { .. })));

        if let Some(Pattern::ContextPattern {
            context_features,
            success_rate,
            ..
        }) = patterns
            .iter()
            .find(|p| matches!(p, Pattern::ContextPattern { .. }))
        {
            assert!(context_features.iter().any(|f| f.contains("rust")));
            assert!(context_features.iter().any(|f| f.contains("testing")));
            assert_eq!(*success_rate, 1.0);
        }
    }

    #[test]
    fn test_custom_thresholds() {
        // Lower threshold to accept less successful episodes
        let extractor = PatternExtractor::with_thresholds(0.5, 2, 5);
        let mut episode = create_test_episode();

        // Add mix of successful and failed steps (60% success)
        for i in 0..5 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
            step.result = if i < 3 {
                Some(ExecutionResult::Success {
                    output: "OK".to_string(),
                })
            } else {
                Some(ExecutionResult::Error {
                    message: "Error".to_string(),
                })
            };
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::PartialSuccess {
            verdict: "Partial".to_string(),
            completed: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            failed: vec!["d".to_string(), "e".to_string()],
        });

        let patterns = extractor.extract(&episode);

        // Should still extract context pattern with lower threshold
        assert!(patterns
            .iter()
            .any(|p| matches!(p, Pattern::ContextPattern { .. })));
    }

    #[test]
    fn test_deduplicate_patterns() {
        let pattern1 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool1".to_string()],
            context: TaskContext::default(),
            success_rate: 0.9,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 1,
        };

        let pattern2 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool2".to_string()],
            context: TaskContext::default(),
            success_rate: 0.8,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 1,
        };

        let patterns = vec![pattern1.clone(), pattern2.clone(), pattern1.clone()];

        let deduped = deduplicate_patterns(patterns);

        // Should only have 2 unique patterns
        assert_eq!(deduped.len(), 2);

        // Should be sorted by success rate (descending)
        assert_eq!(deduped[0].success_rate(), 0.9);
        assert_eq!(deduped[1].success_rate(), 0.8);
    }

    #[test]
    fn test_rank_patterns_by_context() {
        let relevant_context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain: "web-api".to_string(),
            tags: vec![],
        };

        let pattern1 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool1".to_string()],
            context: relevant_context.clone(),
            success_rate: 0.8,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 1,
        };

        let pattern2 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool2".to_string()],
            context: TaskContext {
                domain: "data-processing".to_string(),
                ..Default::default()
            },
            success_rate: 0.9, // Higher success rate but not relevant
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 1,
        };

        let patterns = vec![pattern2, pattern1];

        let query_context = TaskContext {
            domain: "web-api".to_string(),
            ..Default::default()
        };

        let ranked = rank_patterns(patterns, &query_context);

        // Relevant pattern should be first, even with lower success rate
        assert_eq!(ranked[0].context().unwrap().domain, "web-api");
    }
}

//! # Reflection Generator
//!
//! Generates structured reflections from completed episodes by analyzing:
//! - Successful strategies and patterns
//! - Areas for improvement
//! - Key insights and learnings
//!
//! ## Example
//!
//! ```
//! use memory_core::reflection::ReflectionGenerator;
//! use memory_core::{Episode, TaskContext, TaskType, TaskOutcome, ExecutionStep};
//!
//! let context = TaskContext::default();
//! let mut episode = Episode::new("Test task".to_string(), context, TaskType::Testing);
//!
//! let mut step = ExecutionStep::new(1, "test_runner".to_string(), "Run tests".to_string());
//! episode.add_step(step);
//!
//! episode.complete(TaskOutcome::Success {
//!     verdict: "All tests passed".to_string(),
//!     artifacts: vec!["test_results.json".to_string()],
//! });
//!
//! let generator = ReflectionGenerator::new();
//! let reflection = generator.generate(&episode);
//!
//! assert!(!reflection.successes.is_empty() || !reflection.insights.is_empty());
//! ```

use crate::episode::Episode;
use crate::types::{ExecutionResult, Reflection, TaskOutcome};
use chrono::Utc;
use tracing::{debug, instrument};

/// Minimum step count to generate detailed reflections
const MIN_STEPS_FOR_REFLECTION: usize = 2;

/// Maximum items in each reflection category
const MAX_REFLECTION_ITEMS: usize = 5;

/// Generator for episode reflections
#[derive(Clone)]
pub struct ReflectionGenerator {
    /// Maximum items per category
    max_items: usize,
}

impl Default for ReflectionGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ReflectionGenerator {
    /// Create a new reflection generator
    pub fn new() -> Self {
        Self {
            max_items: MAX_REFLECTION_ITEMS,
        }
    }

    /// Create a generator with custom max items
    pub fn with_max_items(max_items: usize) -> Self {
        Self { max_items }
    }

    /// Generate reflection from a completed episode
    #[instrument(skip(self, episode), fields(episode_id = %episode.episode_id))]
    pub fn generate(&self, episode: &Episode) -> Reflection {
        let successes = self.identify_successes(episode);
        let improvements = self.identify_improvements(episode);
        let insights = self.generate_insights(episode);

        debug!(
            successes_count = successes.len(),
            improvements_count = improvements.len(),
            insights_count = insights.len(),
            "Generated reflection"
        );

        Reflection {
            successes,
            improvements,
            insights,
            generated_at: Utc::now(),
        }
    }

    /// Identify what worked well in the episode
    fn identify_successes(&self, episode: &Episode) -> Vec<String> {
        let mut successes = Vec::new();

        // Check overall outcome
        match &episode.outcome {
            Some(TaskOutcome::Success { verdict, artifacts }) => {
                successes.push(format!("Successfully completed task: {}", verdict));

                if !artifacts.is_empty() {
                    successes.push(format!("Generated {} artifact(s)", artifacts.len()));
                }
            }
            Some(TaskOutcome::PartialSuccess {
                verdict, completed, ..
            }) => {
                successes.push(format!("Partial success: {}", verdict));
                if !completed.is_empty() {
                    successes.push(format!("Completed {} subtask(s)", completed.len()));
                }
            }
            _ => {}
        }

        // Identify successful execution patterns
        let successful_steps = episode.successful_steps_count();
        let total_steps = episode.steps.len();

        if successful_steps > 0 && total_steps > 0 {
            let success_rate = successful_steps as f32 / total_steps as f32;
            if success_rate >= 0.8 {
                successes.push(format!(
                    "High execution success rate: {:.1}% ({}/{})",
                    success_rate * 100.0,
                    successful_steps,
                    total_steps
                ));
            }
        }

        // Identify efficient tool usage
        if let Some(tool_pattern) = self.identify_effective_tool_sequence(episode) {
            successes.push(tool_pattern);
        }

        // Identify quick completion
        if let Some(duration) = episode.duration() {
            let duration_secs = duration.num_seconds();
            if duration_secs < 30 && total_steps > 0 {
                successes.push(format!(
                    "Efficient execution: completed in {} seconds",
                    duration_secs
                ));
            }
        }

        // Limit to max items
        successes.truncate(self.max_items);
        successes
    }

    /// Identify areas for improvement
    fn identify_improvements(&self, episode: &Episode) -> Vec<String> {
        let mut improvements = Vec::new();

        // Check for failures
        match &episode.outcome {
            Some(TaskOutcome::Failure { reason, .. }) => {
                improvements.push(format!("Task failed: {}", reason));
            }
            Some(TaskOutcome::PartialSuccess { failed, .. }) if !failed.is_empty() => {
                improvements.push(format!("Failed {} subtask(s)", failed.len()));
            }
            _ => {}
        }

        // Check for failed steps
        let failed_steps = episode.failed_steps_count();
        if failed_steps > 0 {
            improvements.push(format!(
                "Reduce failed execution steps (current: {})",
                failed_steps
            ));

            // Identify specific tools that failed
            if let Some(problematic_tool) = self.identify_problematic_tool(episode) {
                improvements.push(problematic_tool);
            }
        }

        // Check for inefficiency
        if let Some(duration) = episode.duration() {
            let duration_secs = duration.num_seconds();
            if duration_secs > 300 {
                // > 5 minutes
                improvements.push(format!(
                    "Optimize execution time (took {} seconds)",
                    duration_secs
                ));
            }
        }

        // Check for excessive steps
        let step_count = episode.steps.len();
        if step_count > 50 {
            improvements.push(format!(
                "Reduce number of execution steps (current: {})",
                step_count
            ));
        }

        // Check for repeated errors
        if let Some(repeated_error) = self.identify_repeated_errors(episode) {
            improvements.push(repeated_error);
        }

        // If no improvements identified but task failed, add generic improvement
        if improvements.is_empty()
            && matches!(
                episode.outcome,
                Some(TaskOutcome::Failure { .. }) | Some(TaskOutcome::PartialSuccess { .. })
            )
        {
            improvements.push("Review and refine approach for better outcomes".to_string());
        }

        // Limit to max items
        improvements.truncate(self.max_items);
        improvements
    }

    /// Generate key insights from the episode
    fn generate_insights(&self, episode: &Episode) -> Vec<String> {
        let mut insights = Vec::new();

        // Only generate detailed insights for episodes with enough steps
        if episode.steps.len() < MIN_STEPS_FOR_REFLECTION {
            return insights;
        }

        // Insight from step patterns
        if let Some(pattern_insight) = self.analyze_step_patterns(episode) {
            insights.push(pattern_insight);
        }

        // Insight from error recovery
        if let Some(recovery_insight) = self.identify_error_recovery_pattern(episode) {
            insights.push(recovery_insight);
        }

        // Insight from context
        let context_insight = format!(
            "Task in {} domain with {:?} complexity",
            episode.context.domain, episode.context.complexity
        );
        insights.push(context_insight);

        // Insight from tool diversity
        let unique_tools = self.count_unique_tools(episode);
        if unique_tools > 5 {
            insights.push(format!(
                "Task required diverse toolset ({} different tools)",
                unique_tools
            ));
        } else if unique_tools == 1 && episode.steps.len() > 3 {
            insights
                .push("Task accomplished with single tool - potential for automation".to_string());
        }

        // Insight from execution time per step
        if let Some(avg_latency) = self.calculate_average_latency(episode) {
            if avg_latency > 5000 {
                // > 5 seconds
                insights.push(format!(
                    "High average step latency: {}ms - consider optimization",
                    avg_latency
                ));
            }
        }

        // Limit to max items
        insights.truncate(self.max_items);
        insights
    }

    /// Identify effective tool sequence
    fn identify_effective_tool_sequence(&self, episode: &Episode) -> Option<String> {
        if episode.steps.len() < 2 {
            return None;
        }

        let successful_tools: Vec<&str> = episode
            .steps
            .iter()
            .filter(|s| s.is_success())
            .map(|s| s.tool.as_str())
            .collect();

        if successful_tools.len() >= 3 {
            let sequence = successful_tools
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(" → ");
            Some(format!("Effective tool sequence: {}", sequence))
        } else {
            None
        }
    }

    /// Identify problematic tool
    fn identify_problematic_tool(&self, episode: &Episode) -> Option<String> {
        let mut tool_failures: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();

        for step in &episode.steps {
            if !step.is_success() {
                *tool_failures.entry(step.tool.as_str()).or_insert(0) += 1;
            }
        }

        tool_failures
            .iter()
            .max_by_key(|(_, &count)| count)
            .filter(|(_, &count)| count >= 2)
            .map(|(tool, count)| {
                format!("Tool '{}' failed {} times - needs attention", tool, count)
            })
    }

    /// Identify repeated errors
    fn identify_repeated_errors(&self, episode: &Episode) -> Option<String> {
        let mut error_messages: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for step in &episode.steps {
            if let Some(ExecutionResult::Error { message }) = &step.result {
                *error_messages.entry(message.clone()).or_insert(0) += 1;
            }
        }

        error_messages
            .iter()
            .max_by_key(|(_, &count)| count)
            .filter(|(_, &count)| count >= 2)
            .map(|(msg, count)| format!("Repeated error ({} times): {}", count, msg))
    }

    /// Analyze step patterns
    fn analyze_step_patterns(&self, episode: &Episode) -> Option<String> {
        let total_steps = episode.steps.len();
        let successful_steps = episode.successful_steps_count();

        if total_steps == 0 {
            return None;
        }

        let success_rate = successful_steps as f32 / total_steps as f32;

        if success_rate == 1.0 {
            Some("All steps executed successfully - reliable execution pattern".to_string())
        } else if success_rate >= 0.8 {
            Some(format!(
                "High reliability pattern with {:.0}% step success rate",
                success_rate * 100.0
            ))
        } else if success_rate < 0.5 {
            Some(format!(
                "Low reliability pattern ({:.0}% success) - review approach",
                success_rate * 100.0
            ))
        } else {
            None
        }
    }

    /// Identify error recovery patterns
    fn identify_error_recovery_pattern(&self, episode: &Episode) -> Option<String> {
        for i in 0..episode.steps.len().saturating_sub(1) {
            let current = &episode.steps[i];
            let next = &episode.steps[i + 1];

            // Check if error was followed by success
            if !current.is_success() && next.is_success() {
                return Some(format!(
                    "Successfully recovered from error using '{}'",
                    next.tool
                ));
            }
        }

        None
    }

    /// Count unique tools used
    fn count_unique_tools(&self, episode: &Episode) -> usize {
        let unique: std::collections::HashSet<_> = episode.steps.iter().map(|s| &s.tool).collect();
        unique.len()
    }

    /// Calculate average step latency
    fn calculate_average_latency(&self, episode: &Episode) -> Option<u64> {
        if episode.steps.is_empty() {
            return None;
        }

        let total_latency: u64 = episode.steps.iter().map(|s| s.latency_ms).sum();
        Some(total_latency / episode.steps.len() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::ExecutionStep;
    use crate::types::{ComplexityLevel, ExecutionResult, TaskContext, TaskType};

    fn create_test_episode() -> Episode {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain: "testing".to_string(),
            tags: vec![],
        };

        Episode::new("Test task".to_string(), context, TaskType::Testing)
    }

    #[test]
    fn test_successful_episode_reflection() {
        let generator = ReflectionGenerator::new();
        let mut episode = create_test_episode();

        // Add successful steps
        for i in 0..5 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "All tests passed".to_string(),
            artifacts: vec!["test_results.json".to_string()],
        });

        let reflection = generator.generate(&episode);

        assert!(!reflection.successes.is_empty());
        assert!(reflection
            .successes
            .iter()
            .any(|s| s.contains("Successfully completed")));
        assert!(reflection
            .successes
            .iter()
            .any(|s| s.contains("Generated 1 artifact")));
    }

    #[test]
    fn test_failed_episode_reflection() {
        let generator = ReflectionGenerator::new();
        let mut episode = create_test_episode();

        // Add some failed steps
        for i in 0..3 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
            step.result = Some(ExecutionResult::Error {
                message: "Error occurred".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Failure {
            reason: "Tests failed".to_string(),
            error_details: Some("Multiple errors".to_string()),
        });

        let reflection = generator.generate(&episode);

        assert!(!reflection.improvements.is_empty());
        assert!(reflection
            .improvements
            .iter()
            .any(|s| s.contains("Task failed")));
    }

    #[test]
    fn test_partial_success_reflection() {
        let generator = ReflectionGenerator::new();
        let mut episode = create_test_episode();

        episode.complete(TaskOutcome::PartialSuccess {
            verdict: "Some tests passed".to_string(),
            completed: vec!["test1".to_string(), "test2".to_string()],
            failed: vec!["test3".to_string()],
        });

        let reflection = generator.generate(&episode);

        assert!(reflection
            .successes
            .iter()
            .any(|s| s.contains("Partial success")));
        assert!(reflection.improvements.iter().any(|s| s.contains("Failed")));
    }

    #[test]
    fn test_error_recovery_insight() {
        let generator = ReflectionGenerator::new();
        let mut episode = create_test_episode();

        // Add error then recovery
        let mut error_step =
            ExecutionStep::new(1, "failing_tool".to_string(), "Failed action".to_string());
        error_step.result = Some(ExecutionResult::Error {
            message: "Error".to_string(),
        });
        episode.add_step(error_step);

        let mut recovery_step = ExecutionStep::new(
            2,
            "recovery_tool".to_string(),
            "Recovery action".to_string(),
        );
        recovery_step.result = Some(ExecutionResult::Success {
            output: "Recovered".to_string(),
        });
        episode.add_step(recovery_step);

        episode.complete(TaskOutcome::Success {
            verdict: "Recovered and completed".to_string(),
            artifacts: vec![],
        });

        let reflection = generator.generate(&episode);

        assert!(reflection
            .insights
            .iter()
            .any(|s| s.contains("Successfully recovered")));
    }

    #[test]
    fn test_problematic_tool_identification() {
        let generator = ReflectionGenerator::new();
        let mut episode = create_test_episode();

        // Add repeated failures from same tool
        for i in 0..3 {
            let mut step =
                ExecutionStep::new(i + 1, "buggy_tool".to_string(), "Action".to_string());
            step.result = Some(ExecutionResult::Error {
                message: "Tool error".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Failure {
            reason: "Tool errors".to_string(),
            error_details: None,
        });

        let reflection = generator.generate(&episode);

        assert!(reflection
            .improvements
            .iter()
            .any(|s| s.contains("buggy_tool")));
    }

    #[test]
    fn test_tool_diversity_insight() {
        let generator = ReflectionGenerator::new();
        let mut episode = create_test_episode();

        // Add many different tools
        for i in 0..7 {
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

        let reflection = generator.generate(&episode);

        assert!(reflection
            .insights
            .iter()
            .any(|s| s.contains("diverse toolset")));
    }

    #[test]
    fn test_single_tool_automation_insight() {
        let generator = ReflectionGenerator::new();
        let mut episode = create_test_episode();

        // Add many steps with same tool
        for i in 0..5 {
            let mut step = ExecutionStep::new(i + 1, "same_tool".to_string(), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let reflection = generator.generate(&episode);

        assert!(reflection
            .insights
            .iter()
            .any(|s| s.contains("single tool")));
    }

    #[test]
    fn test_custom_max_items() {
        let generator = ReflectionGenerator::with_max_items(2);
        let mut episode = create_test_episode();

        // Add many steps to generate many insights
        for i in 0..10 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        });

        let reflection = generator.generate(&episode);

        // Should be limited to max_items
        assert!(reflection.successes.len() <= 2);
        assert!(reflection.insights.len() <= 2);
    }
}

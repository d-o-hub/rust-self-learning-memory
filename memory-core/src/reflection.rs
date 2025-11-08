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
        let mut successes = self.identify_successes(episode);
        let mut improvements = self.identify_improvements(episode);
        let mut insights = self.generate_insights(episode);

        // Add sophisticated analysis
        successes.extend(self.analyze_success_patterns(episode));
        improvements.extend(self.analyze_improvement_opportunities(episode));
        insights.extend(self.generate_contextual_insights(episode));

        // Limit to max items after combining all sources
        successes.truncate(self.max_items);
        improvements.truncate(self.max_items);
        insights.truncate(self.max_items);

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

    /// Analyze success patterns with detailed context
    ///
    /// Identifies what specific strategies worked and why, with generalizability analysis
    fn analyze_success_patterns(&self, episode: &Episode) -> Vec<String> {
        let mut patterns = Vec::new();

        // Only analyze if episode was successful
        let is_success = matches!(
            episode.outcome,
            Some(TaskOutcome::Success { .. }) | Some(TaskOutcome::PartialSuccess { .. })
        );

        if !is_success || episode.steps.is_empty() {
            return patterns;
        }

        // Analyze tool combination effectiveness
        if let Some(combo) = self.analyze_tool_combination_strategy(episode) {
            patterns.push(combo);
        }

        // Analyze execution flow
        if let Some(flow) = self.analyze_execution_flow(episode) {
            patterns.push(flow);
        }

        // Analyze context-specific success factors
        if let Some(context_factor) = self.analyze_context_success_factors(episode) {
            patterns.push(context_factor);
        }

        // Analyze efficiency achievements
        if let Some(efficiency) = self.analyze_efficiency_achievements(episode) {
            patterns.push(efficiency);
        }

        patterns
    }

    /// Analyze improvement opportunities with actionable recommendations
    ///
    /// Provides specific, actionable improvements with root cause analysis
    fn analyze_improvement_opportunities(&self, episode: &Episode) -> Vec<String> {
        let mut opportunities = Vec::new();

        // Analyze step-level bottlenecks
        if let Some(bottleneck) = self.identify_bottlenecks(episode) {
            opportunities.push(bottleneck);
        }

        // Analyze redundancy and repetition
        if let Some(redundancy) = self.identify_redundancy(episode) {
            opportunities.push(redundancy);
        }

        // Analyze error patterns with root causes
        if let Some(error_pattern) = self.analyze_error_root_causes(episode) {
            opportunities.push(error_pattern);
        }

        // Analyze missed optimization opportunities
        if let Some(optimization) = self.identify_optimization_opportunities(episode) {
            opportunities.push(optimization);
        }

        // Analyze resource utilization
        if let Some(resource) = self.analyze_resource_utilization(episode) {
            opportunities.push(resource);
        }

        opportunities
    }

    /// Generate contextual insights with deep analysis
    ///
    /// Provides deep insights about task approach, lessons learned, and recommendations
    fn generate_contextual_insights(&self, episode: &Episode) -> Vec<String> {
        let mut insights = Vec::new();

        // Insight about task complexity vs execution
        if let Some(complexity_insight) = self.analyze_complexity_alignment(episode) {
            insights.push(complexity_insight);
        }

        // Insight about learning and adaptation
        if let Some(learning_insight) = self.analyze_learning_indicators(episode) {
            insights.push(learning_insight);
        }

        // Insight about strategy effectiveness
        if let Some(strategy_insight) = self.analyze_strategy_effectiveness(episode) {
            insights.push(strategy_insight);
        }

        // Recommendations for similar tasks
        if let Some(recommendation) = self.generate_recommendations_for_similar_tasks(episode) {
            insights.push(recommendation);
        }

        insights
    }

    // Helper methods for success pattern analysis

    fn analyze_tool_combination_strategy(&self, episode: &Episode) -> Option<String> {
        let successful_tools: Vec<&str> = episode
            .steps
            .iter()
            .filter(|s| s.is_success())
            .map(|s| s.tool.as_str())
            .collect();

        if successful_tools.len() >= 3 {
            let unique_tools: std::collections::HashSet<_> = successful_tools.iter().collect();
            let strategy = if unique_tools.len() == successful_tools.len() {
                "diverse tool strategy"
            } else {
                "focused tool strategy with repetition"
            };

            Some(format!(
                "Effective {} with {} tools: {}",
                strategy,
                unique_tools.len(),
                unique_tools
                    .iter()
                    .take(3)
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        } else {
            None
        }
    }

    fn analyze_execution_flow(&self, episode: &Episode) -> Option<String> {
        let success_rate = episode.successful_steps_count() as f32 / episode.steps.len() as f32;

        if success_rate > 0.9 {
            Some(format!(
                "Smooth execution flow with {:.0}% success rate - minimal backtracking required",
                success_rate * 100.0
            ))
        } else if self.detect_iterative_refinement(episode) {
            Some(
                "Iterative refinement approach: successfully adapted strategy based on feedback"
                    .to_string(),
            )
        } else {
            None
        }
    }

    fn analyze_context_success_factors(&self, episode: &Episode) -> Option<String> {
        if let Some(language) = &episode.context.language {
            if episode.successful_steps_count() > 5 {
                return Some(format!(
                    "Successfully leveraged {}-specific tools and patterns in {} domain",
                    language, episode.context.domain
                ));
            }
        }

        if !episode.context.tags.is_empty() {
            return Some(format!(
                "Effectively utilized domain knowledge: {}",
                episode.context.tags.join(", ")
            ));
        }

        None
    }

    fn analyze_efficiency_achievements(&self, episode: &Episode) -> Option<String> {
        if let Some(duration) = episode.duration() {
            let duration_secs = duration.num_seconds();
            let steps = episode.steps.len();

            if duration_secs < 60 && steps < 15 {
                Some(format!(
                    "Highly efficient execution: {} steps in {}s - demonstrates expertise",
                    steps, duration_secs
                ))
            } else if steps < 5 && duration_secs < 120 {
                Some(
                    "Minimalist approach: achieved goal with minimal steps - shows clear strategy"
                        .to_string(),
                )
            } else {
                None
            }
        } else {
            None
        }
    }

    // Helper methods for improvement opportunities

    fn identify_bottlenecks(&self, episode: &Episode) -> Option<String> {
        if episode.steps.is_empty() {
            return None;
        }

        // Find slowest step
        let max_latency = episode.steps.iter().map(|s| s.latency_ms).max()?;
        let avg_latency: u64 =
            episode.steps.iter().map(|s| s.latency_ms).sum::<u64>() / episode.steps.len() as u64;

        if max_latency > avg_latency * 3 && max_latency > 1000 {
            let slow_step = episode.steps.iter().find(|s| s.latency_ms == max_latency)?;
            Some(format!(
                "Performance bottleneck: '{}' took {}ms (3x average) - consider optimization or caching",
                slow_step.tool, max_latency
            ))
        } else {
            None
        }
    }

    fn identify_redundancy(&self, episode: &Episode) -> Option<String> {
        let mut tool_counts: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();

        for step in &episode.steps {
            *tool_counts.entry(step.tool.as_str()).or_insert(0) += 1;
        }

        // Find tools used many times
        let redundant = tool_counts
            .iter()
            .filter(|(_, &count)| count >= 5)
            .max_by_key(|(_, &count)| count);

        if let Some((tool, count)) = redundant {
            Some(format!(
                "High repetition of '{}' ({} times) - consider batching or alternative approach",
                tool, count
            ))
        } else {
            None
        }
    }

    fn analyze_error_root_causes(&self, episode: &Episode) -> Option<String> {
        let error_steps: Vec<_> = episode.steps.iter().filter(|s| !s.is_success()).collect();

        if error_steps.len() >= 3 {
            // Check if errors follow a pattern
            let error_tools: Vec<_> = error_steps.iter().map(|s| s.tool.as_str()).collect();
            let unique_error_tools: std::collections::HashSet<_> = error_tools.iter().collect();

            if unique_error_tools.len() == 1 {
                Some(format!(
                    "Systematic issue with '{}' - {} consecutive failures suggest incompatibility or misconfiguration",
                    error_tools[0], error_steps.len()
                ))
            } else {
                Some(format!(
                    "Multiple failure points ({} tools) - review overall approach and prerequisites",
                    unique_error_tools.len()
                ))
            }
        } else {
            None
        }
    }

    fn identify_optimization_opportunities(&self, episode: &Episode) -> Option<String> {
        // Check for sequential operations that could be parallelized
        if episode.steps.len() >= 4 {
            let consecutive_same_type: Vec<_> = episode
                .steps
                .windows(2)
                .filter(|w| w[0].tool == w[1].tool)
                .collect();

            if consecutive_same_type.len() >= 2 {
                Some(
                    "Potential for parallelization: consecutive similar operations detected"
                        .to_string(),
                )
            } else {
                None
            }
        } else {
            None
        }
    }

    fn analyze_resource_utilization(&self, episode: &Episode) -> Option<String> {
        let total_tokens: usize = episode.steps.iter().filter_map(|s| s.tokens_used).sum();

        if total_tokens > 10000 {
            Some(format!(
                "High token usage ({} tokens) - consider more focused prompts or caching",
                total_tokens
            ))
        } else if total_tokens > 0 && total_tokens < 1000 {
            Some(format!(
                "Efficient token usage ({} tokens) - demonstrates focused communication",
                total_tokens
            ))
        } else {
            None
        }
    }

    // Helper methods for contextual insights

    fn analyze_complexity_alignment(&self, episode: &Episode) -> Option<String> {
        let step_count = episode.steps.len();
        let complexity = &episode.context.complexity;

        let expected_steps = match complexity {
            crate::types::ComplexityLevel::Simple => 5,
            crate::types::ComplexityLevel::Moderate => 15,
            crate::types::ComplexityLevel::Complex => 30,
        };

        if step_count < expected_steps / 2 {
            Some(format!(
                "Task complexity ({:?}) handled more efficiently than expected ({} vs ~{} steps)",
                complexity, step_count, expected_steps
            ))
        } else if step_count > expected_steps * 2 {
            Some(format!(
                "Task required more steps than typical for {:?} complexity - may need approach refinement",
                complexity
            ))
        } else {
            None
        }
    }

    fn analyze_learning_indicators(&self, episode: &Episode) -> Option<String> {
        let pattern_count = episode.patterns.len();

        if pattern_count >= 3 {
            Some(format!(
                "Strong learning episode: discovered {} reusable patterns for future tasks",
                pattern_count
            ))
        } else if pattern_count > 0 && episode.successful_steps_count() > 0 {
            Some(format!(
                "Learning opportunity: {} pattern(s) identified - build on this for similar tasks",
                pattern_count
            ))
        } else if self.detect_error_recovery(episode) {
            Some(
                "Valuable learning from error recovery - demonstrates adaptability and problem-solving"
                    .to_string(),
            )
        } else {
            None
        }
    }

    fn analyze_strategy_effectiveness(&self, episode: &Episode) -> Option<String> {
        if episode.steps.is_empty() {
            return None;
        }

        let success_rate = episode.successful_steps_count() as f32 / episode.steps.len() as f32;
        let duration = episode.duration()?;

        if success_rate > 0.8 && duration.num_seconds() < 120 {
            Some(
                "Highly effective strategy: high success rate with quick execution - replicate for similar tasks"
                    .to_string(),
            )
        } else if success_rate < 0.5 {
            Some(format!(
                "Strategy needs refinement: {:.0}% success rate indicates need for different approach",
                success_rate * 100.0
            ))
        } else {
            None
        }
    }

    fn generate_recommendations_for_similar_tasks(&self, episode: &Episode) -> Option<String> {
        if let Some(TaskOutcome::Success { .. }) = &episode.outcome {
            let key_tools: Vec<_> = episode
                .steps
                .iter()
                .filter(|s| s.is_success())
                .map(|s| s.tool.as_str())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .take(3)
                .collect();

            if !key_tools.is_empty() {
                Some(format!(
                    "For similar {} tasks in {}, prioritize: {}",
                    episode.context.domain,
                    episode
                        .context
                        .language
                        .as_deref()
                        .unwrap_or("any language"),
                    key_tools.join(", ")
                ))
            } else {
                None
            }
        } else {
            None
        }
    }

    // Shared helper methods

    fn detect_error_recovery(&self, episode: &Episode) -> bool {
        for i in 0..episode.steps.len().saturating_sub(1) {
            if !episode.steps[i].is_success() && episode.steps[i + 1].is_success() {
                return true;
            }
        }
        false
    }

    fn detect_iterative_refinement(&self, episode: &Episode) -> bool {
        // Look for pattern: fail -> adjust -> succeed
        let mut refinement_count = 0;

        for i in 0..episode.steps.len().saturating_sub(1) {
            let has_error = !episode.steps[i].is_success();
            let has_recovery = episode.steps[i + 1].is_success();

            if has_error && has_recovery {
                refinement_count += 1;
            }
        }

        refinement_count >= 2
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

    // Tests for sophisticated analysis features

    #[test]
    fn test_analyze_tool_combination_strategy() {
        let generator = ReflectionGenerator::new();
        let mut episode = create_test_episode();

        // Add diverse successful tools
        for i in 0..5 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Success".to_string(),
            artifacts: vec![],
        });

        let reflection = generator.generate(&episode);

        // Should identify diverse tool strategy
        assert!(reflection
            .successes
            .iter()
            .any(|s| s.contains("diverse tool strategy") || s.contains("tools")));
    }

    #[test]
    fn test_identify_bottlenecks() {
        let generator = ReflectionGenerator::new();
        let mut episode = create_test_episode();

        // Add steps with one very slow step
        for i in 0..5 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            step.latency_ms = if i == 2 { 5000 } else { 100 }; // One slow step
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Success".to_string(),
            artifacts: vec![],
        });

        let reflection = generator.generate(&episode);

        // Should identify bottleneck
        assert!(reflection
            .improvements
            .iter()
            .any(|s| s.contains("bottleneck") || s.contains("took")));
    }

    #[test]
    fn test_identify_redundancy() {
        let generator = ReflectionGenerator::new();
        let mut episode = create_test_episode();

        // Add same tool many times
        for i in 0..7 {
            let mut step = ExecutionStep::new(i + 1, "same_tool".to_string(), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Success".to_string(),
            artifacts: vec![],
        });

        let reflection = generator.generate(&episode);

        // Should identify redundancy
        assert!(reflection
            .improvements
            .iter()
            .any(|s| s.contains("repetition") || s.contains("same_tool")));
    }

    #[test]
    fn test_analyze_error_root_causes() {
        let generator = ReflectionGenerator::new();
        let mut episode = create_test_episode();

        // Add multiple failures with same tool
        for i in 0..4 {
            let mut step =
                ExecutionStep::new(i + 1, "problematic_tool".to_string(), "Action".to_string());
            step.result = Some(ExecutionResult::Error {
                message: "Error".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Failure {
            reason: "Tool errors".to_string(),
            error_details: None,
        });

        let reflection = generator.generate(&episode);

        // Should identify systematic issue
        assert!(reflection
            .improvements
            .iter()
            .any(|s| s.contains("Systematic") || s.contains("problematic_tool")));
    }

    #[test]
    fn test_analyze_complexity_alignment() {
        let generator = ReflectionGenerator::new();
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Simple,
            domain: "testing".to_string(),
            tags: vec![],
        };

        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Testing);

        // Add just 2 steps for "Simple" task (expected ~5)
        for i in 0..2 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Efficient completion".to_string(),
            artifacts: vec![],
        });

        let reflection = generator.generate(&episode);

        // Should note efficiency vs complexity
        assert!(reflection
            .insights
            .iter()
            .any(|s| s.contains("complexity") || s.contains("efficiently")));
    }

    #[test]
    fn test_analyze_learning_indicators_with_patterns() {
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

        // Add patterns to simulate pattern discovery
        use uuid::Uuid;
        episode.patterns.push(Uuid::new_v4());
        episode.patterns.push(Uuid::new_v4());
        episode.patterns.push(Uuid::new_v4());

        episode.complete(TaskOutcome::Success {
            verdict: "Learned patterns".to_string(),
            artifacts: vec![],
        });

        let reflection = generator.generate(&episode);

        // Should identify learning
        assert!(reflection
            .insights
            .iter()
            .any(|s| s.contains("learning") || s.contains("pattern")));
    }

    #[test]
    fn test_generate_recommendations_for_similar_tasks() {
        let generator = ReflectionGenerator::new();
        let mut episode = create_test_episode();

        // Add successful steps with specific tools
        for i in 0..5 {
            let mut step =
                ExecutionStep::new(i + 1, format!("key_tool_{}", i), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Success".to_string(),
            artifacts: vec![],
        });

        let reflection = generator.generate(&episode);

        // Should generate recommendations
        assert!(reflection
            .insights
            .iter()
            .any(|s| s.contains("similar") || s.contains("prioritize")));
    }

    #[test]
    fn test_analyze_resource_utilization() {
        let generator = ReflectionGenerator::new();
        let mut episode = create_test_episode();

        // Add steps with token usage
        for i in 0..5 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            step.tokens_used = Some(3000); // High token usage
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Success".to_string(),
            artifacts: vec![],
        });

        let reflection = generator.generate(&episode);

        // Should identify high token usage
        assert!(reflection
            .improvements
            .iter()
            .any(|s| s.contains("token") || s.contains("usage")));
    }

    #[test]
    fn test_iterative_refinement_detection() {
        let generator = ReflectionGenerator::new();
        let mut episode = create_test_episode();

        // Add pattern: error -> success, error -> success
        for i in 0..4 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
            step.result = if i % 2 == 0 {
                Some(ExecutionResult::Error {
                    message: "Error".to_string(),
                })
            } else {
                Some(ExecutionResult::Success {
                    output: "OK".to_string(),
                })
            };
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Success through iteration".to_string(),
            artifacts: vec![],
        });

        let reflection = generator.generate(&episode);

        // Should identify iterative refinement
        assert!(reflection
            .successes
            .iter()
            .any(|s| s.contains("Iterative") || s.contains("adapted")));
    }

    #[test]
    fn test_comprehensive_sophisticated_reflection() {
        let generator = ReflectionGenerator::new();
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity: ComplexityLevel::Complex,
            domain: "web-api".to_string(),
            tags: vec!["async".to_string(), "rest".to_string()],
        };

        let mut episode = Episode::new(
            "Build async API".to_string(),
            context,
            TaskType::CodeGeneration,
        );

        // Add diverse successful execution
        for i in 0..8 {
            let mut step =
                ExecutionStep::new(i + 1, format!("api_tool_{}", i), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            step.latency_ms = 200;
            step.tokens_used = Some(500);
            episode.add_step(step);
        }

        // Add patterns
        use uuid::Uuid;
        episode.patterns.push(Uuid::new_v4());
        episode.patterns.push(Uuid::new_v4());

        episode.complete(TaskOutcome::Success {
            verdict: "API successfully created".to_string(),
            artifacts: vec![
                "api.rs".to_string(),
                "tests.rs".to_string(),
                "docs.md".to_string(),
            ],
        });

        let reflection = generator.generate(&episode);

        // Should have sophisticated insights across all categories
        assert!(!reflection.successes.is_empty());
        assert!(!reflection.insights.is_empty());

        // Check for sophisticated analysis
        let all_text = reflection
            .successes
            .iter()
            .chain(reflection.insights.iter())
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        // Should mention context-specific factors
        assert!(
            all_text.contains("rust")
                || all_text.contains("domain")
                || all_text.contains("pattern")
        );
    }
}

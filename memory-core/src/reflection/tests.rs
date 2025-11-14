//! Tests for reflection module

use crate::episode::Episode;
use crate::reflection::{ReflectionGenerator, success_analyzer, improvement_analyzer, insight_generator};
use crate::types::{TaskContext, TaskType, TaskOutcome, ExecutionResult, ComplexityLevel};
use crate::ExecutionStep;


#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a test episode
    fn create_test_episode(
        description: &str,
        task_type: TaskType,
        steps: Vec<ExecutionStep>,
        outcome: Option<TaskOutcome>,
    ) -> Episode {
        let mut episode = Episode::new(description.to_string(), TaskContext::default(), task_type);
        for step in steps {
            episode.add_step(step);
        }
        if let Some(outcome) = outcome {
            episode.complete(outcome);
        }
        episode
    }

    // Helper function to create a successful execution step
    fn successful_step(step_number: usize, tool: &str, action: &str) -> ExecutionStep {
        let mut step = ExecutionStep::new(step_number, tool.to_string(), action.to_string());
        step.result = Some(ExecutionResult::Success {
            output: "Success".to_string(),
        });
        step.latency_ms = 100;
        step.tokens_used = Some(50);
        step
    }

    // Helper function to create a successful execution step with custom latency
    fn successful_step_with_latency(step_number: usize, tool: &str, action: &str, latency_ms: u64) -> ExecutionStep {
        let mut step = successful_step(step_number, tool, action);
        step.latency_ms = latency_ms;
        step
    }

    // Helper function to create a failed execution step
    fn failed_step(step_number: usize, tool: &str, action: &str, error_msg: &str) -> ExecutionStep {
        let mut step = ExecutionStep::new(step_number, tool.to_string(), action.to_string());
        step.result = Some(ExecutionResult::Error {
            message: error_msg.to_string(),
        });
        step.latency_ms = 200;
        step.tokens_used = Some(25);
        step
    }

    mod reflection_generator_tests {
        use super::*;

        #[test]
        fn test_generate_reflection_successful_episode() {
            let steps = vec![
                successful_step(1, "test_runner", "Run unit tests"),
                successful_step(2, "code_review", "Review code quality"),
                successful_step(3, "build_tool", "Build project"),
            ];

            let outcome = TaskOutcome::Success {
                verdict: "All tests passed".to_string(),
                artifacts: vec!["test_results.json".to_string()],
            };

            let episode = create_test_episode("Test successful task", TaskType::Testing, steps, Some(outcome));
            let generator = ReflectionGenerator::new();
            let reflection = generator.generate(&episode);

            assert!(!reflection.successes.is_empty());
            assert!(reflection.successes.iter().any(|s| s.contains("Successfully completed")));
            assert!(reflection.successes.iter().any(|s| s.contains("artifact")));
        }

        #[test]
        fn test_generate_reflection_failed_episode() {
            let steps = vec![
                successful_step(1, "test_runner", "Run unit tests"),
                failed_step(2, "code_review", "Review code quality", "Code review failed"),
                failed_step(3, "build_tool", "Build project", "Build failed"),
            ];

            let outcome = TaskOutcome::Failure {
                reason: "Multiple failures occurred".to_string(),
                error_details: None,
            };

            let episode = create_test_episode("Test failed task", TaskType::Testing, steps, Some(outcome));
            let generator = ReflectionGenerator::new();
            let reflection = generator.generate(&episode);

            assert!(!reflection.improvements.is_empty());
            assert!(reflection.improvements.iter().any(|i| i.contains("failed")));
        }

        #[test]
        fn test_generate_reflection_empty_episode() {
            let episode = create_test_episode("Empty episode", TaskType::Testing, vec![], None);
            let generator = ReflectionGenerator::new();
            let reflection = generator.generate(&episode);

            // Should handle empty episodes gracefully
            assert!(reflection.successes.is_empty() || reflection.improvements.is_empty() || reflection.insights.is_empty());
        }

        #[test]
        fn test_generate_reflection_partial_success() {
            let steps = vec![
                successful_step(1, "test_runner", "Run unit tests"),
                failed_step(2, "code_review", "Review code quality", "Review failed"),
            ];

            let outcome = TaskOutcome::PartialSuccess {
                verdict: "Partial completion".to_string(),
                completed: vec!["testing".to_string()],
                failed: vec!["review".to_string()],
            };

            let episode = create_test_episode("Partial success", TaskType::Testing, steps, Some(outcome));
            let generator = ReflectionGenerator::new();
            let reflection = generator.generate(&episode);

            assert!(!reflection.successes.is_empty());
            assert!(!reflection.improvements.is_empty());
        }

        #[test]
        fn test_generate_reflection_with_custom_max_items() {
            let steps = vec![
                successful_step(1, "tool1", "action1"),
                successful_step(2, "tool2", "action2"),
                successful_step(3, "tool3", "action3"),
                successful_step(4, "tool4", "action4"),
                successful_step(5, "tool5", "action5"),
                successful_step(6, "tool6", "action6"),
            ];

            let outcome = TaskOutcome::Success {
                verdict: "Success".to_string(),
                artifacts: vec![],
            };

            let episode = create_test_episode("Many steps", TaskType::Testing, steps, Some(outcome));
            let generator = ReflectionGenerator::with_max_items(2);
            let reflection = generator.generate(&episode);

            assert!(reflection.successes.len() <= 2);
            assert!(reflection.improvements.len() <= 2);
            assert!(reflection.insights.len() <= 2);
        }
    }

    mod success_analyzer_tests {
        use super::*;

        #[test]
        fn test_identify_successes_full_success() {
            let steps = vec![
                successful_step(1, "test_runner", "Run tests"),
                successful_step(2, "build_tool", "Build"),
            ];

            let outcome = TaskOutcome::Success {
                verdict: "All good".to_string(),
                artifacts: vec!["results.json".to_string()],
            };

            let episode = create_test_episode("Success test", TaskType::Testing, steps, Some(outcome));
            let successes = success_analyzer::identify_successes(&episode, 5);

            assert!(!successes.is_empty());
            assert!(successes.iter().any(|s| s.contains("Successfully completed")));
            assert!(successes.iter().any(|s| s.contains("artifact")));
        }

        #[test]
        fn test_identify_successes_partial_success() {
            let steps = vec![successful_step(1, "test_runner", "Run tests")];

            let outcome = TaskOutcome::PartialSuccess {
                verdict: "Partial".to_string(),
                completed: vec!["testing".to_string()],
                failed: vec![],
            };

            let episode = create_test_episode("Partial test", TaskType::Testing, steps, Some(outcome));
            let successes = success_analyzer::identify_successes(&episode, 5);

            assert!(!successes.is_empty());
            assert!(successes.iter().any(|s| s.contains("Partial success")));
        }

        #[test]
        fn test_identify_successes_high_success_rate() {
            let steps = vec![
                successful_step(1, "tool1", "action1"),
                successful_step(2, "tool2", "action2"),
                successful_step(3, "tool3", "action3"),
                successful_step(4, "tool4", "action4"),
                successful_step(5, "tool5", "action5"),
            ];

            let outcome = TaskOutcome::Success {
                verdict: "Success".to_string(),
                artifacts: vec![],
            };

            let episode = create_test_episode("High success rate", TaskType::Testing, steps, Some(outcome));
            let successes = success_analyzer::identify_successes(&episode, 5);

            assert!(successes.iter().any(|s| s.contains("success rate")));
        }

        #[test]
        fn test_identify_successes_efficient_execution() {
            let mut step = successful_step(1, "tool", "action");
            step.latency_ms = 10; // Very fast

            let steps = vec![step];
            let outcome = TaskOutcome::Success {
                verdict: "Fast".to_string(),
                artifacts: vec![],
            };

            let episode = create_test_episode("Efficient", TaskType::Testing, steps, Some(outcome));
            let successes = success_analyzer::identify_successes(&episode, 5);

            assert!(successes.iter().any(|s| s.contains("Efficient execution")));
        }

        #[test]
        fn test_identify_successes_effective_tool_sequence() {
            let steps = vec![
                successful_step(1, "tool1", "action1"),
                successful_step(2, "tool2", "action2"),
                successful_step(3, "tool3", "action3"),
            ];

            let outcome = TaskOutcome::Success {
                verdict: "Success".to_string(),
                artifacts: vec![],
            };

            let episode = create_test_episode("Tool sequence", TaskType::Testing, steps, Some(outcome));
            let successes = success_analyzer::identify_successes(&episode, 5);

            assert!(successes.iter().any(|s| s.contains("tool sequence")));
        }

        #[test]
        fn test_analyze_success_patterns_tool_combination() {
            let steps = vec![
                successful_step(1, "tool1", "action1"),
                successful_step(2, "tool2", "action2"),
                successful_step(3, "tool3", "action3"),
            ];

            let outcome = TaskOutcome::Success {
                verdict: "Success".to_string(),
                artifacts: vec![],
            };

            let episode = create_test_episode("Tool combo", TaskType::Testing, steps, Some(outcome));
            let patterns = success_analyzer::analyze_success_patterns(&episode);

            assert!(!patterns.is_empty());
            assert!(patterns.iter().any(|p| p.contains("tool strategy")));
        }

        #[test]
        fn test_analyze_success_patterns_execution_flow() {
            let steps = vec![
                successful_step(1, "tool1", "action1"),
                successful_step(2, "tool2", "action2"),
                successful_step(3, "tool3", "action3"),
                successful_step(4, "tool4", "action4"),
                successful_step(5, "tool5", "action5"),
            ];

            let outcome = TaskOutcome::Success {
                verdict: "Success".to_string(),
                artifacts: vec![],
            };

            let episode = create_test_episode("Smooth flow", TaskType::Testing, steps, Some(outcome));
            let patterns = success_analyzer::analyze_success_patterns(&episode);

            assert!(patterns.iter().any(|p| p.contains("execution flow")));
        }

        #[test]
        fn test_analyze_success_patterns_context_factors() {
            let mut context = TaskContext::default();
            context.language = Some("Rust".to_string());
            context.domain = "testing".to_string();
            context.tags = vec!["unit".to_string(), "integration".to_string()];

            let steps = vec![
                successful_step(1, "tool1", "action1"),
                successful_step(2, "tool2", "action2"),
                successful_step(3, "tool3", "action3"),
                successful_step(4, "tool4", "action4"),
                successful_step(5, "tool5", "action5"),
                successful_step(6, "tool6", "action6"),
            ];

            let outcome = TaskOutcome::Success {
                verdict: "Success".to_string(),
                artifacts: vec![],
            };

            let mut episode = Episode::new("Context test".to_string(), context, TaskType::Testing);
            for step in steps {
                episode.add_step(step);
            }
            episode.complete(outcome);

            let patterns = success_analyzer::analyze_success_patterns(&episode);

            assert!(patterns.iter().any(|p| p.contains("Rust-specific") || p.contains("domain knowledge")));
        }

        #[test]
        fn test_analyze_success_patterns_efficiency() {
            let steps = vec![
                successful_step(1, "tool1", "action1"),
                successful_step(2, "tool2", "action2"),
            ];

            let outcome = TaskOutcome::Success {
                verdict: "Success".to_string(),
                artifacts: vec![],
            };

            let episode = create_test_episode("Efficient", TaskType::Testing, steps, Some(outcome));
            let patterns = success_analyzer::analyze_success_patterns(&episode);

            assert!(patterns.iter().any(|p| p.contains("expertise") || p.contains("minimalist")));
        }

        #[test]
        fn test_analyze_success_patterns_failed_episode() {
            let steps = vec![failed_step(1, "tool", "action", "Failed")];

            let outcome = TaskOutcome::Failure {
                reason: "Failed".to_string(),
                error_details: None,
            };

            let episode = create_test_episode("Failed", TaskType::Testing, steps, Some(outcome));
            let patterns = success_analyzer::analyze_success_patterns(&episode);

            assert!(patterns.is_empty());
        }
    }

    mod improvement_analyzer_tests {
        use super::*;

        #[test]
        fn test_identify_improvements_failed_episode() {
            let steps = vec![
                successful_step(1, "tool1", "action1"),
                failed_step(2, "tool2", "action2", "Error occurred"),
            ];

            let outcome = TaskOutcome::Failure {
                reason: "Task failed".to_string(),
                error_details: None,
            };

            let episode = create_test_episode("Failed task", TaskType::Testing, steps, Some(outcome));
            let improvements = improvement_analyzer::identify_improvements(&episode, 5);

            assert!(!improvements.is_empty());
            assert!(improvements.iter().any(|i| i.contains("failed")));
        }

        #[test]
        fn test_identify_improvements_partial_success() {
            let steps = vec![
                successful_step(1, "tool1", "action1"),
                failed_step(2, "tool2", "action2", "Error"),
            ];

            let outcome = TaskOutcome::PartialSuccess {
                verdict: "Partial".to_string(),
                completed: vec!["step1".to_string()],
                failed: vec!["step2".to_string()],
            };

            let episode = create_test_episode("Partial", TaskType::Testing, steps, Some(outcome));
            let improvements = improvement_analyzer::identify_improvements(&episode, 5);

            assert!(improvements.iter().any(|i| i.contains("failed")));
        }

        #[test]
        fn test_identify_improvements_long_duration() {
            let steps = vec![successful_step(1, "tool", "action")];
            let outcome = TaskOutcome::Success {
                verdict: "Slow".to_string(),
                artifacts: vec![],
            };

            let mut episode = create_test_episode("Slow task", TaskType::Testing, steps, None);
            // Simulate long duration by setting end_time far in the future
            episode.end_time = Some(episode.start_time + chrono::Duration::seconds(400));
            episode.outcome = Some(outcome);

            let improvements = improvement_analyzer::identify_improvements(&episode, 5);

            assert!(improvements.iter().any(|i| i.contains("execution time")));
        }

        #[test]
        fn test_identify_improvements_many_steps() {
            let steps: Vec<_> = (1..60).map(|i| successful_step(i, "tool", &format!("action{}", i))).collect();

            let outcome = TaskOutcome::Success {
                verdict: "Many steps".to_string(),
                artifacts: vec![],
            };

            let episode = create_test_episode("Many steps", TaskType::Testing, steps, Some(outcome));
            let improvements = improvement_analyzer::identify_improvements(&episode, 5);

            assert!(improvements.iter().any(|i| i.contains("execution steps")));
        }

        #[test]
        fn test_identify_improvements_repeated_errors() {
            let steps = vec![
                failed_step(1, "tool", "action1", "Same error"),
                failed_step(2, "tool", "action2", "Same error"),
                failed_step(3, "tool", "action3", "Same error"),
            ];

            let outcome = TaskOutcome::Failure {
                reason: "Repeated errors".to_string(),
                error_details: None,
            };

            let episode = create_test_episode("Repeated errors", TaskType::Testing, steps, Some(outcome));
            let improvements = improvement_analyzer::identify_improvements(&episode, 5);

            assert!(improvements.iter().any(|i| i.contains("Repeated error")));
        }

        #[test]
        fn test_analyze_improvement_opportunities_bottlenecks() {
            let steps = vec![
                successful_step_with_latency(1, "fast_tool", "fast_action", 1),
                successful_step_with_latency(2, "fast_tool", "fast_action2", 1),
                successful_step_with_latency(3, "fast_tool", "fast_action3", 1),
                successful_step_with_latency(4, "slow_tool", "slow_action", 5000), // Very slow
            ];

            let outcome = TaskOutcome::Success {
                verdict: "Success".to_string(),
                artifacts: vec![],
            };

            let episode = create_test_episode("Bottleneck", TaskType::Testing, steps, Some(outcome));
            let opportunities = improvement_analyzer::analyze_improvement_opportunities(&episode);

            assert!(opportunities.iter().any(|o| o.contains("bottleneck")));
        }

        #[test]
        fn test_analyze_improvement_opportunities_redundancy() {
            let steps: Vec<_> = (1..8).map(|i| successful_step(i, "same_tool", "action")).collect();

            let outcome = TaskOutcome::Success {
                verdict: "Redundant".to_string(),
                artifacts: vec![],
            };

            let episode = create_test_episode("Redundant", TaskType::Testing, steps, Some(outcome));
            let opportunities = improvement_analyzer::analyze_improvement_opportunities(&episode);

            assert!(opportunities.iter().any(|o| o.contains("repetition")));
        }

        #[test]
        fn test_analyze_improvement_opportunities_error_patterns() {
            let steps = vec![
                failed_step(1, "tool1", "action1", "Error"),
                failed_step(2, "tool1", "action2", "Error"),
                failed_step(3, "tool1", "action3", "Error"),
            ];

            let outcome = TaskOutcome::Failure {
                reason: "Systematic failure".to_string(),
                error_details: None,
            };

            let episode = create_test_episode("Error pattern", TaskType::Testing, steps, Some(outcome));
            let opportunities = improvement_analyzer::analyze_improvement_opportunities(&episode);

            assert!(opportunities.iter().any(|o| o.contains("Systematic issue")));
        }

        #[test]
        fn test_analyze_improvement_opportunities_parallelization() {
            let steps = vec![
                successful_step(1, "tool", "action1"),
                successful_step(2, "tool", "action2"),
                successful_step(3, "tool", "action3"),
                successful_step(4, "tool", "action4"),
            ];

            let outcome = TaskOutcome::Success {
                verdict: "Sequential".to_string(),
                artifacts: vec![],
            };

            let episode = create_test_episode("Sequential", TaskType::Testing, steps, Some(outcome));
            let opportunities = improvement_analyzer::analyze_improvement_opportunities(&episode);

            assert!(opportunities.iter().any(|o| o.contains("parallelization")));
        }

        #[test]
        fn test_analyze_improvement_opportunities_resource_usage() {
            let mut high_token_step = successful_step(1, "tool", "action");
            high_token_step.tokens_used = Some(15000); // High token usage

            let steps = vec![high_token_step];

            let outcome = TaskOutcome::Success {
                verdict: "High tokens".to_string(),
                artifacts: vec![],
            };

            let episode = create_test_episode("High tokens", TaskType::Testing, steps, Some(outcome));
            let opportunities = improvement_analyzer::analyze_improvement_opportunities(&episode);

            assert!(opportunities.iter().any(|o| o.contains("token usage")));
        }
    }

    mod insight_generator_tests {
        use super::*;

        #[test]
        fn test_generate_insights_minimal_steps() {
            let steps = vec![successful_step(1, "tool", "action")];

            let outcome = TaskOutcome::Success {
                verdict: "Success".to_string(),
                artifacts: vec![],
            };

            let episode = create_test_episode("Minimal", TaskType::Testing, steps, Some(outcome));
            let insights = insight_generator::generate_insights(&episode, 5);

            // Should return minimal insights for episodes with few steps
            assert!(insights.len() <= 2);
        }

        #[test]
        fn test_generate_insights_step_patterns() {
            let steps = vec![
                successful_step(1, "tool1", "action1"),
                successful_step(2, "tool2", "action2"),
                successful_step(3, "tool3", "action3"),
                successful_step(4, "tool4", "action4"),
                successful_step(5, "tool5", "action5"),
            ];

            let outcome = TaskOutcome::Success {
                verdict: "Success".to_string(),
                artifacts: vec![],
            };

            let episode = create_test_episode("Perfect success", TaskType::Testing, steps, Some(outcome));
            let insights = insight_generator::generate_insights(&episode, 5);

            assert!(insights.iter().any(|i| i.contains("reliable") || i.contains("All steps")));
        }

        #[test]
        fn test_generate_insights_error_recovery() {
            let steps = vec![
                failed_step(1, "tool1", "action1", "Failed"),
                successful_step(2, "tool2", "action2"),
            ];

            let outcome = TaskOutcome::Success {
                verdict: "Recovered".to_string(),
                artifacts: vec![],
            };

            let episode = create_test_episode("Recovery", TaskType::Testing, steps, Some(outcome));
            let insights = insight_generator::generate_insights(&episode, 5);

            assert!(insights.iter().any(|i| i.contains("recovered from error")));
        }

        #[test]
        fn test_generate_insights_tool_diversity() {
            let steps: Vec<_> = (1..8).map(|i| successful_step(i, &format!("tool{}", i), "action")).collect();

            let outcome = TaskOutcome::Success {
                verdict: "Diverse".to_string(),
                artifacts: vec![],
            };

            let episode = create_test_episode("Diverse tools", TaskType::Testing, steps, Some(outcome));
            let insights = insight_generator::generate_insights(&episode, 5);

            assert!(insights.iter().any(|i| i.contains("diverse toolset")));
        }

        #[test]
        fn test_generate_insights_single_tool() {
            let steps: Vec<_> = (1..5).map(|i| successful_step(i, "same_tool", "action")).collect();

            let outcome = TaskOutcome::Success {
                verdict: "Single tool".to_string(),
                artifacts: vec![],
            };

            let episode = create_test_episode("Single tool", TaskType::Testing, steps, Some(outcome));
            let insights = insight_generator::generate_insights(&episode, 5);

            assert!(insights.iter().any(|i| i.contains("single tool")));
        }

        #[test]
        fn test_generate_insights_high_latency() {
            let steps = vec![
                successful_step_with_latency(1, "slow_tool", "slow_action1", 10000), // 10 seconds
                successful_step_with_latency(2, "slow_tool", "slow_action2", 10000), // 10 seconds
            ];

            let outcome = TaskOutcome::Success {
                verdict: "Slow".to_string(),
                artifacts: vec![],
            };

            let episode = create_test_episode("Slow steps", TaskType::Testing, steps, Some(outcome));
            let insights = insight_generator::generate_insights(&episode, 5);

            assert!(insights.iter().any(|i| i.contains("latency")));
        }

        #[test]
        fn test_generate_contextual_insights_complexity_alignment() {
            let mut context = TaskContext::default();
            context.complexity = ComplexityLevel::Simple;

            let steps = (1..12).map(|i| successful_step(i, &format!("tool{}", i), &format!("action{}", i))).collect::<Vec<_>>();

            let outcome = TaskOutcome::Success {
                verdict: "Complex for simple".to_string(),
                artifacts: vec![],
            };

            let mut episode = Episode::new("Complex simple".to_string(), context, TaskType::Testing);
            for step in steps {
                episode.add_step(step);
            }
            episode.complete(outcome);

            let insights = insight_generator::generate_contextual_insights(&episode);

            assert!(insights.iter().any(|i| i.contains("more steps than typical")));
        }

        #[test]
        fn test_generate_contextual_insights_learning_indicators() {
            let steps = vec![
                failed_step(1, "tool1", "action1", "Failed"),
                successful_step(2, "tool2", "action2"), // Error recovery
            ];

            let outcome = TaskOutcome::Success {
                verdict: "Recovered".to_string(),
                artifacts: vec![],
            };

            let episode = create_test_episode("Learning", TaskType::Testing, steps, Some(outcome));
            let insights = insight_generator::generate_contextual_insights(&episode);

            assert!(insights.iter().any(|i| i.contains("learning") || i.contains("adaptability")));
        }

        #[test]
        fn test_generate_contextual_insights_strategy_effectiveness() {
            let steps = vec![
                successful_step(1, "tool1", "action1"),
                successful_step(2, "tool2", "action2"),
            ];

            let outcome = TaskOutcome::Success {
                verdict: "Effective".to_string(),
                artifacts: vec![],
            };

            let episode = create_test_episode("Effective", TaskType::Testing, steps, Some(outcome));
            let insights = insight_generator::generate_contextual_insights(&episode);

            assert!(insights.iter().any(|i| i.contains("effective strategy")));
        }

        #[test]
        fn test_generate_contextual_insights_recommendations() {
            let mut context = TaskContext::default();
            context.domain = "testing".to_string();
            context.language = Some("Rust".to_string());

            let steps = vec![
                successful_step(1, "test_runner", "run tests"),
                successful_step(2, "code_review", "review code"),
            ];

            let outcome = TaskOutcome::Success {
                verdict: "Success".to_string(),
                artifacts: vec![],
            };

            let mut episode = Episode::new("Recommendations".to_string(), context, TaskType::Testing);
            for step in steps {
                episode.add_step(step);
            }
            episode.complete(outcome);

            let insights = insight_generator::generate_contextual_insights(&episode);

            assert!(insights.iter().any(|i| i.contains("prioritize")));
        }
    }

    mod helpers_tests {
        use super::*;
        use crate::reflection::helpers;

        #[test]
        fn test_count_unique_tools() {
            let steps = vec![
                successful_step(1, "tool1", "action1"),
                successful_step(2, "tool1", "action2"),
                successful_step(3, "tool2", "action3"),
            ];

            let episode = create_test_episode("Unique tools", TaskType::Testing, steps, None);

            assert_eq!(helpers::count_unique_tools(&episode), 2);
        }

        #[test]
        fn test_calculate_average_latency() {
            let steps = vec![
                successful_step_with_latency(1, "tool1", "action1", 100),
                successful_step_with_latency(2, "tool2", "action2", 200),
            ];

            let episode = create_test_episode("Average latency", TaskType::Testing, steps, None);

            assert_eq!(helpers::calculate_average_latency(&episode), Some(150));
        }

        #[test]
        fn test_calculate_average_latency_empty() {
            let episode = create_test_episode("Empty", TaskType::Testing, vec![], None);

            assert_eq!(helpers::calculate_average_latency(&episode), None);
        }

        #[test]
        fn test_detect_error_recovery() {
            let steps = vec![
                failed_step(1, "tool1", "action1", "Failed"),
                successful_step(2, "tool2", "action2"),
            ];

            let episode = create_test_episode("Error recovery", TaskType::Testing, steps, None);

            assert!(helpers::detect_error_recovery(&episode));
        }

        #[test]
        fn test_detect_error_recovery_no_recovery() {
            let steps = vec![
                successful_step(1, "tool1", "action1"),
                failed_step(2, "tool2", "action2", "Failed"),
            ];

            let episode = create_test_episode("No recovery", TaskType::Testing, steps, None);

            assert!(!helpers::detect_error_recovery(&episode));
        }

        #[test]
        fn test_detect_iterative_refinement() {
            let steps = vec![
                failed_step(1, "tool1", "action1", "Failed"),
                successful_step(2, "tool2", "action2"),
                failed_step(3, "tool3", "action3", "Failed again"),
                successful_step(4, "tool4", "action4"),
            ];

            let episode = create_test_episode("Iterative", TaskType::Testing, steps, None);

            assert!(helpers::detect_iterative_refinement(&episode));
        }

        #[test]
        fn test_detect_iterative_refinement_insufficient() {
            let steps = vec![
                failed_step(1, "tool1", "action1", "Failed"),
                successful_step(2, "tool2", "action2"),
            ];

            let episode = create_test_episode("Single recovery", TaskType::Testing, steps, None);

            assert!(!helpers::detect_iterative_refinement(&episode));
        }
    }
}
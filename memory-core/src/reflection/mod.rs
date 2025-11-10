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

mod helpers;
mod improvement_analyzer;
mod insight_generator;
mod success_analyzer;

use crate::episode::Episode;
use crate::types::Reflection;
use chrono::Utc;
use tracing::{debug, instrument};

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
        let mut successes = success_analyzer::identify_successes(episode, self.max_items);
        let mut improvements = improvement_analyzer::identify_improvements(episode, self.max_items);
        let mut insights = insight_generator::generate_insights(episode, self.max_items);

        // Add sophisticated analysis
        successes.extend(success_analyzer::analyze_success_patterns(episode));
        improvements.extend(improvement_analyzer::analyze_improvement_opportunities(episode));
        insights.extend(insight_generator::generate_contextual_insights(episode));

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::ExecutionStep;
    use crate::types::{ComplexityLevel, ExecutionResult, TaskContext, TaskOutcome, TaskType};

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

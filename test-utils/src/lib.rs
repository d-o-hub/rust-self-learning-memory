//! # Test Utilities
//!
//! Shared test utilities for the self-learning memory system.
//!
//! Provides:
//! - Test data generators
//! - Mock storage backends  
//! - Test fixtures
//! - Helper functions

use memory_core::*;
use uuid::Uuid;

/// Create a test episode with minimal configuration
pub fn create_test_episode(task_description: &str) -> Episode {
    Episode::new(
        task_description.to_string(),
        TaskContext::default(),
        TaskType::Testing,
    )
}

/// Create a test episode with specific context
pub fn create_test_episode_with_context(
    task_description: &str,
    context: TaskContext,
    task_type: TaskType,
) -> Episode {
    Episode::new(task_description.to_string(), context, task_type)
}

/// Create a completed episode for testing
pub fn create_completed_episode(task_description: &str, success: bool) -> Episode {
    let mut episode = create_test_episode(task_description);

    // Add some test steps
    for i in 0..3 {
        let mut step =
            ExecutionStep::new(i + 1, format!("test_tool_{}", i), format!("Action {}", i));
        step.result = Some(if success || i < 2 {
            ExecutionResult::Success {
                output: "OK".to_string(),
            }
        } else {
            ExecutionResult::Error {
                message: "Failed".to_string(),
            }
        });
        step.latency_ms = 100;
        episode.add_step(step);
    }

    // Complete the episode
    let outcome = if success {
        TaskOutcome::Success {
            verdict: "Test passed".to_string(),
            artifacts: vec![],
        }
    } else {
        TaskOutcome::Failure {
            reason: "Test failed".to_string(),
            error_details: None,
        }
    };

    episode.complete(outcome);
    episode
}

/// Create a test execution step
pub fn create_test_step(step_number: usize) -> ExecutionStep {
    let mut step = ExecutionStep::new(
        step_number,
        "test_tool".to_string(),
        "Test action".to_string(),
    );
    step.parameters = serde_json::json!({"test": "value"});
    step.result = Some(ExecutionResult::Success {
        output: "Success".to_string(),
    });
    step.latency_ms = 50;
    step.tokens_used = Some(100);
    step
}

/// Create a test context with specific domain
pub fn create_test_context(domain: &str, language: Option<&str>) -> TaskContext {
    TaskContext {
        language: language.map(|s| s.to_string()),
        framework: None,
        complexity: ComplexityLevel::Moderate,
        domain: domain.to_string(),
        tags: vec![],
    }
}

/// Create multiple test episodes with different contexts
pub fn create_test_episodes(count: usize, domain: &str) -> Vec<Episode> {
    (0..count)
        .map(|i| {
            let context = create_test_context(domain, Some("rust"));
            create_test_episode_with_context(
                &format!("Test task {}", i),
                context,
                TaskType::Testing,
            )
        })
        .collect()
}

/// Create a test pattern
pub fn create_test_pattern(pattern_type: &str, success_rate: f32) -> Pattern {
    let context = TaskContext::default();

    match pattern_type {
        "tool_sequence" => Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool1".to_string(), "tool2".to_string()],
            context,
            success_rate,
            avg_latency: chrono::Duration::milliseconds(100),
            occurrence_count: 5,
            effectiveness: PatternEffectiveness::new(),
        },
        "error_recovery" => Pattern::ErrorRecovery {
            id: Uuid::new_v4(),
            error_type: "connection_error".to_string(),
            recovery_steps: vec!["retry".to_string(), "backoff".to_string()],
            success_rate,
            context,
            effectiveness: PatternEffectiveness::new(),
        },
        _ => panic!("Unknown pattern type: {}", pattern_type),
    }
}

/// Create a test heuristic
pub fn create_test_heuristic(condition: &str, action: &str) -> Heuristic {
    Heuristic::new(condition.to_string(), action.to_string(), 0.8)
}

/// Generate test reward score
pub fn create_test_reward(total: f32) -> RewardScore {
    RewardScore {
        total,
        base: total * 0.7,
        efficiency: 1.2,
        complexity_bonus: 1.1,
        learning_bonus: 1.0,
        quality_multiplier: 1.0,
    }
}

/// Generate test reflection
pub fn create_test_reflection() -> Reflection {
    Reflection {
        successes: vec!["Completed successfully".to_string()],
        improvements: vec!["Could be faster".to_string()],
        insights: vec!["Use caching".to_string()],
        generated_at: chrono::Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_episode() {
        let episode = create_test_episode("Test task");
        assert_eq!(episode.task_description, "Test task");
        assert!(!episode.is_complete());
    }

    #[test]
    fn test_create_completed_episode() {
        let episode = create_completed_episode("Test", true);
        assert!(episode.is_complete());
        assert_eq!(episode.steps.len(), 3);
    }

    #[test]
    fn test_create_test_pattern() {
        let pattern = create_test_pattern("tool_sequence", 0.9);
        assert_eq!(pattern.success_rate(), 0.9);
    }
}

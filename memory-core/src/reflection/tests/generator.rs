//!
//! Tests for the reflection generator module
//!

use crate::ExecutionStep;
use crate::episode::Episode;
use crate::reflection::ReflectionGenerator;
use crate::types::{ExecutionResult, TaskContext, TaskOutcome, TaskType};

fn successful_step(step_number: usize, tool: &str, action: &str) -> ExecutionStep {
    let mut step = ExecutionStep::new(step_number, tool.to_string(), action.to_string());
    step.result = Some(ExecutionResult::Success {
        output: "Success".to_string(),
    });
    step.latency_ms = 100;
    step.tokens_used = Some(50);
    step
}

fn failed_step(step_number: usize, tool: &str, action: &str, error_msg: &str) -> ExecutionStep {
    let mut step = ExecutionStep::new(step_number, tool.to_string(), action.to_string());
    step.result = Some(ExecutionResult::Error {
        message: error_msg.to_string(),
    });
    step.latency_ms = 200;
    step.tokens_used = Some(25);
    step
}

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

    let episode = create_test_episode(
        "Test successful task",
        TaskType::Testing,
        steps,
        Some(outcome),
    );
    let generator = ReflectionGenerator::new();
    let reflection = generator.generate(&episode);

    assert!(!reflection.successes.is_empty());
    assert!(
        reflection
            .successes
            .iter()
            .any(|s| s.contains("Successfully completed"))
    );
    assert!(reflection.successes.iter().any(|s| s.contains("artifact")));
}

#[test]
fn test_generate_reflection_failed_episode() {
    let steps = vec![
        successful_step(1, "test_runner", "Run unit tests"),
        failed_step(
            2,
            "code_review",
            "Review code quality",
            "Code review failed",
        ),
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
    assert!(
        reflection.successes.is_empty()
            || reflection.improvements.is_empty()
            || reflection.insights.is_empty()
    );
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

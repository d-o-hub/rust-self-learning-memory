//!
//! Tests for the improvement analyzer module
//!

use crate::episode::Episode;
use crate::reflection::improvement_analyzer;
use crate::types::{ExecutionResult, TaskContext, TaskOutcome, TaskType};
use crate::ExecutionStep;

use super::create_test_episode;
use super::successful_step;

fn failed_step(step_number: usize, tool: &str, action: &str, error_msg: &str) -> ExecutionStep {
    let mut step = ExecutionStep::new(step_number, tool.to_string(), action.to_string());
    step.result = Some(ExecutionResult::Error {
        message: error_msg.to_string(),
    });
    step.latency_ms = 200;
    step.tokens_used = Some(25);
    step
}

fn successful_step_with_latency(
    step_number: usize,
    tool: &str,
    action: &str,
    latency_ms: u64,
) -> ExecutionStep {
    let mut step = successful_step(step_number, tool, action);
    step.latency_ms = latency_ms;
    step
}

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
    let steps: Vec<_> = (1..60)
        .map(|i| successful_step(i, "tool", &format!("action{i}")))
        .collect();

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
    let steps: Vec<_> = (1..8)
        .map(|i| successful_step(i, "same_tool", "action"))
        .collect();

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

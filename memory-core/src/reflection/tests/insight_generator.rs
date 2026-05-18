//!
//! Tests for the insight generator module
//!

use crate::ExecutionStep;
use crate::episode::Episode;
use crate::reflection::insight_generator;
use crate::types::{ComplexityLevel, ExecutionResult, TaskContext, TaskOutcome, TaskType};

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

    assert!(
        insights
            .iter()
            .any(|i| i.contains("reliable") || i.contains("All steps"))
    );
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
    let steps: Vec<_> = (1..8)
        .map(|i| successful_step(i, &format!("tool{i}"), "action"))
        .collect();

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
    let steps: Vec<_> = (1..5)
        .map(|i| successful_step(i, "same_tool", "action"))
        .collect();

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
    let context = TaskContext {
        complexity: ComplexityLevel::Simple,
        ..Default::default()
    };

    let steps = (1..12)
        .map(|i| successful_step(i, &format!("tool{i}"), &format!("action{i}")))
        .collect::<Vec<_>>();

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

    assert!(
        insights
            .iter()
            .any(|i| i.contains("more steps than typical"))
    );
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

    assert!(
        insights
            .iter()
            .any(|i| i.contains("learning") || i.contains("adaptability"))
    );
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
    let context = TaskContext {
        domain: "testing".to_string(),
        language: Some("Rust".to_string()),
        ..Default::default()
    };

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

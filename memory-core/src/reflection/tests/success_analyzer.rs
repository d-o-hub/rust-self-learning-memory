//!
//! Tests for the success analyzer module
//!

use crate::episode::Episode;
use crate::reflection::success_analyzer;
use crate::types::{TaskContext, TaskOutcome, TaskType};

use super::create_test_episode;
use super::failed_step;
use super::successful_step;

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
    assert!(successes
        .iter()
        .any(|s| s.contains("Successfully completed")));
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
    let context = TaskContext {
        language: Some("Rust".to_string()),
        domain: "testing".to_string(),
        tags: vec!["unit".to_string(), "integration".to_string()],
        ..Default::default()
    };

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

    assert!(patterns
        .iter()
        .any(|p| p.contains("Rust-specific") || p.contains("domain knowledge")));
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

    assert!(patterns
        .iter()
        .any(|p| p.contains("expertise") || p.contains("minimalist")));
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

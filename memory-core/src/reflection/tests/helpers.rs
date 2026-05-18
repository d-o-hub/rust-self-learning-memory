//!
//! Tests for the reflection helper functions
//!

use crate::reflection::helpers;
use crate::types::{ExecutionResult, TaskType};
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

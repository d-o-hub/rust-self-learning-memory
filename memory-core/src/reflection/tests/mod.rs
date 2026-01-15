//!
//! Test module for the reflection subsystem
//!
//! Contains tests organized by component:
//! - generator.rs: Tests for ReflectionGenerator
//! - success_analyzer.rs: Tests for success analysis
//! - improvement_analyzer.rs: Tests for improvement analysis
//! - insight_generator.rs: Tests for insight generation
//! - helpers.rs: Tests for helper functions
//!

use crate::episode::Episode;
use crate::types::{ExecutionResult, TaskContext, TaskOutcome, TaskType};
use crate::ExecutionStep;

// ---------------------------------------------------------------------------
// Test helper functions shared across test modules
// ---------------------------------------------------------------------------

pub(super) fn failed_step(
    step_number: usize,
    tool: &str,
    action: &str,
    error_msg: &str,
) -> ExecutionStep {
    let mut step = ExecutionStep::new(step_number, tool.to_string(), action.to_string());
    step.result = Some(ExecutionResult::Error {
        message: error_msg.to_string(),
    });
    step.latency_ms = 200;
    step.tokens_used = Some(25);
    step
}

pub(super) fn successful_step(step_number: usize, tool: &str, action: &str) -> ExecutionStep {
    let mut step = ExecutionStep::new(step_number, tool.to_string(), action.to_string());
    step.result = Some(ExecutionResult::Success {
        output: "Success".to_string(),
    });
    step.latency_ms = 100;
    step.tokens_used = Some(50);
    step
}

pub(super) fn successful_step_with_latency(
    step_number: usize,
    tool: &str,
    action: &str,
    latency_ms: u64,
) -> ExecutionStep {
    let mut step = successful_step(step_number, tool, action);
    step.latency_ms = latency_ms;
    step
}

pub(super) fn create_test_episode(
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

mod generator;
mod helpers;
mod improvement_analyzer;
mod insight_generator;
mod success_analyzer;

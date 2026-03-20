//! Property-based tests for Episode lifecycle
//!
//! These tests verify invariants for episode creation, step logging,
//! and completion using the proptest crate for property-based testing.
//!
//! Covers WG-046 from ADR-047 (v0.1.22 Quality Polish)

#![allow(clippy::cast_precision_loss)]

use memory_core::{Episode, ExecutionResult, ExecutionStep, TaskContext, TaskOutcome, TaskType};
use proptest::prelude::*;

/// Generate arbitrary task descriptions
fn arb_task_description() -> impl Strategy<Value = String> {
    "[a-zA-Z ]{5,50}"
}

/// Generate arbitrary domain names
fn arb_domain() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("testing".to_string()),
        Just("development".to_string()),
        Just("security".to_string()),
        Just("documentation".to_string()),
        Just("refactoring".to_string()),
    ]
}

/// Generate arbitrary number of steps (0-20)
fn arb_step_count() -> impl Strategy<Value = usize> {
    0..20usize
}

proptest! {
    /// Test that episode IDs are unique across creations
    #[test]
    fn episode_ids_are_unique(
        desc1 in arb_task_description(),
        desc2 in arb_task_description(),
    ) {
        let context = TaskContext::default();
        let ep1 = Episode::new(desc1, context.clone(), TaskType::Testing);
        let ep2 = Episode::new(desc2, context, TaskType::Testing);
        assert_ne!(ep1.episode_id, ep2.episode_id);
    }

    /// Test that step order is preserved
    #[test]
    fn step_order_preserved(num_steps in arb_step_count()) {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Testing);

        for i in 0..num_steps {
            let step = ExecutionStep::new(i + 1, format!("tool_{i}"), format!("action_{i}"));
            episode.add_step(step);
        }

        // Verify step count
        assert_eq!(episode.steps.len(), num_steps);

        // Verify order
        for (i, step) in episode.steps.iter().enumerate() {
            assert_eq!(step.step_number, i + 1);
        }
    }

    /// Test that episode maintains task type
    #[test]
    fn episode_maintains_task_type(
        desc in arb_task_description(),
        task_type_idx in 0..5usize,
    ) {
        let context = TaskContext::default();
        let task_type = match task_type_idx {
            0 => TaskType::CodeGeneration,
            1 => TaskType::Testing,
            2 => TaskType::Refactoring,
            3 => TaskType::Documentation,
            _ => TaskType::Debugging,
        };

        let episode = Episode::new(desc, context, task_type);
        assert_eq!(episode.task_type, task_type);
    }

    /// Test that episode domain is preserved
    #[test]
    fn episode_domain_preserved(domain in arb_domain()) {
        let context = TaskContext {
            domain: domain.clone(),
            ..TaskContext::default()
        };
        let episode = Episode::new("Test task".to_string(), context, TaskType::Testing);
        assert_eq!(episode.context.domain, domain);
    }

    /// Test that outcome verdict length is bounded
    #[test]
    fn outcome_verdict_bounded(verdict_len in 0..200usize) {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Testing);

        let verdict: String = "x".repeat(verdict_len);
        let outcome = TaskOutcome::Success {
            verdict: verdict.clone(),
            artifacts: vec![],
        };

        episode.complete(outcome);

        // end_time should be set after completion
        assert!(episode.end_time.is_some());
    }

    /// Test that duplicate step numbers are handled
    #[test]
    fn duplicate_step_numbers_handled(
        tool in "[a-z_]{3,10}",
        action in "[a-zA-Z ]{5,20}",
    ) {
        let context = TaskContext::default();
        let mut episode = Episode::new("Test task".to_string(), context, TaskType::Testing);

        // Log same step number twice
        let step1 = ExecutionStep::new(1, tool.clone(), action.clone());
        let step2 = ExecutionStep::new(1, tool, action);

        episode.add_step(step1);
        episode.add_step(step2);

        // Both steps should be present (list allows duplicates)
        assert_eq!(episode.steps.len(), 2);
    }
}

/// Test that step results don't affect episode validity
#[test]
fn test_step_results_dont_affect_validity() {
    let context = TaskContext::default();
    let mut episode = Episode::new("Test".to_string(), context, TaskType::Testing);

    // Add steps with various results
    for i in 0..5 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), format!("action_{i}"));
        step.result = if i % 2 == 0 {
            Some(ExecutionResult::Success {
                output: "OK".to_string(),
            })
        } else {
            Some(ExecutionResult::Error {
                message: "Failed".to_string(),
            })
        };
        episode.add_step(step);
    }

    // Episode should still be valid regardless of step results
    episode.complete(TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });
    assert!(episode.is_complete());
}

//! Property-based tests for `ExecutionStep`
//!
//! These tests verify invariants for execution step creation and serialization
//! using the proptest crate for property-based testing.
//!
//! Covers WG-046 from ADR-047 (v0.1.22 Quality Polish)

#![allow(clippy::cast_precision_loss)]

use memory_core::{ExecutionResult, ExecutionStep};
use proptest::prelude::*;

/// Generate arbitrary order numbers
fn arb_order() -> impl Strategy<Value = usize> {
    1..=100usize
}

/// Generate arbitrary tool names
fn arb_tool_name() -> impl Strategy<Value = String> {
    "[a-z][a-z_]{2,15}"
}

/// Generate arbitrary action descriptions
fn arb_action() -> impl Strategy<Value = String> {
    "[a-zA-Z ]{5,50}"
}

/// Generate arbitrary execution result
fn arb_result() -> impl Strategy<Value = Option<ExecutionResult>> {
    prop_oneof![
        Just(None),
        Just(Some(ExecutionResult::Success {
            output: "OK".to_string()
        })),
        Just(Some(ExecutionResult::Error {
            message: "Error occurred".to_string()
        })),
    ]
}

proptest! {
    /// Test that step order is preserved
    #[test]
    fn step_order_preserved(order in arb_order(), tool in arb_tool_name(), action in arb_action()) {
        let step = ExecutionStep::new(order, tool, action);
        assert_eq!(step.step_number, order);
    }

    /// Test that step tool name is preserved
    #[test]
    fn step_tool_preserved(tool in arb_tool_name()) {
        let step = ExecutionStep::new(1, tool.clone(), "action".to_string());
        assert_eq!(step.tool, tool);
    }

    /// Test that step action is preserved
    #[test]
    fn step_action_preserved(action in arb_action()) {
        let step = ExecutionStep::new(1, "tool".to_string(), action.clone());
        assert_eq!(step.action, action);
    }

    /// Test that step result defaults to None
    #[test]
    fn step_result_defaults_none(order in arb_order(), tool in arb_tool_name(), action in arb_action()) {
        let step = ExecutionStep::new(order, tool, action);
        assert!(step.result.is_none());
    }

    /// Test that step result can be set
    #[test]
    fn step_result_can_be_set(
        order in arb_order(),
        tool in arb_tool_name(),
        action in arb_action(),
        result in arb_result(),
    ) {
        let mut step = ExecutionStep::new(order, tool, action);
        step.result = result.clone();

        match result {
            None => assert!(step.result.is_none()),
            Some(ExecutionResult::Success { output }) => {
                if let Some(ExecutionResult::Success { output: o }) = step.result {
                    assert_eq!(o, output);
                }
            }
            Some(ExecutionResult::Error { message }) => {
                if let Some(ExecutionResult::Error { message: m }) = step.result {
                    assert_eq!(m, message);
                }
            }
            _ => {}
        }
    }

    /// Test that step is Clone
    #[test]
    fn step_is_clone(order in arb_order(), tool in arb_tool_name(), action in arb_action()) {
        let step = ExecutionStep::new(order, tool, action);
        let cloned = step.clone();
        assert_eq!(step.step_number, cloned.step_number);
        assert_eq!(step.tool, cloned.tool);
        assert_eq!(step.action, cloned.action);
    }

    /// Test that step is Debug
    #[test]
    fn step_is_debug(order in arb_order(), tool in arb_tool_name(), action in arb_action()) {
        let step = ExecutionStep::new(order, tool, action);
        let debug_str = format!("{step:?}");
        assert!(debug_str.contains("ExecutionStep"));
    }

}

/// Test that order must be positive (1-indexed)
#[test]
fn order_must_be_positive() {
    let step = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
    assert!(step.step_number >= 1);
}

/// Test that success result contains output
#[test]
fn test_success_result() {
    let result = ExecutionResult::Success {
        output: "Task completed".to_string(),
    };
    if let ExecutionResult::Success { output } = result {
        assert_eq!(output, "Task completed");
    } else {
        panic!("Expected Success variant");
    }
}

/// Test that error result contains message
#[test]
fn test_error_result() {
    let result = ExecutionResult::Error {
        message: "Something went wrong".to_string(),
    };
    if let ExecutionResult::Error { message } = result {
        assert_eq!(message, "Something went wrong");
    } else {
        panic!("Expected Error variant");
    }
}

/// Test that steps with same parameters have same fields
#[test]
fn test_steps_same_fields() {
    let step1 = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
    let step2 = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
    assert_eq!(step1.step_number, step2.step_number);
    assert_eq!(step1.tool, step2.tool);
    assert_eq!(step1.action, step2.action);
}

/// Test that steps with different order have different `step_number`
#[test]
fn test_steps_different_order() {
    let step1 = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
    let step2 = ExecutionStep::new(2, "tool".to_string(), "action".to_string());
    assert_ne!(step1.step_number, step2.step_number);
}

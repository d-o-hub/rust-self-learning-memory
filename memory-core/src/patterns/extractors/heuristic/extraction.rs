//! Helper functions for heuristic extraction
//!
//! Provides utilities for identifying decision points and extracting
//! condition-action pairs from execution steps.

use crate::episode::{Episode, ExecutionStep};
use anyhow::Result;

/// Check if an action indicates a decision point
///
/// Decision points are steps that involve conditional logic, verification,
/// or explicit decision-making actions.
///
/// # Arguments
///
/// * `action` - The action string to analyze
///
/// # Returns
///
/// `true` if the action represents a decision point, `false` otherwise
///
/// # Examples
///
/// ```
/// # use memory_core::patterns::extractors::heuristic::is_decision_action;
/// assert!(is_decision_action("Check if input is valid"));
/// assert!(is_decision_action("Verify the output"));
/// assert!(is_decision_action("When ready, proceed"));
/// assert!(!is_decision_action("Read file"));
/// ```
pub fn is_decision_action(action: &str) -> bool {
    let action_lower = action.to_lowercase();
    action_lower.contains("if ")
        || action_lower.contains("when ")
        || action_lower.contains("check ")
        || action_lower.contains("verify ")
        || action_lower.contains("validate ")
        || action_lower.contains("ensure ")
        || action_lower.starts_with("decide ")
        || action_lower.starts_with("determine ")
}

/// Extract the condition from a decision point
///
/// Combines the episode context with the decision step to form the condition
/// part of a heuristic rule.
///
/// # Arguments
///
/// * `episode` - The episode containing the decision
/// * `step` - The execution step representing the decision
/// * `_idx` - Index of the step (currently unused, reserved for future use)
///
/// # Returns
///
/// A string describing the condition under which the decision was made
pub fn extract_condition(episode: &Episode, step: &ExecutionStep, _idx: usize) -> Result<String> {
    // Combine context information with the decision action
    let mut condition_parts = Vec::new();

    // Add domain context
    if !episode.context.domain.is_empty() {
        condition_parts.push(format!("In {} domain", episode.context.domain));
    }

    // Add language context if available
    if let Some(lang) = &episode.context.language {
        condition_parts.push(format!("using {}", lang));
    }

    // Add the decision action itself
    condition_parts.push(step.action.clone());

    Ok(condition_parts.join(", "))
}

/// Extract the action taken after a decision point
///
/// Uses the next step as the action, or the current tool if no next step exists.
///
/// # Arguments
///
/// * `episode` - The episode containing the decision
/// * `step` - The execution step representing the decision
/// * `idx` - Index of the step in the episode
///
/// # Returns
///
/// A string describing the action that followed the decision
pub fn extract_action(episode: &Episode, step: &ExecutionStep, idx: usize) -> Result<String> {
    // Try to use the next step as the action
    if let Some(next_step) = episode.steps.get(idx + 1) {
        Ok(format!("Use {} to {}", next_step.tool, next_step.action))
    } else {
        // Fall back to using the current step's tool
        Ok(format!("Apply {}", step.tool))
    }
}

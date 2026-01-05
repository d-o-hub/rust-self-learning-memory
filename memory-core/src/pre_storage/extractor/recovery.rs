//! Error recovery pattern extraction logic.

use crate::episode::{Episode, ExecutionStep};
use crate::types::ExecutionResult;

/// Helper to extract error message from a failed step.
pub(super) fn get_error_message(step: &ExecutionStep) -> String {
    match &step.result {
        Some(ExecutionResult::Error { message }) => {
            // Truncate long error messages
            if message.len() > 50 {
                format!("{}...", &message[..50])
            } else {
                message.clone()
            }
        }
        Some(ExecutionResult::Timeout) => "Timeout".to_string(),
        _ => format!("Error in {}", step.action),
    }
}

/// Extract multi-step error recovery patterns.
///
/// Helper function to find longer recovery sequences.
pub(super) fn extract_multi_step_recovery(episode: &Episode, patterns: &mut Vec<String>) {
    let mut i = 0;
    while i < episode.steps.len() {
        let step_failed = !episode.steps[i].is_success();
        if step_failed {
            let error_step = &episode.steps[i];
            let error_msg = get_error_message(error_step);

            // Count successful recovery steps
            let mut recovery_steps = Vec::new();
            let mut j = i + 1;
            while j < episode.steps.len()
                && episode.steps[j].is_success()
                && recovery_steps.len() < 3
            {
                recovery_steps.push(&episode.steps[j]);
                j += 1;
            }

            if recovery_steps.len() >= 2 {
                let recovery_desc = recovery_steps
                    .iter()
                    .map(|s| s.action.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                patterns.push(format!("{error_msg} -> [{recovery_desc}]"));
            }

            i = j;
        } else {
            i += 1;
        }
    }
}

/// Extract error recovery patterns from failed steps followed by successes.
///
/// Identifies how errors were detected and resolved, creating
/// reusable recovery strategies.
pub(super) fn extract_error_recovery_patterns(episode: &Episode) -> Vec<String> {
    let mut patterns = Vec::new();

    // Look for error -> success transitions
    for window in episode.steps.windows(2) {
        let error_step = &window[0];
        let recovery_step = &window[1];

        if !error_step.is_success() && recovery_step.is_success() {
            let error_msg = get_error_message(error_step);
            patterns.push(format!(
                "{} -> {} ({})",
                error_msg, recovery_step.action, recovery_step.tool
            ));
        }
    }

    // Look for longer recovery sequences (error -> multiple recovery steps)
    extract_multi_step_recovery(episode, &mut patterns);

    // Deduplicate patterns
    let mut seen = std::collections::HashSet::new();
    patterns.retain(|p| seen.insert(p.clone()));

    // Limit to most relevant patterns
    patterns.truncate(10);
    patterns
}

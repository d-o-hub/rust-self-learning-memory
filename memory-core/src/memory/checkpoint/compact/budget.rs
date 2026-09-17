//! Byte-budget enforcement and text truncation for compact handoff packs.
//!
//! Split out of `compact.rs` to keep that module within the 500 LOC limit
//! (AGENTS.md). The budget contract itself is documented on the parent module.

use super::{CompactHandoff, HandoffBudget};
use crate::types::ExecutionResult;

/// Truncate to a character boundary, reporting whether a cut happened.
pub(super) fn truncate_chars(text: &str, max_chars: usize) -> (String, bool) {
    if text.chars().count() <= max_chars {
        return (text.to_string(), false);
    }
    let cut: String = text.chars().take(max_chars).collect();
    (cut, true)
}

/// Bounded one-line summary of a step result, plus whether it was cut.
pub(super) fn summarize_result(
    result: &Option<ExecutionResult>,
    max_chars: usize,
) -> (Option<String>, bool) {
    let text = match result {
        None => return (None, false),
        Some(ExecutionResult::Success { output }) => output.clone(),
        Some(ExecutionResult::Error { message }) => format!("error: {message}"),
        Some(ExecutionResult::Timeout) => "timeout".to_string(),
    };
    let (cut, was_cut) = truncate_chars(&text, max_chars);
    (Some(cut), was_cut)
}

/// Drop whole items in reverse priority until the JSON fits `max_bytes`.
/// Counts flow back into the omission metadata; the goal truncates last.
/// Best-effort when even the skeleton exceeds the budget (documented).
pub(super) fn enforce_byte_budget(pack: &mut CompactHandoff, budget: &HandoffBudget) {
    loop {
        if pack.payload_bytes() <= budget.max_bytes {
            return;
        }
        if pack.artifact_refs.pop().is_some() {
            pack.omitted.omitted_artifacts += 1;
        } else if pack.heuristic_refs.pop().is_some() {
            pack.omitted.omitted_heuristics += 1;
        } else if pack.pattern_refs.pop().is_some() {
            pack.omitted.omitted_patterns += 1;
        } else if !pack.evidence_excerpts.is_empty() {
            // Excerpts are chronological; the oldest goes first so the most
            // recent context (highest resume value) survives the longest.
            pack.evidence_excerpts.remove(0);
            pack.omitted.omitted_steps += 1;
        } else if pack.pending_actions.pop().is_some() {
            pack.omitted.omitted_pending_actions += 1;
        } else if pack.decisions.pop().is_some() {
            pack.omitted.omitted_decisions += 1;
        } else if pack.verified_findings.pop().is_some() {
            pack.omitted.omitted_findings += 1;
        } else if !pack.current_goal.is_empty() {
            // Last resort: halve the goal (UTF-8 safe) until it fits.
            let half = pack.current_goal.chars().count() / 2;
            let (cut, _) = truncate_chars(&pack.current_goal, half);
            pack.current_goal = cut;
            if !pack
                .omitted
                .truncated_fields
                .contains(&"current_goal".to_string())
            {
                pack.omitted
                    .truncated_fields
                    .push("current_goal".to_string());
            }
            if half == 0 {
                return;
            }
        } else {
            return;
        }
    }
}

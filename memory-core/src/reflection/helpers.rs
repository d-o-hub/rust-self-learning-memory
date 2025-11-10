//! Helper utilities for reflection analysis

use crate::episode::Episode;

/// Count unique tools used in episode
pub(super) fn count_unique_tools(episode: &Episode) -> usize {
    let unique: std::collections::HashSet<_> = episode.steps.iter().map(|s| &s.tool).collect();
    unique.len()
}

/// Calculate average step latency
pub(super) fn calculate_average_latency(episode: &Episode) -> Option<u64> {
    if episode.steps.is_empty() {
        return None;
    }

    let total_latency: u64 = episode.steps.iter().map(|s| s.latency_ms).sum();
    Some(total_latency / episode.steps.len() as u64)
}

/// Detect if episode shows error recovery pattern
pub(super) fn detect_error_recovery(episode: &Episode) -> bool {
    for i in 0..episode.steps.len().saturating_sub(1) {
        if !episode.steps[i].is_success() && episode.steps[i + 1].is_success() {
            return true;
        }
    }
    false
}

/// Detect if episode shows iterative refinement pattern
pub(super) fn detect_iterative_refinement(episode: &Episode) -> bool {
    // Look for pattern: fail -> adjust -> succeed
    let mut refinement_count = 0;

    for i in 0..episode.steps.len().saturating_sub(1) {
        let has_error = !episode.steps[i].is_success();
        let has_recovery = episode.steps[i + 1].is_success();

        if has_error && has_recovery {
            refinement_count += 1;
        }
    }

    refinement_count >= 2
}

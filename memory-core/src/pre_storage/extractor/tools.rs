//! Tool combination extraction logic.

use crate::episode::Episode;

/// Extract effective tool combinations from sequential steps.
///
/// Identifies sequences of 2+ tools used together successfully,
/// which represent reusable patterns.
///
/// # Arguments
///
/// * `episode` - The episode to extract from
/// * `min_sequence_length` - Minimum number of tools for a valid combination
pub(super) fn extract_tool_combinations(
    episode: &Episode,
    min_sequence_length: usize,
) -> Vec<Vec<String>> {
    let mut combinations = Vec::new();

    if episode.steps.len() < min_sequence_length {
        return combinations;
    }

    // Find sequences of successful steps
    let mut current_sequence = Vec::new();

    for step in &episode.steps {
        if step.is_success() {
            current_sequence.push(step.tool.clone());
        } else {
            // Sequence broken by failure
            if current_sequence.len() >= min_sequence_length {
                combinations.push(current_sequence.clone());
            }
            current_sequence.clear();
        }
    }

    // Add final sequence if long enough
    if current_sequence.len() >= min_sequence_length {
        combinations.push(current_sequence);
    }

    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    combinations.retain(|combo| {
        let key = combo.join("->");
        seen.insert(key)
    });

    // Limit to most relevant combinations
    combinations.truncate(5);
    combinations
}

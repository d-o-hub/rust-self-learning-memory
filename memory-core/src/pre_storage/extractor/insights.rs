//! Key insights extraction logic.

use crate::episode::Episode;

/// Extract key insights from reflection and outcome.
///
/// Pulls out important learnings and discoveries that can be
/// applied to future tasks.
pub(super) fn extract_key_insights(episode: &Episode) -> Vec<String> {
    let mut insights = Vec::new();

    // Extract from reflection
    if let Some(ref reflection) = episode.reflection {
        // Add notable successes (limit to most important)
        for success in reflection.successes.iter().take(3) {
            if success.len() > 10 {
                // Filter out trivial successes
                insights.push(format!("Success: {success}"));
            }
        }

        // Add all insights from reflection (these are already curated)
        for insight in &reflection.insights {
            insights.push(format!("Insight: {insight}"));
        }

        // Add key improvements (limit to most important)
        for improvement in reflection.improvements.iter().take(2) {
            if improvement.len() > 10 {
                insights.push(format!("Improvement: {improvement}"));
            }
        }
    }

    // Extract from outcome artifacts (if meaningful)
    if let Some(crate::types::TaskOutcome::Success { artifacts, .. }) = &episode.outcome {
        if !artifacts.is_empty() && artifacts.len() <= 5 {
            // Only include if reasonable number of artifacts
            insights.push(format!("Artifacts produced: {}", artifacts.join(", ")));
        }
    }

    // Deduplicate insights
    let mut seen = std::collections::HashSet::new();
    insights.retain(|i| seen.insert(i.clone()));

    // Limit to most relevant insights
    insights.truncate(15);
    insights
}

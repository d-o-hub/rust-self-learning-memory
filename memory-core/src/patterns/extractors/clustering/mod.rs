//! Pattern clustering and deduplication
//!
//! Provides functionality to deduplicate similar patterns using clustering.

use crate::pattern::Pattern;
use std::collections::HashSet;

mod cluster_types;

/// Deduplicate patterns by removing exact duplicates (same ID)
#[must_use]
pub fn deduplicate_patterns(patterns: Vec<Pattern>) -> Vec<Pattern> {
    let mut unique_patterns = Vec::new();
    let mut seen_ids = HashSet::new();

    for pattern in patterns {
        let id = pattern.id();
        if !seen_ids.contains(&id) {
            seen_ids.insert(id);
            unique_patterns.push(pattern);
        }
    }

    // Sort by success rate (descending)
    unique_patterns.sort_by(|a, b| {
        b.success_rate()
            .partial_cmp(&a.success_rate())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    unique_patterns
}

/// Cluster similar patterns together to avoid redundancy
///
/// This performs a simple similarity-based clustering:
/// - `ToolSequence` patterns with same tools are merged
/// - `DecisionPoint` patterns with same condition are merged
/// - `ErrorRecovery` patterns with same error type are merged
/// - `ContextPattern` patterns with overlapping features are merged
#[must_use]
pub fn cluster_similar_patterns(patterns: Vec<Pattern>) -> Vec<Pattern> {
    let mut clustered = Vec::new();

    // Group by pattern type first
    let mut tool_sequences = Vec::new();
    let mut decision_points = Vec::new();
    let mut error_recoveries = Vec::new();
    let mut context_patterns = Vec::new();

    for pattern in patterns {
        match pattern {
            Pattern::ToolSequence { .. } => tool_sequences.push(pattern),
            Pattern::DecisionPoint { .. } => decision_points.push(pattern),
            Pattern::ErrorRecovery { .. } => error_recoveries.push(pattern),
            Pattern::ContextPattern { .. } => context_patterns.push(pattern),
        }
    }

    // Cluster each type
    clustered.extend(cluster_types::cluster_tool_sequences(tool_sequences));
    clustered.extend(cluster_types::cluster_decision_points(decision_points));
    clustered.extend(cluster_types::cluster_error_recoveries(error_recoveries));
    clustered.extend(cluster_types::cluster_context_patterns(context_patterns));

    // Final deduplication and sorting
    deduplicate_patterns(clustered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TaskContext;
    use chrono::Duration;
    use uuid::Uuid;

    #[test]
    fn test_deduplicate_patterns() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        let pattern1 = Pattern::ToolSequence {
            id: id1,
            tools: vec!["tool1".to_string()],
            context: TaskContext::default(),
            success_rate: 0.9,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 1,
            effectiveness: crate::pattern::PatternEffectiveness::new(),
        };

        let pattern2 = Pattern::ToolSequence {
            id: id2,
            tools: vec!["tool2".to_string()],
            context: TaskContext::default(),
            success_rate: 0.8,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 1,
            effectiveness: crate::pattern::PatternEffectiveness::new(),
        };

        let test_patterns = vec![pattern1.clone(), pattern2.clone(), pattern1.clone()];
        let deduped = deduplicate_patterns(test_patterns);

        assert_eq!(deduped.len(), 2);
        // Should be sorted by success rate
        assert_eq!(deduped[0].success_rate(), 0.9);
        assert_eq!(deduped[1].success_rate(), 0.8);
    }
}

//! Pattern clustering and deduplication - Type-specific clustering.
//!
//! Provides clustering logic for specific pattern types.

use crate::pattern::Pattern;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Cluster tool sequence patterns
pub fn cluster_tool_sequences(patterns: Vec<Pattern>) -> Vec<Pattern> {
    let mut clusters: HashMap<Vec<String>, Vec<Pattern>> = HashMap::new();

    for pattern in patterns {
        if let Pattern::ToolSequence { tools, .. } = &pattern {
            clusters.entry(tools.clone()).or_default().push(pattern);
        }
    }

    // Keep the best pattern from each cluster
    clusters
        .into_values()
        .filter_map(|mut cluster| {
            if cluster.is_empty() {
                None
            } else {
                // Sort by success rate and take the best
                cluster.sort_by(|a, b| {
                    b.success_rate()
                        .partial_cmp(&a.success_rate())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                cluster.into_iter().next()
            }
        })
        .collect()
}

/// Cluster decision point patterns
pub fn cluster_decision_points(patterns: Vec<Pattern>) -> Vec<Pattern> {
    let mut clusters: HashMap<String, Vec<Pattern>> = HashMap::new();

    for pattern in patterns {
        if let Pattern::DecisionPoint { condition, .. } = &pattern {
            // Normalize condition for clustering
            let normalized = condition.to_lowercase().trim().to_string();
            clusters.entry(normalized).or_default().push(pattern);
        }
    }

    // Keep the best pattern from each cluster
    clusters
        .into_values()
        .filter_map(|mut cluster| {
            if cluster.is_empty() {
                None
            } else {
                cluster.sort_by(|a, b| {
                    b.success_rate()
                        .partial_cmp(&a.success_rate())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                cluster.into_iter().next()
            }
        })
        .collect()
}

/// Cluster error recovery patterns
pub fn cluster_error_recoveries(patterns: Vec<Pattern>) -> Vec<Pattern> {
    let mut clusters: HashMap<String, Vec<Pattern>> = HashMap::new();

    for pattern in patterns {
        if let Pattern::ErrorRecovery { error_type, .. } = &pattern {
            // Normalize error type for clustering
            let normalized = error_type.to_lowercase().trim().to_string();
            clusters.entry(normalized).or_default().push(pattern);
        }
    }

    // Merge patterns with same error type
    clusters
        .into_values()
        .filter_map(|cluster| {
            if cluster.is_empty() {
                None
            } else if cluster.len() == 1 {
                cluster.into_iter().next()
            } else {
                // Merge multiple patterns with same error type
                merge_error_recovery_patterns(cluster)
            }
        })
        .collect()
}

/// Merge multiple error recovery patterns
fn merge_error_recovery_patterns(patterns: Vec<Pattern>) -> Option<Pattern> {
    if patterns.is_empty() {
        return None;
    }

    // Extract data from all patterns
    let mut all_recovery_steps = Vec::new();
    let mut total_success_rate = 0.0;
    let mut count = 0;
    let first_pattern = &patterns[0];

    for pattern in &patterns {
        if let Pattern::ErrorRecovery {
            recovery_steps,
            success_rate,
            ..
        } = pattern
        {
            all_recovery_steps.extend(recovery_steps.clone());
            total_success_rate += success_rate;
            count += 1;
        }
    }

    // Deduplicate recovery steps
    let mut unique_steps = Vec::new();
    let mut seen = HashSet::new();
    for step in all_recovery_steps {
        if !seen.contains(&step) {
            seen.insert(step.clone());
            unique_steps.push(step);
        }
    }

    // Create merged pattern
    if let Pattern::ErrorRecovery {
        error_type,
        context,
        ..
    } = first_pattern
    {
        Some(Pattern::ErrorRecovery {
            id: Uuid::new_v4(),
            error_type: error_type.clone(),
            recovery_steps: unique_steps,
            success_rate: total_success_rate / count as f32,
            context: context.clone(),
            effectiveness: crate::pattern::PatternEffectiveness::new(),
        })
    } else {
        None
    }
}

/// Cluster context patterns
pub fn cluster_context_patterns(patterns: Vec<Pattern>) -> Vec<Pattern> {
    if patterns.is_empty() {
        return patterns;
    }

    // For context patterns, we merge those with high feature overlap
    // Keep a reference for lookups while we consume the vector
    let patterns_ref = patterns.clone();
    let mut result = Vec::new();
    let mut remaining: Vec<_> = patterns.into_iter().enumerate().collect();
    let mut similar_indices: Vec<usize>;

    while !remaining.is_empty() {
        let (base_idx, base_pattern) = remaining.remove(0);
        similar_indices = vec![base_idx];

        // Find similar patterns - use indices to avoid cloning
        remaining.retain(|(idx, p)| {
            if are_context_patterns_similar(&base_pattern, p) {
                similar_indices.push(*idx);
                false // Remove from remaining
            } else {
                true // Keep in remaining
            }
        });

        // Collect patterns to merge - this is the unavoidable clone
        let mut similar = Vec::new();
        for idx in similar_indices {
            if let Some((_, pattern)) = patterns_ref.iter().enumerate().find(|(i, _)| *i == idx) {
                similar.push(pattern.clone());
            }
        }

        // Merge similar patterns
        if let Some(merged) = merge_context_patterns(similar) {
            result.push(merged);
        }
    }

    result
}

/// Check if two context patterns are similar
fn are_context_patterns_similar(p1: &Pattern, p2: &Pattern) -> bool {
    if let (
        Pattern::ContextPattern {
            context_features: f1,
            ..
        },
        Pattern::ContextPattern {
            context_features: f2,
            ..
        },
    ) = (p1, p2)
    {
        // Calculate Jaccard similarity
        let set1: HashSet<_> = f1.iter().collect();
        let set2: HashSet<_> = f2.iter().collect();

        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();

        if union == 0 {
            return false;
        }

        let similarity = intersection as f32 / union as f32;
        similarity > 0.7 // 70% overlap threshold
    } else {
        false
    }
}

/// Merge multiple context patterns
pub fn merge_context_patterns(patterns: Vec<Pattern>) -> Option<Pattern> {
    if patterns.is_empty() {
        return None;
    }

    if patterns.len() == 1 {
        return patterns.into_iter().next();
    }

    // Combine features and evidence - optimize cloning
    let mut all_features = HashSet::new();
    let mut all_evidence = Vec::new();
    let mut total_success_rate = 0.0;
    let mut approaches = Vec::new();

    for pattern in &patterns {
        if let Pattern::ContextPattern {
            context_features,
            recommended_approach,
            evidence,
            success_rate,
            ..
        } = pattern
        {
            // Extend with iterators to avoid cloning the inner values
            all_features.extend(context_features.iter().cloned());
            all_evidence.extend(evidence.iter().copied());
            total_success_rate += success_rate;
            approaches.push(recommended_approach.clone());
        }
    }

    // Create merged pattern
    let avg_success_rate = total_success_rate / patterns.len() as f32;
    let combined_approach = approaches.join("; ");

    Some(Pattern::ContextPattern {
        id: Uuid::new_v4(),
        context_features: all_features.into_iter().collect(),
        recommended_approach: combined_approach,
        evidence: all_evidence,
        success_rate: avg_success_rate,
        effectiveness: crate::pattern::PatternEffectiveness::new(),
    })
}

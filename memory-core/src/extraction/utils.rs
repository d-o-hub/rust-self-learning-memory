//! Utility functions for pattern processing

use crate::pattern::Pattern;
use crate::types::TaskContext;

/// Remove duplicate patterns from a list
pub fn deduplicate_patterns(patterns: Vec<Pattern>) -> Vec<Pattern> {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    let mut deduplicated = Vec::new();

    for pattern in patterns {
        let key = pattern.similarity_key();

        if seen.insert(key) {
            deduplicated.push(pattern);
        }
    }

    deduplicated
}

/// Rank patterns by relevance/quality
pub fn rank_patterns(mut patterns: Vec<Pattern>, context: &TaskContext) -> Vec<Pattern> {
    // Sort patterns by a composite score considering multiple factors
    patterns.sort_by(|a, b| {
        let score_a = calculate_pattern_score(a, context);
        let score_b = calculate_pattern_score(b, context);

        // Sort in descending order (higher score first)
        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    patterns
}

/// Calculate a relevance score for a pattern given the current context
fn calculate_pattern_score(pattern: &Pattern, current_context: &TaskContext) -> f64 {
    let mut score = 0.0;

    // Base score from success rate (0-100 points)
    score += pattern.success_rate() as f64 * 100.0;

    // Sample size bonus (0-50 points, diminishing returns)
    let sample_size = pattern.sample_size() as f64;
    score += (sample_size.min(10.0) / 10.0) * 50.0;

    // Context relevance bonus (0-100 points)
    if let Some(pattern_context) = pattern.context() {
        score += calculate_context_similarity(pattern_context, current_context) * 100.0;
    }

    // Pattern type specific bonuses
    match pattern {
        Pattern::ToolSequence { tools, .. } => {
            // Prefer patterns with diverse tool usage
            let unique_tools = tools.iter().collect::<std::collections::HashSet<_>>().len();
            score += (unique_tools as f64 / tools.len() as f64) * 20.0;
        }
        Pattern::ErrorRecovery { .. } => {
            // Error recovery patterns are valuable for robustness
            score += 30.0;
        }
        Pattern::DecisionPoint { outcome_stats, .. } => {
            // Decision points with clear outcomes are more valuable
            if outcome_stats.total_count > 5 {
                score += 25.0;
            }
        }
        Pattern::ContextPattern { evidence, .. } => {
            // Context patterns with more evidence are better
            score += (evidence.len() as f64).min(5.0) * 10.0;
        }
    }

    score
}

/// Calculate similarity between two task contexts (0.0 to 1.0)
fn calculate_context_similarity(a: &TaskContext, b: &TaskContext) -> f64 {
    let mut similarity = 0.0;
    let mut factors = 0.0;

    // Language match (high weight)
    if a.language == b.language {
        similarity += 1.0;
    }
    factors += 1.0;

    // Framework match (high weight)
    if a.framework == b.framework {
        similarity += 1.0;
    }
    factors += 1.0;

    // Domain match (medium weight)
    if a.domain == b.domain {
        similarity += 0.8;
    }
    factors += 1.0;

    // Complexity level match (medium weight)
    if a.complexity == b.complexity {
        similarity += 0.6;
    }
    factors += 1.0;

    // Tag overlap (variable weight based on overlap)
    if !a.tags.is_empty() || !b.tags.is_empty() {
        let a_tags: std::collections::HashSet<_> = a.tags.iter().collect();
        let b_tags: std::collections::HashSet<_> = b.tags.iter().collect();
        let intersection = a_tags.intersection(&b_tags).count();
        let union = a_tags.union(&b_tags).count();

        if union > 0 {
            similarity += (intersection as f64 / union as f64) * 0.7;
        }
        factors += 1.0;
    }

    if factors > 0.0 {
        similarity / factors
    } else {
        0.0
    }
}

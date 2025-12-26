//! Utility functions for pattern processing

use crate::pattern::Pattern;
use crate::types::TaskContext;

/// Remove duplicate patterns from a list
#[must_use]
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
#[must_use]
pub fn rank_patterns(mut patterns: Vec<Pattern>, context: &TaskContext) -> Vec<Pattern> {
    // Sort patterns by a composite score considering multiple factors
    patterns.sort_by(|a, b| {
        let score_a = calculate_pattern_score(a, context);
        let score_b = calculate_pattern_score(b, context);

        // Sort in descending order (higher score first)
        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    patterns
}

/// Calculate a relevance score for a pattern given the current context
///
/// Scoring system (max ~400+ points):
/// - Base success rate: 0-100 points
/// - Sample size: 0-50 points
/// - Context relevance: 0-100 points
/// - Pattern type bonuses: 0-50 points
/// - **Effectiveness tracking: 0-200 points** (NEW)
fn calculate_pattern_score(pattern: &Pattern, current_context: &TaskContext) -> f64 {
    let mut score = 0.0;

    // Base score from success rate (0-100 points)
    score += f64::from(pattern.success_rate()) * 100.0;

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

    // **NEW: Effectiveness-based scoring (0-200 points)**
    // This is the key metric for self-learning!
    let effectiveness = pattern.effectiveness();

    // 1. Effectiveness score boost (0-100 points)
    // Combines success rate, usage confidence, and reward impact
    score += f64::from(effectiveness.effectiveness_score()) * 100.0;

    // 2. Proven usage bonus (0-50 points)
    // Patterns that have been successfully applied get priority
    if effectiveness.times_applied > 0 {
        let success_rate = effectiveness.application_success_rate();
        let usage_confidence = (effectiveness.times_applied as f64).ln().min(3.0) / 3.0;
        score += f64::from(success_rate) * usage_confidence * 50.0;
    }

    // 3. Reward delta bonus (0-50 points)
    // Patterns that improve outcomes get strong preference
    if effectiveness.avg_reward_delta > 0.0 {
        let capped_delta = effectiveness.avg_reward_delta.min(0.5); // Cap at +0.5
        score += f64::from(capped_delta) * 100.0; // 0.5 delta = 50 points
    } else if effectiveness.avg_reward_delta < 0.0 {
        // Penalize patterns that hurt performance
        let capped_penalty = effectiveness.avg_reward_delta.max(-0.5); // Cap at -0.5
        score += f64::from(capped_penalty) * 100.0; // Can subtract up to 50 points
    }

    // 4. Recency bonus (0-10 points)
    // Recently used patterns are more likely to be relevant
    if effectiveness.times_applied > 0 {
        use chrono::Utc;
        let days_since_use = (Utc::now() - effectiveness.last_used).num_days();
        if days_since_use < 30 {
            score += (30.0 - days_since_use as f64) / 30.0 * 10.0;
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

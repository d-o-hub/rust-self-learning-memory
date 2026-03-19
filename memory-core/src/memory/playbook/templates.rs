//! Playbook templates and synthesis logic.
//!
//! Extracted from `generator.rs` to maintain the ≤500 LOC invariant.

use super::builder::{ReflectionData, StepsBuilder};
use super::types::{
    PlaybookPitfall, PlaybookRequest, PlaybookStep, PlaybookSynthesisSource,
};
use crate::pattern::Pattern;
use crate::semantic::EpisodeSummary;
use crate::types::TaskContext;

/// Calculate how well patterns match the task.
pub(super) fn calculate_task_match(request: &PlaybookRequest, patterns: &[Pattern]) -> f32 {
    if patterns.is_empty() {
        return 0.0;
    }

    // Calculate average success rate of patterns
    let avg_success_rate: f32 =
        patterns.iter().map(|p| p.success_rate()).sum::<f32>() / patterns.len() as f32;

    // Calculate context match
    let context_matches: usize = patterns
        .iter()
        .filter_map(|p| p.context())
        .filter(|ctx| {
            ctx.domain == request.domain
                || ctx.tags.iter().any(|t| request.context.tags.contains(t))
        })
        .count();

    let context_match_ratio = if patterns.is_empty() {
        0.0
    } else {
        context_matches as f32 / patterns.len() as f32
    };

    // Weighted combination
    avg_success_rate * 0.6 + context_match_ratio * 0.4
}

/// Synthesize ordered steps from patterns.
pub(super) fn synthesize_steps(
    patterns: &[Pattern],
    source: &mut PlaybookSynthesisSource,
    max_steps: usize,
    max_patterns: usize,
) -> Vec<PlaybookStep> {
    let mut builder = StepsBuilder::new(max_steps);

    for pattern in patterns.iter().take(max_patterns) {
        if builder.is_full() {
            break;
        }

        source.add_pattern(pattern.id());
        builder.add_pattern_steps(pattern, source);
    }

    builder.build()
}

/// Synthesize when to apply and when not to apply rules.
pub(super) fn synthesize_applicability(
    patterns: &[Pattern],
    context: &TaskContext,
    max_patterns: usize,
) -> (Vec<String>, Vec<String>) {
    let mut when_to_apply = Vec::new();
    let mut when_not_to_apply = Vec::new();

    for pattern in patterns.iter().take(max_patterns) {
        match pattern {
            Pattern::ToolSequence { tools, context, .. } => {
                when_to_apply.push(format!(
                    "When working with {} in {} domain",
                    tools.join(", "),
                    context.domain
                ));
            }
            Pattern::DecisionPoint {
                condition, action, ..
            } => {
                when_to_apply.push(format!("When condition '{}' is true", condition));
                when_not_to_apply.push(format!(
                    "When condition '{}' is false - skip {}",
                    condition, action
                ));
            }
            Pattern::ErrorRecovery { error_type, .. } => {
                when_to_apply.push(format!("When encountering {} errors", error_type));
            }
            Pattern::ContextPattern {
                context_features, ..
            } => {
                let features = context_features.join(", ");
                when_to_apply.push(format!("When context includes: {}", features));
                if !context.tags.is_empty() {
                    when_not_to_apply.push("When task has different context tags".to_string());
                }
            }
        }
    }

    // Deduplicate
    when_to_apply.sort();
    when_to_apply.dedup();
    when_not_to_apply.sort();
    when_not_to_apply.dedup();

    (when_to_apply, when_not_to_apply)
}

/// Synthesize pitfalls from reflections.
pub(super) fn synthesize_pitfalls(
    reflections: &[ReflectionData],
    source: &mut PlaybookSynthesisSource,
) -> Vec<PlaybookPitfall> {
    let mut pitfalls = Vec::new();

    for reflection in reflections {
        source.add_episode(reflection.episode_id);

        // Improvements become pitfalls
        for improvement in &reflection.improvements {
            pitfalls.push(
                PlaybookPitfall::new(
                    format!("Potential issue: {}", improvement),
                    "Identified from past execution",
                )
                .with_mitigation("Review and apply this improvement"),
            );
        }

        // Failed steps become warnings
        for failed_step in &reflection.failed_steps {
            pitfalls.push(PlaybookPitfall::new(
                format!("Step may fail: {}", failed_step),
                "Based on historical failures",
            ));
        }
    }

    // Limit to top 5 pitfalls
    pitfalls.truncate(5);
    pitfalls
}

/// Synthesize expected outcome from patterns and summaries.
pub(super) fn synthesize_expected_outcome(
    patterns: &[Pattern],
    summaries: &[EpisodeSummary],
    source: &mut PlaybookSynthesisSource,
) -> String {
    let mut outcome_parts = Vec::new();

    // From patterns - use success rates
    let avg_success: f32 = if patterns.is_empty() {
        0.0
    } else {
        patterns.iter().map(|p| p.success_rate()).sum::<f32>() / patterns.len() as f32
    };

    if avg_success > 0.7 {
        outcome_parts.push("High probability of success".to_string());
    } else if avg_success > 0.5 {
        outcome_parts.push("Moderate probability of success".to_string());
    } else {
        outcome_parts.push("Variable outcomes expected".to_string());
    }

    // From summaries - use key concepts
    for summary in summaries.iter().take(3) {
        source.add_summary(summary.episode_id);
        if !summary.key_concepts.is_empty() {
            outcome_parts.push(format!(
                "Key concepts: {}",
                summary
                    .key_concepts
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    outcome_parts.join(". ")
}

/// Calculate overall confidence from multiple sources.
pub(super) fn calculate_confidence(
    patterns: &[Pattern],
    summaries: &[EpisodeSummary],
    source: &PlaybookSynthesisSource,
) -> f32 {
    if patterns.is_empty() && summaries.is_empty() {
        return 0.0;
    }

    let mut confidence = 0.0;

    // Pattern contribution (0-0.4)
    if !patterns.is_empty() {
        let avg_success: f32 =
            patterns.iter().map(|p| p.success_rate()).sum::<f32>() / patterns.len() as f32;
        confidence += avg_success * 0.4;
    }

    // Summary contribution (0-0.3)
    if !summaries.is_empty() {
        let summary_boost = (summaries.len() as f32).min(3.0) / 3.0 * 0.3;
        confidence += summary_boost;
    }

    // Source diversity contribution (0-0.3)
    let source_diversity = (source.total_sources() as f32).ln().max(0.0) / 3.0 * 0.3;
    confidence += source_diversity;

    confidence.min(1.0)
}

/// Generate why_relevant explanation.
pub(super) fn generate_why_relevant(
    patterns: &[Pattern],
    summaries: &[EpisodeSummary],
    source: &PlaybookSynthesisSource,
) -> String {
    let mut reasons = Vec::new();

    if !patterns.is_empty() {
        reasons.push(format!(
            "Based on {} patterns with {:.0}% average success rate",
            patterns.len(),
            patterns.iter().map(|p| p.success_rate()).sum::<f32>() / patterns.len() as f32
                * 100.0
        ));
    }

    if !summaries.is_empty() {
        reasons.push(format!(
            "Synthesized from {} similar episode summaries",
            summaries.len()
        ));
    }

    if source.total_sources() > 0 {
        reasons.push(format!(
            "Supported by {} historical data points",
            source.total_sources()
        ));
    }

    if reasons.is_empty() {
        "Generated from available memory data".to_string()
    } else {
        reasons.join(". ")
    }
}

//! Pattern extraction functions

use crate::episode::{Episode, PatternId};
use crate::extraction::extractor::PatternExtractor;
use crate::pattern::Pattern;
use crate::types::OutcomeStats;

/// Extract tool sequence patterns from an episode
pub fn extract_tool_sequence(extractor: &PatternExtractor, episode: &Episode) -> Option<Pattern> {
    // Need at least one step to extract a pattern
    if episode.steps.is_empty() {
        return None;
    }

    // Check success rate threshold
    let success_rate = extractor.calculate_step_success_rate(episode);
    if success_rate < extractor.success_threshold {
        return None;
    }

    // Extract tool sequence (limit to max_sequence_len)
    // For episodes with fewer than min_sequence_len steps, we still extract
    // a pattern if the episode was successful (helps with learning from all episodes)
    let tools: Vec<String> = episode
        .steps
        .iter()
        .take(extractor.max_sequence_len)
        .map(|step| step.tool.clone())
        .collect();

    Some(Pattern::ToolSequence {
        id: PatternId::new_v4(),
        tools,
        context: episode.context.clone(),
        success_rate,
        avg_latency: extractor.calculate_average_latency(episode),
        occurrence_count: 1,
    })
}

/// Extract decision point patterns from an episode
pub fn extract_decision_points(extractor: &PatternExtractor, episode: &Episode) -> Vec<Pattern> {
    let mut patterns = Vec::new();

    // Check success rate threshold
    let success_rate = extractor.calculate_step_success_rate(episode);
    if success_rate < extractor.success_threshold {
        return patterns;
    }

    // Look for steps that appear to be decision points
    for step in &episode.steps {
        // Check if the action looks like a decision/condition
        let action_lower = step.action.to_lowercase();
        if action_lower.contains("check")
            || action_lower.contains("verify")
            || action_lower.contains("validate")
            || action_lower.contains("is")
            || action_lower.contains("has") {

            // Calculate outcome stats (simplified - assume success since episode succeeded)
            let outcome_stats = OutcomeStats {
                success_count: 1,
                failure_count: 0,
                total_count: 1,
                avg_duration_secs: step.latency_ms as f32 / 1000.0,
            };

            patterns.push(Pattern::DecisionPoint {
                id: PatternId::new_v4(),
                condition: step.action.clone(),
                action: step.tool.clone(),
                outcome_stats,
                context: episode.context.clone(),
            });
        }
    }

    patterns
}

/// Extract error recovery patterns from an episode
pub fn extract_error_recovery(extractor: &PatternExtractor, episode: &Episode) -> Option<Pattern> {
    use crate::types::ExecutionResult;

    // Need at least 2 steps to have error -> recovery
    if episode.steps.len() < 2 {
        return None;
    }

    // Look for error -> success patterns
    let mut error_type = None;
    let mut recovery_steps = Vec::new();

    for i in 0..episode.steps.len().saturating_sub(1) {
        let current = &episode.steps[i];
        let next = &episode.steps[i + 1];

        // Found an error followed by success
        if !current.is_success() && next.is_success() {
            // Extract error type
            if error_type.is_none() {
                error_type = Some(
                    if let Some(ExecutionResult::Error { message }) = &current.result {
                        message.clone()
                    } else {
                        "Unknown error".to_string()
                    },
                );
            }

            // Extract recovery step
            recovery_steps.push(format!("{}: {}", next.tool, next.action));
        }
    }

    // Need at least one recovery to create a pattern
    if error_type.is_none() || recovery_steps.is_empty() {
        return None;
    }

    // Calculate success rate
    let success_rate = extractor.calculate_step_success_rate(episode);

    // For error recovery patterns, we use a lower threshold (0.3) since they
    // represent valuable learning from failures even when overall success rate is moderate
    // The key is that we recovered from the error, not that all steps succeeded
    if success_rate < 0.3 {
        return None;
    }

    Some(Pattern::ErrorRecovery {
        id: PatternId::new_v4(),
        error_type: error_type.unwrap(),
        recovery_steps,
        context: episode.context.clone(),
        success_rate,
    })
}

/// Extract context-based patterns from an episode
pub fn extract_context_pattern(extractor: &PatternExtractor, episode: &Episode) -> Option<Pattern> {
    // Extract context pattern even for episodes without steps
    // This helps capture high-level patterns about task contexts

    // Calculate success rate
    let success_rate = if episode.steps.is_empty() {
        // If no steps but episode completed successfully, assume 100% success
        if episode.is_complete() && episode.reward.as_ref().is_some_and(|r| r.total > 0.0) {
            1.0
        } else {
            return None;
        }
    } else {
        extractor.calculate_step_success_rate(episode)
    };

    if success_rate < extractor.success_threshold {
        return None;
    }

    // Extract context features
    let mut context_features = Vec::new();
    if let Some(ref lang) = episode.context.language {
        context_features.push(format!("language:{}", lang));
    }
    if let Some(ref framework) = episode.context.framework {
        context_features.push(format!("framework:{}", framework));
    }
    context_features.push(format!("domain:{}", episode.context.domain));
    context_features.push(format!("complexity:{:?}", episode.context.complexity));

    // Recommended approach based on task type
    let recommended_approach = format!("{:?}", episode.task_type);

    Some(Pattern::ContextPattern {
        id: PatternId::new_v4(),
        context_features,
        recommended_approach,
        evidence: vec![episode.episode_id],
        success_rate,
    })
}

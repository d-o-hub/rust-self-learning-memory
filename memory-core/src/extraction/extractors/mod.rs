//! Pattern extraction functions

use crate::episode::{Episode, PatternId};
use crate::extraction::extractor::PatternExtractor;
use crate::pattern::Pattern;

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
pub fn extract_decision_points(_extractor: &PatternExtractor, _episode: &Episode) -> Vec<Pattern> {
    // TODO: Implement decision point extraction
    vec![]
}

/// Extract error recovery patterns from an episode
pub fn extract_error_recovery(
    _extractor: &PatternExtractor,
    _episode: &Episode,
) -> Option<Pattern> {
    // TODO: Implement error recovery extraction
    None
}

/// Extract context-based patterns from an episode
pub fn extract_context_pattern(extractor: &PatternExtractor, episode: &Episode) -> Option<Pattern> {
    // Extract context pattern even for episodes without steps
    // This helps capture high-level patterns about task contexts

    // Calculate success rate
    let success_rate = if episode.steps.is_empty() {
        // If no steps but episode completed successfully, assume 100% success
        if episode.is_complete() && episode.reward.as_ref().map_or(false, |r| r.total > 0.0) {
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

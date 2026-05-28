//! Helper functions for retrieval operations

use crate::episode::Episode;
use std::sync::Arc;

/// Maximum size for caching episodes (100KB)
pub const MAX_CACHEABLE_SIZE: usize = 100 * 1024;

/// Check if episodes should be cached based on estimated size
///
/// Skips caching for large result sets to avoid expensive clone operations.
/// Estimates size based on step count and artifact sizes.
pub fn should_cache_episodes(episodes: &[Arc<Episode>]) -> bool {
    // Quick check: if >50 episodes, likely too large
    if episodes.len() > 50 {
        return false;
    }

    // Estimate total size
    let estimated_size: usize = episodes
        .iter()
        .map(|arc_ep| {
            let ep = arc_ep.as_ref();
            // Base episode overhead: ~1KB
            let mut size = 1024;

            // Steps: ~100 bytes each
            size += ep.steps.len() * 100;

            // Outcome artifacts (can be large)
            if let Some(ref outcome) = ep.outcome {
                match outcome {
                    crate::types::TaskOutcome::Success { artifacts, .. } => {
                        size += artifacts.iter().map(|a| a.len()).sum::<usize>();
                    }
                    crate::types::TaskOutcome::PartialSuccess {
                        completed, failed, ..
                    } => {
                        size += completed.iter().map(|a| a.len()).sum::<usize>();
                        size += failed.iter().map(|a| a.len()).sum::<usize>();
                    }
                    crate::types::TaskOutcome::Failure { .. } => {}
                }
            }

            // Reflection insights
            if let Some(ref reflection) = ep.reflection {
                size += reflection.insights.iter().map(|i| i.len()).sum::<usize>();
            }

            size
        })
        .sum();

    estimated_size < MAX_CACHEABLE_SIZE
}

/// Generate a simple embedding for an episode based on its metadata
/// This is a placeholder until full embedding integration is complete
pub fn generate_simple_embedding(episode: &Episode) -> Vec<f32> {
    let mut embedding = Vec::with_capacity(10);

    // Domain hash
    let domain_hash = episode
        .context
        .domain
        .chars()
        .fold(0u32, |acc, c| acc.wrapping_add(c as u32));
    embedding.push((domain_hash % 100) as f32 / 100.0);

    // Task type encoding
    embedding.push(match episode.task_type {
        crate::types::TaskType::CodeGeneration => 0.9,
        crate::types::TaskType::Analysis => 0.7,
        crate::types::TaskType::Testing => 0.5,
        crate::types::TaskType::Debugging => 0.3,
        crate::types::TaskType::Refactoring => 0.2,
        crate::types::TaskType::Documentation => 0.1,
        crate::types::TaskType::Other => 0.0,
    });

    // Complexity encoding
    embedding.push(match episode.context.complexity {
        crate::types::ComplexityLevel::Simple => 0.2,
        crate::types::ComplexityLevel::Moderate => 0.5,
        crate::types::ComplexityLevel::Complex => 0.8,
    });

    // Language/framework presence
    embedding.push(if episode.context.language.is_some() {
        1.0
    } else {
        0.0
    });
    embedding.push(if episode.context.framework.is_some() {
        1.0
    } else {
        0.0
    });

    // Number of steps (normalized)
    let step_count = episode.steps.len().min(50) as f32 / 50.0;
    embedding.push(step_count);

    // Reward component (if available)
    let reward_value = episode.reward.as_ref().map_or(0.5, |r| r.total.min(1.0));
    embedding.push(reward_value);

    // Duration component
    if let Some(end) = episode.end_time {
        let duration = end - episode.start_time;
        let duration_secs = duration.num_seconds().clamp(0, 3600) as f32 / 3600.0;
        embedding.push(duration_secs);
    } else {
        embedding.push(0.5);
    }

    // Tag count (normalized)
    let tag_count = episode.context.tags.len().min(10) as f32 / 10.0;
    embedding.push(tag_count);

    // Outcome encoding
    embedding.push(match &episode.outcome {
        Some(crate::types::TaskOutcome::Success { .. }) => 1.0,
        Some(crate::types::TaskOutcome::PartialSuccess { .. }) => 0.5,
        Some(crate::types::TaskOutcome::Failure { .. }) => 0.0,
        None => 0.5,
    });

    embedding
}

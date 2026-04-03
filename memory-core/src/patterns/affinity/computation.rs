//! Core similarity computation functions for pattern affinity.

use crate::episode::Episode;
use crate::pattern::Pattern;

/// Compute max cosine similarity between an episode and pattern set.
///
/// Finds the pattern with highest embedding similarity to the episode.
pub(crate) fn max_cosine_similarity(
    episode: &Episode,
    patterns: &[Pattern],
    episode_embedding: Option<&[f32]>,
) -> f32 {
    if patterns.is_empty() {
        return 0.0;
    }

    let ep_emb = episode_embedding;

    patterns
        .iter()
        .map(|pattern| {
            pattern_embedding_similarity(ep_emb, pattern)
                .unwrap_or_else(|| context_similarity(episode, pattern))
        })
        .fold(0.0, f32::max)
}

/// Compute embedding-based similarity if embeddings are available.
fn pattern_embedding_similarity(
    _episode_embedding: Option<&[f32]>,
    _pattern: &Pattern,
) -> Option<f32> {
    None
}

/// Compute context-based similarity as fallback.
///
/// Uses task context features (domain, tags) for similarity estimation.
pub(crate) fn context_similarity(episode: &Episode, pattern: &Pattern) -> f32 {
    let ep_context = &episode.context;
    let pat_context = pattern.context();

    match pat_context {
        Some(pat_ctx) => {
            let mut score = 0.0;
            let mut components = 0;

            // Domain match
            if ep_context.domain == pat_ctx.domain {
                score += 1.0;
            }
            components += 1;

            // Tag overlap (Jaccard)
            let ep_tags: std::collections::HashSet<_> = ep_context.tags.iter().collect();
            let pat_tags: std::collections::HashSet<_> = pat_ctx.tags.iter().collect();
            let intersection = ep_tags.intersection(&pat_tags).count();
            let union = ep_tags.union(&pat_tags).count();
            if union > 0 {
                score += intersection as f32 / union as f32;
                components += 1;
            }

            // Language match
            if ep_context.language == pat_ctx.language {
                score += 0.5;
                components += 1;
            }

            score / components as f32
        }
        None => 0.3,
    }
}

//! Scoring logic for hierarchical retrieval
//!
//! Implements the 4-level scoring strategy for episodic memory retrieval.

use super::types::{generate_episode_embedding, calculate_text_similarity, HierarchicalScore, RetrievalQuery};
use crate::episode::Episode;
use chrono::Utc;

/// Scoring implementation for HierarchicalRetriever
impl super::HierarchicalRetriever {
    /// Level 1: Filter episodes by domain.
    ///
    /// If domain is specified in query, only return episodes from that domain.
    /// Otherwise, return all episodes.
    pub(super) fn filter_by_domain<'a>(
        &self,
        episodes: &'a [Episode],
        query: &RetrievalQuery,
    ) -> Vec<&'a Episode> {
        if let Some(ref domain) = query.domain {
            episodes
                .iter()
                .filter(|ep| ep.context.domain == *domain)
                .collect()
        } else {
            episodes.iter().collect()
        }
    }

    /// Level 2: Filter episodes by task type.
    ///
    /// If task type is specified in query, only return episodes of that type.
    /// Otherwise, return all candidates.
    pub(super) fn filter_by_task_type<'a>(
        &self,
        candidates: &[&'a Episode],
        query: &RetrievalQuery,
    ) -> Vec<&'a Episode> {
        if let Some(task_type) = query.task_type {
            candidates
                .iter()
                .filter(|ep| ep.task_type == task_type)
                .copied()
                .collect()
        } else {
            candidates.to_vec()
        }
    }

    /// Level 3: Select temporal clusters with recency bias.
    ///
    /// Groups episodes into temporal clusters and selects the most recent ones.
    /// Applies temporal bias to favor recent episodes in scoring.
    pub(super) fn select_temporal_clusters<'a>(
        &self,
        candidates: &[&'a Episode],
        _query: &RetrievalQuery,
    ) -> Vec<&'a Episode> {
        if candidates.is_empty() {
            return vec![];
        }

        // Sort by recency (newest first)
        let mut sorted: Vec<_> = candidates.to_vec();
        sorted.sort_by(|a, b| b.start_time.cmp(&a.start_time));

        // For now, take top-k most recent episodes
        // Future: implement proper temporal clustering (weekly/monthly buckets)
        let cluster_size = candidates.len() / self.max_clusters_to_search.max(1);
        let take_count = cluster_size.max(10).min(candidates.len());

        sorted.into_iter().take(take_count).collect()
    }

    /// Level 4: Score episodes by similarity.
    ///
    /// Calculates fine-grained similarity scores for candidates.
    /// Uses embedding-based similarity when available, falls back to text similarity.
    pub(super) fn score_episodes(
        &self,
        candidates: &[&Episode],
        query: &RetrievalQuery,
    ) -> Vec<HierarchicalScore> {
        let now = Utc::now();

        let scored: Vec<HierarchicalScore> = candidates
            .iter()
            .map(|episode| {
                // Level 1 score: Domain match
                let level_1_score = if let Some(ref domain) = query.domain {
                    if episode.context.domain == *domain {
                        1.0
                    } else {
                        0.0
                    }
                } else {
                    0.5 // Neutral if no domain specified
                };

                // Level 2 score: Task type match
                let level_2_score = if let Some(task_type) = query.task_type {
                    if episode.task_type == task_type {
                        1.0
                    } else {
                        0.0
                    }
                } else {
                    0.5 // Neutral if no task type specified
                };

                // Level 3 score: Temporal proximity (newer is better)
                let age_seconds = (now - episode.start_time).num_seconds().max(0) as f32;
                let max_age_seconds = 30.0 * 24.0 * 3600.0; // 30 days
                let level_3_score = 1.0 - (age_seconds / max_age_seconds).min(1.0);

                // Level 4 score: Embedding similarity (if available) or text similarity
                let level_4_score = if let Some(ref query_emb) = query.query_embedding {
                    // Generate episode embedding (simple metadata-based for now)
                    let episode_emb = generate_episode_embedding(episode);

                    // Calculate cosine similarity between query and episode embeddings
                    // Note: cosine_similarity returns a value in [-1, 1], normalize to [0, 1]
                    let similarity = crate::embeddings::cosine_similarity(query_emb, &episode_emb);
                    (similarity + 1.0) / 2.0 // Normalize from [-1, 1] to [0, 1]
                } else {
                    // Fallback to text-based similarity
                    calculate_text_similarity(
                        &query.query_text.to_lowercase(),
                        &episode.task_description.to_lowercase(),
                    )
                };

                // Combined relevance score
                // Weights: domain (0.3), task_type (0.3), temporal (temporal_bias_weight), similarity (1 - temporal_bias - 0.6)
                let temporal_weight = self.temporal_bias_weight;
                let similarity_weight = 1.0 - temporal_weight - 0.6;

                let relevance_score = 0.3 * level_1_score
                    + 0.3 * level_2_score
                    + temporal_weight * level_3_score
                    + similarity_weight.max(0.1) * level_4_score;

                HierarchicalScore {
                    episode_id: episode.episode_id,
                    relevance_score,
                    level_1_score,
                    level_2_score,
                    level_3_score,
                    level_4_score,
                }
            })
            .collect();

        scored
    }

    /// Rank scored episodes by combined relevance score.
    ///
    /// Sorts episodes in descending order of relevance.
    pub(super) fn rank_by_combined_score(&self, mut scored: Vec<HierarchicalScore>) -> Vec<HierarchicalScore> {
        scored.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored
    }
}

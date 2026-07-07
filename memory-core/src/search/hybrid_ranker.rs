//! Hybrid ranker for multi-signal fusion

use crate::episode::Episode;
use crate::types::TaskContext;
use chrono::Utc;

/// Signals used for ranking episodes
pub struct RankingSignals {
    /// Semantic similarity score (0.0 to 1.0)
    pub semantic_score: f32,
    /// Reward/Quality score (normalized, typically 0.0 to 1.0)
    pub reward_score: f32,
    /// Recency score (0.0 to 1.0, 1.0 is most recent)
    pub recency_score: f32,
    /// Context overlap score (0.0 to 1.0)
    pub context_score: f32,
}

/// Configuration for hybrid ranking weights
#[derive(Debug, Clone, Copy)]
pub struct HybridRankingWeights {
    pub semantic: f32,
    pub reward: f32,
    pub recency: f32,
    pub context: f32,
}

impl Default for HybridRankingWeights {
    fn default() -> Self {
        Self {
            semantic: 0.4,
            reward: 0.3,
            recency: 0.1,
            context: 0.2,
        }
    }
}

/// Ranker that combines multiple signals into a final relevance score
pub struct HybridRanker {
    weights: HybridRankingWeights,
}

impl HybridRanker {
    /// Create a new hybrid ranker with default weights
    #[must_use]
    pub fn new() -> Self {
        Self {
            weights: HybridRankingWeights::default(),
        }
    }

    /// Create a new hybrid ranker with custom weights
    #[must_use]
    pub fn with_weights(weights: HybridRankingWeights) -> Self {
        Self { weights }
    }

    /// Rank episodes based on multiple signals
    pub fn rank(&self, signals: &RankingSignals) -> f32 {
        (signals.semantic_score * self.weights.semantic) +
        (signals.reward_score * self.weights.reward) +
        (signals.recency_score * self.weights.recency) +
        (signals.context_score * self.weights.context)
    }

    /// Calculate recency score for an episode (1.0 = now, decays over time)
    pub fn calculate_recency_score(&self, episode: &Episode) -> f32 {
        let now = Utc::now();
        let age = now.signed_duration_since(episode.start_time);
        let age_days = age.num_days() as f32;

        // Simple exponential decay: score = e^(-age_days / 30)
        // 1.0 at 0 days, ~0.37 at 30 days, ~0.13 at 60 days
        (-age_days / 30.0).exp()
    }

    /// Calculate context overlap score (domain, language, framework)
    pub fn calculate_context_score(&self, episode: &Episode, query_context: &TaskContext) -> f32 {
        let mut score = 0.0;
        let mut total_weight = 0.0;

        // Domain match (highest weight)
        total_weight += 2.0;
        if episode.context.domain == query_context.domain {
            score += 2.0;
        }

        // Language match
        total_weight += 1.0;
        if let (Some(e_lang), Some(q_lang)) = (&episode.context.language, &query_context.language) {
            if e_lang == q_lang {
                score += 1.0;
            }
        }

        // Framework match
        total_weight += 1.0;
        if let (Some(e_fw), Some(q_fw)) = (&episode.context.framework, &query_context.framework) {
            if e_fw == q_fw {
                score += 1.0;
            }
        }

        // Tags overlap
        if !query_context.tags.is_empty() {
            total_weight += 1.0;
            let query_tags: std::collections::HashSet<_> = query_context.tags.iter().collect();
            let episode_tags: std::collections::HashSet<_> = episode.context.tags.iter().collect();
            let common = query_tags.intersection(&episode_tags).count();
            score += (common as f32 / query_tags.len() as f32).min(1.0);
        }

        score / total_weight
    }
}

impl Default for HybridRanker {
    fn default() -> Self {
        Self::new()
    }
}

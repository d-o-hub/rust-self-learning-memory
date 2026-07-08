//! Hybrid semantic and keyword retrieval for episodic memory.

use chrono::Utc;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::embeddings::VectorIndex;
use crate::episode::Episode;
use crate::error::Result;
use crate::types::{MemoryConfig, TaskContext};

/// A hit from the hybrid retriever.
#[derive(Debug, Clone)]
pub struct HybridHit {
    /// The episode that was matched.
    pub episode: Arc<Episode>,
    /// The final hybrid score.
    pub score: f32,
    /// Component scores.
    pub components: ScoreComponents,
}

/// Component scores for a hybrid hit.
#[derive(Debug, Clone, Default)]
pub struct ScoreComponents {
    /// Semantic similarity score.
    pub semantic: f32,
    /// Recency score.
    pub recency: f32,
    /// Reward score.
    pub reward: f32,
    /// Context overlap score.
    pub context_overlap: f32,
}

/// Hybrid retriever that combines multiple signals for episode ranking.
pub struct SemanticRetriever {
    config: MemoryConfig,
    pub vector_index: RwLock<Box<dyn VectorIndex>>,
}

impl SemanticRetriever {
    /// Create a new hybrid retriever.
    pub fn new(config: MemoryConfig, vector_index: Box<dyn VectorIndex>) -> Self {
        Self {
            config,
            vector_index: RwLock::new(vector_index),
        }
    }

    /// Retrieve relevant episodes using a hybrid approach.
    pub fn retrieve(
        &self,
        _query_text: &str,
        query_embedding: &[f32],
        context: &TaskContext,
        episodes: HashMap<Uuid, Arc<Episode>>,
        limit: usize,
    ) -> Result<Vec<HybridHit>> {
        // 1. Semantic search
        let semantic_hits = {
            let index = self.vector_index.read();
            index.search(query_embedding, limit * 2)?
        };

        // 2. Score and rank
        let mut hybrid_hits = Vec::new();

        for v_hit in semantic_hits {
            if let Ok(id) = Uuid::parse_str(&v_hit.id) {
                if let Some(episode) = episodes.get(&id) {
                    let components = self.calculate_components(episode, v_hit.score, context);
                    let combined_score = self.combine_scores(&components);

                    hybrid_hits.push(HybridHit {
                        episode: episode.clone(),
                        score: combined_score,
                        components,
                    });
                }
            }
        }

        // Sort by final score
        hybrid_hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply limit
        hybrid_hits.truncate(limit);

        Ok(hybrid_hits)
    }

    /// Add an episode to the index.
    pub fn upsert(&self, id: &str, embedding: Vec<f32>) -> Result<()> {
        let mut index = self.vector_index.write();
        index.upsert(id, &embedding)
    }

    /// Remove an episode from the index.
    pub fn remove(&self, id: &str) -> Result<()> {
        let mut index = self.vector_index.write();
        index.remove(id)
    }

    /// Save the vector index to a file.
    pub fn save(&self, path: &std::path::Path) -> Result<()> {
        let index = self.vector_index.read();
        index.save(path)
    }

    fn calculate_components(
        &self,
        episode: &Episode,
        semantic_score: f32,
        current_context: &TaskContext,
    ) -> ScoreComponents {
        let recency = self.calculate_recency(episode);
        let reward = self.calculate_reward(episode);
        let context_overlap = self.calculate_context_overlap(episode, current_context);

        ScoreComponents {
            semantic: semantic_score,
            recency,
            reward,
            context_overlap,
        }
    }

    fn calculate_recency(&self, episode: &Episode) -> f32 {
        let now = Utc::now();
        let duration = now.signed_duration_since(episode.start_time);
        let days = duration.num_days().max(0) as f32;

        // Exponential decay: score = 0.5 ^ (days / 7)
        // 1.0 for today, 0.5 for a week ago, 0.25 for two weeks ago
        (0.5f32).powf(days / 7.0)
    }

    fn calculate_reward(&self, episode: &Episode) -> f32 {
        // Map RewardScore to 0.0-1.0
        // Assuming base reward is roughly 0-100
        let score = episode.reward.as_ref().map_or(0.0, |r| r.total);
        (score / 100.0f32).clamp(0.0f32, 1.0f32)
    }

    fn calculate_context_overlap(&self, episode: &Episode, current: &TaskContext) -> f32 {
        let mut score = 0.0;
        let mut total_points = 0.0;

        // Domain match
        total_points += 1.0;
        if episode.context.domain == current.domain {
            score += 1.0;
        }

        // Language match
        if current.language.is_some() {
            total_points += 1.0;
            if episode.context.language == current.language {
                score += 1.0;
            }
        }

        // Framework match
        if current.framework.is_some() {
            total_points += 1.0;
            if episode.context.framework == current.framework {
                score += 1.0;
            }
        }

        // Tags overlap
        if !current.tags.is_empty() {
            total_points += 1.0;
            let current_tags: std::collections::HashSet<_> = current.tags.iter().collect();
            let episode_tags: std::collections::HashSet<_> = episode.context.tags.iter().collect();
            let intersection = current_tags.intersection(&episode_tags).count();
            if !current_tags.is_empty() {
                score += intersection as f32 / current_tags.len() as f32;
            }
        }

        if total_points > 0.0 {
            score / total_points
        } else {
            0.0
        }
    }

    fn combine_scores(&self, components: &ScoreComponents) -> f32 {
        (components.semantic * self.config.semantic_weight)
            + (components.recency * self.config.recency_weight)
            + (components.reward * self.config.reward_weight)
            + (components.context_overlap * self.config.context_overlap_weight)
    }

    /// Combine results from keyword search and semantic search using RRF.
    pub fn reciprocal_rank_fusion(
        keyword_results: &[Uuid],
        semantic_results: &[Uuid],
        k: f32,
    ) -> Vec<(Uuid, f32)> {
        let mut scores = HashMap::new();

        for (rank, &id) in keyword_results.iter().enumerate() {
            let score = 1.0 / (k + (rank + 1) as f32);
            *scores.entry(id).or_insert(0.0) += score;
        }

        for (rank, &id) in semantic_results.iter().enumerate() {
            let score = 1.0 / (k + (rank + 1) as f32);
            *scores.entry(id).or_insert(0.0) += score;
        }

        let mut fused: Vec<_> = scores.into_iter().collect();
        fused.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        fused
    }
}

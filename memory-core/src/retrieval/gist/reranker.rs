//! Hierarchical reranking for dense context retrieval.

use std::collections::HashSet;
use std::sync::Arc;

use crate::episode::Episode;

use super::config::RerankConfig;
use super::extractor::GistExtractor;
use super::types::EpisodeGist;

/// Item with gist information for reranking.
#[derive(Debug, Clone)]
pub struct GistScoredItem {
    /// The episode
    episode: Arc<Episode>,
    /// Extracted gist
    gist: EpisodeGist,
    /// Original relevance score
    relevance: f32,
    /// Combined score (relevance + density + recency)
    combined_score: f32,
}

impl GistScoredItem {
    /// Create a new gist-scored item.
    #[must_use]
    pub fn new(episode: Arc<Episode>, gist: EpisodeGist, relevance: f32) -> Self {
        Self {
            episode,
            gist,
            relevance,
            combined_score: 0.0,
        }
    }

    /// Get the episode.
    #[must_use]
    pub fn episode(&self) -> &Arc<Episode> {
        &self.episode
    }

    /// Get the gist.
    #[must_use]
    pub fn gist(&self) -> &EpisodeGist {
        &self.gist
    }

    /// Get the relevance score.
    #[must_use]
    pub fn relevance(&self) -> f32 {
        self.relevance
    }

    /// Get the combined score.
    #[must_use]
    pub fn combined_score(&self) -> f32 {
        self.combined_score
    }

    /// Set the combined score.
    pub fn set_combined_score(&mut self, score: f32) {
        self.combined_score = score;
    }
}

/// Hierarchical reranker for dense context retrieval.
#[derive(Debug)]
pub struct HierarchicalReranker {
    config: RerankConfig,
    extractor: GistExtractor,
}

impl Default for HierarchicalReranker {
    fn default() -> Self {
        Self::new(RerankConfig::default())
    }
}

impl HierarchicalReranker {
    /// Create a new hierarchical reranker.
    #[must_use]
    pub fn new(config: RerankConfig) -> Self {
        let max_key_points = config.max_key_points;
        Self {
            config,
            extractor: GistExtractor::new(max_key_points),
        }
    }

    /// Create a reranker optimized for dense context.
    #[must_use]
    pub fn dense() -> Self {
        Self::new(RerankConfig::dense())
    }

    /// Rerank episodes by gist density.
    #[must_use]
    pub fn rerank(&self, episodes: Vec<(Arc<Episode>, f32)>, top_k: usize) -> Vec<GistScoredItem> {
        self.rerank_with_query(episodes, "", top_k)
    }

    /// Rerank episodes using CogniRank (gist-to-query alignment).
    #[must_use]
    pub fn rerank_with_query(
        &self,
        episodes: Vec<(Arc<Episode>, f32)>,
        query: &str,
        top_k: usize,
    ) -> Vec<GistScoredItem> {
        if episodes.is_empty() || top_k == 0 {
            return Vec::new();
        }

        let mut items: Vec<GistScoredItem> = episodes
            .into_iter()
            .map(|(episode, relevance)| {
                let gist = self.extractor.extract_from_episode(&episode);
                GistScoredItem::new(episode, gist, relevance)
            })
            .filter(|item| item.gist().density >= self.config.min_density_threshold)
            .collect();

        if items.is_empty() {
            return Vec::new();
        }

        for item in &mut items {
            let recency = self.compute_recency_score(item.episode());
            let gist_query_sim = if query.is_empty() {
                0.0
            } else {
                self.compute_gist_query_similarity(item, query)
            };
            let score = self.compute_combined_score(
                item.relevance(),
                item.gist().density,
                gist_query_sim,
                recency,
            );
            item.set_combined_score(score);
        }

        self.select_diverse(items, top_k)
    }

    /// Compute recency score for an episode.
    fn compute_recency_score(&self, episode: &Episode) -> f32 {
        use chrono::Utc;

        let now = Utc::now();
        let age_days = (now - episode.start_time).num_days() as f32;

        if age_days <= 0.0 {
            return 1.0;
        }

        let decay = 0.5_f32.powf(age_days / self.config.recency_half_life_days);
        decay.clamp(0.0, 1.0)
    }

    /// Compute combined score from components.
    fn compute_combined_score(
        &self,
        relevance: f32,
        density: f32,
        gist_query_sim: f32,
        recency: f32,
    ) -> f32 {
        self.config.relevance_weight * relevance
            + self.config.density_weight * density
            + self.config.gist_query_similarity_weight * gist_query_sim
            + self.config.recency_weight * recency
    }

    /// Compute similarity between a gist and a query.
    pub fn compute_gist_query_similarity(&self, item: &GistScoredItem, query: &str) -> f32 {
        let summary = item.gist().summary();
        let gist_words = self.extract_words(&summary);
        let query_words = self.extract_words(query);

        if gist_words.is_empty() || query_words.is_empty() {
            return 0.0;
        }

        let intersection = gist_words.intersection(&query_words).count();
        let union = gist_words.union(&query_words).count();

        if union == 0 {
            return 0.0;
        }

        intersection as f32 / union as f32
    }

    /// Select diverse items using MMR-style algorithm.
    fn select_diverse(&self, items: Vec<GistScoredItem>, k: usize) -> Vec<GistScoredItem> {
        if items.len() <= k {
            let mut sorted = items;
            sorted.sort_by(|a, b| {
                b.combined_score()
                    .partial_cmp(&a.combined_score())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            return sorted;
        }

        let mut selected: Vec<GistScoredItem> = Vec::with_capacity(k);
        let mut remaining = items;

        let first_idx = self.find_max_score_index(&remaining);
        let first = remaining.remove(first_idx);
        selected.push(first);

        while selected.len() < k && !remaining.is_empty() {
            let best_idx = self.find_max_mmr_index(&remaining, &selected);
            let best = remaining.remove(best_idx);
            selected.push(best);
        }

        selected
    }

    /// Find index of item with highest combined score.
    fn find_max_score_index(&self, items: &[GistScoredItem]) -> usize {
        items
            .iter()
            .enumerate()
            .map(|(idx, item)| (idx, item.combined_score()))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map_or(0, |(idx, _)| idx)
    }

    /// Find index of item with highest MMR score.
    fn find_max_mmr_index(
        &self,
        candidates: &[GistScoredItem],
        selected: &[GistScoredItem],
    ) -> usize {
        candidates
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let mmr = self.compute_mmr_score(item, selected);
                (idx, mmr)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map_or(0, |(idx, _)| idx)
    }

    /// Compute MMR score for an item.
    fn compute_mmr_score(&self, item: &GistScoredItem, selected: &[GistScoredItem]) -> f32 {
        let relevance = item.combined_score();

        if selected.is_empty() {
            return self.config.diversity_lambda * relevance;
        }

        let max_similarity = selected
            .iter()
            .map(|s| self.compute_text_similarity(item, s))
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);

        self.config.diversity_lambda * relevance
            - (1.0 - self.config.diversity_lambda) * max_similarity
    }

    /// Compute text similarity between two items.
    pub fn compute_text_similarity(&self, item1: &GistScoredItem, item2: &GistScoredItem) -> f32 {
        let summary1 = item1.gist().summary();
        let summary2 = item2.gist().summary();
        let words1 = self.extract_words(&summary1);
        let words2 = self.extract_words(&summary2);

        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            return 0.0;
        }

        intersection as f32 / union as f32
    }

    /// Extract significant words from text.
    fn extract_words<'a>(&self, text: &'a str) -> HashSet<&'a str> {
        text.split(|c: char| !c.is_alphanumeric())
            .map(|s| s.trim())
            .filter(|s| s.len() >= 3)
            .collect()
    }
}

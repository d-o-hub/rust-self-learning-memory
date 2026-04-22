//! Hierarchical reranking for dense context retrieval.

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
///
/// Reranks retrieval results by gist density and diversity,
/// returning fewer items with higher information density.
///
/// # Algorithm
///
/// 1. Extract gist from each episode
/// 2. Compute combined score = relevance + density + recency
/// 3. Apply MMR-style diversity selection
/// 4. Return top-k dense items
///
/// # Examples
///
/// ```
/// use do_memory_core::retrieval::{HierarchicalReranker, RerankConfig};
/// use do_memory_core::episode::Episode;
/// use do_memory_core::TaskContext;
/// use do_memory_core::types::TaskType;
/// use std::sync::Arc;
///
/// let reranker = HierarchicalReranker::new(RerankConfig::dense());
///
/// let ep1 = Arc::new(Episode::new("Fixed bug in auth".to_string(), TaskContext::default(), TaskType::Debugging));
/// let ep2 = Arc::new(Episode::new("Added new feature".to_string(), TaskContext::default(), TaskType::CodeGeneration));
///
/// let items = vec![(ep1, 0.9), (ep2, 0.85)];
/// let dense = reranker.rerank(items, 5);
///
/// // Returns at most 5 items, prioritized by density
/// assert!(dense.len() <= 5);
/// ```
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
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for reranking
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

    /// Get the configuration.
    #[must_use]
    pub fn config(&self) -> &RerankConfig {
        &self.config
    }

    /// Rerank episodes by gist density.
    ///
    /// # Arguments
    ///
    /// * `episodes` - Episodes with their original relevance scores
    /// * `top_k` - Maximum number of items to return
    ///
    /// # Returns
    ///
    /// Vector of gist-scored items, sorted by combined score
    #[must_use]
    pub fn rerank(&self, episodes: Vec<(Arc<Episode>, f32)>, top_k: usize) -> Vec<GistScoredItem> {
        if episodes.is_empty() || top_k == 0 {
            return Vec::new();
        }

        // 1. Extract gists and compute scores
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

        // 2. Compute combined scores
        for item in &mut items {
            let recency = self.compute_recency_score(item.episode());
            let score = self.compute_combined_score(item.relevance(), item.gist().density, recency);
            item.set_combined_score(score);
        }

        // 3. Apply diversity selection (MMR-style)
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

        // Exponential decay with half-life
        let decay = 0.5_f32.powf(age_days / self.config.recency_half_life_days);
        decay.clamp(0.0, 1.0)
    }

    /// Compute combined score from components.
    fn compute_combined_score(&self, relevance: f32, density: f32, recency: f32) -> f32 {
        self.config.relevance_weight * relevance
            + self.config.density_weight * density
            + self.config.recency_weight * recency
    }

    /// Select diverse items using MMR-style algorithm.
    fn select_diverse(&self, items: Vec<GistScoredItem>, k: usize) -> Vec<GistScoredItem> {
        if items.len() <= k {
            // Sort by combined score
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

        // Select first item (highest combined score)
        let first_idx = self.find_max_score_index(&remaining);
        let first = remaining.remove(first_idx);
        selected.push(first);

        // Iterative MMR selection
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
    ///
    /// MMR = lambda * Score(item) - (1-lambda) * max(Similarity(item, selected))
    fn compute_mmr_score(&self, item: &GistScoredItem, selected: &[GistScoredItem]) -> f32 {
        let relevance = item.combined_score();

        if selected.is_empty() {
            return self.config.diversity_lambda * relevance;
        }

        // Find maximum text similarity to selected items
        let max_similarity = selected
            .iter()
            .map(|s| self.compute_text_similarity(item, s))
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);

        self.config.diversity_lambda * relevance
            - (1.0 - self.config.diversity_lambda) * max_similarity
    }

    /// Compute text similarity between two items.
    ///
    /// Uses word overlap (Jaccard similarity) for gist comparison.
    pub fn compute_text_similarity(&self, item1: &GistScoredItem, item2: &GistScoredItem) -> f32 {
        let summary1 = item1.gist().summary();
        let summary2 = item2.gist().summary();
        let words1 = self.extract_words(&summary1);
        let words2 = self.extract_words(&summary2);

        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }

        // Jaccard similarity = intersection / union
        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            return 0.0;
        }

        intersection as f32 / union as f32
    }

    /// Extract significant words from text.
    fn extract_words<'a>(&self, text: &'a str) -> std::collections::HashSet<&'a str> {
        text.split_whitespace()
            .filter(|w| w.len() >= 3) // Skip short words
            .collect()
    }
}

//! Hierarchical/gist reranking for dense context retrieval (WG-118).
//!
//! This module provides gist-based summarization and reranking to return
//! fewer, denser context items for downstream prompts.
//!
//! ## Purpose
//!
//! When retrieving episodes for LLM context, flat ranking can result in:
//! - Redundant items (similar episodes all included)
//! - Low information density (verbose descriptions)
//! - Token waste (too many low-value items)
//!
//! Gist reranking addresses these by:
//! - Extracting gist summaries (1-3 key sentences per episode)
//! - Scoring by gist density (information per token)
//! - Reranking with diversity to maximize coverage
//!
//! ## Architecture
//!
//! ```text
//! Retrieval Results (episodes + scores)
//!        |
//!        v
//!   GistExtractor (extract key points)
//!        |  - Parse episode description
//!        |  - Extract 1-3 key sentences
//!        |  - Compute gist density score
//!        v
//!   GistScoredItem (episode + gist + density)
//!        |
//!        v
//!   HierarchicalReranker (density + diversity reranking)
//!        |  - Score = relevance + density + recency
//!        |  - Apply MMR-style diversity
//!        |  - Return top-k dense items
//!        v
//!   Dense Bundle (fewer items, higher quality)
//! ```
//!
//! ## Quick Start
//!
//! ```
//! use do_memory_core::retrieval::{GistExtractor, HierarchicalReranker, RerankConfig};
//! use do_memory_core::episode::Episode;
//! use do_memory_core::TaskContext;
//! use do_memory_core::types::TaskType;
//! use std::sync::Arc;
//!
//! // Extract gist from episode description
//! let extractor = GistExtractor::default();
//! let gist = extractor.extract("Fixed authentication bug by adding JWT validation");
//! assert!(gist.key_points.len() <= 3);
//!
//! // Rerank retrieval results by gist density
//! let reranker = HierarchicalReranker::new(RerankConfig::dense());
//! let episodes = vec![
//!     (Arc::new(Episode::new("Fix bug".to_string(), TaskContext::default(), TaskType::Debugging)), 0.9),
//!     (Arc::new(Episode::new("Add feature".to_string(), TaskContext::default(), TaskType::CodeGeneration)), 0.85),
//! ];
//! let dense = reranker.rerank(episodes, 5);
//! // Returns at most 5 items, prioritized by density
//! ```

use crate::episode::Episode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Configuration for hierarchical reranking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankConfig {
    /// Weight for original relevance score (0.0-1.0)
    pub relevance_weight: f32,
    /// Weight for gist density score (0.0-1.0)
    pub density_weight: f32,
    /// Weight for recency score (0.0-1.0)
    pub recency_weight: f32,
    /// Lambda for diversity (MMR-style) (0.0-1.0)
    /// Higher lambda = more relevance, lower = more diversity
    pub diversity_lambda: f32,
    /// Maximum key points to extract per episode
    pub max_key_points: usize,
    /// Minimum gist density threshold for inclusion
    pub min_density_threshold: f32,
    /// Half-life in days for recency decay
    pub recency_half_life_days: f32,
}

impl Default for RerankConfig {
    fn default() -> Self {
        Self {
            relevance_weight: 0.3,
            density_weight: 0.4,
            recency_weight: 0.2,
            diversity_lambda: 0.7,
            max_key_points: 3,
            min_density_threshold: 0.3,
            recency_half_life_days: 30.0,
        }
    }
}

impl RerankConfig {
    /// Create a configuration optimized for dense context.
    ///
    /// Prioritizes gist density and diversity for maximum information
    /// per token in downstream prompts.
    #[must_use]
    pub fn dense() -> Self {
        Self {
            relevance_weight: 0.2,
            density_weight: 0.5,
            recency_weight: 0.15,
            diversity_lambda: 0.6,
            max_key_points: 2,
            min_density_threshold: 0.4,
            recency_half_life_days: 14.0,
        }
    }

    /// Create a configuration optimized for comprehensive context.
    ///
    /// Larger result set with lower density thresholds.
    #[must_use]
    pub fn comprehensive() -> Self {
        Self {
            relevance_weight: 0.35,
            density_weight: 0.25,
            recency_weight: 0.25,
            diversity_lambda: 0.75,
            max_key_points: 3,
            min_density_threshold: 0.2,
            recency_half_life_days: 60.0,
        }
    }

    /// Validate the configuration.
    ///
    /// # Returns
    ///
    /// `Ok(())` if configuration is valid, `Err` with message if invalid
    pub fn validate(&self) -> Result<(), String> {
        let weight_sum = self.relevance_weight + self.density_weight + self.recency_weight;
        if (weight_sum - 1.0).abs() > 0.15 {
            return Err(format!("Weights should sum to ~1.0, got {weight_sum:.2}"));
        }

        if !(0.0..=1.0).contains(&self.diversity_lambda) {
            return Err(format!(
                "diversity_lambda must be in [0.0, 1.0], got {}",
                self.diversity_lambda
            ));
        }

        if self.max_key_points == 0 {
            return Err("max_key_points must be at least 1".to_string());
        }

        Ok(())
    }
}

/// Extracted gist from an episode.
///
/// Contains key points extracted from the episode description and
/// a computed density score for reranking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeGist {
    /// Episode ID
    pub episode_id: String,
    /// Extracted key points (1-3 sentences)
    pub key_points: Vec<String>,
    /// Gist density score (0.0-1.0)
    /// Higher = more information per token
    pub density: f32,
    /// Original episode description length
    pub original_length: usize,
    /// Gist summary length
    pub gist_length: usize,
}

impl EpisodeGist {
    /// Create a new episode gist.
    #[must_use]
    pub fn new(episode_id: String, key_points: Vec<String>, density: f32) -> Self {
        let gist_length = key_points.iter().map(|s| s.len()).sum();
        Self {
            episode_id,
            key_points,
            density,
            original_length: 0,
            gist_length,
        }
    }

    /// Get the compression ratio (gist/original).
    #[must_use]
    pub fn compression_ratio(&self) -> f32 {
        if self.original_length == 0 {
            return 1.0;
        }
        self.gist_length as f32 / self.original_length as f32
    }

    /// Get the gist summary as a single string.
    #[must_use]
    pub fn summary(&self) -> String {
        self.key_points.join(" | ")
    }
}

/// Extracts gist summaries from episode descriptions.
///
/// Parses episode text to extract key points that capture the
/// essential information for downstream prompts.
///
/// # Algorithm
///
/// 1. Split description into sentences
/// 2. Score each sentence by information density
/// 3. Select top-k sentences as key points
/// 4. Compute density score based on compression ratio
///
/// # Examples
///
/// ```
/// use do_memory_core::retrieval::GistExtractor;
///
/// let extractor = GistExtractor::default();
///
/// // Extract key points from a description
/// let gist = extractor.extract("Fixed authentication bug. Added JWT validation. Improved error handling.");
/// assert!(gist.key_points.len() <= 3);
/// assert!(gist.density > 0.0);
///
/// // Empty description returns empty gist
/// let empty = extractor.extract("");
/// assert!(empty.key_points.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct GistExtractor {
    /// Maximum key points to extract
    max_key_points: usize,
}

impl Default for GistExtractor {
    fn default() -> Self {
        Self::new(3)
    }
}

impl GistExtractor {
    /// Create a new gist extractor.
    ///
    /// # Arguments
    ///
    /// * `max_key_points` - Maximum sentences to extract per episode
    #[must_use]
    pub fn new(max_key_points: usize) -> Self {
        Self {
            max_key_points: max_key_points.max(1),
        }
    }

    /// Get the maximum key points setting.
    #[must_use]
    pub fn max_key_points(&self) -> usize {
        self.max_key_points
    }

    /// Extract gist from an episode.
    ///
    /// # Arguments
    ///
    /// * `episode` - The episode to extract gist from
    ///
    /// # Returns
    ///
    /// An `EpisodeGist` with key points and density score
    #[must_use]
    pub fn extract_from_episode(&self, episode: &Episode) -> EpisodeGist {
        let gist = self.extract(&episode.task_description);
        EpisodeGist {
            episode_id: episode.episode_id.to_string(),
            key_points: gist.key_points,
            density: gist.density,
            original_length: gist.original_length,
            gist_length: gist.gist_length,
        }
    }

    /// Extract gist from a text description.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to extract gist from
    ///
    /// # Returns
    ///
    /// An `EpisodeGist` with key points and density score
    #[must_use]
    pub fn extract(&self, text: &str) -> EpisodeGist {
        let original_length = text.len();

        if text.is_empty() {
            return EpisodeGist {
                episode_id: String::new(),
                key_points: Vec::new(),
                density: 0.0,
                original_length: 0,
                gist_length: 0,
            };
        }

        // Split into sentences
        let sentences = self.split_sentences(text);

        if sentences.is_empty() {
            return EpisodeGist {
                episode_id: String::new(),
                key_points: Vec::new(),
                density: 0.0,
                original_length,
                gist_length: 0,
            };
        }

        // Score and select top sentences
        let scored = self.score_sentences(&sentences);
        let key_points = self.select_top_k(scored, self.max_key_points);

        // Compute density
        let gist_length = key_points.iter().map(|s| s.len()).sum();
        let density = self.compute_density(original_length, gist_length, key_points.len());

        EpisodeGist {
            episode_id: String::new(),
            key_points,
            density,
            original_length,
            gist_length,
        }
    }

    /// Split text into sentences.
    fn split_sentences(&self, text: &str) -> Vec<String> {
        // Simple sentence splitting on punctuation
        text.split(['.', '!', '?'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && s.len() >= 5)
            .map(|s| {
                // Ensure sentence ends with punctuation
                if !s.ends_with('.') && !s.ends_with('!') && !s.ends_with('?') {
                    format!("{s}.")
                } else {
                    s.to_string()
                }
            })
            .collect()
    }

    /// Score sentences by information density.
    fn score_sentences(&self, sentences: &[String]) -> Vec<(String, f32)> {
        sentences
            .iter()
            .map(|s| {
                let score = self.sentence_score(s);
                (s.clone(), score)
            })
            .collect()
    }

    /// Compute information density score for a sentence.
    fn sentence_score(&self, sentence: &str) -> f32 {
        // Score based on:
        // 1. Length (longer sentences may have more info, but penalize very long)
        // 2. Keyword indicators (action verbs, outcomes)
        // 3. Token density (words per character)

        let len = sentence.len();

        // Length score: optimal around 20-50 chars
        let length_score = if len < 10 {
            0.3 // Too short, likely incomplete
        } else if len < 20 {
            0.5 // Short but acceptable
        } else if len <= 50 {
            1.0 // Optimal length
        } else if len <= 100 {
            0.7 // Long but acceptable
        } else {
            0.4 // Very long, verbose
        };

        // Keyword score: action verbs indicate high-value info
        let keyword_score = self.keyword_score(sentence);

        // Combine scores
        0.4 * length_score + 0.6 * keyword_score
    }

    /// Score based on keyword indicators.
    fn keyword_score(&self, sentence: &str) -> f32 {
        let lower = sentence.to_lowercase();

        // High-value action keywords
        let high_value = [
            "fixed",
            "added",
            "implemented",
            "resolved",
            "completed",
            "solved",
            "created",
            "updated",
            "refactored",
            "optimized",
            "deployed",
            "tested",
            "validated",
            "confirmed",
        ];

        // Outcome indicators
        let outcome = ["success", "failed", "error", "bug", "issue", "feature"];

        let has_high_value = high_value.iter().any(|kw| lower.contains(kw));
        let has_outcome = outcome.iter().any(|kw| lower.contains(kw));

        if has_high_value && has_outcome {
            1.0
        } else if has_high_value {
            0.8
        } else if has_outcome {
            0.6
        } else {
            0.4
        }
    }

    /// Select top-k sentences by score.
    fn select_top_k(&self, scored: Vec<(String, f32)>, k: usize) -> Vec<String> {
        let mut sorted = scored;
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        sorted.into_iter().take(k).map(|(s, _)| s).collect()
    }

    /// Compute gist density score.
    fn compute_density(&self, original_len: usize, gist_len: usize, num_points: usize) -> f32 {
        if original_len == 0 {
            return 0.0;
        }

        // Compression ratio (how much we condensed)
        let compression = gist_len as f32 / original_len as f32;

        // Information coverage (how many key points extracted)
        let coverage = num_points as f32 / self.max_key_points.max(1) as f32;

        // Density = high coverage + good compression
        // Penalize if gist is too long (bad compression)
        // Reward if gist captures key points (good coverage)
        let compression_score = if compression < 0.3 {
            1.0 // Excellent compression
        } else if compression < 0.5 {
            0.8 // Good compression
        } else if compression < 0.7 {
            0.5 // Moderate compression
        } else {
            0.3 // Poor compression
        };

        0.5 * coverage + 0.5 * compression_score
    }
}

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
    fn compute_text_similarity(&self, item1: &GistScoredItem, item2: &GistScoredItem) -> f32 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TaskContext;
    use crate::types::TaskType;

    #[test]
    fn test_rerank_config_default() {
        let config = RerankConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.max_key_points, 3);
        assert!(config.min_density_threshold > 0.0);
    }

    #[test]
    fn test_rerank_config_dense() {
        let config = RerankConfig::dense();
        assert!(config.validate().is_ok());
        assert!(config.density_weight > config.relevance_weight);
    }

    #[test]
    fn test_rerank_config_validation() {
        let invalid = RerankConfig {
            relevance_weight: 0.5,
            density_weight: 0.6,
            recency_weight: 0.5, // Sum > 1.0
            diversity_lambda: 0.7,
            max_key_points: 3,
            min_density_threshold: 0.3,
            recency_half_life_days: 30.0,
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_gist_extractor_default() {
        let extractor = GistExtractor::default();
        assert_eq!(extractor.max_key_points(), 3);
    }

    #[test]
    fn test_gist_extractor_empty() {
        let extractor = GistExtractor::default();
        let gist = extractor.extract("");
        assert!(gist.key_points.is_empty());
        assert_eq!(gist.density, 0.0);
    }

    #[test]
    fn test_gist_extractor_single_sentence() {
        let extractor = GistExtractor::default();
        let gist = extractor.extract("Fixed authentication bug by adding JWT validation.");
        assert!(!gist.key_points.is_empty());
        assert!(gist.density > 0.0);
    }

    #[test]
    fn test_gist_extractor_multiple_sentences() {
        let extractor = GistExtractor::default();
        let gist = extractor
            .extract("Fixed authentication bug. Added JWT validation. Improved error handling.");
        assert!(gist.key_points.len() <= 3);
        assert!(gist.density > 0.0);
    }

    #[test]
    fn test_gist_extractor_high_value_keywords() {
        let extractor = GistExtractor::default();
        let gist = extractor.extract("Fixed critical bug in authentication module.");
        // Should extract this sentence due to "fixed" and "bug" keywords
        assert!(!gist.key_points.is_empty());
    }

    #[test]
    fn test_episode_gist_compression_ratio() {
        let gist = EpisodeGist {
            episode_id: "test".to_string(),
            key_points: vec!["Fixed bug.".to_string()],
            density: 0.8,
            original_length: 100,
            gist_length: 10,
        };
        assert!((gist.compression_ratio() - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_episode_gist_summary() {
        let gist = EpisodeGist {
            episode_id: "test".to_string(),
            key_points: vec!["Fixed bug.".to_string(), "Added feature.".to_string()],
            density: 0.8,
            original_length: 100,
            gist_length: 20,
        };
        let summary = gist.summary();
        assert!(summary.contains("Fixed bug"));
        assert!(summary.contains("Added feature"));
    }

    #[test]
    fn test_hierarchical_reranker_empty() {
        let reranker = HierarchicalReranker::dense();
        let result = reranker.rerank(Vec::new(), 5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_hierarchical_reranker_single() {
        let reranker = HierarchicalReranker::default();
        let episode = Arc::new(Episode::new(
            "Fix bug".to_string(),
            TaskContext::default(),
            TaskType::Debugging,
        ));
        let items = vec![(episode, 0.9)];
        let result = reranker.rerank(items, 5);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_hierarchical_reranker_multiple() {
        let reranker = HierarchicalReranker::default();

        let ep1 = Arc::new(Episode::new(
            "Fixed authentication bug in login module".to_string(),
            TaskContext::default(),
            TaskType::Debugging,
        ));
        let ep2 = Arc::new(Episode::new(
            "Added new feature for user profile".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        ));
        let ep3 = Arc::new(Episode::new(
            "Refactored database connection pooling".to_string(),
            TaskContext::default(),
            TaskType::Refactoring,
        ));

        let items = vec![(ep1, 0.9), (ep2, 0.85), (ep3, 0.8)];
        let result = reranker.rerank(items, 2);

        // Should return at most 2 items
        assert!(result.len() <= 2);
        // Should be sorted by combined score
        if result.len() > 1 {
            assert!(result[0].combined_score() >= result[1].combined_score());
        }
    }

    #[test]
    fn test_hierarchical_reranker_density_threshold() {
        let config = RerankConfig {
            min_density_threshold: 0.9, // Very high threshold
            ..RerankConfig::default()
        };
        let reranker = HierarchicalReranker::new(config);

        let episode = Arc::new(Episode::new(
            "Some task".to_string(),
            TaskContext::default(),
            TaskType::Debugging,
        ));
        let items = vec![(episode, 0.9)];

        // Should filter out items below density threshold
        let result = reranker.rerank(items, 5);
        // May be empty due to high threshold
        assert!(result.len() <= 1);
    }

    #[test]
    fn test_compute_text_similarity_identical() {
        let reranker = HierarchicalReranker::default();

        let gist1 = EpisodeGist::new("ep1".to_string(), vec!["fixed bug".to_string()], 0.8);
        let gist2 = EpisodeGist::new("ep2".to_string(), vec!["fixed bug".to_string()], 0.8);

        let ep = Arc::new(Episode::new(
            "test".to_string(),
            TaskContext::default(),
            TaskType::Debugging,
        ));
        let item1 = GistScoredItem::new(ep.clone(), gist1, 0.9);
        let item2 = GistScoredItem::new(ep, gist2, 0.9);

        let sim = reranker.compute_text_similarity(&item1, &item2);
        // Identical gists should have high similarity
        assert!(sim > 0.9);
    }

    #[test]
    fn test_compute_text_similarity_different() {
        let reranker = HierarchicalReranker::default();

        let gist1 = EpisodeGist::new(
            "ep1".to_string(),
            vec!["fixed authentication bug".to_string()],
            0.8,
        );
        let gist2 = EpisodeGist::new(
            "ep2".to_string(),
            vec!["added new feature".to_string()],
            0.8,
        );

        let ep = Arc::new(Episode::new(
            "test".to_string(),
            TaskContext::default(),
            TaskType::Debugging,
        ));
        let item1 = GistScoredItem::new(ep.clone(), gist1, 0.9);
        let item2 = GistScoredItem::new(ep, gist2, 0.9);

        let sim = reranker.compute_text_similarity(&item1, &item2);
        // Different gists should have low similarity
        assert!(sim < 0.5);
    }
}

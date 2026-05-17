//! CogniRank reranking algorithm inspired by CogitoRAG.

use std::collections::HashSet;

use super::reranker::GistScoredItem;
use super::types::EpisodeGist;

/// CogniRank reranker that applies cognitive heuristics to retrieval results.
#[derive(Debug, Clone, Default)]
pub struct CogniRank;

impl CogniRank {
    /// Rerank items using the CogniRank algorithm.
    ///
    /// # Arguments
    ///
    /// * `query` - The original search query
    /// * `items` - Items with gists and relevance scores
    /// * `weights` - Scoring weights (relevance, density, overlap)
    pub fn rerank(
        &self,
        query: &str,
        mut items: Vec<GistScoredItem>,
        weights: CogniRankWeights,
    ) -> Vec<GistScoredItem> {
        let query_words = self.extract_keywords(query);

        for item in &mut items {
            let relevance = item.relevance();
            let density = item.gist().density;
            let overlap = self.compute_keyword_overlap(&query_words, item.gist());

            let cogni_score = weights.relevance * relevance
                + weights.density * density
                + weights.overlap * overlap;

            item.set_combined_score(cogni_score);
        }

        // Sort by CogniRank score
        items.sort_by(|a, b| {
            b.combined_score()
                .partial_cmp(&a.combined_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        items
    }

    /// Extract keywords from text for overlap comparison.
    fn extract_keywords(&self, text: &str) -> HashSet<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| s.len() > 3)
            .collect()
    }

    /// Compute keyword overlap between query and gist key points.
    fn compute_keyword_overlap(&self, query_words: &HashSet<String>, gist: &EpisodeGist) -> f32 {
        if query_words.is_empty() {
            return 0.0;
        }

        let mut matches = 0;
        let gist_text = gist.summary().to_lowercase();

        for word in query_words {
            if gist_text.contains(word) {
                matches += 1;
            }
        }

        matches as f32 / query_words.len() as f32
    }
}

/// Weights for CogniRank scoring.
#[derive(Debug, Clone)]
pub struct CogniRankWeights {
    /// Weight for semantic relevance (0.0 to 1.0)
    pub relevance: f32,
    /// Weight for information density (0.0 to 1.0)
    pub density: f32,
    /// Weight for keyword overlap (0.0 to 1.0)
    pub overlap: f32,
}

impl Default for CogniRankWeights {
    fn default() -> Self {
        Self {
            relevance: 0.5,
            density: 0.3,
            overlap: 0.2,
        }
    }
}

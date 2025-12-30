//! Hybrid search combining vector similarity and full-text search
//!
//! This module provides hybrid search capabilities that combine:
//! 1. Vector similarity search (semantic understanding)
//! 2. FTS5 full-text search (keyword matching)
//!
//! The hybrid approach provides better relevance than either method alone,
//! especially for queries that contain both semantic meaning and specific keywords.

use anyhow::Result;
use std::fmt;

/// Configuration for hybrid search weighting
///
/// Determines how much weight to give to vector similarity vs keyword relevance.
/// Both weights should sum to 1.0 for normalized scoring.
#[derive(Debug, Clone, Copy)]
pub struct HybridSearchConfig {
    /// Weight for vector similarity (0.0 to 1.0)
    pub vector_weight: f32,
    /// Weight for full-text search relevance (0.0 to 1.0)
    pub fts_weight: f32,
}

impl HybridSearchConfig {
    /// Create a new configuration with normalized weights
    ///
    /// # Arguments
    ///
    /// * `vector_weight` - Weight for vector similarity (will be normalized)
    /// * `fts_weight` - Weight for FTS relevance (will be normalized)
    ///
    /// # Returns
    ///
    /// Normalized configuration where weights sum to 1.0
    #[must_use]
    pub fn new(vector_weight: f32, fts_weight: f32) -> Self {
        let total = vector_weight + fts_weight;
        Self {
            vector_weight: vector_weight / total,
            fts_weight: fts_weight / total,
        }
    }

    /// Default configuration favoring vector similarity (0.7 vector, 0.3 FTS)
    #[must_use]
    pub fn default_config() -> Self {
        Self::new(0.7, 0.3)
    }

    /// Vector-only configuration (1.0 vector, 0.0 FTS)
    #[must_use]
    pub fn vector_only() -> Self {
        Self {
            vector_weight: 1.0,
            fts_weight: 0.0,
        }
    }

    /// Keyword-only configuration (0.0 vector, 1.0 FTS)
    #[must_use]
    pub fn keyword_only() -> Self {
        Self {
            vector_weight: 0.0,
            fts_weight: 1.0,
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if !(0.0..=1.0).contains(&self.vector_weight) {
            anyhow::bail!("Vector weight must be between 0.0 and 1.0");
        }
        if !(0.0..=1.0).contains(&self.fts_weight) {
            anyhow::bail!("FTS weight must be between 0.0 and 1.0");
        }
        if (self.vector_weight + self.fts_weight - 1.0).abs() > 0.0001 {
            anyhow::bail!(
                "Weights must sum to 1.0 (got {} + {} = {})",
                self.vector_weight,
                self.fts_weight,
                self.vector_weight + self.fts_weight
            );
        }
        Ok(())
    }
}

impl Default for HybridSearchConfig {
    fn default() -> Self {
        Self::default_config()
    }
}

/// Hybrid search result with combined score
#[derive(Debug, Clone)]
pub struct HybridSearchResult<T> {
    /// The matched item
    pub item: T,
    /// Combined hybrid score (0.0 to 1.0)
    pub hybrid_score: f32,
    /// Vector similarity component (0.0 to 1.0)
    pub vector_score: f32,
    /// FTS relevance component (0.0 to 1.0)
    pub fts_score: f32,
}

impl<T> HybridSearchResult<T> {
    /// Create a new hybrid search result
    #[must_use]
    pub fn new(item: T, vector_score: f32, fts_score: f32, config: &HybridSearchConfig) -> Self {
        let hybrid_score = config.vector_weight * vector_score + config.fts_weight * fts_score;
        Self {
            item,
            hybrid_score,
            vector_score,
            fts_score,
        }
    }
}

/// Hybrid search engine
///
/// Combines vector similarity search with FTS5 full-text search
/// to provide enhanced retrieval capabilities.
pub struct HybridSearch {
    config: HybridSearchConfig,
}

impl HybridSearch {
    /// Create a new hybrid search engine with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: HybridSearchConfig::default(),
        }
    }

    /// Create a new hybrid search engine with custom configuration
    pub fn with_config(config: HybridSearchConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self { config })
    }

    /// Search for episodes using hybrid approach
    ///
    /// # Arguments
    ///
    /// * `vector_results` - Vector similarity search results with scores
    /// * `fts_results` - Full-text search results with relevance scores
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    ///
    /// Combined and ranked results based on hybrid scoring
    #[must_use]
    pub fn search_episodes<T>(
        &self,
        vector_results: Vec<(T, f32)>,
        fts_results: Vec<(T, f32)>,
        limit: usize,
    ) -> Vec<HybridSearchResult<T>>
    where
        T: Clone + PartialEq + Eq + std::hash::Hash,
    {
        // Create maps for quick lookup
        let mut vector_map = std::collections::HashMap::new();
        for (item, score) in vector_results {
            vector_map.insert(item, score);
        }

        let mut fts_map = std::collections::HashMap::new();
        for (item, score) in fts_results {
            fts_map.insert(item, score);
        }

        // Combine scores for items that appear in both result sets
        let mut combined = Vec::new();

        // Process items from vector results
        for (item, vector_score) in &vector_map {
            let fts_score = fts_map.get(item).copied().unwrap_or(0.0);
            let result =
                HybridSearchResult::new((*item).clone(), *vector_score, fts_score, &self.config);
            combined.push(result);
        }

        // Add items that only appear in FTS results
        for (item, fts_score) in &fts_map {
            if !vector_map.contains_key(item) {
                let result = HybridSearchResult::new(
                    (*item).clone(),
                    0.0, // No vector score
                    *fts_score,
                    &self.config,
                );
                combined.push(result);
            }
        }

        // Sort by hybrid score (descending) and take limit
        combined.sort_by(|a, b| b.hybrid_score.partial_cmp(&a.hybrid_score).unwrap());
        combined.truncate(limit);

        combined
    }

    /// Get the current configuration
    #[must_use]
    pub fn config(&self) -> &HybridSearchConfig {
        &self.config
    }

    /// Update the configuration
    pub fn update_config(&mut self, config: HybridSearchConfig) -> Result<()> {
        config.validate()?;
        self.config = config;
        Ok(())
    }
}

impl Default for HybridSearch {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for HybridSearch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HybridSearch")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_search_config() {
        let config = HybridSearchConfig::new(0.7, 0.3);
        assert!((config.vector_weight - 0.7).abs() < 0.001);
        assert!((config.fts_weight - 0.3).abs() < 0.001);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_hybrid_search_config_normalization() {
        let config = HybridSearchConfig::new(2.0, 1.0);
        assert!((config.vector_weight - 0.6666667).abs() < 0.001);
        assert!((config.fts_weight - 0.3333333).abs() < 0.001);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_hybrid_search_config_validation() {
        let invalid = HybridSearchConfig {
            vector_weight: 1.5,
            fts_weight: -0.5,
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_hybrid_search_result() {
        let config = HybridSearchConfig::new(0.7, 0.3);
        let item = "test item".to_string();
        let result = HybridSearchResult::new(item.clone(), 0.8, 0.6, &config);

        let expected_score = 0.7 * 0.8 + 0.3 * 0.6; // 0.56 + 0.18 = 0.74
        assert!((result.hybrid_score - expected_score).abs() < 0.001);
        assert!((result.vector_score - 0.8).abs() < 0.001);
        assert!((result.fts_score - 0.6).abs() < 0.001);
        assert_eq!(result.item, item);
    }

    #[test]
    fn test_hybrid_search_engine() {
        let engine = HybridSearch::new();
        let config = engine.config();

        // Test that default config is valid
        assert!(config.validate().is_ok());
        assert!((config.vector_weight + config.fts_weight - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_search_episodes() {
        let config = HybridSearchConfig::new(0.5, 0.5);
        let engine = HybridSearch::with_config(config).unwrap();

        let vector_results = vec![("item1".to_string(), 0.9), ("item2".to_string(), 0.7)];

        let fts_results = vec![("item2".to_string(), 0.8), ("item3".to_string(), 0.6)];

        let results = engine.search_episodes(vector_results, fts_results, 5);

        assert_eq!(results.len(), 3); // item1, item2, item3

        // item2 should have highest score (0.7*0.5 + 0.8*0.5 = 0.75)
        // item1 should have 0.9*0.5 + 0.0*0.5 = 0.45
        // item3 should have 0.0*0.5 + 0.6*0.5 = 0.30

        assert_eq!(results[0].item, "item2");
        assert!((results[0].hybrid_score - 0.75).abs() < 0.001);

        assert_eq!(results[1].item, "item1");
        assert!((results[1].hybrid_score - 0.45).abs() < 0.001);

        assert_eq!(results[2].item, "item3");
        assert!((results[2].hybrid_score - 0.30).abs() < 0.001);
    }

    #[test]
    fn test_search_episodes_with_limit() {
        let config = HybridSearchConfig::new(0.5, 0.5);
        let engine = HybridSearch::with_config(config).unwrap();

        let vector_results = vec![
            ("item1".to_string(), 0.9),
            ("item2".to_string(), 0.7),
            ("item3".to_string(), 0.5),
        ];

        let fts_results = vec![
            ("item1".to_string(), 0.1),
            ("item2".to_string(), 0.8),
            ("item3".to_string(), 0.6),
        ];

        let results = engine.search_episodes(vector_results, fts_results, 2);

        assert_eq!(results.len(), 2);

        // item2 should be first (0.7*0.5 + 0.8*0.5 = 0.75)
        // item3 should be second (0.5*0.5 + 0.6*0.5 = 0.55)
        // item1 should be excluded by limit (0.9*0.5 + 0.1*0.5 = 0.50)

        assert_eq!(results[0].item, "item2");
        assert_eq!(results[1].item, "item3");
    }
}

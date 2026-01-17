//! Pattern search types and configuration

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Result from semantic pattern search with scoring details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternSearchResult {
    /// The matched pattern
    pub pattern: crate::pattern::Pattern,
    /// Overall relevance score (0.0 to 1.0)
    pub relevance_score: f32,
    /// Breakdown of scoring components
    pub score_breakdown: ScoreBreakdown,
}

/// Detailed breakdown of relevance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    /// Semantic similarity from embeddings (0.0 to 1.0)
    pub semantic_similarity: f32,
    /// Context match score (0.0 to 1.0)
    pub context_match: f32,
    /// Effectiveness score based on past usage (0.0 to 1.0+)
    pub effectiveness: f32,
    /// Recency score (0.0 to 1.0)
    pub recency: f32,
    /// Success rate of the pattern (0.0 to 1.0)
    pub success_rate: f32,
}

/// Configuration for pattern search
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Minimum relevance score to include (0.0 to 1.0)
    pub min_relevance: f32,
    /// Weight for semantic similarity (default: 0.4)
    pub semantic_weight: f32,
    /// Weight for context matching (default: 0.2)
    pub context_weight: f32,
    /// Weight for effectiveness (default: 0.2)
    pub effectiveness_weight: f32,
    /// Weight for recency (default: 0.1)
    pub recency_weight: f32,
    /// Weight for success rate (default: 0.1)
    pub success_weight: f32,
    /// Whether to filter by domain
    pub filter_by_domain: bool,
    /// Whether to filter by task type
    pub filter_by_task_type: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            min_relevance: 0.3,
            semantic_weight: 0.4,
            context_weight: 0.2,
            effectiveness_weight: 0.2,
            recency_weight: 0.1,
            success_weight: 0.1,
            filter_by_domain: false,
            filter_by_task_type: false,
        }
    }
}

impl SearchConfig {
    /// Create a strict search config (high threshold, domain filtering)
    #[must_use]
    pub fn strict() -> Self {
        Self {
            min_relevance: 0.6,
            filter_by_domain: true,
            filter_by_task_type: true,
            ..Default::default()
        }
    }

    /// Create a relaxed search config (low threshold, no filtering)
    #[must_use]
    pub fn relaxed() -> Self {
        Self {
            min_relevance: 0.2,
            filter_by_domain: false,
            filter_by_task_type: false,
            ..Default::default()
        }
    }
}

/// Calculate Jaccard similarity between two sets
fn jaccard_similarity(set1: &HashSet<&String>, set2: &HashSet<&String>) -> f32 {
    if set1.is_empty() && set2.is_empty() {
        return 1.0;
    }

    let intersection = set1.intersection(set2).count();
    let union = set1.union(set2).count();

    if union > 0 {
        intersection as f32 / union as f32
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_config_defaults() {
        let config = SearchConfig::default();
        assert_eq!(config.min_relevance, 0.3);
        assert_eq!(config.semantic_weight, 0.4);
        assert!(!config.filter_by_domain);
    }

    #[test]
    fn test_search_config_strict() {
        let config = SearchConfig::strict();
        assert_eq!(config.min_relevance, 0.6);
        assert!(config.filter_by_domain);
    }

    #[test]
    fn test_jaccard_similarity() {
        let set1: HashSet<&String> = vec!["a", "b", "c"].iter().collect();
        let set2: HashSet<&String> = vec!["b", "c", "d"].iter().collect();

        let similarity = jaccard_similarity(&set1, &set2);
        // Intersection: {b, c} = 2, Union: {a, b, c, d} = 4
        assert_eq!(similarity, 0.5);
    }

    #[test]
    fn test_jaccard_similarity_empty() {
        let set1: HashSet<&String> = vec![].iter().collect();
        let set2: HashSet<&String> = vec![].iter().collect();

        let similarity = jaccard_similarity(&set1, &set2);
        assert_eq!(similarity, 1.0);
    }
}

//! Configuration and result types for the cascading retrieval pipeline.
//!
//! Split out of `mod.rs` to keep individual source files under the 500 LOC
//! quality gate (WG-185).

/// Configuration for the cascading retrieval pipeline.
#[derive(Debug, Clone)]
pub struct CascadeConfig {
    /// Number of results to return from each tier.
    pub top_k: usize,
    /// Minimum score threshold for BM25 results (0.0-1.0).
    pub bm25_threshold: f32,
    /// Minimum similarity threshold for HDC results (0.0-1.0).
    pub hdc_threshold: f32,
    /// Minimum confidence threshold for ConceptGraph results (0.0-1.0).
    pub concept_graph_threshold: f32,
    /// Whether to merge results across tiers.
    pub merge_results: bool,
    /// Minimum results before escalating to next tier.
    pub min_results: usize,
    /// Enable/disable ConceptGraph expansion.
    pub enable_concept_expansion: bool,
}

impl Default for CascadeConfig {
    fn default() -> Self {
        Self {
            top_k: 10,
            bm25_threshold: 0.3,
            hdc_threshold: 0.5,
            concept_graph_threshold: 0.4,
            merge_results: true,
            min_results: 3,
            enable_concept_expansion: true,
        }
    }
}

/// Result from a single tier in the cascade.
#[derive(Debug, Clone)]
pub struct TierResult {
    /// Tier identifier (bm25, hdc, concept_graph, api).
    pub tier: String,
    /// Retrieved episode IDs with scores as tuples.
    pub results: Vec<(String, f32)>,
    /// Whether this tier produced sufficient results.
    pub sufficient: bool,
}

impl TierResult {
    /// Get episode IDs from results.
    #[must_use]
    pub fn ids(&self) -> Vec<String> {
        self.results.iter().map(|(id, _)| id.clone()).collect()
    }

    /// Get scores from results.
    #[must_use]
    pub fn scores(&self) -> Vec<f32> {
        self.results.iter().map(|(_, score)| *score).collect()
    }

    /// Check if results are empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Get number of results.
    #[must_use]
    pub fn len(&self) -> usize {
        self.results.len()
    }
}

/// Final result from the cascading retrieval pipeline.
#[derive(Debug, Clone)]
pub struct CascadeResult {
    /// Final merged/re-ranked episode IDs.
    pub episode_ids: Vec<String>,
    /// Final merged/re-ranked scores.
    pub scores: Vec<f32>,
    /// Which tier(s) contributed to the final result.
    pub contributing_tiers: Vec<String>,
    /// Number of API calls made (should be 0 or 1).
    pub api_calls: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cascade_config_default_returns_expected_values() {
        let config = CascadeConfig::default();

        assert_eq!(config.top_k, 10);
        assert_eq!(config.bm25_threshold, 0.3);
        assert_eq!(config.hdc_threshold, 0.5);
        assert_eq!(config.concept_graph_threshold, 0.4);
        assert!(config.merge_results);
        assert_eq!(config.min_results, 3);
        assert!(config.enable_concept_expansion);
    }

    #[test]
    fn tier_result_ids_returns_episode_ids() {
        let result = TierResult {
            tier: "bm25".to_string(),
            results: vec![
                ("ep1".to_string(), 0.9),
                ("ep2".to_string(), 0.8),
                ("ep3".to_string(), 0.7),
            ],
            sufficient: true,
        };

        let ids = result.ids();

        assert_eq!(ids, vec!["ep1", "ep2", "ep3"]);
    }

    #[test]
    fn tier_result_scores_returns_score_values() {
        let result = TierResult {
            tier: "hdc".to_string(),
            results: vec![("ep1".to_string(), 0.95), ("ep2".to_string(), 0.85)],
            sufficient: true,
        };

        let scores = result.scores();

        assert_eq!(scores.len(), 2);
        assert!((scores[0] - 0.95).abs() < f32::EPSILON);
        assert!((scores[1] - 0.85).abs() < f32::EPSILON);
    }

    #[test]
    fn tier_result_is_empty_true_when_no_results() {
        let result = TierResult {
            tier: "concept_graph".to_string(),
            results: vec![],
            sufficient: false,
        };

        assert!(result.is_empty());
    }

    #[test]
    fn tier_result_is_empty_false_when_has_results() {
        let result = TierResult {
            tier: "bm25".to_string(),
            results: vec![("ep1".to_string(), 0.9)],
            sufficient: true,
        };

        assert!(!result.is_empty());
    }

    #[test]
    fn tier_result_len_returns_correct_count() {
        let result = TierResult {
            tier: "api".to_string(),
            results: vec![
                ("ep1".to_string(), 0.9),
                ("ep2".to_string(), 0.8),
                ("ep3".to_string(), 0.7),
                ("ep4".to_string(), 0.6),
            ],
            sufficient: true,
        };

        assert_eq!(result.len(), 4);
    }

    #[test]
    fn tier_result_empty_results() {
        let result = TierResult {
            tier: "bm25".to_string(),
            results: vec![],
            sufficient: false,
        };

        assert!(result.is_empty());
        assert_eq!(result.len(), 0);
        assert!(result.ids().is_empty());
        assert!(result.scores().is_empty());
    }

    #[test]
    fn cascade_result_construction_and_field_access() {
        let result = CascadeResult {
            episode_ids: vec!["ep1".to_string(), "ep2".to_string()],
            scores: vec![0.9, 0.8],
            contributing_tiers: vec!["bm25".to_string(), "hdc".to_string()],
            api_calls: 0,
        };

        assert_eq!(result.episode_ids, vec!["ep1", "ep2"]);
        assert_eq!(result.scores.len(), 2);
        assert!((result.scores[0] - 0.9).abs() < f32::EPSILON);
        assert!((result.scores[1] - 0.8).abs() < f32::EPSILON);
        assert_eq!(result.contributing_tiers, vec!["bm25", "hdc"]);
        assert_eq!(result.api_calls, 0);
    }
}

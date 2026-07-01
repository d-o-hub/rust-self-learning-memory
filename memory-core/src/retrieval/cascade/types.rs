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

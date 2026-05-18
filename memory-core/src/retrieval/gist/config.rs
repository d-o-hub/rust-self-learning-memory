//! Configuration for gist reranking.

use serde::{Deserialize, Serialize};

/// Configuration for hierarchical gist reranking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankConfig {
    /// Weight for original relevance score (0.0-1.0)
    pub relevance_weight: f32,
    /// Weight for gist density score (0.0-1.0)
    pub density_weight: f32,
    /// Weight for gist-to-query similarity (0.0-1.0, CogniRank)
    pub gist_query_similarity_weight: f32,
    /// Weight for recency score (0.0-1.0)
    pub recency_weight: f32,
    /// Lambda for MMR diversity (0.0-1.0)
    /// 1.0 = relevance only, 0.0 = diversity only
    pub diversity_lambda: f32,
    /// Maximum key points to extract per episode
    pub max_key_points: usize,
    /// Minimum density threshold to include in results
    pub min_density_threshold: f32,
    /// Half-life for recency decay in days
    pub recency_half_life_days: f32,
}

impl Default for RerankConfig {
    fn default() -> Self {
        Self {
            relevance_weight: 0.3,
            density_weight: 0.3,
            gist_query_similarity_weight: 0.2,
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
    #[must_use]
    pub fn dense() -> Self {
        Self {
            relevance_weight: 0.2,
            density_weight: 0.5,
            gist_query_similarity_weight: 0.15,
            recency_weight: 0.15,
            diversity_lambda: 0.6,
            max_key_points: 2,
            min_density_threshold: 0.4,
            recency_half_life_days: 14.0,
        }
    }

    /// Create a configuration optimized for CogniRank (query alignment).
    #[must_use]
    pub fn cognirank() -> Self {
        Self {
            relevance_weight: 0.2,
            density_weight: 0.3,
            gist_query_similarity_weight: 0.4,
            recency_weight: 0.1,
            diversity_lambda: 0.7,
            max_key_points: 3,
            min_density_threshold: 0.3,
            recency_half_life_days: 30.0,
        }
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), String> {
        let weight_sum = self.relevance_weight
            + self.density_weight
            + self.gist_query_similarity_weight
            + self.recency_weight;
        if (weight_sum - 1.0).abs() > 0.15 {
            return Err(format!("Weights should sum to ~1.0, got {weight_sum:.2}"));
        }
        if !(0.0..=1.0).contains(&self.diversity_lambda) {
            return Err("diversity_lambda must be in [0.0, 1.0]".to_string());
        }
        Ok(())
    }
}

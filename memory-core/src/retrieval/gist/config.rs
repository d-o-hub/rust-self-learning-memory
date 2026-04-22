//! Configuration for hierarchical reranking.

use serde::{Deserialize, Serialize};

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

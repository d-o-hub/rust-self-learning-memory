//! Quality configuration and feature types.
//!
//! Contains the configuration and feature enums used by the quality assessor.

/// Configuration for quality assessment.
///
/// Controls the quality threshold and feature weights used to assess
/// episode quality before storage.
///
/// # Examples
///
/// ```
/// use memory_core::pre_storage::{QualityConfig, QualityFeature};
/// use std::collections::HashMap;
///
/// // Default configuration (quality threshold 0.7)
/// let config = QualityConfig::default();
///
/// // Custom configuration with higher threshold
/// let mut custom_config = QualityConfig::new(0.8);
/// custom_config.set_weight(QualityFeature::TaskComplexity, 0.3);
/// custom_config.set_weight(QualityFeature::StepDiversity, 0.2);
/// ```
#[derive(Debug, Clone)]
pub struct QualityConfig {
    /// Minimum quality score required for storage (0.0 to 1.0)
    pub quality_threshold: f32,
    /// Weights for each quality feature
    feature_weights: std::collections::HashMap<QualityFeature, f32>,
}

impl QualityConfig {
    /// Create a new quality configuration with the specified threshold.
    ///
    /// Uses default feature weights that sum to 1.0.
    ///
    /// # Arguments
    ///
    /// * `quality_threshold` - Minimum quality score (0.0 to 1.0)
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::pre_storage::QualityConfig;
    ///
    /// let config = QualityConfig::new(0.75);
    /// assert_eq!(config.quality_threshold, 0.75);
    /// ```
    #[must_use]
    pub fn new(quality_threshold: f32) -> Self {
        let mut feature_weights = std::collections::HashMap::new();
        feature_weights.insert(QualityFeature::TaskComplexity, 0.25);
        feature_weights.insert(QualityFeature::StepDiversity, 0.20);
        feature_weights.insert(QualityFeature::ErrorRate, 0.20);
        feature_weights.insert(QualityFeature::ReflectionDepth, 0.20);
        feature_weights.insert(QualityFeature::PatternNovelty, 0.15);

        Self {
            quality_threshold,
            feature_weights,
        }
    }

    /// Set the weight for a specific quality feature.
    ///
    /// # Arguments
    ///
    /// * `feature` - The quality feature to set weight for
    /// * `weight` - The weight value (should sum to 1.0 across all features)
    pub fn set_weight(&mut self, feature: QualityFeature, weight: f32) {
        self.feature_weights.insert(feature, weight);
    }

    /// Get the weight for a specific quality feature.
    ///
    /// # Arguments
    ///
    /// * `feature` - The quality feature to get weight for
    ///
    /// # Returns
    ///
    /// The weight value, or 0.0 if not set
    #[must_use]
    pub fn get_weight(&self, feature: QualityFeature) -> f32 {
        *self.feature_weights.get(&feature).unwrap_or(&0.0)
    }
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self::new(0.7)
    }
}

/// Quality features used to assess episode quality.
///
/// Each feature measures a different aspect of episode quality and
/// contributes to the overall quality score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QualityFeature {
    /// Task complexity based on number of steps and tool diversity
    TaskComplexity,
    /// Diversity of steps taken during execution
    StepDiversity,
    /// Error rate and recovery quality
    ErrorRate,
    /// Depth and quality of reflections
    ReflectionDepth,
    /// Novelty of patterns discovered
    PatternNovelty,
}

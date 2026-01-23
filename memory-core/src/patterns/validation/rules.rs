//! Validation rules for pattern quality
//!
//! Provides pattern validation logic including confidence checking,
//! effectiveness tracking, and pattern matching rules.

use crate::pattern::Pattern;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, instrument};
use uuid::Uuid;

/// Pattern accuracy metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternMetrics {
    /// Precision: TP / (TP + FP) - proportion of predicted positives that are correct
    pub precision: f32,
    /// Recall: TP / (TP + FN) - proportion of actual positives that were identified
    pub recall: f32,
    /// F1 Score: 2 * (precision * recall) / (precision + recall) - harmonic mean
    pub f1_score: f32,
    /// Accuracy: (TP + TN) / Total - overall correctness
    pub accuracy: f32,
    /// True Positives
    pub true_positives: usize,
    /// False Positives
    pub false_positives: usize,
    /// False Negatives
    pub false_negatives: usize,
    /// True Negatives
    pub true_negatives: usize,
}

impl PatternMetrics {
    /// Create metrics from confusion matrix counts
    #[must_use]
    pub fn from_counts(tp: usize, fp: usize, fn_: usize, tn: usize) -> Self {
        let precision = if tp + fp > 0 {
            tp as f32 / (tp + fp) as f32
        } else {
            0.0
        };

        let recall = if tp + fn_ > 0 {
            tp as f32 / (tp + fn_) as f32
        } else {
            0.0
        };

        let f1_score = if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };

        let total = tp + fp + fn_ + tn;
        let accuracy = if total > 0 {
            (tp + tn) as f32 / total as f32
        } else {
            0.0
        };

        Self {
            precision,
            recall,
            f1_score,
            accuracy,
            true_positives: tp,
            false_positives: fp,
            false_negatives: fn_,
            true_negatives: tn,
        }
    }

    /// Check if metrics meet target thresholds
    #[must_use]
    pub fn meets_target(&self, target_precision: f32, target_recall: f32) -> bool {
        self.precision >= target_precision && self.recall >= target_recall
    }

    /// Get overall quality score (0.0 to 1.0)
    #[must_use]
    pub fn quality_score(&self) -> f32 {
        // Weight F1 score (60%), precision (25%), and recall (15%)
        (self.f1_score * 0.6) + (self.precision * 0.25) + (self.recall * 0.15)
    }
}

/// Configuration for pattern validation
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Minimum confidence threshold for patterns
    pub min_confidence: f32,
    /// Similarity threshold for pattern matching (0.0 to 1.0)
    pub similarity_threshold: f32,
    /// Maximum allowed false positive rate
    pub max_false_positive_rate: f32,
    /// Minimum required recall
    pub min_recall: f32,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
            similarity_threshold: 0.8,
            max_false_positive_rate: 0.2,
            min_recall: 0.7,
        }
    }
}

/// Pattern validator for calculating accuracy metrics
pub struct PatternValidator {
    config: ValidationConfig,
    /// Cache of confidence scores by pattern ID
    confidence_cache: HashMap<Uuid, f32>,
}

impl PatternValidator {
    /// Create a new validator with default config
    #[must_use]
    pub fn new(config: ValidationConfig) -> Self {
        Self {
            config,
            confidence_cache: HashMap::new(),
        }
    }

    /// Validate that a pattern meets confidence threshold
    #[must_use]
    pub fn validate_confidence(&self, pattern: &Pattern) -> bool {
        let success_rate = pattern.success_rate();
        success_rate >= self.config.min_confidence
    }

    /// Track effectiveness of a pattern usage
    pub fn track_effectiveness(&mut self, pattern_id: Uuid, used: bool, successful: bool) {
        if used {
            // Update confidence based on success
            let current_confidence = self
                .confidence_cache
                .get(&pattern_id)
                .copied()
                .unwrap_or(0.5);

            // Simple moving average update
            let new_confidence = if successful {
                (current_confidence * 0.9) + (1.0 * 0.1)
            } else {
                (current_confidence * 0.9) + (0.0 * 0.1)
            };

            self.confidence_cache.insert(pattern_id, new_confidence);

            debug!(
                pattern_id = %pattern_id,
                used = used,
                successful = successful,
                new_confidence = new_confidence,
                "Tracked pattern effectiveness"
            );
        }
    }

    /// Get the tracked confidence for a pattern
    #[must_use]
    pub fn get_confidence(&self, pattern_id: Uuid) -> Option<f32> {
        self.confidence_cache.get(&pattern_id).copied()
    }

    /// Get validation configuration
    #[must_use]
    pub fn config(&self) -> &ValidationConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComplexityLevel, TaskContext};
    use chrono::Duration;
    use uuid::Uuid;

    fn create_test_context() -> TaskContext {
        TaskContext {
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity: ComplexityLevel::Moderate,
            domain: "testing".to_string(),
            tags: vec!["async".to_string()],
        }
    }

    #[test]
    fn test_pattern_metrics_calculation() {
        // Perfect classification: TP=5, FP=0, FN=0, TN=5
        let metrics = PatternMetrics::from_counts(5, 0, 0, 5);
        assert_eq!(metrics.precision, 1.0);
        assert_eq!(metrics.recall, 1.0);
        assert_eq!(metrics.f1_score, 1.0);
        assert_eq!(metrics.accuracy, 1.0);

        // Some errors: TP=3, FP=2, FN=1, TN=4
        let metrics = PatternMetrics::from_counts(3, 2, 1, 4);
        assert_eq!(metrics.precision, 0.6); // 3 / (3 + 2)
        assert_eq!(metrics.recall, 0.75); // 3 / (3 + 1)
        assert_eq!(metrics.accuracy, 0.7); // (3 + 4) / 10

        // F1 = 2 * (0.6 * 0.75) / (0.6 + 0.75) = 0.666...
        assert!((metrics.f1_score - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_pattern_metrics_edge_cases() {
        // No predictions
        let metrics = PatternMetrics::from_counts(0, 0, 5, 5);
        assert_eq!(metrics.precision, 0.0);
        assert_eq!(metrics.recall, 0.0);
        assert_eq!(metrics.f1_score, 0.0);

        // All false positives
        let metrics = PatternMetrics::from_counts(0, 5, 0, 5);
        assert_eq!(metrics.precision, 0.0);
        assert_eq!(metrics.recall, 0.0);
    }

    #[test]
    fn test_validate_confidence() {
        let config = ValidationConfig {
            min_confidence: 0.7,
            ..Default::default()
        };
        let validator = PatternValidator::new(config);

        let high_conf_pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool1".to_string()],
            context: create_test_context(),
            success_rate: 0.9,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 10,
            effectiveness: crate::pattern::PatternEffectiveness::new(),
        };

        let low_conf_pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool2".to_string()],
            context: create_test_context(),
            success_rate: 0.5,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 3,
            effectiveness: crate::pattern::PatternEffectiveness::new(),
        };

        assert!(validator.validate_confidence(&high_conf_pattern));
        assert!(!validator.validate_confidence(&low_conf_pattern));
    }

    #[test]
    fn test_track_effectiveness() {
        let config = ValidationConfig::default();
        let mut validator = PatternValidator::new(config);

        let pattern_id = Uuid::new_v4();

        // Track successful usage
        validator.track_effectiveness(pattern_id, true, true);
        let conf1 = validator.get_confidence(pattern_id).unwrap();
        assert!(conf1 > 0.5);

        // Track another success
        validator.track_effectiveness(pattern_id, true, true);
        let conf2 = validator.get_confidence(pattern_id).unwrap();
        assert!(conf2 > conf1);

        // Track a failure
        validator.track_effectiveness(pattern_id, true, false);
        let conf3 = validator.get_confidence(pattern_id).unwrap();
        assert!(conf3 < conf2);
    }

    #[test]
    fn test_quality_score() {
        let metrics = PatternMetrics::from_counts(8, 2, 1, 9);
        let score = metrics.quality_score();
        assert!(score > 0.0 && score <= 1.0);

        // Perfect metrics should give score close to 1.0
        let perfect = PatternMetrics::from_counts(10, 0, 0, 10);
        assert!(perfect.quality_score() > 0.95);
    }
}

//! Metrics calculation for pattern validation
//!
//! Provides metrics calculation including precision, recall, F1 score,
//! pattern matching, and similarity calculations.

use super::rules::{PatternMetrics, PatternValidator, ValidationConfig};
use crate::pattern::Pattern;
use std::collections::HashMap;
use tracing::{debug, instrument};
use uuid::Uuid;

impl PatternValidator {
    /// Calculate precision, recall, F1, and accuracy metrics
    #[instrument(skip(self, ground_truth, extracted))]
    pub fn calculate_metrics(
        &self,
        ground_truth: &[Pattern],
        extracted: &[Pattern],
    ) -> PatternMetrics {
        let mut tp = 0; // Correctly extracted patterns
        let mut fp = 0; // Incorrectly extracted patterns

        // Build similarity map for ground truth patterns
        let gt_map = self.build_pattern_map(ground_truth);

        // Track which ground truth patterns were matched
        let mut matched_gt: std::collections::HashSet<Uuid> = std::collections::HashSet::new();

        // Check each extracted pattern
        for extracted_pattern in extracted {
            if let Some(gt_pattern) = self.find_matching_pattern(extracted_pattern, &gt_map) {
                // Check if this is a valid match (high enough similarity)
                if self.patterns_match(extracted_pattern, gt_pattern) {
                    tp += 1;
                    matched_gt.insert(gt_pattern.id());
                } else {
                    fp += 1;
                }
            } else {
                fp += 1; // Extracted but not in ground truth
            }
        }

        // Count false negatives (ground truth patterns not extracted)
        let fn_ = ground_truth.len() - matched_gt.len();

        // For TN, we'd need to know the total pattern space,
        // which is not well-defined. Set to 0 for now.
        let tn = 0;

        debug!(
            tp = tp,
            fp = fp,
            fn_ = fn_,
            tn = tn,
            "Calculated pattern validation metrics"
        );

        PatternMetrics::from_counts(tp, fp, fn_, tn)
    }

    /// Build a map of patterns by type for efficient lookup
    fn build_pattern_map<'a>(&self, patterns: &'a [Pattern]) -> HashMap<String, Vec<&'a Pattern>> {
        let mut map: HashMap<String, Vec<&'a Pattern>> = HashMap::new();

        for pattern in patterns {
            let key = self.pattern_type_key(pattern);
            map.entry(key).or_default().push(pattern);
        }

        map
    }

    /// Get a type key for pattern matching
    fn pattern_type_key(&self, pattern: &Pattern) -> String {
        match pattern {
            Pattern::ToolSequence { tools, .. } => format!("tool_seq_{}", tools.join("_")),
            Pattern::DecisionPoint { condition, .. } => format!("decision_{condition}"),
            Pattern::ErrorRecovery { error_type, .. } => format!("error_{error_type}"),
            Pattern::ContextPattern {
                context_features, ..
            } => format!("context_{}", context_features.join("_")),
        }
    }

    /// Find a matching pattern in the ground truth
    fn find_matching_pattern<'a>(
        &self,
        extracted: &Pattern,
        gt_map: &'a HashMap<String, Vec<&'a Pattern>>,
    ) -> Option<&'a Pattern> {
        let key = self.pattern_type_key(extracted);

        if let Some(candidates) = gt_map.get(&key) {
            // Find the best matching candidate
            for candidate in candidates {
                if self.patterns_match(extracted, candidate) {
                    return Some(candidate);
                }
            }
        }

        None
    }

    /// Check if two patterns match (are similar enough)
    fn patterns_match(&self, p1: &Pattern, p2: &Pattern) -> bool {
        // Check type match first
        match (p1, p2) {
            (Pattern::ToolSequence { tools: t1, .. }, Pattern::ToolSequence { tools: t2, .. }) => {
                // Calculate sequence similarity
                self.sequence_similarity(t1, t2) >= self.config.similarity_threshold
            }
            (
                Pattern::DecisionPoint {
                    condition: c1,
                    action: a1,
                    ..
                },
                Pattern::DecisionPoint {
                    condition: c2,
                    action: a2,
                    ..
                },
            ) => {
                // Check if condition and action match
                self.string_similarity(c1, c2) >= self.config.similarity_threshold
                    && self.string_similarity(a1, a2) >= self.config.similarity_threshold
            }
            (
                Pattern::ErrorRecovery {
                    error_type: e1,
                    recovery_steps: r1,
                    ..
                },
                Pattern::ErrorRecovery {
                    error_type: e2,
                    recovery_steps: r2,
                    ..
                },
            ) => {
                // Check error type and recovery steps
                self.string_similarity(e1, e2) >= self.config.similarity_threshold
                    && self.sequence_similarity(r1, r2) >= self.config.similarity_threshold
            }
            (
                Pattern::ContextPattern {
                    context_features: f1,
                    ..
                },
                Pattern::ContextPattern {
                    context_features: f2,
                    ..
                },
            ) => {
                // Check context features overlap
                self.sequence_similarity(f1, f2) >= self.config.similarity_threshold
            }
            _ => false, // Different pattern types don't match
        }
    }

    /// Calculate similarity between two sequences (0.0 to 1.0)
    fn sequence_similarity(&self, seq1: &[String], seq2: &[String]) -> f32 {
        if seq1.is_empty() && seq2.is_empty() {
            return 1.0;
        }
        if seq1.is_empty() || seq2.is_empty() {
            return 0.0;
        }

        // Calculate Jaccard similarity (intersection / union)
        let seq1_set: std::collections::HashSet<_> = seq1.iter().collect();
        let seq2_set: std::collections::HashSet<_> = seq2.iter().collect();

        let intersection = seq1_set.intersection(&seq2_set).count();
        let union = seq1_set.union(&seq2_set).count();

        if union > 0 {
            intersection as f32 / union as f32
        } else {
            0.0
        }
    }

    /// Calculate similarity between two strings (0.0 to 1.0)
    fn string_similarity(&self, s1: &str, s2: &str) -> f32 {
        if s1 == s2 {
            return 1.0;
        }

        // Simple word-based similarity
        let s1_lower = s1.to_lowercase();
        let s2_lower = s2.to_lowercase();
        let words1: std::collections::HashSet<_> = s1_lower.split_whitespace().collect();
        let words2: std::collections::HashSet<_> = s2_lower.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union > 0 {
            intersection as f32 / union as f32
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComplexityLevel, TaskContext};
    use chrono::Duration;

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
    fn test_calculate_metrics_perfect_match() {
        let config = ValidationConfig::default();
        let validator = PatternValidator::new(config);

        let ground_truth = vec![
            Pattern::ToolSequence {
                id: Uuid::new_v4(),
                tools: vec!["tool1".to_string(), "tool2".to_string()],
                context: create_test_context(),
                success_rate: 0.9,
                avg_latency: Duration::milliseconds(100),
                occurrence_count: 5,
                effectiveness: crate::pattern::PatternEffectiveness::new(),
            },
            Pattern::ErrorRecovery {
                id: Uuid::new_v4(),
                error_type: "timeout".to_string(),
                recovery_steps: vec!["retry".to_string()],
                success_rate: 0.8,
                context: create_test_context(),
                effectiveness: crate::pattern::PatternEffectiveness::new(),
            },
        ];

        // Exact same patterns
        let extracted = ground_truth.clone();

        let metrics = validator.calculate_metrics(&ground_truth, &extracted);

        assert_eq!(metrics.true_positives, 2);
        assert_eq!(metrics.false_positives, 0);
        assert_eq!(metrics.false_negatives, 0);
        assert_eq!(metrics.precision, 1.0);
        assert_eq!(metrics.recall, 1.0);
        assert_eq!(metrics.f1_score, 1.0);
    }

    #[test]
    fn test_calculate_metrics_partial_match() {
        let config = ValidationConfig::default();
        let validator = PatternValidator::new(config);

        let ground_truth = vec![
            Pattern::ToolSequence {
                id: Uuid::new_v4(),
                tools: vec!["tool1".to_string(), "tool2".to_string()],
                context: create_test_context(),
                success_rate: 0.9,
                avg_latency: Duration::milliseconds(100),
                occurrence_count: 5,
                effectiveness: crate::pattern::PatternEffectiveness::new(),
            },
            Pattern::ErrorRecovery {
                id: Uuid::new_v4(),
                error_type: "timeout".to_string(),
                recovery_steps: vec!["retry".to_string()],
                success_rate: 0.8,
                context: create_test_context(),
                effectiveness: crate::pattern::PatternEffectiveness::new(),
            },
        ];

        // Only extract one pattern correctly
        let extracted = vec![Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool1".to_string(), "tool2".to_string()],
            context: create_test_context(),
            success_rate: 0.9,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 5,
            effectiveness: crate::pattern::PatternEffectiveness::new(),
        }];

        let metrics = validator.calculate_metrics(&ground_truth, &extracted);

        assert_eq!(metrics.true_positives, 1);
        assert_eq!(metrics.false_positives, 0);
        assert_eq!(metrics.false_negatives, 1);
        assert_eq!(metrics.precision, 1.0); // 1 / 1
        assert_eq!(metrics.recall, 0.5); // 1 / 2
        assert!((metrics.f1_score - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_sequence_similarity() {
        let config = ValidationConfig::default();
        let validator = PatternValidator::new(config);

        // Identical sequences
        let seq1 = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let seq2 = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(validator.sequence_similarity(&seq1, &seq2), 1.0);

        // Partial overlap
        let seq3 = vec!["a".to_string(), "b".to_string()];
        let sim = validator.sequence_similarity(&seq1, &seq3);
        assert!(sim > 0.5 && sim < 1.0);

        // No overlap
        let seq4 = vec!["x".to_string(), "y".to_string()];
        let sim = validator.sequence_similarity(&seq1, &seq4);
        assert!(sim < 0.5);
    }

    #[test]
    fn test_string_similarity() {
        let config = ValidationConfig::default();
        let validator = PatternValidator::new(config);

        // Identical strings
        assert_eq!(
            validator.string_similarity("hello world", "hello world"),
            1.0
        );

        // Partial match
        let sim = validator.string_similarity("hello world", "hello there");
        assert!(sim > 0.3 && sim < 1.0);

        // No match
        let sim = validator.string_similarity("hello", "goodbye");
        assert_eq!(sim, 0.0);
    }
}

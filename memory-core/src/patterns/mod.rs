//! # Pattern Validation and Effectiveness Tracking
//!
//! This module provides tools for validating pattern extraction quality
//! and tracking pattern effectiveness over time.
//!
//! ## Components
//!
//! - `affinity`: DyMoE routing-drift protection with pattern affinity classification
//! - `changepoint`: Changepoint detection for pattern metric monitoring
//! - `clustering`: Pattern clustering and deduplication
//! - `dbscan`: DBSCAN-based anomaly detection for episodes
//! - `effectiveness`: Pattern usage and success tracking
//! - `extractors`: Hybrid pattern extraction system with specialized extractors
//! - `optimized_validator`: Enhanced pattern validation
//! - `validation`: Pattern accuracy metrics (precision, recall, F1)

pub mod affinity;
pub mod changepoint;
pub mod clustering;
pub mod dbscan;
pub mod effectiveness;
pub mod extractors;
pub mod heuristic;
pub mod optimized_validator;
pub mod similarity;
#[cfg(test)]
mod tests;
pub mod types;
pub mod validation;

pub use affinity::{
    DEFAULT_AFFINITY_THRESHOLD, DEFAULT_MIN_SUCCESS_RATE, EpisodeAssignmentGuard,
    PatternAffinityClassifier, RejectionReason, RelativeAffinity,
};

pub use changepoint::{
    ChangeDirection, ChangeType, Changepoint, ChangepointConfig, ChangepointDetector,
    SegmentComparison, SegmentComparisonConfig, SegmentStats,
};
pub use clustering::{ClusterCentroid, ClusteringConfig, EpisodeCluster, PatternClusterer};
pub use dbscan::{
    Anomaly, AnomalyReason, DBSCANAnomalyDetector, DBSCANClusterResult, DBSCANConfig, DBSCANStats,
    FeatureWeights,
};
pub use effectiveness::{EffectivenessTracker, OverallStats, PatternUsage, UsageStats};
pub use extractors::{
    ContextPatternExtractor, DecisionPointExtractor, ErrorRecoveryExtractor,
    HybridPatternExtractor, PatternExtractor, ToolSequenceExtractor,
};
pub use heuristic::Heuristic;
pub use optimized_validator::{
    EnhancedPatternApplicator, OptimizedPatternValidator, RiskAssessment,
};
pub use types::{Pattern, PatternEffectiveness};
pub use validation::{PatternMetrics, PatternValidator, ValidationConfig};

use crate::types::TaskContext;

impl Pattern {
    /// Check if this pattern is relevant to a given context
    #[must_use]
    pub fn is_relevant_to(&self, query_context: &TaskContext) -> bool {
        if let Some(pattern_context) = self.context() {
            // Match on domain
            if pattern_context.domain == query_context.domain {
                return true;
            }

            // Match on language
            if pattern_context.language == query_context.language
                && pattern_context.language.is_some()
            {
                return true;
            }

            // Match on tags
            let common_tags: Vec<_> = pattern_context
                .tags
                .iter()
                .filter(|t| query_context.tags.contains(t))
                .collect();

            if !common_tags.is_empty() {
                return true;
            }
        }

        false
    }

    /// Get a similarity key for pattern deduplication
    /// Patterns with identical keys are considered duplicates
    #[must_use]
    pub fn similarity_key(&self) -> String {
        match self {
            Pattern::ToolSequence { tools, context, .. } => {
                format!("tool_seq:{}:{}", tools.join(","), context.domain)
            }
            Pattern::DecisionPoint {
                condition,
                action,
                context,
                ..
            } => {
                format!("decision:{}:{}:{}", condition, action, context.domain)
            }
            Pattern::ErrorRecovery {
                error_type,
                recovery_steps,
                context,
                ..
            } => {
                format!(
                    "error_recovery:{}:{}:{}",
                    error_type,
                    recovery_steps.join(","),
                    context.domain
                )
            }
            Pattern::ContextPattern {
                context_features,
                recommended_approach,
                ..
            } => {
                format!(
                    "context:{}:{}",
                    context_features.join(","),
                    recommended_approach
                )
            }
        }
    }

    /// Calculate similarity score between this pattern and another (0.0 to 1.0)
    /// Uses edit distance for sequences and context matching
    #[must_use]
    pub fn similarity_score(&self, other: &Self) -> f32 {
        // Different pattern types have zero similarity
        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return 0.0;
        }

        match (self, other) {
            (
                Pattern::ToolSequence {
                    tools: tools1,
                    context: ctx1,
                    ..
                },
                Pattern::ToolSequence {
                    tools: tools2,
                    context: ctx2,
                    ..
                },
            ) => similarity::tool_sequence_similarity(tools1, ctx1, tools2, ctx2),
            (
                Pattern::DecisionPoint {
                    condition: cond1,
                    action: act1,
                    context: ctx1,
                    ..
                },
                Pattern::DecisionPoint {
                    condition: cond2,
                    action: act2,
                    context: ctx2,
                    ..
                },
            ) => similarity::decision_point_similarity(cond1, act1, ctx1, cond2, act2, ctx2),
            (
                Pattern::ErrorRecovery {
                    error_type: err1,
                    recovery_steps: steps1,
                    context: ctx1,
                    ..
                },
                Pattern::ErrorRecovery {
                    error_type: err2,
                    recovery_steps: steps2,
                    context: ctx2,
                    ..
                },
            ) => similarity::error_recovery_similarity(err1, steps1, ctx1, err2, steps2, ctx2),
            (
                Pattern::ContextPattern {
                    context_features: feat1,
                    recommended_approach: rec1,
                    ..
                },
                Pattern::ContextPattern {
                    context_features: feat2,
                    recommended_approach: rec2,
                    ..
                },
            ) => similarity::context_pattern_similarity(feat1, rec1, feat2, rec2),
            _ => 0.0,
        }
    }

    /// Calculate confidence score for this pattern
    /// Confidence = `success_rate` * `sqrt(sample_size)`
    #[must_use]
    pub fn confidence(&self) -> f32 {
        let success_rate = self.success_rate();
        let sample_size = self.sample_size() as f32;

        if sample_size == 0.0 {
            return 0.0;
        }

        success_rate * sample_size.sqrt()
    }

    /// Merge this pattern with another similar pattern
    /// Combines evidence and updates statistics
    pub fn merge_with(&mut self, other: &Self) {
        // Can only merge patterns of the same type
        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return;
        }

        match (self, other) {
            (
                Pattern::ToolSequence {
                    success_rate: sr1,
                    occurrence_count: oc1,
                    avg_latency: lat1,
                    ..
                },
                Pattern::ToolSequence {
                    success_rate: sr2,
                    occurrence_count: oc2,
                    avg_latency: lat2,
                    ..
                },
            ) => types::merge_tool_sequence(sr1, oc1, lat1, *sr2, *oc2, lat2),
            (
                Pattern::DecisionPoint {
                    outcome_stats: stats1,
                    ..
                },
                Pattern::DecisionPoint {
                    outcome_stats: stats2,
                    ..
                },
            ) => types::merge_decision_point(stats1, stats2),
            (
                Pattern::ErrorRecovery {
                    success_rate: sr1, ..
                },
                Pattern::ErrorRecovery {
                    success_rate: sr2, ..
                },
            ) => types::merge_error_recovery(sr1, *sr2),
            (
                Pattern::ContextPattern {
                    evidence: ev1,
                    success_rate: sr1,
                    ..
                },
                Pattern::ContextPattern {
                    evidence: ev2,
                    success_rate: sr2,
                    ..
                },
            ) => types::merge_context_pattern(ev1, sr1, ev2, *sr2),
            _ => {}
        }
    }
}

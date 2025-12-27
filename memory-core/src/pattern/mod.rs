//! Pattern extraction and management
//!
//! This module provides types and functions for working with patterns extracted from episodes.

mod heuristic;
mod similarity;
mod types;

pub use heuristic::Heuristic;
pub use types::{Pattern, PatternEffectiveness};

use crate::types::TaskContext;
use chrono::Duration;

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
            ) => {
                let sequence_similarity = similarity::sequence_similarity(tools1, tools2);
                let context_similarity = similarity::context_similarity(ctx1, ctx2);
                // Weight: 70% sequence, 30% context
                sequence_similarity * 0.7 + context_similarity * 0.3
            }
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
            ) => {
                let condition_sim = similarity::string_similarity(cond1, cond2);
                let action_sim = similarity::string_similarity(act1, act2);
                let context_sim = similarity::context_similarity(ctx1, ctx2);
                // Weight: 40% condition, 40% action, 20% context
                condition_sim * 0.4 + action_sim * 0.4 + context_sim * 0.2
            }
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
            ) => {
                let error_sim = similarity::string_similarity(err1, err2);
                let steps_sim = similarity::sequence_similarity(steps1, steps2);
                let context_sim = similarity::context_similarity(ctx1, ctx2);
                // Weight: 40% error type, 40% recovery steps, 20% context
                error_sim * 0.4 + steps_sim * 0.4 + context_sim * 0.2
            }
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
            ) => {
                let features_sim = similarity::sequence_similarity(feat1, feat2);
                let approach_sim = similarity::string_similarity(rec1, rec2);
                // Weight: 60% features, 40% approach
                features_sim * 0.6 + approach_sim * 0.4
            }
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
            ) => {
                let total_count = *oc1 + *oc2;
                // Weighted average of success rates
                *sr1 = (*sr1 * *oc1 as f32 + *sr2 * *oc2 as f32) / total_count as f32;
                // Weighted average of latencies
                *lat1 = Duration::milliseconds(
                    (lat1.num_milliseconds() * *oc1 as i64 + lat2.num_milliseconds() * *oc2 as i64)
                        / total_count as i64,
                );
                *oc1 = total_count;
            }
            (
                Pattern::DecisionPoint {
                    outcome_stats: stats1,
                    ..
                },
                Pattern::DecisionPoint {
                    outcome_stats: stats2,
                    ..
                },
            ) => {
                stats1.success_count += stats2.success_count;
                stats1.failure_count += stats2.failure_count;
                stats1.total_count += stats2.total_count;
                // Weighted average of durations
                stats1.avg_duration_secs = (stats1.avg_duration_secs
                    * (stats1.total_count - stats2.total_count) as f32
                    + stats2.avg_duration_secs * stats2.total_count as f32)
                    / stats1.total_count as f32;
            }
            (
                Pattern::ErrorRecovery {
                    success_rate: sr1, ..
                },
                Pattern::ErrorRecovery {
                    success_rate: sr2, ..
                },
            ) => {
                // Simple average for error recovery patterns
                *sr1 = (*sr1 + *sr2) / 2.0;
                // Keep the richer context (more tags)
                // Context is already part of self
            }
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
            ) => {
                let size1 = ev1.len();
                let size2 = ev2.len();
                // Combine evidence
                ev1.extend_from_slice(ev2);
                // Weighted average of success rates
                *sr1 = (*sr1 * size1 as f32 + *sr2 * size2 as f32) / (size1 + size2) as f32;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ComplexityLevel;
    use uuid::Uuid;

    #[test]
    fn test_pattern_id() {
        let pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool1".to_string(), "tool2".to_string()],
            context: TaskContext::default(),
            success_rate: 0.9,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 5,
            effectiveness: PatternEffectiveness::new(),
        };

        assert!(pattern.success_rate() > 0.8);
        assert!(pattern.context().is_some());
    }

    #[test]
    fn test_pattern_similarity_key() {
        let pattern1 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["read".to_string(), "write".to_string()],
            context: TaskContext {
                domain: "web-api".to_string(),
                ..Default::default()
            },
            success_rate: 0.9,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 5,
            effectiveness: PatternEffectiveness::new(),
        };

        let pattern2 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["read".to_string(), "write".to_string()],
            context: TaskContext {
                domain: "web-api".to_string(),
                ..Default::default()
            },
            success_rate: 0.8,
            avg_latency: Duration::milliseconds(120),
            occurrence_count: 3,
            effectiveness: PatternEffectiveness::new(),
        };

        // Same tools and domain = same key
        assert_eq!(pattern1.similarity_key(), pattern2.similarity_key());
    }

    #[test]
    fn test_pattern_similarity_score() {
        let pattern1 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["read".to_string(), "write".to_string()],
            context: TaskContext {
                domain: "web-api".to_string(),
                language: Some("rust".to_string()),
                ..Default::default()
            },
            success_rate: 0.9,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 5,
            effectiveness: PatternEffectiveness::new(),
        };

        let pattern2 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["read".to_string(), "write".to_string()],
            context: TaskContext {
                domain: "web-api".to_string(),
                language: Some("rust".to_string()),
                ..Default::default()
            },
            success_rate: 0.8,
            avg_latency: Duration::milliseconds(120),
            occurrence_count: 3,
            effectiveness: PatternEffectiveness::new(),
        };

        let similarity = pattern1.similarity_score(&pattern2);

        // Identical tools and context should have high similarity
        assert!(similarity > 0.9);
    }

    #[test]
    fn test_pattern_confidence() {
        let pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool1".to_string()],
            context: TaskContext::default(),
            success_rate: 0.8,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 16, // sqrt(16) = 4
            effectiveness: PatternEffectiveness::new(),
        };

        let confidence = pattern.confidence();

        // 0.8 * sqrt(16) = 0.8 * 4 = 3.2
        assert!((confidence - 3.2).abs() < 0.01);
    }

    #[test]
    fn test_pattern_merge() {
        let mut pattern1 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["read".to_string(), "write".to_string()],
            context: TaskContext::default(),
            success_rate: 0.8,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 10,
            effectiveness: PatternEffectiveness::new(),
        };

        let pattern2 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["read".to_string(), "write".to_string()],
            context: TaskContext::default(),
            success_rate: 0.9,
            avg_latency: Duration::milliseconds(200),
            occurrence_count: 10,
            effectiveness: PatternEffectiveness::new(),
        };

        pattern1.merge_with(&pattern2);

        // Should have combined occurrence count
        match pattern1 {
            Pattern::ToolSequence {
                occurrence_count,
                success_rate,
                ..
            } => {
                assert_eq!(occurrence_count, 20);
                // Average: (0.8 * 10 + 0.9 * 10) / 20 = 0.85
                assert!((success_rate - 0.85).abs() < 0.01);
            }
            _ => panic!("Expected ToolSequence"),
        }
    }

    #[test]
    fn test_pattern_relevance() {
        let pattern_context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain: "web-api".to_string(),
            tags: vec!["async".to_string()],
        };

        let pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![],
            context: pattern_context.clone(),
            success_rate: 0.9,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 1,
            effectiveness: PatternEffectiveness::new(),
        };

        // Should match on domain
        let query_context = TaskContext {
            domain: "web-api".to_string(),
            ..Default::default()
        };
        assert!(pattern.is_relevant_to(&query_context));

        // Should match on language
        let query_context2 = TaskContext {
            language: Some("rust".to_string()),
            domain: "cli".to_string(),
            ..Default::default()
        };
        assert!(pattern.is_relevant_to(&query_context2));

        // Should not match
        let query_context3 = TaskContext {
            language: Some("python".to_string()),
            domain: "data-science".to_string(),
            ..Default::default()
        };
        assert!(!pattern.is_relevant_to(&query_context3));
    }

    #[test]
    fn test_heuristic_evidence_update() {
        let mut heuristic = Heuristic::new(
            "When refactoring async code".to_string(),
            "Use tokio::spawn for CPU-intensive tasks".to_string(),
            0.7,
        );

        assert_eq!(heuristic.evidence.sample_size, 0);

        // Add successful evidence
        heuristic.update_evidence(Uuid::new_v4(), true);
        assert_eq!(heuristic.evidence.sample_size, 1);
        assert_eq!(heuristic.evidence.success_rate, 1.0);

        // Add failed evidence
        heuristic.update_evidence(Uuid::new_v4(), false);
        assert_eq!(heuristic.evidence.sample_size, 2);
        assert_eq!(heuristic.evidence.success_rate, 0.5);

        // Add more successful evidence
        heuristic.update_evidence(Uuid::new_v4(), true);
        assert_eq!(heuristic.evidence.sample_size, 3);
        assert!((heuristic.evidence.success_rate - 0.666).abs() < 0.01);
    }
}

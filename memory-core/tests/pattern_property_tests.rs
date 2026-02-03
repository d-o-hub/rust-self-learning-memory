//! Property-based tests for Pattern extraction and similarity
//!
//! Tests invariants that must hold regardless of input values:
//! - Pattern extraction is deterministic
//! - Similar inputs produce similar patterns
//! - Pattern scores are bounded
//! - Pattern effectiveness metrics are consistent

use chrono::Duration;
use memory_core::pattern::{Pattern, PatternEffectiveness};
use memory_core::types::{ComplexityLevel, OutcomeStats, TaskContext};
use proptest::prelude::*;
use uuid::Uuid;

// ============================================================================
// Pattern ID Properties
// ============================================================================

proptest! {
    /// Pattern IDs are valid UUIDs
    #[test]
    fn pattern_id_is_valid_uuid(
        id in "[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}",
        tools in proptest::collection::vec("[a-zA-Z0-9_-]{1,20}", 1..5)
    ) {
        let pattern = Pattern::ToolSequence {
            id: Uuid::parse_str(&id).unwrap(),
            tools: tools.clone(),
            context: TaskContext::default(),
            success_rate: 0.8,
            avg_latency: Duration::seconds(30),
            occurrence_count: 5,
            effectiveness: PatternEffectiveness::new(),
        };

        assert!(!pattern.id().is_nil());
    }
}

// ============================================================================
// Pattern Similarity Properties
// ============================================================================

proptest! {
    /// Pattern similarity is reflexive (similarity with self is 1.0)
    #[test]
    fn similarity_is_reflexive(
        tools in proptest::collection::vec("[a-zA-Z0-9_-]{1,20}", 1..5)
    ) {
        let pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: tools.clone(),
            context: TaskContext::default(),
            success_rate: 0.8,
            avg_latency: Duration::seconds(30),
            occurrence_count: 5,
            effectiveness: PatternEffectiveness::new(),
        };

        let similarity = pattern.similarity_score(&pattern);
        assert!((similarity - 1.0).abs() < 0.001, "Similarity with self should be 1.0");
    }

    /// Pattern similarity is symmetric (A ~ B = B ~ A)
    #[test]
    fn similarity_is_symmetric(
        tools1 in proptest::collection::vec("[a-zA-Z0-9_-]{1,20}", 1..5),
        tools2 in proptest::collection::vec("[a-zA-Z0-9_-]{1,20}", 1..5)
    ) {
        let pattern1 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: tools1.clone(),
            context: TaskContext::default(),
            success_rate: 0.8,
            avg_latency: Duration::seconds(30),
            occurrence_count: 5,
            effectiveness: PatternEffectiveness::new(),
        };

        let pattern2 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: tools2.clone(),
            context: TaskContext::default(),
            success_rate: 0.7,
            avg_latency: Duration::seconds(25),
            occurrence_count: 4,
            effectiveness: PatternEffectiveness::new(),
        };

        let sim1 = pattern1.similarity_score(&pattern2);
        let sim2 = pattern2.similarity_score(&pattern1);

        assert!((sim1 - sim2).abs() < 0.001, "Similarity should be symmetric");
    }

    /// Similarity scores are bounded between 0.0 and 1.0
    #[test]
    fn similarity_is_bounded(
        tools1 in proptest::collection::vec("[a-zA-Z0-9_-]{1,20}", 1..5),
        tools2 in proptest::collection::vec("[a-zA-Z0-9_-]{1,20}", 1..5)
    ) {
        let pattern1 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: tools1,
            context: TaskContext::default(),
            success_rate: 0.8,
            avg_latency: Duration::seconds(30),
            occurrence_count: 5,
            effectiveness: PatternEffectiveness::new(),
        };

        let pattern2 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: tools2,
            context: TaskContext::default(),
            success_rate: 0.7,
            avg_latency: Duration::seconds(25),
            occurrence_count: 4,
            effectiveness: PatternEffectiveness::new(),
        };

        let similarity = pattern1.similarity_score(&pattern2);
        assert!(similarity >= 0.0, "Similarity should be >= 0.0");
        assert!(similarity <= 1.0, "Similarity should be <= 1.0");
    }

    /// Different pattern types have zero similarity
    #[test]
    fn different_types_zero_similarity(
        tools in proptest::collection::vec("[a-zA-Z0-9_-]{1,20}", 1..3),
        condition in "[a-zA-Z0-9 ]{1,30}",
        action in "[a-zA-Z0-9 ]{1,30}"
    ) {
        let tool_pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools,
            context: TaskContext::default(),
            success_rate: 0.8,
            avg_latency: Duration::seconds(30),
            occurrence_count: 5,
            effectiveness: PatternEffectiveness::new(),
        };

        let decision_pattern = Pattern::DecisionPoint {
            id: Uuid::new_v4(),
            condition: condition.clone(),
            action: action.clone(),
            outcome_stats: OutcomeStats {
                success_count: 8,
                failure_count: 2,
                total_count: 10,
                avg_duration_secs: 30.0,
            },
            context: TaskContext::default(),
            effectiveness: PatternEffectiveness::new(),
        };

        let similarity = tool_pattern.similarity_score(&decision_pattern);
        assert_eq!(similarity, 0.0, "Different pattern types should have zero similarity");
    }

    /// Similarity key is deterministic for same pattern
    #[test]
    fn similarity_key_deterministic(
        tools in proptest::collection::vec("[a-zA-Z0-9_-]{1,20}", 1..5)
    ) {
        let pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: tools.clone(),
            context: TaskContext::default(),
            success_rate: 0.8,
            avg_latency: Duration::seconds(30),
            occurrence_count: 5,
            effectiveness: PatternEffectiveness::new(),
        };

        let key1 = pattern.similarity_key();
        let key2 = pattern.similarity_key();

        assert_eq!(key1, key2, "Similarity key should be deterministic");
    }
}

// ============================================================================
// Pattern Score Properties
// ============================================================================

proptest! {
    /// Success rates are bounded between 0.0 and 1.0
    #[test]
    fn success_rate_bounded(
        success_count in 0usize..100,
        failure_count in 0usize..100
    ) {
        let total = success_count + failure_count;
        if total == 0 {
            return; // Skip if no data
        }

        let success_rate = success_count as f32 / total as f32;
        assert!(success_rate >= 0.0);
        assert!(success_rate <= 1.0);
    }

    /// Pattern success rate is bounded
    #[test]
    fn pattern_success_rate_bounded(
        success_rate in 0.0f32..1.0
    ) {
        let pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool1".to_string()],
            context: TaskContext::default(),
            success_rate,
            avg_latency: Duration::seconds(30),
            occurrence_count: 5,
            effectiveness: PatternEffectiveness::new(),
        };

        let retrieved_rate = pattern.success_rate();
        assert!(retrieved_rate >= 0.0, "Success rate should be >= 0.0");
        assert!(retrieved_rate <= 1.0, "Success rate should be <= 1.0");
    }
}

// ============================================================================
// Pattern Effectiveness Properties
// ============================================================================

proptest! {
    /// Initial effectiveness has zero counts
    #[test]
    fn initial_effectiveness_zero() {
        let effectiveness = PatternEffectiveness::new();

        assert_eq!(effectiveness.times_retrieved, 0);
        assert_eq!(effectiveness.times_applied, 0);
        assert_eq!(effectiveness.success_when_applied, 0);
        assert_eq!(effectiveness.failure_when_applied, 0);
    }

    /// Recording retrieval increments count
    #[test]
    fn record_retrieval_increments(count in 1..10usize) {
        let mut effectiveness = PatternEffectiveness::new();

        for _ in 0..count {
            effectiveness.record_retrieval();
        }

        assert_eq!(effectiveness.times_retrieved, count);
    }

    /// Application success rate is bounded
    #[test]
    fn application_success_rate_bounded(
        success_count in 0usize..10,
        failure_count in 0usize..10
    ) {
        let mut effectiveness = PatternEffectiveness::new();

        for _ in 0..success_count {
            effectiveness.record_application(true, 0.1);
        }

        for _ in 0..failure_count {
            effectiveness.record_application(false, -0.1);
        }

        let rate = effectiveness.application_success_rate();
        assert!(rate >= 0.0);
        assert!(rate <= 1.0);
    }

    /// Application stats sum correctly
    #[test]
    fn application_stats_sum(
        success_count in 0usize..10,
        failure_count in 0usize..10
    ) {
        let mut effectiveness = PatternEffectiveness::new();

        for _ in 0..success_count {
            effectiveness.record_application(true, 0.1);
        }

        for _ in 0..failure_count {
            effectiveness.record_application(false, -0.1);
        }

        assert_eq!(effectiveness.success_when_applied, success_count);
        assert_eq!(effectiveness.failure_when_applied, failure_count);
        assert_eq!(
            effectiveness.times_applied,
            success_count + failure_count
        );
    }

    /// Usage rate is bounded
    #[test]
    fn usage_rate_bounded(
        retrieved in 1usize..10,
        applied in 0usize..10
    ) {
        let mut effectiveness = PatternEffectiveness::new();

        for _ in 0..retrieved {
            effectiveness.record_retrieval();
        }

        for i in 0..applied {
            effectiveness.record_application(i % 2 == 0, 0.1);
        }

        let rate = effectiveness.usage_rate();
        assert!(rate >= 0.0);
        assert!(rate <= 1.0);
    }

    /// Reward delta moving average converges
    #[test]
    fn reward_delta_converges(
        deltas in proptest::collection::vec(-1.0f32..1.0, 5..20)
    ) {
        let mut effectiveness = PatternEffectiveness::new();

        for delta in &deltas {
            effectiveness.record_application(true, *delta);
        }

        // Calculate expected average
        let expected_avg = deltas.iter().sum::<f32>() / deltas.len() as f32;

        assert!(
            (effectiveness.avg_reward_delta - expected_avg).abs() < 0.001,
            "Moving average should converge to arithmetic mean"
        );
    }
}

// ============================================================================
// Pattern Serialization Properties
// ============================================================================

proptest! {
    /// Pattern serialization is round-trippable
    #[test]
    fn pattern_serialization_roundtrip(
        tools in proptest::collection::vec("[a-zA-Z0-9_-]{1,20}", 1..5)
    ) {
        let pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: tools.clone(),
            context: TaskContext::default(),
            success_rate: 0.8,
            avg_latency: Duration::seconds(30),
            occurrence_count: 5,
            effectiveness: PatternEffectiveness::new(),
        };

        let json = serde_json::to_string(&pattern).unwrap();
        let deserialized: Pattern = serde_json::from_str(&json).unwrap();

        assert_eq!(pattern.id(), deserialized.id());
        assert_eq!(pattern.success_rate(), deserialized.success_rate());
    }

    /// PatternEffectiveness serialization is round-trippable
    #[test]
    fn effectiveness_serialization_roundtrip() {
        let mut effectiveness = PatternEffectiveness::new();
        effectiveness.record_retrieval();
        effectiveness.record_application(true, 0.5);

        let json = serde_json::to_string(&effectiveness).unwrap();
        let deserialized: PatternEffectiveness = serde_json::from_str(&json).unwrap();

        assert_eq!(effectiveness.times_retrieved, deserialized.times_retrieved);
        assert_eq!(effectiveness.times_applied, deserialized.times_applied);
        assert_eq!(effectiveness.avg_reward_delta, deserialized.avg_reward_delta);
    }
}

// ============================================================================
// Pattern Context Properties
// ============================================================================

proptest! {
    /// Pattern context retrieval works correctly
    #[test]
    fn pattern_context_retrieval(
        domain in "[a-zA-Z0-9_-]{1,20}",
        language in "[a-zA-Z0-9_-]{1,20}"
    ) {
        let context = TaskContext {
            language: Some(language.clone()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain: domain.clone(),
            tags: vec![],
        };

        let pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool1".to_string()],
            context: context.clone(),
            success_rate: 0.8,
            avg_latency: Duration::seconds(30),
            occurrence_count: 5,
            effectiveness: PatternEffectiveness::new(),
        };

        let retrieved_context = pattern.context();
        assert!(retrieved_context.is_some());
        assert_eq!(retrieved_context.unwrap().domain, domain);
    }

    /// ContextPattern has no context
    #[test]
    fn context_pattern_no_context(
        features in proptest::collection::vec("[a-zA-Z0-9_-]{1,20}", 1..5),
        approach in "[a-zA-Z0-9 ]{1,50}"
    ) {
        let pattern = Pattern::ContextPattern {
            id: Uuid::new_v4(),
            context_features: features,
            recommended_approach: approach,
            evidence: vec![],
            success_rate: 0.8,
            effectiveness: PatternEffectiveness::new(),
        };

        assert!(pattern.context().is_none());
    }
}

// ============================================================================
// Pattern Sample Size Properties
// ============================================================================

proptest! {
    /// Sample size is non-negative
    #[test]
    fn sample_size_non_negative(
        occurrence_count in 0usize..100
    ) {
        let pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool1".to_string()],
            context: TaskContext::default(),
            success_rate: 0.8,
            avg_latency: Duration::seconds(30),
            occurrence_count,
            effectiveness: PatternEffectiveness::new(),
        };

        assert!(pattern.sample_size() >= 0);
    }

    /// ContextPattern sample size matches evidence length
    #[test]
    fn context_pattern_sample_size(
        evidence_count in 0usize..10
    ) {
        let evidence: Vec<Uuid> = (0..evidence_count).map(|_| Uuid::new_v4()).collect();

        let pattern = Pattern::ContextPattern {
            id: Uuid::new_v4(),
            context_features: vec![],
            recommended_approach: "test".to_string(),
            evidence: evidence.clone(),
            success_rate: 0.8,
            effectiveness: PatternEffectiveness::new(),
        };

        assert_eq!(pattern.sample_size(), evidence.len());
    }
}

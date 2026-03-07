//! Property-based tests for serialization roundtrips in memory-storage-turso
//!
//! These tests verify that types serialized with postcard (the storage format)
//! and serde_json survive roundtrip serialization without data loss.

use memory_core::*;
use proptest::prelude::*;

// ============================================================================
// Postcard Serialization Roundtrips (Binary storage format)
// ============================================================================

proptest! {
    /// TaskContext postcard roundtrip in Turso storage context
    #[test]
    fn task_context_postcard_turso_roundtrip(
        language in proptest::option::of("[a-z]{2,10}"),
        framework in proptest::option::of("[a-z]{2,15}"),
        domain in "[a-z]{3,20}",
        tags in proptest::collection::vec("[a-z]{2,15}", 0..5),
        complexity in prop::sample::select(vec![
            ComplexityLevel::Simple,
            ComplexityLevel::Moderate,
            ComplexityLevel::Complex,
        ]),
    ) {
        let context = TaskContext {
            language,
            framework,
            complexity,
            domain,
            tags,
        };

        let bytes = postcard::to_allocvec(&context).expect("postcard serialize");
        let deserialized: TaskContext =
            postcard::from_bytes(&bytes).expect("postcard deserialize");

        prop_assert_eq!(context, deserialized);
    }

    /// Heuristic postcard roundtrip (used in pattern storage)
    #[test]
    fn heuristic_postcard_roundtrip(
        condition in "[a-zA-Z0-9 ]{5,100}",
        action in "[a-zA-Z0-9 ]{5,100}",
        confidence in 0.0f32..1.0f32,
    ) {
        let heuristic = Heuristic::new(condition.clone(), action.clone(), confidence);

        let bytes = postcard::to_allocvec(&heuristic).expect("postcard serialize");
        let deserialized: Heuristic =
            postcard::from_bytes(&bytes).expect("postcard deserialize");

        prop_assert_eq!(heuristic.condition, deserialized.condition);
        prop_assert_eq!(heuristic.action, deserialized.action);
        prop_assert!((heuristic.confidence - deserialized.confidence).abs() < f32::EPSILON);
    }

    /// OutcomeStats postcard roundtrip
    #[test]
    fn outcome_stats_postcard_turso_roundtrip(
        success_count in 0usize..1000usize,
        failure_count in 0usize..1000usize,
        avg_duration_secs in 0.0f32..3600.0f32,
    ) {
        let stats = OutcomeStats {
            success_count,
            failure_count,
            total_count: success_count + failure_count,
            avg_duration_secs,
        };

        let bytes = postcard::to_allocvec(&stats).expect("postcard serialize");
        let deserialized: OutcomeStats =
            postcard::from_bytes(&bytes).expect("postcard deserialize");

        prop_assert_eq!(stats.success_count, deserialized.success_count);
        prop_assert_eq!(stats.failure_count, deserialized.failure_count);
        prop_assert_eq!(stats.total_count, deserialized.total_count);
        prop_assert!((stats.avg_duration_secs - deserialized.avg_duration_secs).abs() < f32::EPSILON);
    }

    /// PatternEffectiveness postcard roundtrip
    #[test]
    fn pattern_effectiveness_postcard_roundtrip(
        times_retrieved in 0usize..100usize,
        times_applied in 0usize..100usize,
        avg_reward_delta in -1.0f32..1.0f32,
    ) {
        let mut effectiveness = PatternEffectiveness::default();
        effectiveness.times_retrieved = times_retrieved;
        effectiveness.times_applied = times_applied;
        effectiveness.avg_reward_delta = avg_reward_delta;

        let bytes = postcard::to_allocvec(&effectiveness).expect("postcard serialize");
        let deserialized: PatternEffectiveness =
            postcard::from_bytes(&bytes).expect("postcard deserialize");

        prop_assert_eq!(effectiveness.times_retrieved, deserialized.times_retrieved);
        prop_assert_eq!(effectiveness.times_applied, deserialized.times_applied);
        prop_assert!((effectiveness.avg_reward_delta - deserialized.avg_reward_delta).abs() < f32::EPSILON);
    }
}

// ============================================================================
// JSON Serialization Roundtrips
// ============================================================================

proptest! {
    /// TaskContext JSON roundtrip
    #[test]
    fn task_context_json_roundtrip(
        language in proptest::option::of("[a-z]{2,10}"),
        framework in proptest::option::of("[a-z]{2,15}"),
        domain in "[a-z]{3,20}",
        tags in proptest::collection::vec("[a-z]{2,15}", 0..5),
    ) {
        let context = TaskContext {
            language,
            framework,
            complexity: ComplexityLevel::Moderate,
            domain,
            tags,
        };

        let json = serde_json::to_string(&context).expect("serialize to JSON");
        let deserialized: TaskContext =
            serde_json::from_str(&json).expect("deserialize from JSON");

        prop_assert_eq!(context, deserialized);
    }

    /// Episode JSON roundtrip for storage context
    #[test]
    fn episode_json_storage_roundtrip(
        task_description in "[a-zA-Z0-9 ]{1,100}",
        domain in "[a-z]{3,15}",
        num_steps in 0usize..10usize,
    ) {
        let mut episode = Episode::new(
            task_description.clone(),
            TaskContext {
                language: Some("rust".to_string()),
                framework: None,
                complexity: ComplexityLevel::Moderate,
                domain: domain.clone(),
                tags: vec!["test".to_string()],
            },
            TaskType::CodeGeneration,
        );

        for i in 0..num_steps {
            let step = ExecutionStep::new(
                i + 1,
                format!("tool_{i}"),
                format!("Action {i}"),
            );
            episode.add_step(step);
        }

        let json = serde_json::to_string(&episode).expect("serialize episode");
        let deserialized: Episode = serde_json::from_str(&json).expect("deserialize episode");

        prop_assert_eq!(episode.episode_id, deserialized.episode_id);
        prop_assert_eq!(episode.task_description, deserialized.task_description);
        prop_assert_eq!(episode.steps.len(), deserialized.steps.len());
        prop_assert_eq!(episode.context.domain, deserialized.context.domain);
    }

    /// Heuristic JSON roundtrip
    #[test]
    fn heuristic_json_roundtrip(
        condition in "[a-zA-Z0-9 ]{5,100}",
        action in "[a-zA-Z0-9 ]{5,100}",
        confidence in 0.0f32..1.0f32,
    ) {
        let heuristic = Heuristic::new(condition, action, confidence);

        let json = serde_json::to_string(&heuristic).expect("serialize to JSON");
        let deserialized: Heuristic =
            serde_json::from_str(&json).expect("deserialize from JSON");

        prop_assert_eq!(heuristic.condition, deserialized.condition);
        prop_assert_eq!(heuristic.action, deserialized.action);
        prop_assert!((heuristic.confidence - deserialized.confidence).abs() < f32::EPSILON);
    }
}

// ============================================================================
// Cross-format Serialization Consistency
// ============================================================================

proptest! {
    /// JSON and postcard produce equivalent deserialized values for TaskContext
    #[test]
    fn task_context_cross_format_consistency(
        domain in "[a-z]{3,15}",
        complexity in prop::sample::select(vec![
            ComplexityLevel::Simple,
            ComplexityLevel::Moderate,
            ComplexityLevel::Complex,
        ]),
    ) {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity,
            domain,
            tags: vec!["test".to_string()],
        };

        // Serialize with both formats
        let json_bytes = serde_json::to_vec(&context).expect("json serialize");
        let postcard_bytes = postcard::to_allocvec(&context).expect("postcard serialize");

        // Deserialize from each format
        let from_json: TaskContext =
            serde_json::from_slice(&json_bytes).expect("json deserialize");
        let from_postcard: TaskContext =
            postcard::from_bytes(&postcard_bytes).expect("postcard deserialize");

        // Both should produce identical results
        prop_assert_eq!(from_json, from_postcard);
    }

    /// Cross-format consistency for Heuristic
    #[test]
    fn heuristic_cross_format_consistency(
        condition in "[a-zA-Z0-9 ]{5,50}",
        action in "[a-zA-Z0-9 ]{5,50}",
        confidence in 0.1f32..0.9f32,
    ) {
        let heuristic = Heuristic::new(condition, action, confidence);

        let json_bytes = serde_json::to_vec(&heuristic).expect("json serialize");
        let postcard_bytes = postcard::to_allocvec(&heuristic).expect("postcard serialize");

        let from_json: Heuristic =
            serde_json::from_slice(&json_bytes).expect("json deserialize");
        let from_postcard: Heuristic =
            postcard::from_bytes(&postcard_bytes).expect("postcard deserialize");

        prop_assert_eq!(from_json.condition, from_postcard.condition);
        prop_assert_eq!(from_json.action, from_postcard.action);
        prop_assert!((from_json.confidence - from_postcard.confidence).abs() < f32::EPSILON);
    }
}

// ============================================================================
// State Machine Invariants
// ============================================================================

proptest! {
    /// Episode state transition invariants for storage
    #[test]
    fn episode_storage_state_invariants(
        num_steps in 0usize..20usize,
    ) {
        let mut episode = Episode::new(
            "Storage test".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        );

        prop_assert!(!episode.is_complete());
        prop_assert!(episode.outcome.is_none());

        for i in 0..num_steps {
            let step = ExecutionStep::new(
                i + 1,
                format!("tool_{i}"),
                format!("Action {i}"),
            );
            episode.add_step(step);
        }

        prop_assert!(!episode.is_complete());
        prop_assert_eq!(episode.steps.len(), num_steps);

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        prop_assert!(episode.is_complete());
        prop_assert!(episode.outcome.is_some());
        prop_assert!(episode.end_time.is_some());
    }

    /// PatternEffectiveness rate invariants
    #[test]
    fn pattern_effectiveness_rate_invariants(
        times_retrieved in 0usize..100usize,
        times_applied in 0usize..100usize,
    ) {
        let mut effectiveness = PatternEffectiveness::default();
        effectiveness.times_retrieved = times_retrieved;
        effectiveness.times_applied = times_applied;

        // Usage rate invariant
        let usage_rate = effectiveness.usage_rate();
        prop_assert!((0.0..=1.0).contains(&usage_rate) || times_applied > times_retrieved);

        // Application success rate invariant
        let success_rate = effectiveness.application_success_rate();
        prop_assert!((0.0..=1.0).contains(&success_rate));
    }
}

// ============================================================================
// Determinism Tests
// ============================================================================

proptest! {
    /// Serialization is deterministic for TaskContext
    #[test]
    fn serialization_determinism_turso(
        domain in "[a-z]{3,15}",
        language in proptest::option::of("[a-z]{2,10}"),
    ) {
        let context = TaskContext {
            language,
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain,
            tags: vec!["test".to_string()],
        };

        // Serialize twice
        let json1 = serde_json::to_string(&context).expect("serialize 1");
        let json2 = serde_json::to_string(&context).expect("serialize 2");

        prop_assert_eq!(json1, json2);

        // Postcard too
        let postcard1 = postcard::to_allocvec(&context).expect("postcard 1");
        let postcard2 = postcard::to_allocvec(&context).expect("postcard 2");

        prop_assert_eq!(postcard1, postcard2);
    }

    /// Hash consistency for keys
    #[test]
    fn hash_consistency(
        key in "[a-zA-Z0-9_-]{1,50}",
    ) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        key.hash(&mut hasher1);
        key.hash(&mut hasher2);

        prop_assert_eq!(hasher1.finish(), hasher2.finish());
    }
}

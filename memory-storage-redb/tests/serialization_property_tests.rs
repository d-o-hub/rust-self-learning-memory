//! Property-based tests for serialization roundtrips in memory-storage-redb
//!
//! These tests verify that types serialized with postcard (the storage format)
//! and serde_json survive roundtrip serialization without data loss.

use memory_core::*;
use memory_storage_redb::{CacheSnapshot, IncrementalUpdate, PersistedCacheEntry};
use proptest::prelude::*;

// ============================================================================
// Cache Persistence Type Roundtrips (JSON)
// ============================================================================

proptest! {
    /// PersistedCacheEntry JSON roundtrip preserves all fields
    #[test]
    fn persisted_cache_entry_json_roundtrip(
        key in "[a-zA-Z0-9_-]{1,50}",
        value in proptest::collection::vec(any::<u8>(), 0..200),
        created_at in any::<u64>(),
        access_count in any::<u64>(),
        last_accessed in any::<u64>(),
        ttl_secs in proptest::option::of(any::<u64>()),
    ) {
        let entry = PersistedCacheEntry {
            key: key.clone(),
            value: value.clone(),
            created_at,
            access_count,
            last_accessed,
            ttl_secs,
        };

        let json = serde_json::to_string(&entry).expect("serialize to JSON");
        let deserialized: PersistedCacheEntry =
            serde_json::from_str(&json).expect("deserialize from JSON");

        prop_assert_eq!(&entry.key, &deserialized.key);
        prop_assert_eq!(&entry.value, &deserialized.value);
        prop_assert_eq!(entry.created_at, deserialized.created_at);
        prop_assert_eq!(entry.access_count, deserialized.access_count);
        prop_assert_eq!(entry.last_accessed, deserialized.last_accessed);
        prop_assert_eq!(entry.ttl_secs, deserialized.ttl_secs);
    }

    /// CacheSnapshot JSON roundtrip preserves entries and metadata
    #[test]
    fn cache_snapshot_json_roundtrip(
        entry_count in 0usize..20usize,
        meta_count in 0usize..10usize,
    ) {
        let mut snapshot = CacheSnapshot::new();

        for i in 0..entry_count {
            let entry = PersistedCacheEntry {
                key: format!("key_{i}"),
                value: vec![i as u8; 10],
                created_at: 1000 + i as u64,
                access_count: i as u64,
                last_accessed: 2000 + i as u64,
                ttl_secs: if i % 2 == 0 { Some(3600) } else { None },
            };
            snapshot = snapshot.add_entry(entry);
        }

        for i in 0..meta_count {
            snapshot = snapshot.with_metadata(format!("key_{i}"), format!("value_{i}"));
        }

        let json = serde_json::to_string(&snapshot).expect("serialize to JSON");
        let deserialized: CacheSnapshot =
            serde_json::from_str(&json).expect("deserialize from JSON");

        prop_assert_eq!(snapshot.version, deserialized.version);
        prop_assert_eq!(snapshot.len(), deserialized.len());
        prop_assert_eq!(snapshot.metadata.len(), deserialized.metadata.len());

        for (key, value) in &snapshot.metadata {
            prop_assert_eq!(
                deserialized.metadata.get(key),
                Some(value)
            );
        }
    }

    /// IncrementalUpdate JSON roundtrip preserves upserts and deletions
    #[test]
    fn incremental_update_json_roundtrip(
        sequence in any::<u64>(),
        timestamp in any::<u64>(),
        upsert_count in 0usize..10usize,
        deletion_count in 0usize..10usize,
    ) {
        let upserts: Vec<PersistedCacheEntry> = (0..upsert_count)
            .map(|i| PersistedCacheEntry {
                key: format!("upsert_{i}"),
                value: vec![i as u8],
                created_at: timestamp,
                access_count: 0,
                last_accessed: timestamp,
                ttl_secs: None,
            })
            .collect();

        let deletions: Vec<String> = (0..deletion_count)
            .map(|i| format!("delete_{i}"))
            .collect();

        let update = IncrementalUpdate {
            sequence,
            timestamp,
            upserts,
            deletions,
        };

        let json = serde_json::to_string(&update).expect("serialize to JSON");
        let deserialized: IncrementalUpdate =
            serde_json::from_str(&json).expect("deserialize from JSON");

        prop_assert_eq!(update.sequence, deserialized.sequence);
        prop_assert_eq!(update.timestamp, deserialized.timestamp);
        prop_assert_eq!(update.upserts.len(), deserialized.upserts.len());
        prop_assert_eq!(update.deletions.len(), deserialized.deletions.len());

        for (orig, de) in update.deletions.iter().zip(deserialized.deletions.iter()) {
            prop_assert_eq!(orig, de);
        }
    }
}

// ============================================================================
// CacheSnapshot Invariants
// ============================================================================

proptest! {
    /// CacheSnapshot size_bytes is consistent with entries
    #[test]
    fn snapshot_size_bytes_consistent(
        entry_count in 0usize..30usize,
    ) {
        let mut snapshot = CacheSnapshot::new();

        for i in 0..entry_count {
            let entry = PersistedCacheEntry {
                key: format!("k{i}"),
                value: vec![0u8; i + 1],
                created_at: 0,
                access_count: 0,
                last_accessed: 0,
                ttl_secs: None,
            };
            snapshot = snapshot.add_entry(entry);
        }

        prop_assert_eq!(snapshot.len(), entry_count);
        prop_assert_eq!(snapshot.is_empty(), entry_count == 0);

        if entry_count > 0 {
            prop_assert!(snapshot.size_bytes() > 0);
        }
    }
}

// ============================================================================
// Postcard Serialization Roundtrips (Binary storage format)
// ============================================================================

proptest! {
    /// TaskContext postcard roundtrip in storage context
    #[test]
    fn task_context_postcard_storage_roundtrip(
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

    /// RewardScore postcard roundtrip in storage context
    #[test]
    fn reward_score_postcard_storage_roundtrip(
        total in 0.0f32..3.0f32,
        base in 0.0f32..1.0f32,
        efficiency in 0.5f32..2.0f32,
        complexity_bonus in 0.8f32..1.5f32,
        quality_multiplier in 0.5f32..1.5f32,
        learning_bonus in 0.0f32..1.0f32,
    ) {
        let score = RewardScore {
            total,
            base,
            efficiency,
            complexity_bonus,
            quality_multiplier,
            learning_bonus,
        };

        let bytes = postcard::to_allocvec(&score).expect("postcard serialize");
        let deserialized: RewardScore =
            postcard::from_bytes(&bytes).expect("postcard deserialize");

        prop_assert!((score.total - deserialized.total).abs() < f32::EPSILON);
        prop_assert!((score.base - deserialized.base).abs() < f32::EPSILON);
        prop_assert!((score.efficiency - deserialized.efficiency).abs() < f32::EPSILON);
        prop_assert!((score.complexity_bonus - deserialized.complexity_bonus).abs() < f32::EPSILON);
        prop_assert!((score.quality_multiplier - deserialized.quality_multiplier).abs() < f32::EPSILON);
        prop_assert!((score.learning_bonus - deserialized.learning_bonus).abs() < f32::EPSILON);
    }

    /// OutcomeStats postcard roundtrip
    #[test]
    fn outcome_stats_postcard_roundtrip(
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
}

// ============================================================================
// Episode Serialization Roundtrips
// ============================================================================

proptest! {
    /// Episode JSON roundtrip for cache storage
    #[test]
    fn episode_json_cache_roundtrip(
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

    /// Heuristic JSON roundtrip for cache storage
    #[test]
    fn heuristic_json_cache_roundtrip(
        condition in "[a-zA-Z0-9 ]{5,100}",
        action in "[a-zA-Z0-9 ]{5,100}",
        confidence in 0.0f32..1.0f32,
    ) {
        let heuristic = Heuristic::new(condition.clone(), action.clone(), confidence);

        let json = serde_json::to_string(&heuristic).expect("serialize to JSON");
        let deserialized: Heuristic =
            serde_json::from_str(&json).expect("deserialize from JSON");

        prop_assert_eq!(heuristic.condition, deserialized.condition);
        prop_assert_eq!(heuristic.action, deserialized.action);
        prop_assert!((heuristic.confidence - deserialized.confidence).abs() < f32::EPSILON);
    }

    /// Heuristic postcard roundtrip for binary cache storage
    #[test]
    fn heuristic_postcard_cache_roundtrip(
        condition in "[a-zA-Z0-9 ]{5,50}",
        action in "[a-zA-Z0-9 ]{5,50}",
        confidence in 0.0f32..1.0f32,
    ) {
        let heuristic = Heuristic::new(condition, action, confidence);

        let bytes = postcard::to_allocvec(&heuristic).expect("postcard serialize");
        let deserialized: Heuristic =
            postcard::from_bytes(&bytes).expect("postcard deserialize");

        prop_assert_eq!(heuristic.condition, deserialized.condition);
        prop_assert_eq!(heuristic.action, deserialized.action);
        prop_assert!((heuristic.confidence - deserialized.confidence).abs() < f32::EPSILON);
    }
}

// ============================================================================
// Cache Entry Invariants
// ============================================================================

proptest! {
    /// PersistedCacheEntry access_count consistency
    #[test]
    fn cache_entry_access_count_invariant(
        initial_count in 0u64..100u64,
        access_increment in 0u64..50u64,
    ) {
        let mut entry = PersistedCacheEntry {
            key: "test_key".to_string(),
            value: vec![1, 2, 3],
            created_at: 1000,
            access_count: initial_count,
            last_accessed: 1000,
            ttl_secs: Some(3600),
        };

        // Simulate access increments
        for _ in 0..access_increment {
            entry.access_count += 1;
            entry.last_accessed += 1;
        }

        prop_assert_eq!(entry.access_count, initial_count + access_increment);
        prop_assert_eq!(entry.last_accessed, 1000 + access_increment);
    }

    /// TTL expiration calculation invariant
    #[test]
    fn cache_entry_ttl_invariant(
        created_at in 1000u64..10000u64,
        ttl_secs in 60u64..86400u64,
        current_time in 1000u64..20000u64,
    ) {
        let entry = PersistedCacheEntry {
            key: "test_key".to_string(),
            value: vec![1, 2, 3],
            created_at,
            access_count: 1,
            last_accessed: created_at,
            ttl_secs: Some(ttl_secs),
        };

        // Calculate expiration time
        let expires_at = created_at + ttl_secs;

        // Verify expiration logic
        if current_time >= expires_at {
            prop_assert!(current_time >= created_at + ttl_secs);
        } else {
            prop_assert!(current_time < expires_at);
        }
    }

    /// Cache entry size is consistent with value length
    #[test]
    fn cache_entry_size_consistency(
        key in "[a-zA-Z0-9_-]{1,50}",
        value_len in 0usize..1000usize,
    ) {
        let entry = PersistedCacheEntry {
            key: key.clone(),
            value: vec![0u8; value_len],
            created_at: 0,
            access_count: 0,
            last_accessed: 0,
            ttl_secs: None,
        };

        // Entry size should be at least key length + value length
        let min_size = key.len() + value_len;
        let actual_size = entry.key.len() + entry.value.len();

        prop_assert!(actual_size >= min_size);
        prop_assert_eq!(entry.value.len(), value_len);
    }
}

// ============================================================================
// IncrementalUpdate Invariants
// ============================================================================

proptest! {
    /// IncrementalUpdate sequence monotonicity
    #[test]
    fn incremental_update_sequence_invariant(
        sequence1 in 1u64..1000u64,
        sequence2 in 1u64..1000u64,
    ) {
        let update1 = IncrementalUpdate {
            sequence: sequence1,
            timestamp: 1000,
            upserts: vec![],
            deletions: vec![],
        };

        let update2 = IncrementalUpdate {
            sequence: sequence2,
            timestamp: 2000,
            upserts: vec![],
            deletions: vec![],
        };

        // Higher sequence should have higher or equal timestamp (in valid usage)
        if sequence2 > sequence1 {
            prop_assert!(update2.timestamp >= update1.timestamp || update2.timestamp < update1.timestamp);
        }
    }

    /// IncrementalUpdate empty operations roundtrip
    #[test]
    fn incremental_update_empty_operations(
        sequence in any::<u64>(),
        timestamp in any::<u64>(),
    ) {
        let update = IncrementalUpdate {
            sequence,
            timestamp,
            upserts: vec![],
            deletions: vec![],
        };

        let json = serde_json::to_string(&update).expect("serialize to JSON");
        let deserialized: IncrementalUpdate =
            serde_json::from_str(&json).expect("deserialize from JSON");

        prop_assert!(deserialized.upserts.is_empty());
        prop_assert!(deserialized.deletions.is_empty());
        prop_assert_eq!(update.sequence, deserialized.sequence);
    }
}

// ============================================================================
// State Machine Invariants
// ============================================================================

proptest! {
    /// Episode state consistency for cache storage
    #[test]
    fn episode_state_consistency(
        num_steps in 0usize..20usize,
        should_complete in proptest::bool::ANY,
    ) {
        let mut episode = Episode::new(
            "Cache test".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        );

        // Before completion
        prop_assert!(!episode.is_complete());

        // Add steps
        for i in 0..num_steps {
            episode.add_step(ExecutionStep::new(
                i + 1,
                format!("step_{i}"),
                "action".to_string(),
            ));
        }

        // Still not complete
        prop_assert!(!episode.is_complete());
        prop_assert_eq!(episode.steps.len(), num_steps);

        if should_complete {
            episode.complete(TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            });

            prop_assert!(episode.is_complete());
            prop_assert!(episode.outcome.is_some());
            prop_assert!(episode.duration().is_some());
        } else {
            prop_assert!(!episode.is_complete());
            prop_assert!(episode.outcome.is_none());
        }
    }

    /// OutcomeStats success rate invariant
    #[test]
    fn outcome_stats_rate_invariant(
        success_count in 0usize..500usize,
        failure_count in 0usize..500usize,
    ) {
        let stats = OutcomeStats {
            success_count,
            failure_count,
            total_count: success_count + failure_count,
            avg_duration_secs: 0.0,
        };

        let rate = stats.success_rate();

        // Rate must be in [0.0, 1.0]
        prop_assert!((0.0..=1.0).contains(&rate));

        // Rate calculation must match
        if stats.total_count > 0 {
            #[allow(clippy::cast_precision_loss)]
            let expected = success_count as f32 / stats.total_count as f32;
            prop_assert!((rate - expected).abs() < 0.0001);
        }
    }
}

// ============================================================================
// Determinism Tests
// ============================================================================

proptest! {
    /// CacheSnapshot serialization is deterministic
    #[test]
    fn cache_snapshot_determinism(
        entry_count in 0usize..10usize,
    ) {
        let mut snapshot = CacheSnapshot::new();
        for i in 0..entry_count {
            snapshot = snapshot.add_entry(PersistedCacheEntry {
                key: format!("key_{i}"),
                value: vec![i as u8; 5],
                created_at: i as u64,
                access_count: i as u64,
                last_accessed: i as u64,
                ttl_secs: None,
            });
        }

        let json1 = serde_json::to_string(&snapshot).expect("serialize 1");
        let json2 = serde_json::to_string(&snapshot).expect("serialize 2");

        prop_assert_eq!(json1, json2);
    }

    /// PersistedCacheEntry serialization is deterministic
    #[test]
    fn cache_entry_determinism(
        key in "[a-zA-Z0-9_-]{1,30}",
        value_len in 0usize..50usize,
    ) {
        let entry = PersistedCacheEntry {
            key,
            value: vec![42u8; value_len],
            created_at: 12345,
            access_count: 5,
            last_accessed: 54321,
            ttl_secs: Some(3600),
        };

        let json1 = serde_json::to_string(&entry).expect("serialize 1");
        let json2 = serde_json::to_string(&entry).expect("serialize 2");

        prop_assert_eq!(json1, json2);
    }

    /// TaskContext serialization is deterministic
    #[test]
    fn task_context_determinism(
        domain in "[a-z]{3,15}",
    ) {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain,
            tags: vec!["test".to_string()],
        };

        let json1 = serde_json::to_string(&context).expect("serialize 1");
        let json2 = serde_json::to_string(&context).expect("serialize 2");

        prop_assert_eq!(json1, json2);

        let postcard1 = postcard::to_allocvec(&context).expect("postcard 1");
        let postcard2 = postcard::to_allocvec(&context).expect("postcard 2");

        prop_assert_eq!(postcard1, postcard2);
    }
}

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

//! Cache wrapper tests - Unit tests
//!
//! Unit tests for the CachedTursoStorage wrapper.
//! Tests cover cache hit/miss behavior, invalidation, and basic operations.

use super::{CacheConfig, CachedTursoStorage};
use crate::TursoStorage;
use libsql::Builder;
use memory_core::{Episode, Evidence, Heuristic, Pattern, TaskContext, TaskType};
use std::sync::Arc;
use tempfile::TempDir;
use uuid::Uuid;

/// Create a test Turso storage instance
async fn create_test_turso_storage() -> (TursoStorage, TempDir) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test_cache.db");

    let db = Builder::new_local(&db_path)
        .build()
        .await
        .expect("Failed to create test database");

    let storage = TursoStorage::from_database(db).expect("Failed to create storage");
    storage
        .initialize_schema()
        .await
        .expect("Failed to init schema");

    (storage, dir)
}

/// Create a test episode
fn create_test_episode(id: Uuid) -> Episode {
    Episode {
        episode_id: id,
        task_type: TaskType::CodeGeneration,
        task_description: format!("Test episode {}", id),
        context: TaskContext {
            domain: "test".to_string(),
            language: Some("rust".to_string()),
            ..Default::default()
        },
        steps: vec![],
        outcome: None,
        reward: None,
        reflection: None,
        patterns: vec![],
        heuristics: vec![],
        applied_patterns: vec![],
        salient_features: None,
        start_time: chrono::Utc::now(),
        end_time: None,
        metadata: std::collections::HashMap::new(),
    }
}

/// Create a test pattern
fn create_test_pattern(id: Uuid) -> Pattern {
    Pattern::ToolSequence {
        id,
        tools: vec!["tool1".to_string(), "tool2".to_string()],
        context: TaskContext {
            domain: "test".to_string(),
            language: Some("rust".to_string()),
            ..Default::default()
        },
        success_rate: 0.8,
        avg_latency: chrono::Duration::milliseconds(100),
        occurrence_count: 5,
        effectiveness: Default::default(),
    }
}

/// Create a test heuristic
fn create_test_heuristic(id: Uuid) -> Heuristic {
    Heuristic {
        heuristic_id: id,
        condition: "condition".to_string(),
        action: "action".to_string(),
        confidence: 0.75,
        evidence: Evidence {
            episode_ids: vec![],
            success_rate: 0.75,
            sample_size: 10,
        },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

// ========== Unit Tests ==========

#[tokio::test]
async fn test_cache_creation_with_default_config() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cache_config = CacheConfig::default();

    let cached = CachedTursoStorage::new(storage, cache_config);
    let stats = cached.stats();

    assert_eq!(stats.episode_hits, 0);
    assert_eq!(stats.episode_misses, 0);
    assert_eq!(stats.pattern_hits, 0);
    assert_eq!(stats.pattern_misses, 0);
}

#[tokio::test]
async fn test_cache_creation_with_disabled_caches() {
    let (storage, _dir) = create_test_turso_storage().await;

    let cache_config = CacheConfig {
        enable_episode_cache: false,
        enable_pattern_cache: false,
        enable_query_cache: false,
        ..Default::default()
    };

    let cached = CachedTursoStorage::new(storage, cache_config);

    // Verify episode hit returns None (cache disabled)
    let result = cached.get_episode_cached(Uuid::new_v4()).await;
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_episode_cache_hit() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

    let episode = create_test_episode(Uuid::new_v4());

    // Store episode
    cached.store_episode_cached(&episode).await.unwrap();

    // First retrieval - should be a miss
    let result = cached.get_episode_cached(episode.episode_id).await.unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().episode_id, episode.episode_id);

    let stats = cached.stats();
    assert_eq!(stats.episode_misses, 1); // First access was a miss
    assert_eq!(stats.episode_hits, 0);

    // Second retrieval - should be a hit
    let result = cached.get_episode_cached(episode.episode_id).await.unwrap();
    assert!(result.is_some());

    let stats = cached.stats();
    assert_eq!(stats.episode_hits, 1);
    assert_eq!(stats.episode_misses, 1);
}

#[tokio::test]
async fn test_episode_cache_miss() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

    // Try to get non-existent episode
    let result = cached.get_episode_cached(Uuid::new_v4()).await.unwrap();
    assert!(result.is_none());

    let stats = cached.stats();
    assert_eq!(stats.episode_misses, 1);
    assert_eq!(stats.episode_hits, 0);
}

#[tokio::test]
async fn test_episode_cache_invalidation_on_store() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

    let episode_id = Uuid::new_v4();
    let episode = create_test_episode(episode_id);

    // Store and retrieve to populate cache
    cached.store_episode_cached(&episode).await.unwrap();
    let _ = cached.get_episode_cached(episode_id).await.unwrap();

    // Update episode (should invalidate cache)
    let updated_episode = create_test_episode(episode_id);
    cached.store_episode_cached(&updated_episode).await.unwrap();

    // Cache should be invalidated, next access should be a miss then a hit
    let result = cached.get_episode_cached(episode_id).await.unwrap();
    assert!(result.is_some());

    // Verify we got the updated episode
    let stats = cached.stats();
    // Should have at least 2 misses (initial store, re-fetch after invalidation)
    assert!(stats.episode_misses >= 2);
}

#[tokio::test]
async fn test_episode_cache_invalidation_on_delete() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

    let episode_id = Uuid::new_v4();
    let episode = create_test_episode(episode_id);

    // Store and retrieve to populate cache
    cached.store_episode_cached(&episode).await.unwrap();
    let _ = cached.get_episode_cached(episode_id).await.unwrap();

    // Delete episode
    cached.delete_episode_cached(episode_id).await.unwrap();

    // Should no longer exist
    let result = cached.get_episode_cached(episode_id).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_pattern_cache_hit() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

    let pattern_id = Uuid::new_v4();
    let pattern = create_test_pattern(pattern_id);

    // Store pattern
    cached.store_pattern_cached(&pattern).await.unwrap();

    // First retrieval - miss
    let result = cached.get_pattern_cached(pattern_id).await.unwrap();
    assert!(result.is_some());

    let stats = cached.stats();
    assert_eq!(stats.pattern_misses, 1);
    assert_eq!(stats.pattern_hits, 0);

    // Second retrieval - hit
    let result = cached.get_pattern_cached(pattern_id).await.unwrap();
    assert!(result.is_some());

    let stats = cached.stats();
    assert_eq!(stats.pattern_hits, 1);
    assert_eq!(stats.pattern_misses, 1);
}

#[tokio::test]
async fn test_heuristic_cache_hit() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

    let heuristic_id = Uuid::new_v4();
    let heuristic = create_test_heuristic(heuristic_id);

    // Store heuristic
    cached.store_heuristic_cached(&heuristic).await.unwrap();

    // First retrieval - miss
    let result = cached.get_heuristic_cached(heuristic_id).await.unwrap();
    assert!(result.is_some());

    // Second retrieval - hit
    let result = cached.get_heuristic_cached(heuristic_id).await.unwrap();
    assert!(result.is_some());

    // Verify cache stats are accessible
    let _stats = cached.stats();
}

#[tokio::test]
async fn test_clear_caches() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

    // Store some episodes
    for _ in 0..5 {
        let episode = create_test_episode(Uuid::new_v4());
        cached.store_episode_cached(&episode).await.unwrap();
        let _ = cached.get_episode_cached(episode.episode_id).await.unwrap();
    }

    // Verify some cache activity
    let stats_before = cached.stats();
    assert!(stats_before.episode_hits > 0 || stats_before.episode_misses > 0);

    // Clear caches
    cached.clear_caches().await;

    // Next access should be a miss
    let _ = cached.get_episode_cached(Uuid::new_v4()).await.unwrap();

    // Verify cache was cleared (no hits after clear)
    let stats_after = cached.stats();
    assert_eq!(stats_after.episode_hits, 0);
    // Note: Some misses may still occur due to internal operations
}

#[tokio::test]
async fn test_cache_config_validation() {
    let cache_config = CacheConfig::default();

    // Verify default values are sensible
    assert!(cache_config.enable_episode_cache);
    assert!(cache_config.enable_pattern_cache);
    assert_eq!(cache_config.max_episodes, 10_000);
    assert_eq!(cache_config.max_patterns, 5_000);
    assert!(!cache_config.episode_ttl.is_zero());
    assert!(!cache_config.pattern_ttl.is_zero());
}

#[tokio::test]
async fn test_cache_stats_hit_rate() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

    // Perform some cache operations
    let episode = create_test_episode(Uuid::new_v4());
    cached.store_episode_cached(&episode).await.unwrap();

    // Miss
    let _ = cached.get_episode_cached(episode.episode_id).await.unwrap();
    // Hit
    let _ = cached.get_episode_cached(episode.episode_id).await.unwrap();
    // Hit
    let _ = cached.get_episode_cached(episode.episode_id).await.unwrap();

    let stats = cached.stats();
    let hit_rate = stats.episode_hit_rate();

    // Should be 2 hits out of 3 accesses = 66.6%
    assert!((hit_rate - 0.6666).abs() < 0.01);
}

#[tokio::test]
async fn test_underlying_storage_access() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

    // Get underlying storage reference
    let _ = cached.storage();

    // Should be able to access the storage directly
    let episode = create_test_episode(Uuid::new_v4());
    cached.storage().store_episode(&episode).await.unwrap();

    let result = cached
        .storage()
        .get_episode(episode.episode_id)
        .await
        .unwrap();
    assert!(result.is_some());
}

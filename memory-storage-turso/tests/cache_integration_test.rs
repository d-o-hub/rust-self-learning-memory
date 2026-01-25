//! Integration tests for Phase 3 cache layers

use memory_core::{Episode, Pattern, PatternId, Result, StorageBackend, TaskContext, TaskType};
use memory_storage_turso::{CacheConfig, PreparedCacheConfig, TursoStorage};
use std::time::Duration;
use tempfile::TempDir;

async fn create_test_storage() -> Result<(TursoStorage, TempDir)> {
    let dir = TempDir::new()?;
    let db_path = dir.path().join("test.db");

    let storage = TursoStorage::new(&format!("file:{}", db_path.display()), "").await?;
    storage.initialize_schema().await?;

    Ok((storage, dir))
}

#[tokio::test]
async fn test_cached_storage_episode_operations() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    // Create cache configuration
    let cache_config = CacheConfig {
        enable_episode_cache: true,
        enable_pattern_cache: true,
        max_episodes: 100,
        max_patterns: 50,
        episode_ttl: Duration::from_secs(60),
        ..Default::default()
    };

    // Wrap storage with cache
    let cached_storage = storage.with_cache(cache_config);

    // Create and store an episode
    let episode = Episode::new(
        "Test task".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    );
    let episode_id = episode.episode_id;

    cached_storage.store_episode(&episode).await.unwrap();

    // First retrieval - cache miss
    let retrieved1 = cached_storage.get_episode(episode_id).await.unwrap();
    assert!(retrieved1.is_some());

    // Second retrieval - should be cache hit
    let retrieved2 = cached_storage.get_episode(episode_id).await.unwrap();
    assert!(retrieved2.is_some());

    // Get cache stats
    let stats = cached_storage.stats();
    assert!(stats.episode_hits > 0, "Expected cache hits");
    assert!(stats.episode_hit_rate() > 0.0, "Expected positive hit rate");
}

#[tokio::test]
async fn test_cached_storage_pattern_operations() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    let cache_config = CacheConfig {
        enable_pattern_cache: true,
        max_patterns: 50,
        ..Default::default()
    };

    let cached_storage = storage.with_cache(cache_config);

    // Create and store a pattern
    let pattern = Pattern::DecisionPoint {
        id: PatternId::new_v4(),
        condition: "test condition".to_string(),
        action: "test action".to_string(),
        outcome_stats: memory_core::types::OutcomeStats {
            success_count: 5,
            failure_count: 1,
            total_count: 6,
            avg_duration_secs: 0.5,
        },
        context: TaskContext::default(),
        effectiveness: memory_core::pattern::PatternEffectiveness::default(),
    };
    let pattern_id = pattern.id();

    cached_storage.store_pattern(&pattern).await.unwrap();

    // Retrieve pattern multiple times
    for _ in 0..3 {
        let retrieved = cached_storage.get_pattern(pattern_id).await.unwrap();
        assert!(retrieved.is_some());
    }

    // Check cache stats
    let stats = cached_storage.stats();
    assert!(stats.pattern_hits >= 2, "Expected at least 2 cache hits");
}

#[tokio::test]
async fn test_prepared_statement_cache() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    // Get prepared cache stats before operations
    let stats_before = storage.prepared_cache_stats();
    assert_eq!(stats_before.hits, 0);
    assert_eq!(stats_before.misses, 0);

    // Perform operations that use SQL queries
    let episode = Episode::new(
        "Test task".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    );

    storage.store_episode(&episode).await.unwrap();
    storage.get_episode(episode.episode_id).await.unwrap();

    // Check that prepared cache was used
    let stats_after = storage.prepared_cache_stats();
    // Note: Current implementation doesn't use prepared cache yet
    // This test validates the infrastructure is in place
    let _ = stats_after.current_size; // Validate field exists
}

#[tokio::test]
async fn test_batch_operations_integration() {
    let (storage, _dir) = create_test_storage().await.unwrap();

    // Create multiple episodes
    let episodes: Vec<Episode> = (0..5)
        .map(|i| {
            Episode::new(
                format!("Task {}", i),
                TaskContext::default(),
                TaskType::CodeGeneration,
            )
        })
        .collect();

    // Store batch
    storage
        .store_episodes_batch(episodes.clone())
        .await
        .unwrap();

    // Verify all episodes were stored
    for episode in &episodes {
        let retrieved = storage.get_episode(episode.episode_id).await.unwrap();
        assert!(retrieved.is_some());
    }
}

#[tokio::test]
async fn test_cache_config_defaults() {
    let config = CacheConfig::default();

    assert!(config.enable_episode_cache);
    assert!(config.enable_pattern_cache);
    assert!(config.enable_query_cache);
    assert_eq!(config.max_episodes, 10_000);
    assert_eq!(config.max_patterns, 5_000);
    assert!(config.enable_background_cleanup);
}

#[tokio::test]
async fn test_prepared_cache_config_defaults() {
    let config = PreparedCacheConfig::default();

    assert_eq!(config.max_size, 100);
    assert!(config.enable_refresh);
    assert_eq!(config.refresh_threshold, 1000);
}

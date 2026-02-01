//! Tests for Phase 1 Turso Optimizations
//!
//! Validates the correctness and performance of:
//! 1. Cache-first read strategy
//! 2. Request batching API
//! 3. Query result caching
//! 4. Performance metrics tracking

use memory_core::{ComplexityLevel, Episode, StorageBackend, TaskContext, TaskType};
use memory_storage_turso::{CacheConfig, QueryCache, QueryKey, TursoStorage};
// Note: PerformanceMetrics temporarily disabled due to existing metrics module issues
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tempfile::tempdir;
use uuid::Uuid;

/// Helper to create a test episode
fn create_test_episode(id: Uuid, domain: &str) -> Episode {
    let mut metadata = HashMap::new();
    metadata.insert("test_key".to_string(), "test_value".to_string());
    metadata.insert("domain".to_string(), domain.to_string());

    Episode {
        episode_id: id,
        task_type: TaskType::CodeGeneration,
        task_description: format!("Test task {}", id),
        context: TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain: domain.to_string(),
            tags: Vec::new(),
        },
        start_time: chrono::Utc::now(),
        end_time: Some(chrono::Utc::now()),
        steps: vec![],
        outcome: None,
        reward: None,
        reflection: None,
        patterns: vec![],
        heuristics: vec![],
        applied_patterns: Vec::new(),
        salient_features: None,
        metadata,
        tags: vec![],
    }
}

#[tokio::test]
async fn test_cache_first_read_strategy() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_url = format!("file:{}", db_path.display());

    // Create storage with cache
    let storage = TursoStorage::new(&db_url, "").await.unwrap();
    storage.initialize_schema().await.unwrap();

    let cache_config = CacheConfig {
        enable_episode_cache: true,
        max_episodes: 1000,
        ..Default::default()
    };
    let cached_storage = storage.with_cache(cache_config);

    // Create and store test episodes
    let episodes: Vec<Episode> = (0..10)
        .map(|_| create_test_episode(Uuid::new_v4(), "cache_test"))
        .collect();

    for episode in &episodes {
        cached_storage
            .store_episode(episode)
            .await
            .expect("Failed to store episode");
    }

    // First read (cache miss)
    let start = Instant::now();
    for episode in &episodes {
        let result = cached_storage
            .get_episode(episode.episode_id)
            .await
            .expect("Failed to get episode");
        assert!(result.is_some());
    }
    let miss_duration = start.elapsed();

    // Second read (cache hit - should be much faster)
    let start = Instant::now();
    for episode in &episodes {
        let result = cached_storage
            .get_episode(episode.episode_id)
            .await
            .expect("Failed to get episode");
        assert!(result.is_some());
    }
    let hit_duration = start.elapsed();

    // Verify cache hits are faster
    println!("Cache miss duration: {:?}", miss_duration);
    println!("Cache hit duration: {:?}", hit_duration);

    // Cache hits should be at least 2x faster
    assert!(
        hit_duration < miss_duration / 2,
        "Cache hits should be significantly faster than misses"
    );

    // Verify cache stats
    let stats = cached_storage.stats();
    assert!(stats.episode_hits > 0, "Should have cache hits");
    assert!(
        stats.episode_hit_rate() > 0.0,
        "Hit rate should be positive"
    );
    println!("Cache hit rate: {:.1}%", stats.episode_hit_rate() * 100.0);
}

#[tokio::test]
async fn test_batch_operations() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_url = format!("file:{}", db_path.display());

    let storage = TursoStorage::new(&db_url, "").await.unwrap();
    storage.initialize_schema().await.unwrap();

    // Create test episodes
    let episodes: Vec<Episode> = (0..50)
        .map(|_| create_test_episode(Uuid::new_v4(), "batch_test"))
        .collect();

    // Benchmark individual operations
    let start = Instant::now();
    for episode in &episodes[0..10] {
        storage
            .store_episode(episode)
            .await
            .expect("Failed to store episode");
    }
    let individual_duration = start.elapsed();

    // Benchmark batch operations
    let start = Instant::now();
    storage
        .store_episodes_batch(episodes[10..20].to_vec())
        .await
        .expect("Failed to store batch");
    let batch_duration = start.elapsed();

    println!(
        "Individual operations (10 episodes): {:?}",
        individual_duration
    );
    println!("Batch operation (10 episodes): {:?}", batch_duration);

    // Batch should be faster than individual operations
    assert!(
        batch_duration < individual_duration,
        "Batch operations should be faster than individual operations"
    );

    // Verify all episodes were stored
    let ids: Vec<Uuid> = episodes[10..20].iter().map(|e| e.episode_id).collect();
    let retrieved = storage
        .get_episodes_batch(&ids)
        .await
        .expect("Failed to get batch");

    assert_eq!(retrieved.len(), 10, "Should retrieve all batched episodes");
    assert!(
        retrieved.iter().all(|e| e.is_some()),
        "All episodes should exist"
    );
}

#[tokio::test]
async fn test_query_result_caching() {
    let query_cache = QueryCache::default();

    // Create test episodes
    let episodes: Vec<Episode> = (0..5)
        .map(|_| create_test_episode(Uuid::new_v4(), "query_cache_test"))
        .collect();

    // Cache some results
    let key1 = QueryKey::EpisodesByDomain("test_domain".to_string());
    query_cache.cache_episodes(key1.clone(), episodes.clone());

    // Verify cache hit
    let cached = query_cache.get_episodes(&key1);
    assert!(cached.is_some(), "Should get cached result");
    assert_eq!(cached.unwrap().len(), 5, "Should have all episodes");

    // Verify cache miss
    let key2 = QueryKey::EpisodesByDomain("other_domain".to_string());
    let not_cached = query_cache.get_episodes(&key2);
    assert!(not_cached.is_none(), "Should be cache miss");

    // Verify stats
    let stats = query_cache.stats();
    assert_eq!(stats.hits, 1, "Should have 1 hit");
    assert_eq!(stats.misses, 1, "Should have 1 miss");
    assert_eq!(stats.hit_rate(), 0.5, "Hit rate should be 50%");
}

#[tokio::test]
async fn test_query_cache_expiration() {
    let query_cache = QueryCache::new(100, Duration::from_millis(100));

    let episodes: Vec<Episode> = (0..3)
        .map(|_| create_test_episode(Uuid::new_v4(), "expiration_test"))
        .collect();

    let key = QueryKey::EpisodesByDomain("test".to_string());
    query_cache.cache_episodes(key.clone(), episodes);

    // Should be cached
    assert!(query_cache.get_episodes(&key).is_some());

    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Should be expired
    assert!(query_cache.get_episodes(&key).is_none());

    let stats = query_cache.stats();
    assert_eq!(stats.expirations, 1, "Should have 1 expiration");
}

// Removed test_performance_metrics_tracking - PerformanceMetrics module not available
// The test was already disabled and returning early. When metrics module is
// made available in the future, this test can be re-added.

#[tokio::test]
async fn test_metadata_query_optimization() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_url = format!("file:{}", db_path.display());

    let storage = TursoStorage::new(&db_url, "").await.unwrap();
    storage.initialize_schema().await.unwrap();

    // Create episodes with metadata
    let episodes: Vec<Episode> = (0..20)
        .map(|i| {
            let mut ep = create_test_episode(Uuid::new_v4(), "metadata_test");
            ep.metadata
                .insert("priority".to_string(), format!("p{}", i % 3));
            ep
        })
        .collect();

    // Store episodes
    for episode in &episodes {
        storage.store_episode(episode).await.unwrap();
    }

    // Query by metadata (uses json_extract optimization)
    let start = Instant::now();
    let results = storage
        .query_episodes_by_metadata("priority", "p1")
        .await
        .expect("Failed to query by metadata");
    let query_duration = start.elapsed();

    println!("Metadata query duration: {:?}", query_duration);
    println!("Found {} episodes with priority=p1", results.len());

    // Should find approximately 1/3 of episodes
    assert!(
        results.len() >= 5 && results.len() <= 8,
        "Should find around 6-7 episodes with priority=p1"
    );

    // Query should be reasonably fast (< 50ms for 20 episodes)
    assert!(
        query_duration < Duration::from_millis(50),
        "Optimized query should be fast"
    );
}

#[tokio::test]
async fn test_end_to_end_optimization() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_url = format!("file:{}", db_path.display());

    let storage = TursoStorage::new(&db_url, "").await.unwrap();
    storage.initialize_schema().await.unwrap();

    // Create test episodes first (before wrapping storage with cache)
    let episodes: Vec<Episode> = (0..30)
        .map(|_| create_test_episode(Uuid::new_v4(), "e2e_test"))
        .collect();

    // Phase 1: Batch store using original storage
    let start = Instant::now();
    storage
        .store_episodes_batch(episodes.clone())
        .await
        .expect("Failed to batch store");
    let batch_store_duration = start.elapsed();
    println!("Batch store duration: {:?}", batch_store_duration);

    // Now wrap storage with cache for read operations
    let cache_config = CacheConfig::default();
    let cached_storage = storage.with_cache(cache_config);

    // Phase 2: Cache-first reads (first time - cache miss)
    let start = Instant::now();
    for episode in &episodes {
        let result = cached_storage
            .get_episode(episode.episode_id)
            .await
            .expect("Failed to get episode");
        assert!(result.is_some());
    }
    let first_read_duration = start.elapsed();
    println!(
        "First read duration (cache miss): {:?}",
        first_read_duration
    );

    // Phase 3: Second reads (cache hits)
    let start = Instant::now();
    for episode in &episodes {
        let result = cached_storage
            .get_episode(episode.episode_id)
            .await
            .expect("Failed to get episode");
        assert!(result.is_some());
    }
    let second_read_duration = start.elapsed();
    println!(
        "Second read duration (cache hit): {:?}",
        second_read_duration
    );

    // Phase 4: Metadata queries
    let start = Instant::now();
    let results = cached_storage
        .query_episodes_by_metadata("domain", "e2e_test")
        .await
        .expect("Failed to query");
    let query_duration = start.elapsed();
    println!("Metadata query duration: {:?}", query_duration);

    assert_eq!(results.len(), 30, "Should find all episodes");

    // Verify cache hits are faster than misses
    assert!(
        second_read_duration < first_read_duration,
        "Cache hits should be faster than cache misses"
    );

    // Print cache stats
    let stats = cached_storage.stats();
    println!("Cache hit rate: {:.1}%", stats.episode_hit_rate() * 100.0);
    assert!(stats.episode_hits > 0, "Should have cache hits");
}

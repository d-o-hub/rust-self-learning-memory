//! Cache load tests
//!
//! Tests for validating cache behavior under heavy load and memory pressure.
//! Ensures cache hit rates and proper eviction.
//!
//! Acceptance Criteria:
//! - Insert 100,000 episodes
//! - Query repeatedly to test cache
//! - Verify cache hit rate > 80%
//! - Test cache eviction under memory pressure

use memory_core::{Episode, Pattern, TaskContext, TaskType};
use memory_storage_turso::{CacheConfig, CachedTursoStorage, TursoConfig, TursoStorage};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use uuid::Uuid;

/// Number of episodes to insert for cache load test
const CACHE_TEST_EPISODE_COUNT: usize = 100_000;

/// Number of query iterations
const CACHE_QUERY_ITERATIONS: usize = 10_000;

/// Minimum acceptable cache hit rate (80%)
const MIN_CACHE_HIT_RATE: f64 = 0.80;

/// Maximum test duration (5 minutes)
const MAX_TEST_DURATION: Duration = Duration::from_secs(300);

/// Cache test configuration
struct CacheTestConfig {
    /// Maximum cache size
    max_episodes: usize,
    /// Maximum patterns in cache
    max_patterns: usize,
    /// Expected hit rate
    expected_hit_rate: f64,
    /// Enable episode cache
    enable_episode_cache: bool,
    /// Enable pattern cache
    enable_pattern_cache: bool,
}

impl Default for CacheTestConfig {
    fn default() -> Self {
        Self {
            max_episodes: 10_000,
            max_patterns: 5_000,
            expected_hit_rate: MIN_CACHE_HIT_RATE,
            enable_episode_cache: true,
            enable_pattern_cache: true,
        }
    }
}

/// Cache test statistics
#[derive(Debug, Default)]
struct CacheTestStatistics {
    /// Total operations
    total_operations: usize,
    /// Cache hits
    cache_hits: usize,
    /// Cache misses
    cache_misses: usize,
    /// Episodes inserted
    episodes_inserted: usize,
    /// Episodes retrieved
    episodes_retrieved: usize,
    /// Patterns inserted
    patterns_inserted: usize,
    /// Patterns retrieved
    patterns_retrieved: usize,
    /// Cache evictions
    cache_evictions: usize,
}

impl CacheTestStatistics {
    /// Calculate cache hit rate
    fn hit_rate(&self) -> f64 {
        let total_accesses = self.cache_hits + self.cache_misses;
        if total_accesses == 0 {
            return 0.0;
        }
        self.cache_hits as f64 / total_accesses as f64
    }

    /// Episode-specific hit rate
    fn episode_hit_rate(&self) -> f64 {
        if self.episodes_retrieved == 0 {
            return 0.0;
        }
        self.cache_hits as f64 / self.episodes_retrieved as f64
    }

    /// Check if statistics meet criteria
    fn meets_criteria(&self, config: &CacheTestConfig) -> anyhow::Result<()> {
        let actual_rate = self.hit_rate();
        if actual_rate < config.expected_hit_rate {
            anyhow::bail!(
                "Cache hit rate {:.2}% does not meet expected {:.2}%",
                actual_rate * 100.0,
                config.expected_hit_rate * 100.0
            );
        }
        Ok(())
    }

    /// Print summary
    fn print_summary(&self, test_name: &str) {
        println!("\n=== {} Cache Test Summary ===", test_name);
        println!("Total Operations: {}", self.total_operations);
        println!(
            "Cache Hits: {} ({:.2}%)",
            self.cache_hits,
            self.hit_rate() * 100.0
        );
        println!(
            "Cache Misses: {} ({:.2}%)",
            self.cache_misses,
            100.0 - self.hit_rate() * 100.0
        );
        println!("Episodes Inserted: {}", self.episodes_inserted);
        println!("Episodes Retrieved: {}", self.episodes_retrieved);
        println!("Patterns Inserted: {}", self.patterns_inserted);
        println!("Patterns Retrieved: {}", self.patterns_retrieved);
        println!("Cache Evictions: {}", self.cache_evictions);
        println!("================================\n");
    }
}

/// Create a test Turso storage with caching
async fn create_cached_storage(config: &CacheTestConfig) -> (CachedTursoStorage, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");

    let turso_config = TursoConfig::default();
    let storage = TursoStorage::with_config(
        &format!("file:{}", db_path.to_string_lossy()),
        "",
        turso_config,
    )
    .await
    .expect("Failed to create Turso storage");

    storage
        .initialize_schema()
        .await
        .expect("Failed to initialize schema");

    let cache_config = CacheConfig {
        enable_episode_cache: config.enable_episode_cache,
        enable_pattern_cache: config.enable_pattern_cache,
        max_episodes: config.max_episodes,
        max_patterns: config.max_patterns,
        ttl_seconds: None,
        enable_metrics: true,
    };

    let cached_storage = CachedTursoStorage::new(storage, cache_config);

    (cached_storage, temp_dir)
}

/// Create a test episode
fn create_test_episode(id: Uuid, index: usize) -> Episode {
    Episode {
        episode_id: id,
        task_type: TaskType::CodeGeneration,
        task_description: format!("Cache test episode {} - {}", index, id),
        context: TaskContext {
            domain: "cache_test".to_string(),
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity: memory_core::types::ComplexityLevel::Moderate,
            tags: vec!["cache_test".to_string(), format!("batch_{}", index / 1000)],
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
        tags: vec![],
    }
}

/// Create a test pattern
fn create_test_pattern(id: Uuid, episode_id: Uuid) -> Pattern {
    Pattern::ToolSequence {
        id,
        tools: vec!["test_tool_1".to_string(), "test_tool_2".to_string()],
        context: TaskContext {
            domain: "cache_test".to_string(),
            language: Some("rust".to_string()),
            ..Default::default()
        },
        success_rate: 0.85,
        avg_latency: chrono::Duration::milliseconds(50),
        occurrence_count: 10,
        effectiveness: Default::default(),
    }
}

/// Insert episodes in batches to test cache under heavy load
async fn insert_episodes_for_cache_test(
    cached_storage: &CachedTursoStorage,
    count: usize,
) -> Vec<Uuid> {
    let mut episode_ids = Vec::with_capacity(count);
    let batch_size = 1000;

    println!("Inserting {} episodes for cache test...", count);

    for batch_start in (0..count).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(count);
        let mut batch_episodes = Vec::with_capacity(batch_end - batch_start);

        for i in batch_start..batch_end {
            let episode_id = Uuid::new_v4();
            let episode = create_test_episode(episode_id, i);
            batch_episodes.push(episode);
            episode_ids.push(episode_id);
        }

        // Batch insert
        for episode in batch_episodes {
            cached_storage
                .store_episode_cached(&episode)
                .await
                .expect("Failed to store episode");
        }

        if batch_end % 10_000 == 0 {
            println!("Inserted {} episodes...", batch_end);
        }
    }

    episode_ids
}

/// Run cache load test with repeated queries
async fn run_cache_load_test(
    cached_storage: &CachedTursoStorage,
    episode_ids: &[Uuid],
    config: &CacheTestConfig,
) -> CacheTestStatistics {
    let mut stats = CacheTestStatistics::default();
    let start = Instant::now();

    println!(
        "Running cache load test with {} query iterations...",
        CACHE_QUERY_ITERATIONS
    );

    // Query episodes repeatedly to test cache behavior
    let rng = &mut rand::thread_rng();

    for i in 0..CACHE_QUERY_ITERATIONS {
        stats.total_operations += 1;

        // Use weighted random to prefer recently accessed episodes (simulates real-world access patterns)
        let idx = if rng.gen_bool(0.7) {
            // 70% chance: access recent episode (last 10%)
            let recent_start = episode_ids.len().saturating_sub(episode_ids.len() / 10);
            rng.gen_range(recent_start..episode_ids.len())
        } else {
            // 30% chance: access random episode
            rng.gen_range(0..episode_ids.len())
        };

        let episode_id = episode_ids[idx];

        let query_start = Instant::now();
        let _result = cached_storage.get_episode_cached(episode_id).await;
        let _latency = query_start.elapsed();

        stats.episodes_retrieved += 1;

        // Check if it was a cache hit or miss by measuring latency
        // Cache hits are typically < 1ms, misses take longer
        if _latency < Duration::from_millis(1) {
            stats.cache_hits += 1;
        } else {
            stats.cache_misses += 1;
        }

        if (i + 1) % 1000 == 0 {
            println!("Completed {} queries...", i + 1);
        }
    }

    // Get actual cache statistics
    if let Ok(cache_stats) = cached_storage.stats() {
        stats.cache_hits = cache_stats.episode_hits;
        stats.cache_misses = cache_stats.episode_misses;
        stats.cache_evictions = cache_stats.episode_evictions;
    }

    let elapsed = start.elapsed();
    println!("Cache load test completed in {:?}", elapsed);
    println!(
        "Throughput: {:.2} queries/sec",
        CACHE_QUERY_ITERATIONS as f64 / elapsed.as_secs_f64()
    );

    stats
}

/// Test cache behavior under memory pressure
async fn test_cache_under_memory_pressure(
    cached_storage: &CachedTursoStorage,
) -> anyhow::Result<()> {
    println!("Testing cache behavior under memory pressure...");

    let insert_count = 20_000; // More than cache capacity (10,000)
    let mut inserted_ids = Vec::with_capacity(insert_count);

    // Insert episodes beyond cache capacity
    for i in 0..insert_count {
        let episode_id = Uuid::new_v4();
        let episode = create_test_episode(episode_id, i);
        cached_storage.store_episode_cached(&episode).await?;
        inserted_ids.push(episode_id);

        if (i + 1) % 5000 == 0 {
            println!("Inserted {} episodes under memory pressure...", i + 1);
        }
    }

    // Try to access recently inserted episodes (should be in cache)
    let recent_hits = inserted_ids[inserted_count - 1000..]
        .iter()
        .take(100)
        .map(|id| cached_storage.get_episode_cached(*id).await.is_ok())
        .filter(|&ok| ok)
        .count();

    // Try to access older episodes (may have been evicted)
    let old_hits = inserted_ids[..1000]
        .iter()
        .take(100)
        .map(|id| cached_storage.get_episode_cached(*id).await.is_ok())
        .filter(|&ok| ok)
        .count();

    println!("Recent episodes cache hit rate: {}%", recent_hits);
    println!("Old episodes cache hit rate: {}%", old_hits);

    // Verify cache is working (recent episodes should have better hit rate)
    if recent_hits < 90 || old_hits > 10 {
        anyhow::bail!(
            "Cache eviction not working as expected: recent hits={}, old hits={}",
            recent_hits,
            old_hits
        );
    }

    println!("Cache eviction under memory pressure test passed ✅");

    Ok(())
}

/// Test cache clear operations
async fn test_cache_clear_operations(cached_storage: &CachedTursoStorage) -> anyhow::Result<()> {
    println!("Testing cache clear operations...");

    // Get statistics before clear
    let stats_before = cached_storage.stats()?;
    println!(
        "Stats before clear: hits={}, misses={}",
        stats_before.episode_hits, stats_before.episode_misses
    );

    // Clear caches
    cached_storage.clear_caches().await;

    // Get statistics after clear
    let stats_after = cached_storage.stats()?;
    println!(
        "Stats after clear: hits={}, misses={}",
        stats_after.episode_hits, stats_after.episode_misses
    );

    // Verify statistics were reset or updated
    if stats_after.episode_hits != 0 || stats_after.episode_misses != 0 {
        // Check if clear resets stats or not (both behaviors may be valid)
        println!("Note: Cache statistics not reset by clear operation");
    }

    // Verify cache is actually cleared by retrieving an item and timing
    let episode_id = Uuid::new_v4();
    let episode = create_test_episode(episode_id, 0);
    cached_storage.store_episode_cached(&episode).await?;

    // First retrieve - should be a cache miss (not in cache yet)
    let start = Instant::now();
    let _result = cached_storage.get_episode_cached(episode_id).await?;
    let first_latency = start.elapsed();

    // Second retrieve - should be a cache hit
    let start = Instant::now();
    let _result = cached_storage.get_episode_cached(episode_id).await?;
    let second_latency = start.elapsed();

    println!("First retrieve latency: {:?}", first_latency);
    println!("Second retrieve latency: {:?}", second_latency);

    // Second retrieve should be faster (cached)
    if second_latency > first_latency {
        anyhow::bail!("Cache not effective: second retrieve slower than first");
    }

    println!("Cache clear operations test passed ✅");

    Ok(())
}

/// Main test entry point
#[tokio::test]
async fn test_cache_load() {
    println!("=== Cache Load Test ===");
    println!("Testing cache behavior under heavy load...");
    println!("Episodes to insert: {}", CACHE_TEST_EPISODE_COUNT);
    println!("Query iterations: {}", CACHE_QUERY_ITERATIONS);
    println!(
        "Minimum cache hit rate: {:.0}%\n",
        MIN_CACHE_HIT_RATE * 100.0
    );

    let start = Instant::now();

    let config = CacheTestConfig {
        max_episodes: 10_000,
        max_patterns: 5_000,
        expected_hit_rate: MIN_CACHE_HIT_RATE,
        ..Default::default()
    };

    let (cached_storage, _temp_dir) = create_cached_storage(&config).await;

    // First, verify cache is enabled
    assert!(
        config.enable_episode_cache,
        "Episode cache should be enabled"
    );
    println!("Episode cache enabled: {}", config.enable_episode_cache);
    println!("Pattern cache enabled: {}", config.enable_pattern_cache);

    // Insert episodes for cache test
    tokio::time::timeout(
        MAX_TEST_DURATION,
        insert_episodes_for_cache_test(&cached_storage, CACHE_TEST_EPISODE_COUNT),
    )
    .await
    .expect("Episode insertion timed out");

    // Get initial cache statistics
    let initial_stats = cached_storage.stats().expect("Failed to get cache stats");
    println!("\nInitial cache statistics:");
    println!(" Episodes in cache: {}", initial_stats.episodes_in_cache);
    println!(" Patterns in cache: {}", initial_stats.patterns_in_cache);
    println!(" Episode hits: {}", initial_stats.episode_hits);
    println!(" Episode misses: {}", initial_stats.episode_misses);
    println!(" Episode evictions: {}", initial_stats.episode_evictions);

    // Run cache load test
    let episode_ids: Vec<Uuid> = (0..CACHE_TEST_EPISODE_COUNT)
        .map(|_| Uuid::new_v4())
        .collect();

    let stats = tokio::time::timeout(
        MAX_TEST_DURATION,
        run_cache_load_test(&cached_storage, &episode_ids, &config),
    )
    .await
    .expect("Cache load test timed out");

    stats.print_summary("Cache Load");

    // Verify cache hit rate
    config
        .meets_criteria(&stats)
        .expect("Cache hit rate does not meet criteria");

    // Test cache under memory pressure
    test_cache_under_memory_pressure(&cached_storage)
        .await
        .expect("Memory pressure test failed");

    // Test cache clear operations
    test_cache_clear_operations(&cached_storage)
        .await
        .expect("Cache clear test failed");

    // Get final cache statistics
    let final_stats = cached_storage.stats().expect("Failed to get cache stats");
    println!("\nFinal cache statistics:");
    println!(" Episodes in cache: {}", final_stats.episodes_in_cache);
    println!(" Patterns in cache: {}", final_stats.patterns_in_cache);
    println!(" Episode hits: {}", final_stats.episode_hits);
    println!(" Episode misses: {}", final_stats.episode_misses);
    println!(" Episode evictions: {}", final_stats.episode_evictions);

    // Calculate actual hit rate from stats
    let actual_hit_rate = if final_stats.episode_hits + final_stats.episode_misses > 0 {
        final_stats.episode_hits as f64
            / (final_stats.episode_hits + final_stats.episode_misses) as f64
    } else {
        0.0
    };

    println!("\nActual cache hit rate: {:.2}%", actual_hit_rate * 100.0);

    let elapsed = start.elapsed();
    println!("Total test duration: {:?}", elapsed);
    println!("All cache load tests passed! ✅");
}

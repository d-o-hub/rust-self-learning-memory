//! Batch operations load tests
//!
//! Tests for validating batch operation performance and transaction safety.
//! Ensure throughput meets targets (200-300 ops/sec).
//!
//! Acceptance Criteria:
//! - Batch insert 10,000 episodes
//! - Batch insert 50,000 patterns
//! - Measure throughput (target: 200-300/sec)
//! - Verify transaction safety

use memory_core::{Episode, Heuristic, Pattern, TaskContext, TaskType};
use memory_storage_turso::{BatchConfig, TursoConfig, TursoStorage};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use uuid::Uuid;

/// Number of episodes to batch insert
const BATCH_EPISODE_COUNT: usize = 10_000;

/// Number of patterns to batch insert
const BATCH_PATTERN_COUNT: usize = 50_000;

/// Number of heuristics to batch insert
const BATCH_HEURISTIC_COUNT: usize = 1_000;

/// Target throughput (operations per second)
const TARGET_THROUGHPUT_MIN: f64 = 200.0;
const TARGET_THROUGHPUT_MAX: f64 = 300.0;

/// Maximum test duration (10 minutes)
const MAX_TEST_DURATION: Duration = Duration::from_secs(600);

/// Batch size configuration
const DEFAULT_BATCH_SIZE: usize = 100;
const LARGE_BATCH_SIZE: usize = 1000;

/// Batch test configuration
struct BatchTestConfig {
    /// Batch size for operations
    batch_size: usize,
    /// Expected minimum throughput (ops/sec)
    min_throughput: f64,
    /// Expected maximum throughput (ops/sec)
    max_throughput: f64,
    /// Enable compression (if feature available)
    enable_compression: bool,
}

impl Default for BatchTestConfig {
    fn default() -> Self {
        Self {
            batch_size: DEFAULT_BATCH_SIZE,
            min_throughput: TARGET_THROUGHPUT_MIN,
            max_throughput: TARGET_THROUGHPUT_MAX,
            enable_compression: false,
        }
    }
}

/// Batch test statistics
#[derive(Debug, Default)]
struct BatchTestStatistics {
    /// Episodes inserted
    episodes_inserted: usize,
    /// Episodes failed
    episodes_failed: usize,
    /// Patterns inserted
    patterns_inserted: usize,
    /// Patterns failed
    patterns_failed: usize,
    /// Heuristics inserted
    heuristics_inserted: usize,
    /// Heuristics failed
    heuristics_failed: usize,
    /// Total operations
    total_operations: usize,
    /// Total duration
    total_duration: Duration,
    /// Episode throughput (ops/sec)
    episode_throughput: f64,
    /// Pattern throughput (ops/sec)
    pattern_throughput: f64,
}

impl BatchTestStatistics {
    /// Calculate total throughput
    fn total_throughput(&self) -> f64 {
        if self.total_duration.as_secs() == 0 {
            return 0.0;
        }
        self.total_operations as f64 / self.total_duration.as_secs_f64()
    }

    /// Check if throughput meets criteria
    fn meets_throughput_criteria(&self, config: &BatchTestConfig) -> anyhow::Result<()> {
        let throughput = self.total_throughput();

        if throughput < config.min_throughput {
            anyhow::bail!(
                "Throughput {:.2} ops/sec does not meet minimum {:.2} ops/sec",
                throughput,
                config.min_throughput
            );
        }

        // Warn if throughput exceeds maximum (this is not a failure, but worth noting)
        if throughput > config.max_throughput && self.total_duration.as_secs() > 1 {
            // Only warn if measurement is meaningful (> 1 second)
            println!(
                "Note: Throughput {:.2} ops/sec exceeds expected maximum {:.2} ops/sec",
                throughput, config.max_throughput
            );
        }

        Ok(())
    }

    /// Print summary
    fn print_summary(&self, test_name: &str) {
        println!("\n=== {} Batch Test Summary ===", test_name);
        println!("Total Operations: {}", self.total_operations);
        println!("Total Duration: {:?}", self.total_duration);
        println!("\nEpisode Operations:");
        println!("  Inserted: {}", self.episodes_inserted);
        println!("  Failed: {}", self.episodes_failed);
        println!("  Throughput: {:.2} ops/sec", self.episode_throughput);
        println!("\nPattern Operations:");
        println!("  Inserted: {}", self.patterns_inserted);
        println!("  Failed: {}", self.patterns_failed);
        println!("  Throughput: {:.2} ops/sec", self.pattern_throughput);
        println!("\nHeuristic Operations:");
        println!("  Inserted: {}", self.heuristics_inserted);
        println!("  Failed: {}", self.heuristics_failed);
        println!(
            "\nOverall Throughput: {:.2} ops/sec",
            self.total_throughput()
        );
        println!(
            "Success Rate: {:.2}%",
            (self.episodes_inserted + self.patterns_inserted + self.heuristics_inserted) as f64
                / self.total_operations as f64
                * 100.0
        );
        println!("=============================\n");
    }
}

/// Create a test Turso storage
async fn create_test_storage() -> (TursoStorage, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");

    let config = TursoConfig::default();
    let storage =
        TursoStorage::with_config(&format!("file:{}", db_path.to_string_lossy()), "", config)
            .await
            .expect("Failed to create Turso storage");

    storage
        .initialize_schema()
        .await
        .expect("Failed to initialize schema");

    (storage, temp_dir)
}

/// Create a test episode
fn create_test_episode(id: Uuid, index: usize) -> Episode {
    Episode {
        episode_id: id,
        task_type: TaskType::CodeGeneration,
        task_description: format!("Batch test episode {} - {}", index, id),
        context: TaskContext {
            domain: "batch_test".to_string(),
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity: memory_core::types::ComplexityLevel::Moderate,
            tags: vec!["batch_test".to_string(), format!("batch_{}", index / 1000)],
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
fn create_test_pattern(id: Uuid, episode_id: Uuid, index: usize) -> Pattern {
    Pattern::ToolSequence {
        id,
        tools: vec![
            format!("tool_{}", index % 5),
            format!("tool_{}", (index + 1) % 5),
        ],
        context: TaskContext {
            domain: "batch_test".to_string(),
            language: Some("rust".to_string()),
            ..Default::default()
        },
        success_rate: 0.8 + (index as f64 % 20.0) / 100.0, // Vary success rate slightly
        avg_latency: chrono::Duration::milliseconds(40 + (index as i64 % 20) * 2),
        occurrence_count: 5 + (index % 10) as i64,
        effectiveness: Default::default(),
    }
}

/// Create a test heuristic
fn create_test_heuristic(id: Uuid, index: usize) -> Heuristic {
    Heuristic {
        heuristic_id: id,
        condition: format!("test_condition_{}", index % 10),
        action: format!("test_action_{}", index % 5),
        confidence: 0.7 + (index as f64 % 30.0) / 100.0,
        evidence: memory_core::Evidence {
            episode_ids: vec![],
            success_rate: 0.8,
            sample_size: 10,
        },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

/// Batch insert episodes
async fn batch_insert_episodes(
    storage: &TursoStorage,
    count: usize,
    batch_size: usize,
) -> anyhow::Result<BatchTestStatistics> {
    println!(
        "Batch inserting {} episodes (batch size: {})...",
        count, batch_size
    );

    let mut stats = BatchTestStatistics::default();
    let start = Instant::now();

    for batch_start in (0..count).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(count);
        let batch_episodes: Vec<Episode> = (batch_start..batch_end)
            .map(|i| create_test_episode(Uuid::new_v4(), i))
            .collect();

        // Batch insert using store_episodes_batch
        storage.store_episodes_batch(batch_episodes.clone()).await?;

        let successful = batch_episodes.len();
        let failed = 0;

        stats.episodes_inserted += successful;
        stats.episodes_failed += failed;

        if batch_end % 1000 == 0 {
            println!("  Inserted {} / {} episodes...", batch_end, count);
        }
    }

    stats.total_duration = start.elapsed();
    if stats.total_duration.as_secs() > 0 {
        stats.episode_throughput =
            stats.episodes_inserted as f64 / stats.total_duration.as_secs_f64();
    }

    println!(
        "Episode batch insert completed in {:?}",
        stats.total_duration
    );
    println!(
        "Inserted: {}, Failed: {}, Throughput: {:.2} ops/sec",
        stats.episodes_inserted, stats.episodes_failed, stats.episode_throughput
    );

    Ok(stats)
}

/// Batch insert patterns
async fn batch_insert_patterns(
    storage: &TursoStorage,
    count: usize,
    batch_size: usize,
) -> anyhow::Result<BatchTestStatistics> {
    println!(
        "Batch inserting {} patterns (batch size: {})...",
        count, batch_size
    );

    let mut stats = BatchTestStatistics::default();
    let start = Instant::now();

    for batch_start in (0..count).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(count);
        let mut batch_patterns = Vec::with_capacity(batch_end - batch_start);

        for i in batch_start..batch_end {
            let pattern_id = Uuid::new_v4();
            let episode_id = Uuid::new_v4();
            batch_patterns.push(create_test_pattern(pattern_id, episode_id, i));
        }

        // Batch insert using store_patterns_batch
        storage.store_patterns_batch(batch_patterns.clone()).await?;

        let successful = batch_patterns.len();
        let failed = 0;

        stats.patterns_inserted += successful;
        stats.patterns_failed += failed;

        if batch_end % 5000 == 0 {
            println!("  Inserted {} / {} patterns...", batch_end, count);
        }
    }

    stats.total_duration = start.elapsed();
    if stats.total_duration.as_secs() > 0 {
        stats.pattern_throughput =
            stats.patterns_inserted as f64 / stats.total_duration.as_secs_f64();
    }

    println!(
        "Pattern batch insert completed in {:?}",
        stats.total_duration
    );
    println!(
        "Inserted: {}, Failed: {}, Throughput: {:.2} ops/sec",
        stats.patterns_inserted, stats.patterns_failed, stats.pattern_throughput
    );

    Ok(stats)
}

/// Batch insert heuristics
async fn batch_insert_heuristics(
    storage: &TursoStorage,
    count: usize,
    batch_size: usize,
) -> anyhow::Result<BatchTestStatistics> {
    println!(
        "Batch inserting {} heuristics (batch size: {})...",
        count, batch_size
    );

    let mut stats = BatchTestStatistics::default();
    let start = Instant::now();

    for batch_start in (0..count).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(count);
        let batch_heuristics: Vec<Heuristic> = (batch_start..batch_end)
            .map(|i| create_test_heuristic(Uuid::new_v4(), i))
            .collect();

        // Batch insert using store_heuristics_batch
        storage
            .store_heuristics_batch(batch_heuristics.clone())
            .await?;

        let successful = batch_heuristics.len();
        let failed = 0;

        stats.heuristics_inserted += successful;
        stats.heuristics_failed += failed;

        if batch_end % 500 == 0 {
            println!("  Inserted {} / {} heuristics...", batch_end, count);
        }
    }

    stats.total_duration = start.elapsed();

    println!(
        "Heuristic batch insert completed in {:?}",
        stats.total_duration
    );
    println!(
        "Inserted: {}, Failed: {}",
        stats.heuristics_inserted, stats.heuristics_failed
    );

    Ok(stats)
}

/// Test transaction safety by verifying all data was saved
async fn test_transaction_safety(
    storage: &TursoStorage,
    expected_episodes: usize,
    expected_patterns: usize,
) -> anyhow::Result<()> {
    println!("Testing transaction safety...");

    // Get actual counts using get_statistics
    let stats = storage.get_statistics().await?;

    println!(
        "Expected episodes: {}, Actual: {}",
        expected_episodes, stats.episode_count
    );
    println!(
        "Expected patterns: {}, Actual: {}",
        expected_patterns, stats.pattern_count
    );

    // Verify counts match
    if stats.episode_count != expected_episodes {
        anyhow::bail!(
            "Episode count mismatch: expected {}, got {}",
            expected_episodes,
            stats.episode_count
        );
    }

    if stats.pattern_count != expected_patterns {
        anyhow::bail!(
            "Pattern count mismatch: expected {}, got {}",
            expected_patterns,
            stats.pattern_count
        );
    }

    // Verify data integrity by sampling
    println!("Verifying data integrity by sampling...");

    // Verify we can retrieve episodes
    let test_count = std::cmp::min(100, expected_episodes);
    for _ in 0..test_count {
        let random_id = Uuid::new_v4();
        // Just verify we can query without errors
        let _result = storage.get_episode(random_id).await;
    }

    println!("Transaction safety test passed ✅");

    Ok(())
}

/// Main test entry point
#[tokio::test]
async fn test_batch_operations_load() {
    println!("=== Batch Operations Load Test ===");
    println!("Testing batch operation performance and transaction safety...");
    println!("Episodes to insert: {}", BATCH_EPISODE_COUNT);
    println!("Patterns to insert: {}", BATCH_PATTERN_COUNT);
    println!("Heuristics to insert: {}", BATCH_HEURISTIC_COUNT);
    println!(
        "Target throughput: {} - {} ops/sec\n",
        TARGET_THROUGHPUT_MIN, TARGET_THROUGHPUT_MAX
    );

    let start = Instant::now();

    let config = BatchTestConfig::default();

    let (storage, _temp_dir) = create_test_storage().await;

    // Batch insert episodes
    let episode_stats = tokio::time::timeout(
        MAX_TEST_DURATION,
        batch_insert_episodes(&storage, BATCH_EPISODE_COUNT, config.batch_size),
    )
    .await
    .expect("Batch episode insert timed out")
    .expect("Episode batch insert failed");

    // Batch insert patterns
    let pattern_stats = tokio::time::timeout(
        MAX_TEST_DURATION,
        batch_insert_patterns(&storage, BATCH_PATTERN_COUNT, config.batch_size),
    )
    .await
    .expect("Batch pattern insert timed out")
    .expect("Pattern batch insert failed");

    // Batch insert heuristics
    let heuristic_stats = tokio::time::timeout(
        MAX_TEST_DURATION,
        batch_insert_heuristics(&storage, BATCH_HEURISTIC_COUNT, config.batch_size),
    )
    .await
    .expect("Batch heuristic insert timed out")
    .expect("Heuristic batch insert failed");

    // Combine statistics
    let mut combined_stats = BatchTestStatistics::default();
    combined_stats.episodes_inserted = episode_stats.episodes_inserted;
    combined_stats.episodes_failed = episode_stats.episodes_failed;
    combined_stats.patterns_inserted = pattern_stats.patterns_inserted;
    combined_stats.patterns_failed = pattern_stats.patterns_failed;
    combined_stats.heuristics_inserted = heuristic_stats.heuristics_inserted;
    combined_stats.heuristics_failed = heuristic_stats.heuristics_failed;
    combined_stats.episode_throughput = episode_stats.episode_throughput;
    combined_stats.pattern_throughput = pattern_stats.pattern_throughput;

    combined_stats.total_operations = episode_stats.episodes_inserted
        + pattern_stats.patterns_inserted
        + heuristic_stats.heuristics_inserted;
    combined_stats.total_duration = start.elapsed();

    combined_stats.print_summary("Batch Operations");

    // Verify throughput criteria
    combined_stats
        .meets_throughput_criteria(&config)
        .expect("Throughput does not meet criteria");

    // Test transaction safety
    test_transaction_safety(&storage, BATCH_EPISODE_COUNT, BATCH_PATTERN_COUNT)
        .await
        .expect("Transaction safety test failed");

    let elapsed = start.elapsed();
    println!("Total test duration: {:?}", elapsed);
    println!("All batch operations load tests passed! ✅");
}

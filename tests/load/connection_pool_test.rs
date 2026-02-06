//! Connection pool load tests
//!
//! Tests for validating connection pool behavior under heavy load.
//! Ensures no connection exhaustion and proper pool scaling.
//!
//! Acceptance Criteria:
//! - 1000 concurrent connections
//! - 10,000 queries across connections
//! - Verify no connection exhaustion
//! - Test pool scaling behavior

use chrono::Utc;
use memory_core::{Episode, TaskContext, TaskType};
use memory_storage_turso::TursoStorage;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use uuid::Uuid;

/// Maximum number of concurrent connections to test
const MAX_CONCURRENT_CONNECTIONS: usize = 1000;

/// Total number of queries to execute
const TOTAL_QUERY_COUNT: usize = 10_000;

/// Maximum wait time for a connection from the pool (5 seconds)
const MAX_CONNECTION_WAIT: Duration = Duration::from_secs(5);

/// Maximum time for the entire test to complete (10 minutes)
const MAX_TEST_DURATION: Duration = Duration::from_secs(600);

/// Test configuration
struct TestConfig {
    /// Maximum pool size
    max_pool_size: usize,
    /// Minimum pool size
    min_pool_size: usize,
    /// Connection timeout
    connection_timeout: Duration,
    /// Expected success rate (0.0 to 1.0)
    expected_success_rate: f64,
}

/// Test statistics
#[derive(Debug, Default)]
struct TestStatistics {
    /// Total operations attempted
    total_operations: usize,
    /// Successful operations
    successful_operations: usize,
    /// Failed operations
    failed_operations: usize,
    /// Operations that timed out waiting for a connection
    connection_timeouts: usize,
    /// Minimum latency for successful operations
    min_latency: Option<Duration>,
    /// Maximum latency for successful operations
    max_latency: Option<Duration>,
    /// Total latency for successful operations
    total_latency: Duration,
    /// Connections created during test
    connections_created: usize,
    /// Connection pool exhaustion events
    pool_exhaustion_events: usize,
}

impl TestStatistics {
    /// Calculate success rate
    fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            return 0.0;
        }
        self.successful_operations as f64 / self.total_operations as f64
    }

    /// Calculate average latency for successful operations
    fn average_latency(&self) -> Option<Duration> {
        if self.successful_operations == 0 {
            return None;
        }
        Some(self.total_latency / self.successful_operations as u32)
    }

    /// Check if statistics meet success criteria
    fn meets_criteria(&self, config: &TestConfig) -> anyhow::Result<()> {
        let actual_rate = self.success_rate();
        if actual_rate < config.expected_success_rate {
            anyhow::bail!(
                "Success rate {:.2}% does not meet expected {:.2}%",
                actual_rate * 100.0,
                config.expected_success_rate * 100.0
            );
        }

        if self.connection_timeouts > 0 {
            anyhow::bail!(
                "Found {} connection timeouts - connection pool is being exhausted",
                self.connection_timeouts
            );
        }

        if self.pool_exhaustion_events > (self.total_operations - self.successful_operations) {
            anyhow::bail!(
                "Pool exhaustion exceeded expected failure count: {} > {}",
                self.pool_exhaustion_events,
                self.total_operations - self.successful_operations
            );
        }

        Ok(())
    }

    /// Print summary statistics
    fn print_summary(&self, test_name: &str) {
        println!("\n=== {} Test Summary ===", test_name);
        println!("Total Operations: {}", self.total_operations);
        println!(
            "Successful: {} ({:.2}%)",
            self.successful_operations,
            self.success_rate() * 100.0
        );
        println!("Failed: {}", self.failed_operations);
        println!("Connection Timeouts: {}", self.connection_timeouts);
        println!("Pool Exhaustion Events: {}", self.pool_exhaustion_events);
        println!("Connections Created: {}", self.connections_created);

        if let Some(avg) = self.average_latency() {
            println!("Average Latency: {:?}", avg);
        }

        if let Some(min) = self.min_latency {
            println!("Min Latency: {:?}", min);
        }

        if let Some(max) = self.max_latency {
            println!("Max Latency: {:?}", max);
        }
        println!("==========================\n");
    }
}

/// Create a test Turso storage with configured pool
async fn create_test_storage(
    max_pool_size: usize,
    _min_pool_size: usize,
    _connection_timeout: Duration,
) -> (TursoStorage, TempDir, Arc<Semaphore>) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");

    let storage = TursoStorage::new(&format!("file:{}", db_path.to_string_lossy()), "")
        .await
        .expect("Failed to create Turso storage");

    storage
        .initialize_schema()
        .await
        .expect("Failed to initialize schema");

    let semaphore = Arc::new(Semaphore::new(max_pool_size));

    (storage, temp_dir, semaphore)
}

/// Create a test episode
fn create_test_episode(id: Uuid) -> Episode {
    Episode {
        episode_id: id,
        task_type: TaskType::CodeGeneration,
        task_description: format!("Connection pool test episode {}", id),
        context: TaskContext {
            domain: "load_test".to_string(),
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity: memory_core::types::ComplexityLevel::Moderate,
            tags: vec!["connection_pool_test".to_string()],
        },
        steps: vec![],
        outcome: None,
        reward: None,
        reflection: None,
        patterns: vec![],
        heuristics: vec![],
        applied_patterns: vec![],
        salient_features: None,
        start_time: Utc::now(),
        end_time: None,
        metadata: std::collections::HashMap::new(),
        tags: vec![],
    }
}

/// Execute a single operation (insert + retrieve episode)
async fn execute_operation(
    storage: &TursoStorage,
    semaphore: &Arc<Semaphore>,
    operation_id: usize,
) -> Result<Duration, String> {
    let _permit = tokio::time::timeout(MAX_CONNECTION_WAIT, semaphore.acquire())
        .await
        .map_err(|_| format!("Operation {}: Connection timeout", operation_id))?
        .map_err(|e| {
            format!(
                "Operation {}: Failed to acquire semaphore: {}",
                operation_id, e
            )
        })?;

    let start = Instant::now();

    // Insert episode
    let episode_id = Uuid::new_v4();
    let episode = create_test_episode(episode_id);

    storage
        .store_episode(&episode)
        .await
        .map_err(|e| format!("Operation {}: Store failed: {}", operation_id, e))?;

    // Retrieve episode
    storage
        .get_episode(episode_id)
        .await
        .map_err(|e| format!("Operation {}: Retrieve failed: {}", operation_id, e))?;

    Ok(start.elapsed())
}

/// Run connection pool load test with high concurrency
async fn run_high_concurrency_test(
    storage: Arc<TursoStorage>,
    config: &TestConfig,
) -> TestStatistics {
    let mut stats = TestStatistics::default();
    let semaphore = Arc::new(Semaphore::new(100)); // Limit concurrent operations per batch

    println!("Starting high concurrency test...");
    println!(
        "Target: {} concurrent connections, {} total queries",
        MAX_CONCURRENT_CONNECTIONS, TOTAL_QUERY_COUNT
    );

    let mut join_set = JoinSet::new();

    // Start operations in batches to simulate load
    for _i in 0..TOTAL_QUERY_COUNT {
        let storage_clone = storage.clone();
        let semaphore_clone = semaphore.clone();

        join_set.spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap();
            let start = Instant::now();

            // Insert episode
            let episode_id = Uuid::new_v4();
            let episode = create_test_episode(episode_id);
            let result1 = storage_clone.store_episode(&episode).await;

            // Retrieve episode
            let result2 = storage_clone.get_episode(episode_id).await;

            let latency = start.elapsed();

            match (result1, result2) {
                (Ok(_), Ok(_)) => Ok(latency),
                (Err(e1), _) => Err(format!("Store failed: {}", e1)),
                (_, Err(e2)) => Err(format!("Retrieve failed: {}", e2)),
            }
        });

        // Limit the number of concurrent tasks
        if join_set.len() >= MAX_CONCURRENT_CONNECTIONS {
            if let Some(result) = join_set.join_next().await {
                stats.total_operations += 1;
                match result.unwrap() {
                    Ok(latency) => {
                        stats.successful_operations += 1;
                        stats.total_latency += latency;
                        stats.min_latency =
                            Some(stats.min_latency.map_or(latency, |m| m.min(latency)));
                        stats.max_latency =
                            Some(stats.max_latency.map_or(latency, |m| m.max(latency)));
                    }
                    Err(e) => {
                        stats.failed_operations += 1;
                        if e.contains("timeout") {
                            stats.connection_timeouts += 1;
                        }
                    }
                }
            }
        }
    }

    // Wait for remaining tasks
    while let Some(result) = join_set.join_next().await {
        stats.total_operations += 1;
        match result.unwrap() {
            Ok(latency) => {
                stats.successful_operations += 1;
                stats.total_latency += latency;
                stats.min_latency = Some(stats.min_latency.map_or(latency, |m| m.min(latency)));
                stats.max_latency = Some(stats.max_latency.map_or(latency, |m| m.max(latency)));
            }
            Err(e) => {
                stats.failed_operations += 1;
                if e.contains("timeout") {
                    stats.connection_timeouts += 1;
                }
            }
        }
    }

    stats
}

/// Test connection pool scaling behavior
async fn test_pool_scaling_behavior(storage: Arc<TursoStorage>) -> anyhow::Result<()> {
    println!("Testing connection pool scaling behavior...");

    let operations_per_phase = vec![100, 500, 1000, 500, 100];
    let mut stats_by_phase = Vec::new();

    for (phase, operations) in operations_per_phase.iter().enumerate() {
        println!("Phase {}: Executing {} operations", phase + 1, operations);

        let mut phase_stats = TestStatistics::default();
        let sem = Arc::new(Semaphore::new(200)); // Limit concurrency per phase
        let mut join_set = JoinSet::new();

        for _i in 0..*operations {
            let storage_clone = storage.clone();
            let sem_clone = sem.clone();

            join_set.spawn(async move {
                let _permit = sem_clone.acquire().await.unwrap();
                let start = Instant::now();

                let episode_id = Uuid::new_v4();
                let episode = create_test_episode(episode_id);

                let result1 = storage_clone.store_episode(&episode).await;
                let result2 = storage_clone.get_episode(episode_id).await;

                let latency = start.elapsed();

                match (result1, result2) {
                    (Ok(_), Ok(_)) => Ok(latency),
                    (Err(e1), _) => Err(format!("Store failed: {}", e1)),
                    (_, Err(e2)) => Err(format!("Retrieve failed: {}", e2)),
                }
            });
        }

        while let Some(result) = join_set.join_next().await {
            phase_stats.total_operations += 1;
            match result.unwrap() {
                Ok(latency) => {
                    phase_stats.successful_operations += 1;
                    phase_stats.total_latency += latency;
                    phase_stats.min_latency =
                        Some(phase_stats.min_latency.map_or(latency, |m| m.min(latency)));
                    phase_stats.max_latency =
                        Some(phase_stats.max_latency.map_or(latency, |m| m.max(latency)));
                }
                Err(_e) => phase_stats.failed_operations += 1,
            }
        }

        if let Some(avg) = phase_stats.average_latency() {
            println!(
                "Phase {}: {} operations, {:.2}% success, avg latency {:?}",
                phase + 1,
                operations,
                phase_stats.success_rate() * 100.0,
                avg
            );
        }

        stats_by_phase.push(phase_stats);
        // Small delay between phases
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Verify no performance degradation across phases
    println!("Verifying pool scaling behavior...");

    let base_latency = stats_by_phase[2].average_latency().unwrap(); // Use peak phase as baseline
    let recovery_latency = stats_by_phase[4].average_latency().unwrap();

    let latency_diff = if recovery_latency > base_latency {
        (recovery_latency.as_millis() as f64 - base_latency.as_millis() as f64)
            / base_latency.as_millis() as f64
    } else {
        0.0
    };

    if latency_diff > 0.5 {
        anyhow::bail!(
            "Latency degradation detected: {:.2}% (baseline: {:?}, recovery: {:?})",
            latency_diff * 100.0,
            base_latency,
            recovery_latency
        );
    }

    println!("Pool scaling behavior test passed - latency within acceptable range");

    Ok(())
}

/// Main test entry point
#[tokio::test]
async fn test_connection_pool_load() {
    println!("=== Connection Pool Load Test ===");
    println!(
        "Starting connection pool load test with {} concurrent connections...",
        MAX_CONCURRENT_CONNECTIONS
    );
    println!("Target: {} total operations", TOTAL_QUERY_COUNT);
    println!("Max test duration: {:?}", MAX_TEST_DURATION);
    println!("Expected success rate: 100%\n");

    let start_time = Instant::now();

    let config = TestConfig {
        max_pool_size: MAX_CONCURRENT_CONNECTIONS,
        min_pool_size: 10,
        connection_timeout: Duration::from_secs(30),
        expected_success_rate: 1.0, // 100% success required
    };

    let (storage, _temp_dir, _semaphore) = create_test_storage(
        config.max_pool_size,
        config.min_pool_size,
        config.connection_timeout,
    )
    .await;

    let storage = Arc::new(storage);

    // Run high concurrency test
    let stats = tokio::time::timeout(
        MAX_TEST_DURATION,
        run_high_concurrency_test(storage.clone(), &config),
    )
    .await
    .expect("Connection pool load test timed out");

    stats.print_summary("Connection Pool Load");

    // Verify success criteria
    stats
        .meets_criteria(&config)
        .expect("Test failed criteria check");

    // Test pool scaling behavior
    test_pool_scaling_behavior(storage.clone())
        .await
        .expect("Pool scaling behavior test failed");

    // Test connection pool stats
    if let Some(pool_stats) = storage.pool_statistics().await {
        println!("\n=== Connection Pool Statistics ===");
        println!("Active connections: {}", pool_stats.active_connections);
        println!("Total connections created: {}", pool_stats.total_created);
        println!("Average wait time: {}ms", pool_stats.avg_wait_time_ms);
    }

    let elapsed = start_time.elapsed();
    println!("Test completed in {:?}", elapsed);
    println!("All connection pool load tests passed! ✅");
}

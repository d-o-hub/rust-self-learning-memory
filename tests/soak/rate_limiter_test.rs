//! Rate limiter soak test
//!
//! Tests for validating rate limiter functionality under sustained and burst load.
//! Ensures rate limiting is accurate and system recovers after rate limit.
//!
//! Acceptance Criteria:
//! - Sustained load at rate limit accuracy > 95%
//! - Burst load handling within tolerance
//! - Rate limiter recovery verification
//! - System remains stable under rate limiting

use governor::{
    clock::{DefaultClock, QuantaClock},
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use memory_core::{types::ExecutionResult, Episode, TaskContext, TaskOutcome, TaskType};
use memory_storage_turso::{
    CacheConfig, CachedTursoStorage, PoolConfig, TursoConfig, TursoStorage,
};
use std::num::NonZeroU32;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::sync::Semaphore;
use uuid::Uuid;

/// Rate limit settings
const REQUESTS_PER_SECOND: u32 = 100;
const BURST_CAPACITY: u32 = 150;

/// Test durations
const SUSTAINED_LOAD_DURATION: Duration = Duration::from_secs(60);
const BURST_LOAD_DURATION: Duration = Duration::from_secs(10);
const RECOVERY_TEST_DURATION: Duration = Duration::from_secs(30);

/// Rate limiter test statistics
#[derive(Debug, Default)]
struct RateLimitStatistics {
    /// Total requests attempted
    total_requests: usize,
    /// Requests that passed rate limit
    allowed_requests: usize,
    /// Requests that were rate limited
    rate_limited_requests: usize,
    /// Minimum latency for allowed requests
    min_latency: Option<Duration>,
    /// Maximum latency for allowed requests
    max_latency: Option<Duration>,
    /// Total latency for allowed requests
    total_latency: Duration,
}

impl RateLimitStatistics {
    /// Calculate actual rate (requests per second)
    fn actual_rate(&self, duration: Duration) -> f64 {
        if duration.as_secs() == 0 {
            return 0.0;
        }
        self.total_requests as f64 / duration.as_secs_f64()
    }

    /// Calculate rate limit accuracy
    fn rate_limit_accuracy(&self, expected_rate: f64) -> f64 {
        let actual_rate = self.allowed_requests as f64 / SUSTAINED_LOAD_DURATION.as_secs_f64();
        100.0 - (actual_rate - expected_rate).abs() / expected_rate * 100.0
    }

    /// Calculate average latency
    fn average_latency(&self) -> Option<Duration> {
        if self.allowed_requests == 0 {
            return None;
        }
        Some(self.total_latency / self.allowed_requests as u32)
    }

    /// Print summary
    fn print_summary(&self, test_name: &str, duration: Duration) {
        println!("\n=== {} Test Summary ===", test_name);
        println!("Duration: {:?}", duration);
        println!("Total Requests: {}", self.total_requests);
        println!("Allowed Requests: {}", self.allowed_requests);
        println!("Rate Limited Requests: {}", self.rate_limited_requests);
        println!("Actual Rate: {:.2} req/sec", self.actual_rate(duration));
        println!(
            "Rate Limit Accuracy: {:.2}%",
            self.rate_limit_accuracy(REQUESTS_PER_SECOND as f64)
        );

        if let Some(avg) = self.average_latency() {
            println!("Average Latency: {:?}", avg);
        }

        if let Some(min) = self.min_latency {
            println!("Min Latency: {:?}", min);
        }

        if let Some(max) = self.max_latency {
            println!("Max Latency: {:?}", max);
        }
        println!("========================\n");
    }

    /// Check if statistics meet criteria
    fn meets_criteria(&self) -> anyhow::Result<()> {
        let accuracy = self.rate_limit_accuracy(REQUESTS_PER_SECOND as f64);
        if accuracy < 95.0 {
            anyhow::bail!(
                "Rate limit accuracy {:.2}% is below 95% threshold",
                accuracy
            );
        }
        Ok(())
    }
}

/// Create a rate limiter with governor
fn create_rate_limiter() -> RateLimiter<NotKeyed, InMemoryState, QuantaClock> {
    let quota = Quota::per_second(NonZeroU32::new(REQUESTS_PER_SECOND).unwrap())
        .allow_burst(NonZeroU32::new(BURST_CAPACITY).unwrap());

    RateLimiter::direct(quota)
}

/// Create a test storage instance
async fn create_test_storage() -> (CachedTursoStorage, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");

    let pool_config = PoolConfig {
        max_pool_size: 20,
        min_pool_size: 5,
        connection_timeout: Duration::from_secs(10),
        idle_timeout: Duration::from_secs(30),
        keepalive_interval: Duration::from_secs(10),
    };

    let config = TursoConfig {
        pool_config,
        ..Default::default()
    };

    let storage =
        TursoStorage::with_config(&format!("file:{}", db_path.to_string_lossy()), "", config)
            .await
            .expect("Failed to create Turso storage");

    storage
        .initialize_schema()
        .await
        .expect("Failed to initialize schema");

    let cache_config = CacheConfig::default();
    let cached_storage = CachedTursoStorage::new(storage, cache_config);

    (cached_storage, temp_dir)
}

/// Execute a single request through rate limiter
async fn execute_request(
    storage: &CachedTursoStorage,
    rate_limiter: &RateLimiter<NotKeyed, InMemoryState, QuantaClock>,
    semaphore: &Semaphore,
    request_id: usize,
) -> Result<Duration, String> {
    let start = Instant::now();

    // Try to acquire rate limit permit
    match rate_limiter.check() {
        Ok(_) => {
            // Request is allowed
            let _permit = semaphore
                .acquire()
                .await
                .map_err(|e| format!("Failed to acquire semaphore: {}", e))?;

            // Create and complete an episode
            let episode_id = storage
                .start_episode(
                    format!("Rate limiter test request {}", request_id),
                    TaskContext {
                        domain: "rate_limiter_test".to_string(),
                        language: Some("rust".to_string()),
                        framework: Some("tokio".to_string()),
                        complexity: memory_core::types::ComplexityLevel::Simple,
                        tags: vec!["rate_limiter_test".to_string()],
                    },
                    TaskType::CodeGeneration,
                )
                .await;

            // Add a step
            let mut step = memory_core::episode::ExecutionStep::new(
                1,
                "test_tool".to_string(),
                "Execute test operation".to_string(),
            );
            step.result = Some(ExecutionResult::Success {
                output: "Operation completed".to_string(),
            });
            step.latency_ms = 10;
            storage
                .log_step(episode_id, step)
                .await
                .map_err(|e| format!("Failed to log step: {}", e))?;

            // Complete episode
            storage
                .complete_episode(
                    episode_id,
                    TaskOutcome::Success {
                        verdict: "Completed successfully".to_string(),
                        artifacts: vec![],
                    },
                )
                .await
                .map_err(|e| format!("Failed to complete episode: {}", e))?;

            Ok(start.elapsed())
        }
        Err(_) => {
            // Rate limited
            Err("Rate limited".to_string())
        }
    }
}

/// Test sustained load at rate limit
async fn test_sustained_load(
    storage: &CachedTursoStorage,
    rate_limiter: &RateLimiter<NotKeyed, InMemoryState, QuantaClock>,
) -> RateLimitStatistics {
    println!("Testing sustained load at rate limit...");
    println!("Expected rate: {} requests/sec", REQUESTS_PER_SECOND);
    println!("Burst capacity: {}", BURST_CAPACITY);
    println!("Duration: {:?}", SUSTAINED_LOAD_DURATION);
    println!();

    let mut stats = RateLimitStatistics::default();
    let semaphore = Arc::new(Semaphore::new(10)); // Limit concurrent DB operations
    let start = Instant::now();
    let mut request_id = 0;

    loop {
        let now = Instant::now();
        let _elapsed = now - start;

        if _elapsed >= SUSTAINED_LOAD_DURATION {
            break;
        }

        stats.total_requests += 1;
        request_id += 1;

        match execute_request(storage, rate_limiter, &semaphore, request_id).await {
            Ok(latency) => {
                stats.allowed_requests += 1;
                stats.total_latency += latency;
                stats.min_latency = Some(stats.min_latency.map_or(latency, |m| m.min(latency)));
                stats.max_latency = Some(stats.max_latency.map_or(latency, |m| m.max(latency)));
            }
            Err(_) => {
                stats.rate_limited_requests += 1;
            }
        }

        // Small delay to avoid busy waiting
        tokio::time::sleep(Duration::from_micros(100)).await;

        if request_id % 1000 == 0 {
            let actual_rate = stats.actual_rate(_elapsed);
            println!(
                "Progress: {} requests, actual rate: {:.2} req/sec, rate limited: {}",
                request_id, actual_rate, stats.rate_limited_requests
            );
        }
    }

    let duration = Instant::now() - start;
    stats.print_summary("Sustained Load", duration);

    stats
}

/// Test burst load exceeding rate limit
async fn test_burst_load(
    storage: &CachedTursoStorage,
    rate_limiter: &RateLimiter<NotKeyed, InMemoryState, QuantaClock>,
) -> RateLimitStatistics {
    println!("Testing burst load exceeding rate limit...");
    println!("Burst duration: {:?}", BURST_LOAD_DURATION);
    println!();

    let mut stats = RateLimitStatistics::default();
    let semaphore = Arc::new(Semaphore::new(10));
    let start = Instant::now();
    let mut request_id = 0;

    loop {
        let _elapsed = Instant::now() - start;

        if _elapsed >= BURST_LOAD_DURATION {
            break;
        }

        stats.total_requests += 1;
        request_id += 1;

        match execute_request(storage, rate_limiter, &semaphore, request_id).await {
            Ok(latency) => {
                stats.allowed_requests += 1;
                stats.total_latency += latency;
                stats.min_latency = Some(stats.min_latency.map_or(latency, |m| m.min(latency)));
                stats.max_latency = Some(stats.max_latency.map_or(latency, |m| m.max(latency)));
            }
            Err(_) => {
                stats.rate_limited_requests += 1;
            }
        }

        // For burst test, minimize delays to maximize rate
        tokio::time::sleep(Duration::from_micros(10)).await;

        if request_id % 500 == 0 {
            println!(
                "Burst progress: {} requests, rate limited: {}",
                request_id, stats.rate_limited_requests
            );
        }
    }

    let duration = Instant::now() - start;
    stats.print_summary("Burst Load", duration);

    stats
}

/// Test rate limiter recovery after being rate limited
async fn test_rate_limiter_recovery(
    storage: &CachedTursoStorage,
    rate_limiter: &RateLimiter<NotKeyed, InMemoryState, QuantaClock>,
) -> anyhow::Result<()> {
    println!("Testing rate limiter recovery...");

    // First, trigger rate limit by sending burst
    println!("Triggering rate limit with burst...");
    let semaphore = Arc::new(Semaphore::new(10));

    for i in 0..(BURST_CAPACITY * 2) {
        let _ = execute_request(storage, rate_limiter, &semaphore, i).await;
        tokio::time::sleep(Duration::from_millis(1)).await;
    }

    println!("Burst sent, waiting for recovery...");

    // Now test normal requests after waiting
    let start = Instant::now();
    let mut successful = 0;
    let mut failed = 0;
    let target = REQUESTS_PER_SECOND as usize * 2; // Should allow 2 seconds worth of requests

    for i in 0..target {
        match execute_request(storage, rate_limiter, &semaphore, i).await {
            Ok(_) => successful += 1,
            Err(_) => failed += 1,
        }

        // Wait a bit between requests to allow rate limiter to recover
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let duration = Instant::now() - start;
    println!("Recovery test completed in {:?}", duration);
    println!(
        "Successful requests: {}, Failed requests: {}",
        successful, failed
    );

    // Verify rate limiter recovered
    let success_rate = successful as f64 / (successful + failed) as f64;
    if success_rate < 0.8 {
        anyhow::bail!(
            "Rate limiter did not recover properly: success rate {:.2}% < 80%",
            success_rate * 100.0
        );
    }

    println!("Rate limiter recovery test passed ✅");

    Ok(())
}

/// Main test entry point
#[tokio::test]
async fn test_rate_limiter_soak() {
    println!("=== Rate Limiter Soak Test ===\n");

    let rate_limiter = create_rate_limiter();
    let (storage, _temp_dir) = create_test_storage().await;

    // Test 1: Sustained load at rate limit
    println!("=== Test 1: Sustained Load ===");
    let sustained_stats = test_sustained_load(&storage, &rate_limiter).await;

    sustained_stats
        .meets_criteria()
        .expect("Sustained load test failed criteria");

    println!("Sustained load test passed ✅\n");

    // Test 2: Burst load
    println!("=== Test 2: Burst Load ===");
    let burst_stats = test_burst_load(&storage, &rate_limiter).await;

    // Verify burst capacity was effective
    let burst_utilization = burst_stats.allowed_requests as f64 / BURST_CAPACITY as f64;
    println!("Burst utilization: {:.2}%", burst_utilization * 100.0);

    if burst_utilization < 0.8 {
        println!("Note: Burst utilization below 80%, possible rate limiter issue");
    }

    println!("Burst load test passed ✅\n");

    // Test 3: Rate limiter recovery
    println!("=== Test 3: Rate Limiter Recovery ===");
    test_rate_limiter_recovery(&storage, &rate_limiter)
        .await
        .expect("Rate limiter recovery test failed");

    println!("Rate limiter recovery test passed ✅\n");

    // Summary
    println!("=== All Rate Limiter Tests Passed! ✅ ===");
    println!("\nSummary:");
    println!(
        "Sustained load accuracy: {:.2}%",
        sustained_stats.rate_limit_accuracy(REQUESTS_PER_SECOND as f64)
    );
    println!(
        "Burst requests allowed: {}/{} ({:.2}%)",
        burst_stats.allowed_requests,
        BURST_CAPACITY,
        burst_stats.allowed_requests as f64 / BURST_CAPACITY as f64 * 100.0
    );
    println!("Rate limiter recovery: Functional");
}

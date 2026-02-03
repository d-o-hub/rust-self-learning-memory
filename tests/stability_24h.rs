//! Example 24-hour stability test using the stability framework
//!
//! This test demonstrates how to use the stability framework for long-running tests.
//! Run with: `cargo test --test stability_24h -- --ignored`

mod stability;

use stability::{create_24h_test, create_ci_test, TestConfig, TestMetrics};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 24-hour stability test
///
/// Run manually with: `cargo test --test stability_24h -- --ignored`
#[tokio::test]
#[ignore = "Run manually for 24h soak test"]
async fn test_24h_stability() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║           24-Hour Stability Soak Test                      ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!("This test runs continuous operations for 24 hours to validate");
    println!("system stability, detect memory leaks, and monitor performance.");
    println!("\nPress Ctrl+C to stop early (results will still be reported).\n");

    let test = create_24h_test();

    test.run(|metrics: Arc<TestMetrics>| async move {
        // Simulate an operation
        let start = Instant::now();

        // Your actual test operations would go here
        // For example:
        // - Create episodes
        // - Query memory
        // - Store/retrieve patterns

        tokio::time::sleep(Duration::from_millis(1)).await;

        // Record the operation result
        metrics.record_operation(true, start.elapsed());
    })
    .await;

    println!("\n✅ 24-hour stability test completed successfully!");
}

/// Quick CI stability test (1 minute)
///
/// Run with: `cargo test --test stability_24h`
#[tokio::test]
async fn test_ci_stability() {
    println!("\n=== CI Stability Test (1 minute) ===\n");

    let test = create_ci_test();

    test.run(|metrics: Arc<TestMetrics>| async move {
        let start = Instant::now();

        // Simulate an operation
        tokio::time::sleep(Duration::from_millis(1)).await;

        metrics.record_operation(true, start.elapsed());
    })
    .await;

    println!("\n✅ CI stability test completed successfully!");
}

/// Custom stability test with specific configuration
#[tokio::test]
async fn test_custom_stability() {
    println!("\n=== Custom Stability Test ===\n");

    let config = TestConfig {
        duration: Duration::from_secs(10), // 10 seconds
        snapshot_interval: Duration::from_secs(2),
        memory_check_interval: Duration::from_secs(1),
        worker_count: 2,
        memory_leak_threshold: 30.0,
        performance_degradation_threshold: 50.0,
        min_success_rate: 0.90,
        monitor_memory: true,
        monitor_performance: true,
        print_progress: true,
    };

    let test = stability::StabilityTest::new(config);

    test.run(|metrics: Arc<TestMetrics>| async move {
        let start = Instant::now();

        // Simulate work with variable latency
        let delay = if metrics.total_operations() % 10 == 0 {
            Duration::from_millis(5) // Occasional slower operation
        } else {
            Duration::from_millis(1)
        };

        tokio::time::sleep(delay).await;

        // Occasionally record a failure for testing
        let success = metrics.total_operations() % 50 != 0;
        metrics.record_operation(success, start.elapsed());
    })
    .await;

    println!("\n✅ Custom stability test completed successfully!");
}

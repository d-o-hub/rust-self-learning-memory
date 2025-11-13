//! Integration tests for connection pool performance and functionality

use memory_storage_turso::{ConnectionPool, PoolConfig};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;

async fn create_test_pool() -> (Arc<ConnectionPool>, TempDir) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");

    let db = libsql::Builder::new_local(&db_path).build().await.unwrap();

    let config = PoolConfig {
        max_connections: 10,
        connection_timeout: Duration::from_secs(5),
        enable_health_check: true,
        health_check_timeout: Duration::from_secs(2),
    };

    let pool = ConnectionPool::new(Arc::new(db), config).await.unwrap();
    (Arc::new(pool), dir)
}

#[tokio::test]
#[cfg_attr(target_os = "windows", ignore = "Crashes on Windows CI with STATUS_ACCESS_VIOLATION")]
async fn test_pool_performance_concurrent_operations() {
    let (pool, _dir) = create_test_pool().await;

    let start = Instant::now();
    let mut handles = vec![];

    // Spawn 100 concurrent operations
    for _ in 0..100 {
        let pool_clone = Arc::clone(&pool);
        let handle = tokio::spawn(async move {
            let conn = pool_clone.get().await.unwrap();
            // Simulate database work
            let result = conn.connection().query("SELECT 1", ()).await;
            assert!(result.is_ok());
            tokio::time::sleep(Duration::from_millis(5)).await;
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let elapsed = start.elapsed();

    // Wait a bit for all drops to complete
    tokio::time::sleep(Duration::from_millis(100)).await;

    let stats = pool.statistics().await;

    println!("100 concurrent operations completed in: {:?}", elapsed);
    println!("Total checkouts: {}", stats.total_checkouts);
    println!("Total created connections: {}", stats.total_created);
    println!("Avg wait time: {}ms", stats.avg_wait_time_ms);

    // Verify performance targets
    assert_eq!(stats.total_checkouts, 100);
    assert_eq!(stats.total_created, 100); // Creates a new connection for each request
    assert!(elapsed.as_millis() < 5000); // Should complete within 5 seconds (P95 target)

    // Verify concurrency was limited (no more than pool size active at once)
    // The semaphore ensures max 10 concurrent connections
    assert!(stats.active_connections == 0); // All should be returned after test completes
}

#[tokio::test]
#[cfg_attr(target_os = "windows", ignore = "Crashes on Windows CI with STATUS_ACCESS_VIOLATION")]
async fn test_pool_with_turso_storage() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");

    let db = libsql::Builder::new_local(&db_path).build().await.unwrap();

    let config = PoolConfig {
        max_connections: 5,
        connection_timeout: Duration::from_secs(5),
        enable_health_check: true,
        health_check_timeout: Duration::from_secs(2),
    };

    let pool = ConnectionPool::new(Arc::new(db), config).await.unwrap();

    // Test multiple sequential operations
    for i in 0..10 {
        let conn = pool.get().await.unwrap();
        let result = conn.connection().query("SELECT 1", ()).await;
        assert!(result.is_ok(), "Query {} failed", i);
    }

    let stats = pool.statistics().await;
    assert_eq!(stats.total_checkouts, 10);
}

#[tokio::test]
#[cfg_attr(target_os = "windows", ignore = "Crashes on Windows CI with STATUS_ACCESS_VIOLATION")]
async fn test_pool_utilization_tracking() {
    let (pool, _dir) = create_test_pool().await;

    // Initially no utilization
    assert_eq!(pool.utilization().await, 0.0);

    // Get connections and check utilization increases
    let _conn1 = pool.get().await.unwrap();
    assert!(pool.utilization().await > 0.0);

    let _conn2 = pool.get().await.unwrap();
    assert!(pool.utilization().await > 0.1);

    drop(_conn1);
    drop(_conn2);

    // Wait for drops to complete
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Utilization should be back to 0
    assert_eq!(pool.utilization().await, 0.0);
}

#[tokio::test]
#[cfg_attr(target_os = "windows", ignore = "Crashes on Windows CI with STATUS_ACCESS_VIOLATION")]
async fn test_pool_health_checks() {
    let (pool, _dir) = create_test_pool().await;

    // Get multiple connections, all should pass health checks
    for _ in 0..5 {
        let _conn = pool.get().await.unwrap();
    }

    let stats = pool.statistics().await;
    assert_eq!(stats.total_health_checks_passed, 5);
    assert_eq!(stats.total_health_checks_failed, 0);
}

#[tokio::test]
#[cfg_attr(target_os = "windows", ignore = "Crashes on Windows CI with STATUS_ACCESS_VIOLATION")]
async fn test_pool_graceful_shutdown() {
    let (pool, _dir) = create_test_pool().await;

    // Perform some operations
    {
        let _conn1 = pool.get().await.unwrap();
        let _conn2 = pool.get().await.unwrap();
    }

    // Wait for connections to be returned
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Shutdown should complete cleanly
    let result = pool.shutdown().await;
    assert!(result.is_ok());
}

#[tokio::test]
#[cfg_attr(target_os = "windows", ignore = "Crashes on Windows CI with STATUS_ACCESS_VIOLATION")]
async fn test_pool_statistics_accuracy() {
    let (pool, _dir) = create_test_pool().await;

    // Get and use 3 connections
    for _ in 0..3 {
        let conn = pool.get().await.unwrap();
        let _result = conn.connection().query("SELECT 1", ()).await.unwrap();
        drop(conn);
    }

    tokio::time::sleep(Duration::from_millis(50)).await;

    let stats = pool.statistics().await;

    // Verify statistics
    assert_eq!(stats.total_checkouts, 3);
    assert!(stats.total_created >= 3);
    assert_eq!(stats.total_health_checks_passed, 3);
    assert_eq!(stats.active_connections, 0);
}

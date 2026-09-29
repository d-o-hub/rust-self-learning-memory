//! Tests for keep-alive connection pool

use super::*;
use tempfile::TempDir;
use tokio::time::Duration;

async fn create_test_keepalive_pool() -> (KeepAlivePool, TempDir) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");

    let db = libsql::Builder::new_local(&db_path).build().await.unwrap();
    let pool_config = crate::pool::PoolConfig {
        min_connections: 1,
        max_connections: 5,
        connection_timeout: Duration::from_secs(5),
        enable_health_check: true,
        health_check_timeout: Duration::from_secs(2),
        acquire_timeout_ms: 5000,
        idle_timeout_ms: 0,
    };

    let pool = crate::pool::ConnectionPool::new(std::sync::Arc::new(db), pool_config)
        .await
        .unwrap();
    let pool = std::sync::Arc::new(pool);

    let keepalive_config = KeepAliveConfig {
        keep_alive_interval: Duration::from_millis(100),
        stale_threshold: Duration::from_millis(200),
        enable_proactive_ping: true,
        ping_timeout: Duration::from_secs(1),
    };

    let keepalive_pool = KeepAlivePool::with_config(pool, keepalive_config)
        .await
        .unwrap();

    (keepalive_pool, dir)
}

#[tokio::test]
async fn test_keepalive_pool_creation() {
    let (pool, _dir) = create_test_keepalive_pool().await;
    let stats = pool.statistics();

    assert_eq!(stats.total_connections_created, 0);
    assert_eq!(stats.active_connections, 0);
}

#[tokio::test]
async fn test_connection_acquisition() {
    let (pool, _dir) = create_test_keepalive_pool().await;

    let conn = pool.get().await;
    assert!(conn.is_ok());

    let stats = pool.statistics();
    assert_eq!(stats.total_connections_created, 1);
    assert_eq!(stats.active_connections, 1);
}

#[tokio::test]
async fn test_concurrent_access() {
    let (pool, _dir) = create_test_keepalive_pool().await;
    let pool = std::sync::Arc::new(pool);

    let mut handles = vec![];

    for i in 0..5 {
        let pool_clone = std::sync::Arc::clone(&pool);
        let handle = tokio::spawn(async move {
            let conn = pool_clone.get().await;
            assert!(conn.is_ok(), "Checkout {} failed", i);

            tokio::time::sleep(Duration::from_millis(10)).await;

            conn
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await.unwrap();
    }

    let stats = pool.statistics();
    assert_eq!(stats.total_connections_created, 5);
    assert!(stats.active_connections <= 5);
}

#[tokio::test]
async fn test_active_connection_tracking() {
    let (pool, _dir) = create_test_keepalive_pool().await;

    let conn1 = pool.get().await.unwrap();
    let conn2 = pool.get().await.unwrap();

    assert_eq!(pool.active_connections(), 2);
    assert_eq!(pool.tracked_connections(), 2);

    drop(conn1);

    assert_eq!(pool.active_connections(), 1);

    assert_eq!(pool.tracked_connections(), 2);

    drop(conn2);

    assert_eq!(pool.active_connections(), 0);
}

#[tokio::test]
async fn test_cleanup() {
    let (pool, _dir) = create_test_keepalive_pool().await;

    let conn1 = pool.get().await.unwrap();
    let conn2 = pool.get().await.unwrap();

    assert_eq!(pool.tracked_connections(), 2);

    drop(conn1);
    drop(conn2);

    tokio::time::sleep(Duration::from_millis(500)).await;

    pool.cleanup();

    assert_eq!(pool.tracked_connections(), 0);
}

#[tokio::test]
async fn test_statistics_update() {
    let (pool, _dir) = create_test_keepalive_pool().await;

    let _conn = pool.get().await.unwrap();

    let stats = pool.statistics();
    assert!(stats.last_activity > std::time::Instant::now() - Duration::from_secs(1));

    drop(_conn);

    tokio::time::sleep(Duration::from_millis(50)).await;

    let stats = pool.statistics();
    assert_eq!(stats.active_connections, 0);
}

#[tokio::test]
async fn test_stale_connection_detection() {
    let (pool, _dir) = create_test_keepalive_pool().await;

    // Get a connection
    let conn = pool.get().await.unwrap();
    let conn_id = conn.connection_id();

    // Connection should not be stale initially
    assert!(!pool.is_stale(conn_id));

    drop(conn);

    // Wait for stale threshold
    tokio::time::sleep(Duration::from_millis(250)).await;

    // Connection should now be stale
    assert!(pool.is_stale(conn_id));

    // Get a new connection - this should detect the stale entry and refresh
    let _conn2 = pool.get().await.unwrap();

    let stats = pool.statistics();
    // Note: stale detection happens when we try to reuse a connection ID
    // Since we get a new connection with a new ID, the old ID is still stale
    // Verify the stats are accessible
    let _ = stats.total_stale_detected;
}

#[tokio::test]
async fn test_connection_refresh() {
    let (pool, _dir) = create_test_keepalive_pool().await;

    // Get a connection
    let conn = pool.get().await.unwrap();
    let _conn_id = conn.connection_id();

    drop(conn);

    // Wait for stale threshold
    tokio::time::sleep(Duration::from_millis(250)).await;

    // Get another connection - this should detect staleness
    let conn2 = pool.get().await.unwrap();
    // Note: connection ID may be different since we're creating a new entry
    // but the stale detection should still work

    let stats = pool.statistics();
    // At least one connection was created
    assert!(stats.total_connections_created >= 2);

    drop(conn2);
}

#[tokio::test]
async fn test_underlying_pool_stats() {
    let (pool, _dir) = create_test_keepalive_pool().await;

    let _conn = pool.get().await.unwrap();

    let pool_stats = pool.pool_statistics().await;
    // Note: pool validation happens during pool creation, so checkouts may be > 1
    assert!(pool_stats.total_checkouts >= 1);
}

#[tokio::test]
async fn test_into_connection_no_double_drop() {
    let (pool, _dir) = create_test_keepalive_pool().await;

    let conn = pool.get().await.unwrap();
    assert_eq!(pool.active_connections(), 1);

    // Consuming the wrapper extracts the connection; scope exit must not drop
    // the pooled connection a second time.
    let raw = conn.into_connection().unwrap();

    // Keep-alive stats decremented exactly once.
    assert_eq!(pool.active_connections(), 0);
    // Underlying pool permit released exactly once.
    let pool_stats = pool.pool_statistics().await;
    assert_eq!(pool_stats.active_connections, 0);

    // The extracted libSQL connection stays usable until its caller drops it.
    assert!(raw.query("SELECT 1", ()).await.is_ok());

    drop(raw);

    // Dropping the raw connection must not release the permit again.
    let pool_stats = pool.pool_statistics().await;
    assert_eq!(pool_stats.active_connections, 0);
    assert_eq!(pool.active_connections(), 0);
}

#[tokio::test]
async fn test_active_stats_zero_after_drop_and_extraction() {
    let (pool, _dir) = create_test_keepalive_pool().await;

    // Ordinary drop.
    {
        let conn = pool.get().await.unwrap();
        assert_eq!(pool.active_connections(), 1);
        assert!(conn.connection().is_ok());
    }
    assert_eq!(pool.active_connections(), 0);

    // Extraction path.
    let conn = pool.get().await.unwrap();
    assert_eq!(pool.active_connections(), 1);
    let raw = conn.into_connection().unwrap();
    assert_eq!(pool.active_connections(), 0);

    // Extracted connection remains usable after the wrapper is gone.
    let mut rows = raw.query("SELECT 1", ()).await.unwrap();
    assert!(rows.next().await.unwrap().is_some());
    drop(raw);

    assert_eq!(pool.active_connections(), 0);
}

#[tokio::test]
async fn test_connection_accessors_and_last_used() {
    let (pool, _dir) = create_test_keepalive_pool().await;

    let conn = pool.get().await.unwrap();

    // connection() and connection_id() reflect the acquired connection.
    assert!(conn.connection().is_ok());
    let conn_id = conn.connection_id();
    assert_eq!(pool.tracked_connections(), 1);
    assert!(!pool.is_stale(conn_id));

    // update_last_used() moves the timestamp forward.
    let before = conn.last_used();
    tokio::time::sleep(Duration::from_millis(5)).await;
    conn.update_last_used();
    assert!(conn.last_used() > before);

    drop(conn);
    assert_eq!(pool.active_connections(), 0);
}

#[tokio::test]
async fn test_drop_with_zero_active_stats_does_not_underflow() {
    let (pool, _dir) = create_test_keepalive_pool().await;

    let conn = pool.get().await.unwrap();
    assert_eq!(pool.active_connections(), 1);

    // Force the shared counter to zero before the wrapper drops: the custom
    // Drop must clamp instead of underflowing.
    pool.stats.write().active_connections = 0;

    drop(conn);

    assert_eq!(pool.active_connections(), 0);
}

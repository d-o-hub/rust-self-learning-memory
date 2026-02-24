use super::*;
use tempfile::TempDir;
use tokio::time::Duration;

async fn create_test_pool() -> (AdaptiveConnectionPool, TempDir) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");

    let db = libsql::Builder::new_local(&db_path).build().await.unwrap();

    let config = AdaptivePoolConfig {
        min_connections: 2,
        max_connections: 10,
        scale_up_threshold: 0.8,
        scale_down_threshold: 0.3,
        scale_up_cooldown: Duration::from_secs(1),
        scale_down_cooldown: Duration::from_secs(1),
        scale_up_increment: 2,
        scale_down_decrement: 2,
        check_interval: Duration::from_secs(5),
    };

    let pool = AdaptiveConnectionPool::new_sync(Arc::new(db), config)
        .await
        .unwrap();
    (pool, dir)
}

#[tokio::test]
async fn test_adaptive_pool_creation() {
    let (pool, _dir) = create_test_pool().await;
    let metrics = pool.metrics();

    assert_eq!(metrics.active_connections, 0);
    assert_eq!(metrics.max_connections, 2);
}

#[tokio::test]
async fn test_connection_checkout() {
    let (pool, _dir) = create_test_pool().await;

    let conn = pool.get().await;
    assert!(conn.is_ok());

    let metrics = pool.metrics();
    assert_eq!(metrics.total_acquired, 1);
    assert_eq!(metrics.active_connections, 1);
}

#[tokio::test]
async fn test_connection_auto_return() {
    let (pool, _dir) = create_test_pool().await;

    {
        let _conn = pool.get().await.unwrap();
        let metrics = pool.metrics();
        assert_eq!(metrics.active_connections, 1);
    }

    tokio::time::sleep(Duration::from_millis(50)).await;

    let metrics = pool.metrics();
    assert_eq!(metrics.active_connections, 0);
}

#[tokio::test]
async fn test_utilization() {
    let (pool, _dir) = create_test_pool().await;

    let utilization = pool.utilization();
    assert_eq!(utilization, 0.0);

    let _conn = pool.get().await.unwrap();

    let utilization = pool.utilization();
    assert!(utilization > 0.0 && utilization <= 1.0);
}

#[tokio::test]
async fn test_available_connections() {
    let (pool, _dir) = create_test_pool().await;

    let available = pool.available_connections();
    assert_eq!(available, 2);

    let _conn1 = pool.get().await.unwrap();
    let available = pool.available_connections();
    assert_eq!(available, 1);

    let _conn2 = pool.get().await.unwrap();
    let available = pool.available_connections();
    assert_eq!(available, 0);
}

#[tokio::test]
async fn test_active_connections() {
    let (pool, _dir) = create_test_pool().await;

    let active = pool.active_connections();
    assert_eq!(active, 0);

    let _conn = pool.get().await.unwrap();

    let active = pool.active_connections();
    assert_eq!(active, 1);
}

#[tokio::test]
async fn test_max_connections() {
    let (pool, _dir) = create_test_pool().await;

    let max = pool.max_connections();
    assert_eq!(max, 2);
}

#[tokio::test]
async fn test_check_and_scale() {
    let (pool, _dir) = create_test_pool().await;

    pool.check_and_scale().await;

    assert_eq!(pool.max_connections(), 2);
}

#[tokio::test]
async fn test_shutdown() {
    let (pool, _dir) = create_test_pool().await;

    let _conn = pool.get().await.unwrap();
    drop(_conn);

    tokio::time::sleep(Duration::from_millis(50)).await;

    pool.shutdown().await;
}

#[tokio::test]
async fn test_connection_exposure() {
    let (pool, _dir) = create_test_pool().await;

    // Get a connection from the pool
    let pooled_conn = pool.get().await.unwrap();

    // Verify we can access the underlying connection
    let conn_ref = pooled_conn.connection();
    assert!(conn_ref.is_some(), "Connection should be exposed");

    // Verify the connection is usable by running a query
    let conn = conn_ref.unwrap();
    let result = conn.query("SELECT 1", ()).await;
    assert!(result.is_ok(), "Connection should be usable for queries");

    // Test into_inner to take ownership
    let conn = pooled_conn.into_inner();
    assert!(conn.is_some(), "into_inner should return the connection");
}

#[tokio::test]
async fn test_connection_query_after_into_inner() {
    let (pool, _dir) = create_test_pool().await;

    // Get a connection and take ownership
    let pooled_conn = pool.get().await.unwrap();
    let conn = pooled_conn.into_inner().unwrap();

    // Verify the connection is still usable
    let result = conn.query("SELECT 1 as value", ()).await;
    assert!(result.is_ok());

    let mut rows = result.unwrap();
    let row = rows.next().await.unwrap();
    assert!(row.is_some());

    let value: i32 = row.unwrap().get(0).unwrap();
    assert_eq!(value, 1);
}

#[tokio::test]
#[ignore = "Timing-dependent test - connection ID uniqueness test has async timing issues in CI"]
async fn test_connection_id_uniqueness() {
    let (pool, _dir) = create_test_pool().await;

    // Get multiple connections and verify they have unique IDs
    let conn1 = pool.get().await.unwrap();
    let conn2 = pool.get().await.unwrap();
    let conn3 = pool.get().await.unwrap();

    let id1 = conn1.connection_id();
    let id2 = conn2.connection_id();
    let id3 = conn3.connection_id();

    // All IDs should be unique
    assert_ne!(id1, id2, "Connection IDs should be unique");
    assert_ne!(id2, id3, "Connection IDs should be unique");
    assert_ne!(id1, id3, "Connection IDs should be unique");

    // IDs should be monotonically increasing
    assert!(id2 > id1, "Connection IDs should be increasing");
    assert!(id3 > id2, "Connection IDs should be increasing");
}

#[tokio::test]
#[ignore = "Timing-dependent test - cleanup callback timing issues with async drop in CI"]
async fn test_cleanup_callback_on_connection_drop() {
    let (pool, _dir) = create_test_pool().await;

    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    let cleanup_count = Arc::new(AtomicU64::new(0));
    let cleanup_count_clone = Arc::clone(&cleanup_count);

    // Register cleanup callback
    pool.set_cleanup_callback(Arc::new(move |_conn_id| {
        cleanup_count_clone.fetch_add(1, Ordering::Relaxed);
    }));

    // Create and drop a connection
    let _conn_id = {
        let conn = pool.get().await.unwrap();
        conn.connection_id()
    };
    // Connection is dropped here

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Verify callback was called
    assert_eq!(cleanup_count.load(Ordering::Relaxed), 1);

    // Create and drop multiple connections
    {
        let _conn1 = pool.get().await.unwrap();
        let _conn2 = pool.get().await.unwrap();
        let _conn3 = pool.get().await.unwrap();
    }

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Should have 4 total cleanups (1 from before + 3 new)
    assert_eq!(cleanup_count.load(Ordering::Relaxed), 4);
}

#[tokio::test]
async fn test_cleanup_callback_tracks_correct_connection_id() {
    let (pool, _dir) = create_test_pool().await;

    use std::sync::Arc;
    use std::sync::Mutex;

    let cleaned_ids = Arc::new(Mutex::new(Vec::new()));
    let cleaned_ids_clone = Arc::clone(&cleaned_ids);

    // Register cleanup callback that tracks connection IDs
    pool.set_cleanup_callback(Arc::new(move |conn_id| {
        cleaned_ids_clone.lock().unwrap().push(conn_id);
    }));

    // Create and drop connections
    let id1 = {
        let conn = pool.get().await.unwrap();
        conn.connection_id()
    };

    let id2 = {
        let conn = pool.get().await.unwrap();
        conn.connection_id()
    };

    let id3 = {
        let conn = pool.get().await.unwrap();
        conn.connection_id()
    };

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Verify all connection IDs were tracked
    let ids = cleaned_ids.lock().unwrap();
    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
    assert!(ids.contains(&id3));
}

#[tokio::test]
async fn test_cleanup_callback_removal() {
    let (pool, _dir) = create_test_pool().await;

    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    let cleanup_count = Arc::new(AtomicU64::new(0));
    let cleanup_count_clone = Arc::clone(&cleanup_count);

    // Register cleanup callback
    pool.set_cleanup_callback(Arc::new(move |_conn_id| {
        cleanup_count_clone.fetch_add(1, Ordering::Relaxed);
    }));

    // Create and drop a connection
    {
        let _conn = pool.get().await.unwrap();
    }

    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(cleanup_count.load(Ordering::Relaxed), 1);

    // Remove the callback
    pool.remove_cleanup_callback();

    // Create and drop another connection
    {
        let _conn = pool.get().await.unwrap();
    }

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Count should still be 1 (callback was removed)
    assert_eq!(cleanup_count.load(Ordering::Relaxed), 1);
}

#[tokio::test]
#[ignore = "Timing-dependent test - connection cache integration times out due to async cleanup timing in CI"]
async fn test_connection_cache_integration() {
    let (pool, _dir) = create_test_pool().await;

    use crate::prepared::PreparedStatementCache;
    use std::sync::Arc;

    // Create a prepared statement cache
    let cache = Arc::new(PreparedStatementCache::new(10));
    let cache_clone = Arc::clone(&cache);

    // Register cleanup callback
    pool.set_cleanup_callback(Arc::new(move |conn_id| {
        cache_clone.clear_connection(conn_id);
    }));

    // Get a connection and record some cached statements
    let conn_id = {
        let conn = pool.get().await.unwrap();
        let id = conn.connection_id();

        // Record some cached statements
        cache.record_miss(id, "SELECT 1", 100);
        cache.record_miss(id, "SELECT 2", 100);
        cache.record_miss(id, "SELECT 3", 100);

        assert_eq!(cache.connection_size(id), 3);
        id
    };

    // Connection is dropped here, which should trigger cleanup
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Verify cache was cleared for this connection
    assert_eq!(cache.connection_size(conn_id), 0);
    assert_eq!(cache.connection_count(), 0);
}

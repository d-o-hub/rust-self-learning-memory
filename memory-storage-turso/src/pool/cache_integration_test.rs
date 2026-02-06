//! Integration tests for prepared statement cache with connection lifecycle
//!
//! This module tests the integration between the adaptive connection pool's
//! cleanup callback mechanism and the prepared statement cache.

use memory_storage_turso::pool::{AdaptiveConnectionPool, AdaptivePoolConfig, ConnectionId};
use memory_storage_turso::prepared::{PreparedCacheConfig, PreparedStatementCache};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

/// Create a test pool and database
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
async fn test_cache_cleanup_on_connection_return() {
    let (pool, _dir) = create_test_pool().await;

    // Create a prepared statement cache
    let cache = Arc::new(PreparedStatementCache::new(10));
    let cache_clone = Arc::clone(&cache);

    // Register cleanup callback
    pool.set_cleanup_callback(Arc::new(move |conn_id: ConnectionId| {
        let cleared = cache_clone.clear_connection(conn_id);
        tracing::info!("Cleared {} statements for connection {}", cleared, conn_id);
    }));

    // Simulate connection usage
    {
        let conn = pool.get().await.unwrap();
        let conn_id = conn.connection_id();

        // Record some cached statements
        for i in 0..5 {
            let sql = format!("SELECT {}", i);
            cache.record_miss(conn_id, &sql, 100);
        }

        assert_eq!(cache.connection_size(conn_id), 5);
        assert_eq!(cache.connection_count(), 1);
    } // Connection dropped here, cleanup should be triggered

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify cache was cleared
    assert_eq!(cache.total_size(), 0);
    assert_eq!(cache.connection_count(), 0);
}

#[tokio::test]
async fn test_cache_tracks_multiple_connections() {
    let (pool, _dir) = create_test_pool().await;

    // Create a prepared statement cache
    let cache = Arc::new(PreparedStatementCache::new(10));
    let cache_clone = Arc::clone(&cache);

    // Register cleanup callback
    pool.set_cleanup_callback(Arc::new(move |conn_id: ConnectionId| {
        cache_clone.clear_connection(conn_id);
    }));

    let mut conn_ids = Vec::new();

    // Create multiple connections
    {
        let conn1 = pool.get().await.unwrap();
        let id1 = conn1.connection_id();
        conn_ids.push(id1);
        cache.record_miss(id1, "SELECT 1", 100);
        cache.record_miss(id1, "SELECT 2", 100);

        let conn2 = pool.get().await.unwrap();
        let id2 = conn2.connection_id();
        conn_ids.push(id2);
        cache.record_miss(id2, "SELECT 3", 100);
        cache.record_miss(id2, "SELECT 4", 100);

        assert_eq!(cache.connection_count(), 2);
        assert_eq!(cache.connection_size(id1), 2);
        assert_eq!(cache.connection_size(id2), 2);
    }

    tokio::time::sleep(Duration::from_millis(100)).await;

    // All connections cleaned up
    assert_eq!(cache.connection_count(), 0);
    assert_eq!(cache.total_size(), 0);
}

#[tokio::test]
async fn test_cache_statistics_with_cleanup() {
    let (pool, _dir) = create_test_pool().await;

    // Create a prepared statement cache
    let cache = Arc::new(PreparedStatementCache::new(10));
    let cache_clone = Arc::clone(&cache);

    // Register cleanup callback
    pool.set_cleanup_callback(Arc::new(move |conn_id: ConnectionId| {
        cache_clone.clear_connection(conn_id);
    }));

    // Simulate connection usage with cache hits and misses
    {
        let conn = pool.get().await.unwrap();
        let conn_id = conn.connection_id();

        // Record some cache activity
        cache.record_miss(conn_id, "SELECT 1", 100);
        cache.record_miss(conn_id, "SELECT 2", 100);
        cache.record_hit(conn_id, "SELECT 1");
        cache.record_hit(conn_id, "SELECT 2");
    }

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Check statistics
    let stats = cache.stats();
    assert_eq!(stats.hits, 2);
    assert_eq!(stats.misses, 2);
    assert_eq!(stats.active_connections, 0); // Cleaned up
}

#[tokio::test]
async fn test_no_callback_registered() {
    let (pool, _dir) = create_test_pool().await;

    // Create a cache but don't register a callback
    let cache = PreparedStatementCache::new(10);

    // Use a connection
    let conn_id = {
        let conn = pool.get().await.unwrap();
        let id = conn.connection_id();
        cache.record_miss(id, "SELECT 1", 100);
        id
    };

    // Connection dropped, but no cleanup callback registered
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Cache should still have data (no cleanup)
    assert_eq!(cache.connection_size(conn_id), 1);
}

#[tokio::test]
async fn test_callback_removal_during_runtime() {
    let (pool, _dir) = create_test_pool().await;

    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    let cleanup_count = Arc::new(AtomicU64::new(0));
    let cleanup_count_clone = Arc::clone(&cleanup_count);

    // Register cleanup callback
    pool.set_cleanup_callback(Arc::new(move |conn_id| {
        cleanup_count_clone.fetch_add(1, Ordering::Relaxed);
    }));

    // Use a connection
    {
        let _conn = pool.get().await.unwrap();
    }

    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(cleanup_count.load(Ordering::Relaxed), 1);

    // Remove callback
    pool.remove_cleanup_callback();

    // Use another connection
    {
        let _conn = pool.get().await.unwrap();
    }

    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(cleanup_count.load(Ordering::Relaxed), 1); // Still 1, callback was removed
}

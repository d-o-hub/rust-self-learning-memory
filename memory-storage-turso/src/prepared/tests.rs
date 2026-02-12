//! Integration tests for connection-aware prepared statement cache
//!
//! These tests validate that the prepared statement cache works correctly
//! with connection lifecycle management and provides accurate statistics.

use crate::prepared::{PreparedCacheConfig, PreparedStatementCache};
use std::time::Duration;
use tempfile::TempDir;

/// Helper to create a test database and cache
async fn create_test_cache() -> (PreparedStatementCache, libsql::Database, TempDir) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");

    let db = libsql::Builder::new_local(&db_path).build().await.unwrap();

    let config = PreparedCacheConfig {
        max_size: 100,
        max_connections: 10,
        enable_refresh: true,
        refresh_threshold: 1000,
    };

    let cache = PreparedStatementCache::with_config(config);

    (cache, db, dir)
}

/// Helper to create just a cache without database (for cache-only tests)
fn create_test_cache_only() -> PreparedStatementCache {
    let config = PreparedCacheConfig {
        max_size: 100,
        max_connections: 10,
        enable_refresh: true,
        refresh_threshold: 1000,
    };

    PreparedStatementCache::with_config(config)
}

#[tokio::test]
async fn test_cache_stores_metadata() {
    let cache = create_test_cache_only();
    let conn_id = cache.get_connection_id();
    let sql = "SELECT * FROM episodes WHERE id = ?";

    // Record a miss (simulating statement preparation)
    cache.record_miss(conn_id, sql, 150);

    // Verify the cache knows about this statement
    assert!(cache.is_cached(conn_id, sql));

    // Verify connection size
    assert_eq!(cache.connection_size(conn_id), 1);

    // Verify stats
    let stats = cache.stats();
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.prepared, 1);
    assert_eq!(stats.active_connections, 1);
}

#[tokio::test]
async fn test_cache_tracks_hits_and_misses() {
    let cache = create_test_cache_only();
    let conn_id = cache.get_connection_id();
    let sql = "SELECT * FROM episodes WHERE id = ?";

    // First access - miss
    cache.record_miss(conn_id, sql, 100);

    // Subsequent accesses - hits
    cache.record_hit(conn_id, sql);
    cache.record_hit(conn_id, sql);
    cache.record_hit(conn_id, sql);

    let stats = cache.stats();
    assert_eq!(stats.hits, 3);
    assert_eq!(stats.misses, 1);

    let hit_rate = stats.hit_rate();
    assert!((hit_rate - 0.75).abs() < 0.01, "Hit rate should be ~75%");
}

#[tokio::test]
async fn test_cache_per_connection_isolation() {
    let cache = create_test_cache_only();

    let conn_id1 = cache.get_connection_id();
    let conn_id2 = cache.get_connection_id();

    let sql = "SELECT * FROM episodes WHERE id = ?";

    // Record on connection 1
    cache.record_miss(conn_id1, sql, 100);

    // Connection 1 should have the statement
    assert!(cache.is_cached(conn_id1, sql));
    assert_eq!(cache.connection_size(conn_id1), 1);

    // Connection 2 should NOT have the statement
    assert!(!cache.is_cached(conn_id2, sql));
    assert_eq!(cache.connection_size(conn_id2), 0);

    // Record on connection 2
    cache.record_miss(conn_id2, sql, 100);

    // Now both should have it
    assert!(cache.is_cached(conn_id1, sql));
    assert!(cache.is_cached(conn_id2, sql));

    // Verify connection count
    assert_eq!(cache.connection_count(), 2);
}

#[tokio::test]
async fn test_cache_lru_eviction() {
    let _cache = create_test_cache_only();

    let config = PreparedCacheConfig {
        max_size: 3,
        ..Default::default()
    };
    let small_cache = PreparedStatementCache::with_config(config);
    let conn_id = small_cache.get_connection_id();

    // Add 4 statements to a cache with max size 3
    for i in 0..4 {
        let sql = format!("SELECT * FROM table_{} WHERE id = ?", i);
        small_cache.record_miss(conn_id, &sql, 100);
    }

    // Should have evicted one statement
    let stats = small_cache.stats();
    assert_eq!(stats.evictions, 1);

    // Cache size should be at max
    assert_eq!(small_cache.connection_size(conn_id), 3);
}

#[tokio::test]
async fn test_connection_cleanup() {
    let cache = create_test_cache_only();
    let conn_id = cache.get_connection_id();

    // Add some statements
    for i in 0..5 {
        let sql = format!("SELECT * FROM episodes WHERE id = {}", i);
        cache.record_miss(conn_id, &sql, 100);
    }

    assert_eq!(cache.connection_size(conn_id), 5);
    assert_eq!(cache.connection_count(), 1);

    // Clear the connection
    let cleared = cache.clear_connection(conn_id);

    assert_eq!(cleared, 5);
    assert_eq!(cache.connection_size(conn_id), 0);
    assert_eq!(cache.connection_count(), 0);

    // Stats should reflect this
    let stats = cache.stats();
    assert_eq!(stats.active_connections, 0);
}

#[tokio::test]
async fn test_cache_statistics_tracking() {
    let cache = create_test_cache_only();
    let conn_id = cache.get_connection_id();

    // Record some operations
    for i in 0..10 {
        let sql = format!("SELECT * FROM episodes WHERE id = {}", i);
        cache.record_miss(conn_id, &sql, 100 + i * 10);

        if i % 2 == 0 {
            cache.record_hit(conn_id, &sql);
        }
    }

    let stats = cache.stats();
    assert_eq!(stats.misses, 10);
    assert_eq!(stats.prepared, 10);
    assert_eq!(stats.hits, 5);

    // Check average preparation time
    assert!(stats.avg_preparation_time_us > 0.0);

    // Check total preparation time
    assert_eq!(stats.preparation_time_us, 1450); // Sum of 100 + 110 + ... + 190
}

#[tokio::test]
async fn test_cache_cleanup_idle_connections() {
    let cache = create_test_cache_only();

    // Create multiple connections
    let conn_ids: Vec<_> = (0..5).map(|_| cache.get_connection_id()).collect();

    for &conn_id in &conn_ids {
        cache.record_miss(conn_id, "SELECT 1", 100);
    }

    assert_eq!(cache.connection_count(), 5);

    // Cleanup with zero duration should remove all
    let cleaned = cache.cleanup_idle_connections(Duration::from_secs(0));
    assert_eq!(cleaned, 5);
    assert_eq!(cache.connection_count(), 0);

    let stats = cache.stats();
    assert_eq!(stats.active_connections, 0);
    assert_eq!(stats.connection_evictions, 5);
}

#[tokio::test]
async fn test_cache_statement_removal() {
    let cache = create_test_cache_only();
    let conn_id = cache.get_connection_id();

    let sql = "SELECT * FROM episodes WHERE id = ?";
    cache.record_miss(conn_id, sql, 100);

    assert!(cache.is_cached(conn_id, sql));

    // Remove the specific statement
    let removed = cache.remove(conn_id, sql);
    assert!(removed);

    // Should no longer be cached
    assert!(!cache.is_cached(conn_id, sql));

    // Removing again should return false
    let removed_again = cache.remove(conn_id, sql);
    assert!(!removed_again);
}

#[tokio::test]
async fn test_cache_clear_all() {
    let cache = create_test_cache_only();

    // Create multiple connections with statements
    for _ in 0..3 {
        let conn_id = cache.get_connection_id();
        for i in 0..5 {
            let sql = format!("SELECT * FROM table_{} WHERE id = ?", i);
            cache.record_miss(conn_id, &sql, 100);
        }
    }

    assert_eq!(cache.connection_count(), 3);
    assert_eq!(cache.total_size(), 15);

    // Clear all
    cache.clear();

    assert!(cache.is_empty());
    assert_eq!(cache.connection_count(), 0);
    assert_eq!(cache.total_size(), 0);
}

#[tokio::test]
async fn test_cache_concurrent_access() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let (cache, _db, _dir): (PreparedStatementCache, _, _) = create_test_cache().await;
        let cache = std::sync::Arc::new(cache);

        let handles: Vec<_> = (0..10)
            .map(|_i| {
                let cache = std::sync::Arc::clone(&cache);
                tokio::spawn(async move {
                    let conn_id = cache.get_connection_id();
                    for j in 0..100 {
                        let sql = format!("SELECT * FROM episodes WHERE id = {}", j);
                        cache.record_miss(conn_id, &sql, 100);
                        cache.is_cached(conn_id, &sql);
                        cache.record_hit(conn_id, &sql);
                    }
                })
            })
            .collect();

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        let stats = cache.stats();
        // Should have 10 connections * 100 statements each
        assert_eq!(stats.active_connections, 10);
        assert_eq!(stats.current_size, 1000);
        assert_eq!(stats.misses, 1000);
        assert_eq!(stats.hits, 1000);
    })
    .await
    .expect("Test timed out after 10 seconds");
}

#[tokio::test]
async fn test_cache_with_actual_db_queries() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let (cache, db, _dir) = create_test_cache().await;

        let conn = db.connect().unwrap();
        let conn_id = cache.get_connection_id();

        // Create a table
        conn.execute(
            "CREATE TABLE test_cache (id INTEGER PRIMARY KEY, value TEXT)",
            (),
        )
        .await
        .unwrap();

        // Prepare and execute a statement
        let sql = "INSERT INTO test_cache (id, value) VALUES (?, ?)";

        // Record cache miss
        cache.record_miss(conn_id, sql, 150);

        // Actually prepare and execute
        let stmt = conn.prepare(sql).await.unwrap();
        stmt.execute((1, "test")).await.unwrap();

        // Verify cache knows about this statement
        assert!(cache.is_cached(conn_id, sql));

        // Record a hit for subsequent use
        cache.record_hit(conn_id, sql);

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    })
    .await
    .expect("Test timed out after 10 seconds");
}

#[tokio::test]
async fn test_cache_hit_rate_calculation() {
    let cache = create_test_cache_only();
    let conn_id = cache.get_connection_id();

    // Simulate repeated query pattern
    let queries = vec![
        "SELECT * FROM episodes WHERE id = ?",
        "SELECT * FROM patterns WHERE success_rate > ?",
        "SELECT COUNT(*) FROM episodes",
    ];

    for _ in 0..10 {
        for query in &queries {
            if !cache.is_cached(conn_id, query) {
                cache.record_miss(conn_id, query, 100);
            }
            cache.record_hit(conn_id, query);
        }
    }

    let stats = cache.stats();
    let hit_rate = stats.hit_rate();

    // Should have high hit rate (30 hits out of 33 total operations)
    assert!(hit_rate > 0.8, "Hit rate should be > 80%");
}

#[tokio::test]
async fn test_cache_max_connections_enforcement() {
    let config = PreparedCacheConfig {
        max_size: 100,
        max_connections: 3,
        ..Default::default()
    };
    let cache = PreparedStatementCache::with_config(config);

    // Create 5 connections (should evict 2)
    for _i in 0..5 {
        let conn_id = cache.get_connection_id();
        cache.record_miss(conn_id, "SELECT 1", 100);
    }

    let stats = cache.stats();
    // Should have evicted at least one connection
    assert!(stats.connection_evictions >= 1);

    // Connection count should be at or below max
    assert!(cache.connection_count() <= 3);
}

#[tokio::test]
async fn test_cache_size_tracking() {
    let cache = create_test_cache_only();

    let conn_id = cache.get_connection_id();

    // Add statements
    for i in 0..10 {
        let sql = format!("SELECT * FROM table_{} WHERE id = ?", i);
        cache.record_miss(conn_id, &sql, 100);
    }

    assert_eq!(cache.connection_size(conn_id), 10);
    assert_eq!(cache.total_size(), 10);

    let stats = cache.stats();
    assert_eq!(stats.current_size, 10);
    assert!(stats.max_size_reached >= 10);
}

#[tokio::test]
async fn test_cache_use_count_tracking() {
    let cache = create_test_cache_only();
    let conn_id = cache.get_connection_id();

    let sql = "SELECT * FROM episodes WHERE id = ?";
    cache.record_miss(conn_id, sql, 100);

    // Use the statement multiple times
    for _ in 0..10 {
        cache.record_hit(conn_id, sql);
    }

    // We can't directly access the use count, but we can verify
    // the statement is still cached and working
    assert!(cache.is_cached(conn_id, sql));

    let stats = cache.stats();
    assert_eq!(stats.hits, 10);
}

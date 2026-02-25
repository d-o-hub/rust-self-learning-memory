use super::*;

#[test]
fn test_cache_basic_operations() {
    let cache = PreparedStatementCache::new(10);
    let conn_id = cache.get_connection_id();

    // Record a miss (preparing a statement)
    cache.record_miss(conn_id, "SELECT 1", 100);

    // Record a hit
    cache.record_hit(conn_id, "SELECT 1");

    let stats = cache.stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.prepared, 1);
    assert_eq!(stats.active_connections, 1);
}

#[test]
fn test_cache_per_connection() {
    let cache = PreparedStatementCache::new(10);
    let conn_id1 = cache.get_connection_id();
    let conn_id2 = cache.get_connection_id();

    let sql = "SELECT 1";

    // Record on connection 1
    cache.record_miss(conn_id1, sql, 100);

    // Record on connection 2
    cache.record_miss(conn_id2, sql, 100);

    // Should have 2 separate caches
    assert_eq!(cache.connection_count(), 2);
    assert_eq!(cache.connection_size(conn_id1), 1);
    assert_eq!(cache.connection_size(conn_id2), 1);

    let stats = cache.stats();
    assert_eq!(stats.active_connections, 2);
}

#[test]
fn test_cache_eviction() {
    let cache = PreparedStatementCache::new(3);
    let conn_id = cache.get_connection_id();

    // Record 4 statements (will trigger eviction)
    for i in 0..4 {
        let sql = format!("SELECT {}", i);
        cache.record_miss(conn_id, &sql, 100);
    }

    let stats = cache.stats();
    assert_eq!(stats.evictions, 1);
    assert_eq!(cache.connection_size(conn_id), 3);
}

#[test]
fn test_clear_connection() {
    let cache = PreparedStatementCache::new(10);
    let conn_id = cache.get_connection_id();

    // Record some statements
    for i in 0..5 {
        let sql = format!("SELECT {}", i);
        cache.record_miss(conn_id, &sql, 100);
    }

    assert_eq!(cache.connection_size(conn_id), 5);

    // Clear connection cache
    let cleared = cache.clear_connection(conn_id);
    assert_eq!(cleared, 5);
    assert_eq!(cache.connection_size(conn_id), 0);

    let stats = cache.stats();
    assert_eq!(stats.active_connections, 0);
}

#[test]
fn test_cache_clear() {
    let cache = PreparedStatementCache::new(10);
    let conn_id1 = cache.get_connection_id();
    let conn_id2 = cache.get_connection_id();

    // Record some statements on both connections
    for i in 0..5 {
        let sql = format!("SELECT {}", i);
        cache.record_miss(conn_id1, &sql, 100);
        cache.record_miss(conn_id2, &sql, 100);
    }

    assert_eq!(cache.total_size(), 10);

    // Clear all caches
    cache.clear();
    assert!(cache.is_empty());

    let stats = cache.stats();
    assert_eq!(stats.current_size, 0);
    assert_eq!(stats.active_connections, 0);
}

#[test]
fn test_connection_eviction() {
    let config = PreparedCacheConfig {
        max_size: 10,
        max_connections: 2,
        ..Default::default()
    };
    let cache = PreparedStatementCache::with_config(config);

    // Create 3 connections (will trigger eviction of 1)
    let conn_id1 = cache.get_connection_id();
    let conn_id2 = cache.get_connection_id();
    let conn_id3 = cache.get_connection_id();

    cache.record_miss(conn_id1, "SELECT 1", 100);
    cache.record_miss(conn_id2, "SELECT 2", 100);
    cache.record_miss(conn_id3, "SELECT 3", 100);

    // Should have evicted one connection
    assert_eq!(cache.connection_count(), 2);

    let stats = cache.stats();
    assert_eq!(stats.connection_evictions, 1);
}

#[test]
fn test_cleanup_idle_connections() {
    let cache = PreparedStatementCache::new(10);
    let conn_id = cache.get_connection_id();

    cache.record_miss(conn_id, "SELECT 1", 100);

    assert_eq!(cache.connection_count(), 1);

    // Cleanup with zero duration should remove the connection
    let cleaned = cache.cleanup_idle_connections(Duration::from_secs(0));
    assert_eq!(cleaned, 1);
    assert_eq!(cache.connection_count(), 0);
}

#[test]
fn test_cache_hit_rate() {
    let cache = PreparedStatementCache::new(10);
    let conn_id = cache.get_connection_id();

    let sql = "SELECT 1";

    // First call - miss
    cache.record_miss(conn_id, sql, 100);

    // Second call - hit
    cache.record_hit(conn_id, sql);

    // Third call - hit
    cache.record_hit(conn_id, sql);

    let stats = cache.stats();
    assert_eq!(stats.hits, 2);
    assert_eq!(stats.misses, 1);
    assert!((stats.hit_rate() - 0.666).abs() < 0.01);
}

#[test]
fn test_remove_statement() {
    let cache = PreparedStatementCache::new(10);
    let conn_id = cache.get_connection_id();

    let sql = "SELECT 1";
    cache.record_miss(conn_id, sql, 100);

    assert!(!cache.is_empty());

    // Remove the statement
    assert!(cache.remove(conn_id, sql));
    assert!(cache.is_empty());

    // Try to remove again (should return false)
    assert!(!cache.remove(conn_id, sql));
}

#[test]
fn test_is_cached() {
    let cache = PreparedStatementCache::new(10);
    let conn_id = cache.get_connection_id();

    let sql = "SELECT 1";

    // Not cached initially
    assert!(!cache.is_cached(conn_id, sql));

    // Record miss (caches the statement)
    cache.record_miss(conn_id, sql, 100);

    // Now it should be cached
    assert!(cache.is_cached(conn_id, sql));
}

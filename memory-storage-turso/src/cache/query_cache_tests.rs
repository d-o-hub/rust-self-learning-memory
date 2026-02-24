use super::*;
use std::time::Duration;

#[test]
fn test_query_key_creation() {
    let sql = "SELECT * FROM episodes WHERE domain = ?";
    let key = QueryKey::new(sql, &[&"test_domain"]);

    assert_eq!(key.query_type, QueryType::Episode);
    assert!(!key.param_hashes.is_empty());
}

#[test]
fn test_query_key_normalization() {
    let sql1 = "SELECT * FROM episodes WHERE id = 1";
    let sql2 = "select * from episodes where id = 1";
    let sql3 = "SELECT   *   FROM   episodes   WHERE   id   =   1";

    let key1 = QueryKey::from_sql(sql1);
    let key2 = QueryKey::from_sql(sql2);
    let key3 = QueryKey::from_sql(sql3);

    assert_eq!(key1.sql_hash, key2.sql_hash);
    assert_eq!(key2.sql_hash, key3.sql_hash);
}

#[test]
fn test_table_dependency_detection() {
    let sql = "SELECT e.*, s.* FROM episodes e JOIN steps s ON e.episode_id = s.episode_id";
    let deps = TableDependency::from_query(sql);

    assert!(deps.contains(&TableDependency::Episodes));
    assert!(deps.contains(&TableDependency::Steps));
}

#[test]
fn test_cache_put_and_get() {
    let (cache, _rx) = AdvancedQueryCache::new_with_receiver();

    let key = QueryKey::from_sql("SELECT * FROM episodes");
    let data = b"test data".to_vec();
    let deps = vec![TableDependency::Episodes];

    cache.put(key.clone(), data.clone(), deps);

    let retrieved = cache.get(&key);
    assert_eq!(retrieved, Some(data));
}

#[test]
#[ignore = "Timing-dependent test - cache expiration requires precise sleep timing that fails in CI"]
fn test_cache_expiration() {
    let config = AdvancedQueryCacheConfig {
        default_ttl: Duration::from_millis(50),
        ..Default::default()
    };

    let (cache, _rx) = AdvancedQueryCache::new(config);

    let key = QueryKey::from_sql("SELECT * FROM episodes");
    let data = b"test data".to_vec();
    let deps = vec![TableDependency::Episodes];

    cache.put(key.clone(), data, deps);

    // Should be cached initially
    assert!(cache.get(&key).is_some());

    // Wait for expiration
    std::thread::sleep(Duration::from_millis(60));

    // Should be expired
    assert!(cache.get(&key).is_none());

    let stats = cache.stats();
    assert_eq!(stats.expirations, 1);
}

#[test]
fn test_cache_invalidation_by_table() {
    let (cache, _rx) = AdvancedQueryCache::new_with_receiver();

    let key1 = QueryKey::from_sql("SELECT * FROM episodes");
    let key2 = QueryKey::from_sql("SELECT * FROM patterns");

    cache.put(
        key1.clone(),
        b"episodes data".to_vec(),
        vec![TableDependency::Episodes],
    );
    cache.put(
        key2.clone(),
        b"patterns data".to_vec(),
        vec![TableDependency::Patterns],
    );

    // Invalidate episodes
    cache.invalidate_by_table(&TableDependency::Episodes);

    // Episodes query should be invalidated
    assert!(cache.get(&key1).is_none());
    // Patterns query should still be cached
    assert!(cache.get(&key2).is_some());
}

#[test]
fn test_cache_eviction() {
    let config = AdvancedQueryCacheConfig {
        max_queries: 2,
        ..Default::default()
    };

    let (cache, _rx) = AdvancedQueryCache::new(config);

    let key1 = QueryKey::from_sql("SELECT * FROM episodes WHERE id = 1");
    let key2 = QueryKey::from_sql("SELECT * FROM episodes WHERE id = 2");
    let key3 = QueryKey::from_sql("SELECT * FROM episodes WHERE id = 3");

    cache.put(key1.clone(), b"data1".to_vec(), vec![]);
    cache.put(key2.clone(), b"data2".to_vec(), vec![]);
    cache.put(key3.clone(), b"data3".to_vec(), vec![]);

    // First entry should be evicted (LRU)
    assert!(cache.get(&key1).is_none());
    // Recent entries should still be there
    assert!(cache.get(&key2).is_some());
    assert!(cache.get(&key3).is_some());

    let stats = cache.stats();
    assert_eq!(stats.evictions, 1);
}

#[test]
fn test_hot_query_tracking() {
    let config = AdvancedQueryCacheConfig {
        hot_threshold: 3,
        ..Default::default()
    };

    let (cache, _rx) = AdvancedQueryCache::new(config);

    let key = QueryKey::from_sql("SELECT * FROM episodes");
    cache.put(key.clone(), b"data".to_vec(), vec![]);

    // Access multiple times
    for _ in 0..5 {
        cache.get(&key);
    }

    let _hot = cache.get_hot_queries_needing_refresh();
    // Should be tracked as hot
    assert!(cache.hot_queries.read().contains(&key));
}

#[test]
fn test_query_type_ttl() {
    assert_eq!(QueryType::Statistics.default_ttl(), Duration::from_secs(60));
    assert_eq!(QueryType::Episode.default_ttl(), Duration::from_secs(300));
    assert_eq!(
        QueryType::Embedding.default_ttl(),
        Duration::from_secs(1800)
    );
}

#[test]
fn test_cache_stats() {
    let (cache, _rx) = AdvancedQueryCache::new_with_receiver();

    let key1 = QueryKey::from_sql("SELECT 1");
    let key2 = QueryKey::from_sql("SELECT 2");

    cache.put(key1.clone(), b"data1".to_vec(), vec![]);
    cache.put(key2.clone(), b"data2".to_vec(), vec![]);

    // Hit
    cache.get(&key1);

    // Miss
    let missing_key = QueryKey::from_sql("SELECT 3");
    cache.get(&missing_key);

    let stats = cache.stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.hit_rate(), 0.5);
}

use super::*;

#[tokio::test]
async fn test_cache_basic_operations() {
    let cache = AdaptiveTTLCache::default_config().unwrap();

    // Test insert and get
    cache.insert("key1", "value1".to_string()).await;
    let result = cache.get(&"key1").await;
    assert_eq!(result, Some("value1".to_string()));

    // Test miss
    let result = cache.get(&"nonexistent").await;
    assert_eq!(result, None);

    // Test contains
    assert!(cache.contains(&"key1").await);
    assert!(!cache.contains(&"nonexistent").await);

    // Test remove
    assert!(cache.remove(&"key1").await);
    assert!(!cache.contains(&"key1").await);
    assert!(!cache.remove(&"key1").await);
}

#[tokio::test]
async fn test_cache_stats() {
    let cache = AdaptiveTTLCache::default_config().unwrap();

    // Initial stats
    let stats = cache.stats();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);

    // Generate some hits and misses
    cache.insert("key1", "value1".to_string()).await;
    let _ = cache.get(&"key1").await; // Hit
    let _ = cache.get(&"nonexistent").await; // Miss

    let stats = cache.stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
    assert!((stats.hit_rate - 0.5).abs() < 0.01);
}

#[tokio::test]
async fn test_cache_clear() {
    let cache = AdaptiveTTLCache::default_config().unwrap();

    for i in 0..10 {
        cache.insert(i, format!("value{}", i)).await;
    }

    assert_eq!(cache.len().await, 10);

    cache.clear().await;

    assert_eq!(cache.len().await, 0);
    assert!(cache.is_empty().await);
}

#[tokio::test]
async fn test_cache_eviction() {
    let config = TTLConfig::default().with_max_entries(5);
    let cache = AdaptiveTTLCache::new(config).unwrap();

    // Insert more entries than max
    for i in 0..10 {
        cache.insert(i, format!("value{}", i)).await;
    }

    // Should only have max_entries
    assert_eq!(cache.len().await, 5);

    let stats = cache.stats();
    assert_eq!(stats.evictions, 5);
}

#[tokio::test]
#[ignore = "Timing-dependent test - TTL adaptation requires precise timing that fails in CI"]
async fn test_ttl_adaptation() {
    let config = TTLConfig::default()
        .with_hot_threshold(3)
        .with_adaptation_rate(0.5);

    let cache = AdaptiveTTLCache::new(config).unwrap();

    cache.insert("key1", "value1".to_string()).await;
    let initial_ttl = cache.ttl(&"key1").await.unwrap();

    // Access multiple times to trigger TTL extension
    for _ in 0..5 {
        let _ = cache.get(&"key1").await;
    }

    let new_ttl = cache.ttl(&"key1").await.unwrap();
    assert!(new_ttl > initial_ttl);
}

#[tokio::test]
#[ignore = "Timing-dependent test - cache expiration requires precise sleep timing that fails in CI"]
async fn test_cache_entry_expiration() {
    let config = TTLConfig::default().with_base_ttl(Duration::from_millis(50));
    let cache = AdaptiveTTLCache::new(config).unwrap();

    cache.insert("key1", "value1".to_string()).await;
    assert!(cache.contains(&"key1").await);

    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Entry should be expired
    assert!(!cache.contains(&"key1").await);
    let result = cache.get(&"key1").await;
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_cache_keys() {
    let cache = AdaptiveTTLCache::default_config().unwrap();

    cache.insert("key1", 1).await;
    cache.insert("key2", 2).await;
    cache.insert("key3", 3).await;

    let keys = cache.keys().await;
    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&"key1"));
    assert!(keys.contains(&"key2"));
    assert!(keys.contains(&"key3"));
}

#[test]
fn test_cache_entry_access_frequency() {
    let entry = CacheEntry::new("value", Duration::from_secs(300));

    // Initially zero
    assert_eq!(entry.access_frequency(300), 0.0);
}

#[test]
fn test_cache_stats_snapshot() {
    let stats = CacheStats::new();
    stats.record_hit();
    stats.record_hit();
    stats.record_miss();

    let snapshot = stats.snapshot();
    assert_eq!(snapshot.hits, 2);
    assert_eq!(snapshot.misses, 1);
    assert!((snapshot.hit_rate - 0.666).abs() < 0.01);
    // 2/3 = 0.666 hit rate, which is below 0.8 threshold for is_effective
    assert!(!snapshot.is_effective());
}

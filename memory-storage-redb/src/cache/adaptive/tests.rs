//! Tests for adaptive cache implementation

use super::{AdaptiveCache, AdaptiveCacheConfig};
use tokio::time::sleep as async_sleep;
use tokio::time::Duration as TokioDuration;
use uuid::Uuid;

fn create_test_config() -> AdaptiveCacheConfig {
    AdaptiveCacheConfig {
        max_size: 100,
        default_ttl: Duration::from_secs(60),
        min_ttl: Duration::from_secs(10),
        max_ttl: Duration::from_secs(300),
        hot_threshold: 5,
        cold_threshold: 1,
        adaptation_rate: 0.25,
        window_size: 10,
        cleanup_interval_secs: 1,
        enable_background_cleanup: false,
    }
}

use std::time::Duration;

#[tokio::test]
async fn test_adaptive_cache_creation() {
    let cache = AdaptiveCache::<String>::new(create_test_config());
    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.base.item_count, 0);
}

#[tokio::test]
async fn test_adaptive_cache_hit_and_miss() {
    let cache = AdaptiveCache::<String>::new(create_test_config());
    let id = Uuid::new_v4();

    // First access: miss
    let hit = cache
        .record_access(id, false, Some("value".to_string()))
        .await;
    assert!(!hit);

    // Second access: hit
    let hit = cache.record_access(id, true, None).await;
    assert!(hit);

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.base.hits, 1);
    assert_eq!(metrics.base.misses, 1);
    assert_eq!(metrics.base.hit_rate, 0.5);
}

#[tokio::test]
async fn test_hot_item_ttl_increase() {
    let config = create_test_config();
    let cache = AdaptiveCache::<String>::new(config);
    let id = Uuid::new_v4();

    // Create entry
    cache
        .record_access(id, false, Some("value".to_string()))
        .await;

    // Access multiple times to become "hot"
    for _ in 0..6 {
        cache.record_access(id, true, None).await;
    }

    // Check TTL was increased
    let ttl = cache.ttl(id).await;
    assert!(ttl.is_some());
    let ttl_secs = ttl.unwrap().as_secs();
    assert!(
        ttl_secs > 60,
        "TTL should increase for hot items, got {}s",
        ttl_secs
    );
}

#[tokio::test]
async fn test_cold_item_ttl_decrease() {
    let config = create_test_config();
    let cache = AdaptiveCache::<String>::new(config);
    let id = Uuid::new_v4();

    // Create entry
    cache
        .record_access(id, false, Some("value".to_string()))
        .await;

    // Access once (cold threshold is 1)
    cache.record_access(id, true, None).await;

    // Check TTL was decreased
    let ttl = cache.ttl(id).await;
    assert!(ttl.is_some());
    let ttl_secs = ttl.unwrap().as_secs();
    assert!(
        ttl_secs < 60,
        "TTL should decrease for cold items, got {}s",
        ttl_secs
    );
}

#[tokio::test]
async fn test_ttl_bounds_enforcement() {
    let config = AdaptiveCacheConfig {
        max_size: 100,
        default_ttl: Duration::from_secs(60),
        min_ttl: Duration::from_secs(10),
        max_ttl: Duration::from_secs(120),
        hot_threshold: 5,
        cold_threshold: 1,
        adaptation_rate: 0.5, // Aggressive adaptation
        window_size: 10,
        cleanup_interval_secs: 1,
        enable_background_cleanup: false,
    };
    let cache = AdaptiveCache::<String>::new(config);
    let id = Uuid::new_v4();

    // Create entry
    cache
        .record_access(id, false, Some("value".to_string()))
        .await;

    // Access many times to hit max TTL
    for _ in 0..20 {
        cache.record_access(id, true, None).await;
    }

    // TTL should be capped at max (120s)
    let ttl = cache.ttl(id).await.unwrap();
    assert!(
        ttl.as_secs() <= 120,
        "TTL should be capped at max, got {}s",
        ttl.as_secs()
    );
}

#[tokio::test]
async fn test_access_count_tracking() {
    let cache = AdaptiveCache::<String>::new(create_test_config());
    let id = Uuid::new_v4();

    // Create entry
    cache
        .record_access(id, false, Some("value".to_string()))
        .await;

    // Access multiple times
    for i in 0..5 {
        let hit = cache.record_access(id, true, None).await;
        assert!(hit, "Access {} should be a hit", i + 1);
    }

    // Check access count
    let count = cache.access_count(id).await;
    assert_eq!(count, Some(5));
}

#[tokio::test]
async fn test_hot_cold_distribution() {
    let config = create_test_config();
    let cache = AdaptiveCache::<String>::new(config);

    let hot_id = Uuid::new_v4();
    let cold_id = Uuid::new_v4();

    // Create hot item
    cache
        .record_access(hot_id, false, Some("hot".to_string()))
        .await;
    for _ in 0..6 {
        cache.record_access(hot_id, true, None).await;
    }

    // Create cold item
    cache
        .record_access(cold_id, false, Some("cold".to_string()))
        .await;
    cache.record_access(cold_id, true, None).await;

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.hot_item_count, 1);
    assert_eq!(metrics.cold_item_count, 1);
}

#[tokio::test]
async fn test_ttl_expiration() {
    let config = AdaptiveCacheConfig {
        max_size: 100,
        default_ttl: Duration::from_secs(1),
        min_ttl: Duration::from_secs(1),
        max_ttl: Duration::from_secs(1),
        hot_threshold: 10,
        cold_threshold: 1,
        adaptation_rate: 0.1,
        window_size: 10,
        cleanup_interval_secs: 1,
        enable_background_cleanup: false,
    };
    let cache = AdaptiveCache::<String>::new(config);
    let id = Uuid::new_v4();

    // Create entry
    cache
        .record_access(id, false, Some("value".to_string()))
        .await;
    assert!(cache.contains(id).await);

    // Wait for expiration
    async_sleep(TokioDuration::from_secs(3)).await;

    // Should be expired
    assert!(!cache.contains(id).await);
}

#[tokio::test]
async fn test_manual_cleanup() {
    let config = AdaptiveCacheConfig {
        max_size: 100,
        default_ttl: Duration::from_secs(1),
        min_ttl: Duration::from_secs(1),
        max_ttl: Duration::from_secs(1),
        hot_threshold: 10,
        cold_threshold: 1,
        adaptation_rate: 0.1,
        window_size: 10,
        cleanup_interval_secs: 1,
        enable_background_cleanup: false,
    };
    let cache = AdaptiveCache::<String>::new(config);

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();

    cache
        .record_access(id1, false, Some("v1".to_string()))
        .await;
    cache
        .record_access(id2, false, Some("v2".to_string()))
        .await;

    // Wait for expiration
    async_sleep(TokioDuration::from_secs(3)).await;

    // Run manual cleanup
    let cleaned = cache.cleanup_expired().await;
    assert_eq!(cleaned, 2);

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.base.item_count, 0);
}

#[tokio::test]
async fn test_remove_entry() {
    let cache = AdaptiveCache::<String>::new(create_test_config());
    let id = Uuid::new_v4();

    cache
        .record_access(id, false, Some("value".to_string()))
        .await;
    assert!(cache.contains(id).await);

    cache.remove(id).await;
    assert!(!cache.contains(id).await);
}

#[tokio::test]
async fn test_clear_all() {
    let cache = AdaptiveCache::<String>::new(create_test_config());

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();

    cache
        .record_access(id1, false, Some("v1".to_string()))
        .await;
    cache
        .record_access(id2, false, Some("v2".to_string()))
        .await;

    cache.clear().await;

    assert!(cache.is_empty().await);
    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.base.item_count, 0);
}

#[tokio::test]
async fn test_get_and_record() {
    let cache = AdaptiveCache::<String>::new(create_test_config());
    let id = Uuid::new_v4();

    // Store value
    cache
        .record_access(id, false, Some("value".to_string()))
        .await;

    // Get and record access
    let result = cache.get_and_record(id).await;
    assert_eq!(result, Some("value".to_string()));

    // Access count should be updated
    let count = cache.access_count(id).await;
    assert_eq!(count, Some(1));
}

#[tokio::test]
async fn test_nonexistent_entry() {
    let cache = AdaptiveCache::<String>::new(create_test_config());
    let id = Uuid::new_v4();

    assert!(!cache.contains(id).await);
    assert!(cache.access_count(id).await.is_none());
    assert!(cache.ttl(id).await.is_none());
    assert!(cache.get(id).await.is_none());
}

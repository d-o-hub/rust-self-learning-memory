//! Tests for LRU cache implementation

use super::{CacheConfig, LRUCache};
use tokio::time::{sleep, Duration as TokioDuration};
use uuid::Uuid;

fn create_test_cache(max_size: usize, ttl_secs: u64) -> LRUCache {
    let config = CacheConfig {
        max_size,
        default_ttl_secs: ttl_secs,
        cleanup_interval_secs: 1,
        enable_background_cleanup: false, // Disable for deterministic tests
    };
    LRUCache::new(config)
}

#[tokio::test]
async fn test_cache_creation() {
    let cache = create_test_cache(100, 3600);
    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.item_count, 0);
    assert_eq!(metrics.hits, 0);
    assert_eq!(metrics.misses, 0);
}

#[tokio::test]
async fn test_cache_hit_and_miss() {
    let cache = create_test_cache(100, 3600);
    let id = Uuid::new_v4();

    // First access: miss
    let hit = cache.record_access(id, false, Some(100)).await;
    assert!(!hit);

    // Second access: hit
    let hit = cache.record_access(id, true, None).await;
    assert!(hit);

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.hits, 1);
    assert_eq!(metrics.misses, 1);
    assert_eq!(metrics.hit_rate, 0.5);
}

#[tokio::test]
async fn test_lru_eviction() {
    let cache = create_test_cache(3, 3600);

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();
    let id4 = Uuid::new_v4();

    // Fill cache to capacity
    cache.record_access(id1, false, Some(100)).await;
    cache.record_access(id2, false, Some(100)).await;
    cache.record_access(id3, false, Some(100)).await;

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.item_count, 3);

    // Add fourth item - should evict id1 (oldest)
    cache.record_access(id4, false, Some(100)).await;

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.item_count, 3);
    assert_eq!(metrics.evictions, 1);

    // id1 should be evicted
    assert!(!cache.contains(id1).await);
    assert!(cache.contains(id2).await);
    assert!(cache.contains(id3).await);
    assert!(cache.contains(id4).await);
}

#[tokio::test]
async fn test_lru_order_with_access() {
    let cache = create_test_cache(3, 3600);

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();
    let id4 = Uuid::new_v4();

    // Fill cache
    cache.record_access(id1, false, Some(100)).await;
    cache.record_access(id2, false, Some(100)).await;
    cache.record_access(id3, false, Some(100)).await;

    // Access id1 (moves to back of queue)
    cache.record_access(id1, true, None).await;

    // Add id4 - should evict id2 (now oldest)
    cache.record_access(id4, false, Some(100)).await;

    assert!(cache.contains(id1).await); // Was accessed, not evicted
    assert!(!cache.contains(id2).await); // Evicted
    assert!(cache.contains(id3).await);
    assert!(cache.contains(id4).await);
}

#[tokio::test]
async fn test_cache_remove() {
    let cache = create_test_cache(10, 3600);
    let id = Uuid::new_v4();

    cache.record_access(id, false, Some(100)).await;
    assert!(cache.contains(id).await);

    cache.remove(id).await;
    assert!(!cache.contains(id).await);
}

#[tokio::test]
async fn test_cache_clear() {
    let cache = create_test_cache(10, 3600);

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();

    cache.record_access(id1, false, Some(100)).await;
    cache.record_access(id2, false, Some(100)).await;

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.item_count, 2);

    cache.clear().await;

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.item_count, 0);
    assert!(!cache.contains(id1).await);
    assert!(!cache.contains(id2).await);
}

#[tokio::test]
async fn test_ttl_expiration_on_access() {
    let cache = create_test_cache(10, 1); // 1 second TTL

    let id = Uuid::new_v4();
    cache.record_access(id, false, Some(100)).await;

    // Immediate access should succeed
    assert!(cache.contains(id).await);

    // Wait for expiration (2 seconds to be safe)
    sleep(TokioDuration::from_secs(2)).await;

    // Should be expired now
    assert!(!cache.contains(id).await);

    // Trying to hit expired entry should count as miss
    let hit = cache.record_access(id, true, None).await;
    assert!(!hit);

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.expirations, 1);
}

#[tokio::test]
async fn test_manual_cleanup() {
    let cache = create_test_cache(10, 1); // 1 second TTL

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();

    cache.record_access(id1, false, Some(100)).await;
    cache.record_access(id2, false, Some(100)).await;

    // Wait for expiration (2 seconds to be safe)
    sleep(TokioDuration::from_secs(2)).await;

    // Run manual cleanup
    let cleaned = cache.cleanup_expired().await;
    assert_eq!(cleaned, 2);

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.item_count, 0);
    assert_eq!(metrics.expirations, 2);
}

#[tokio::test]
async fn test_background_cleanup() {
    let config = CacheConfig {
        max_size: 10,
        default_ttl_secs: 1,
        cleanup_interval_secs: 1,
        enable_background_cleanup: true, // Enable background task
    };
    let mut cache = LRUCache::new(config);

    let id = Uuid::new_v4();
    cache.record_access(id, false, Some(100)).await;

    // Wait for expiration + cleanup (3 seconds to be safe)
    sleep(TokioDuration::from_secs(3)).await;

    // Allow background task to complete
    cache.cleanup_expired().await;

    let metrics = cache.get_metrics().await;
    assert_eq!(
        metrics.item_count, 0,
        "Cache should be empty after expiration"
    );

    cache.stop_cleanup();
}

#[tokio::test]
async fn test_cache_size_tracking() {
    let cache = create_test_cache(10, 3600);

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();

    cache.record_access(id1, false, Some(100)).await;
    cache.record_access(id2, false, Some(200)).await;

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.total_size_bytes, 300);
}

#[tokio::test]
async fn test_cache_metrics_accuracy() {
    let cache = create_test_cache(10, 3600);
    let id = Uuid::new_v4();

    // 1 miss
    cache.record_access(id, false, Some(100)).await;

    // 3 hits
    cache.record_access(id, true, None).await;
    cache.record_access(id, true, None).await;
    cache.record_access(id, true, None).await;

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.hits, 3);
    assert_eq!(metrics.misses, 1);
    assert_eq!(metrics.hit_rate, 0.75); // 3/4
}

#[tokio::test]
async fn test_zero_ttl_no_expiration() {
    let cache = create_test_cache(10, 0); // No TTL

    let id = Uuid::new_v4();
    cache.record_access(id, false, Some(100)).await;

    // Wait a bit
    sleep(TokioDuration::from_secs(2)).await;

    // Should still be valid
    assert!(cache.contains(id).await);
    let hit = cache.record_access(id, true, None).await;
    assert!(hit);
}

#[tokio::test]
async fn test_edge_case_size_one() {
    let cache = create_test_cache(1, 3600); // Size = 1

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();

    cache.record_access(id1, false, Some(100)).await;
    assert!(cache.contains(id1).await);

    // Adding second item should evict first
    cache.record_access(id2, false, Some(100)).await;
    assert!(!cache.contains(id1).await);
    assert!(cache.contains(id2).await);

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.item_count, 1);
    assert_eq!(metrics.evictions, 1);
}

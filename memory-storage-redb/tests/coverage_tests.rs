//! Adaptive cache coverage tests for memory-storage-redb (ACT-027)
//!
//! This module adds comprehensive tests for:
//! - Adaptive cache TTL edge cases and adjustments
//! - Hot/cold item detection
//! - Window overflow handling

use memory_storage_redb::{AdaptiveCache, AdaptiveCacheConfig};
use std::time::Duration;
use uuid::Uuid;

// ============================================================================
// Adaptive Cache TTL Adjustment Tests
// ============================================================================

/// Test that adaptive cache increases TTL for hot items beyond the default
#[tokio::test]
async fn test_adaptive_cache_ttl_increase_for_hot_items() {
    let config = AdaptiveCacheConfig {
        max_size: 100,
        default_ttl: Duration::from_secs(60),
        min_ttl: Duration::from_secs(10),
        max_ttl: Duration::from_secs(300),
        hot_threshold: 3,
        cold_threshold: 1,
        adaptation_rate: 0.5, // 50% increase per access above threshold
        window_size: 10,
        cleanup_interval_secs: 0,
        enable_background_cleanup: false,
    };

    let cache = AdaptiveCache::<String>::new(config);
    let id = Uuid::new_v4();

    // Create entry with default TTL
    cache
        .record_access(id, false, Some("value".to_string()))
        .await;
    let initial_ttl = cache.ttl(id).await.expect("Should have TTL");
    assert_eq!(initial_ttl, Duration::from_secs(60));

    // Access multiple times to become hot
    for _ in 0..5 {
        cache.record_access(id, true, None).await;
    }

    let hot_ttl = cache.ttl(id).await.expect("Should have TTL");
    assert!(
        hot_ttl > initial_ttl,
        "Hot item TTL ({:?}) should be greater than initial ({:?})",
        hot_ttl,
        initial_ttl
    );
}

/// Test that adaptive cache decreases TTL for cold items
#[tokio::test]
async fn test_adaptive_cache_ttl_decrease_for_cold_items() {
    let config = AdaptiveCacheConfig {
        max_size: 100,
        default_ttl: Duration::from_secs(60),
        min_ttl: Duration::from_secs(10),
        max_ttl: Duration::from_secs(300),
        hot_threshold: 10,
        cold_threshold: 1, // Items with <= 1 access are cold
        adaptation_rate: 0.3,
        window_size: 10,
        cleanup_interval_secs: 0,
        enable_background_cleanup: false,
    };

    let cache = AdaptiveCache::<String>::new(config);
    let id = Uuid::new_v4();

    // Create entry
    cache
        .record_access(id, false, Some("value".to_string()))
        .await;

    // Just one access makes it cold (threshold is <= 1)
    cache.record_access(id, true, None).await;

    let cold_ttl = cache.ttl(id).await.expect("Should have TTL");
    assert!(
        cold_ttl < Duration::from_secs(60),
        "Cold item TTL ({:?}) should be less than default (60s)",
        cold_ttl
    );
}

/// Test TTL bounds are enforced - TTL cannot exceed max_ttl
#[tokio::test]
async fn test_adaptive_cache_ttl_max_bound() {
    let config = AdaptiveCacheConfig {
        max_size: 100,
        default_ttl: Duration::from_secs(60),
        min_ttl: Duration::from_secs(10),
        max_ttl: Duration::from_secs(100), // Low max
        hot_threshold: 2,
        cold_threshold: 0,
        adaptation_rate: 0.5,
        window_size: 10,
        cleanup_interval_secs: 0,
        enable_background_cleanup: false,
    };

    let cache = AdaptiveCache::<String>::new(config);
    let id = Uuid::new_v4();

    // Create entry
    cache
        .record_access(id, false, Some("value".to_string()))
        .await;

    // Access many times to try to exceed max TTL
    for _ in 0..20 {
        cache.record_access(id, true, None).await;
    }

    let ttl = cache.ttl(id).await.expect("Should have TTL");
    assert!(
        ttl <= Duration::from_secs(100),
        "TTL ({:?}) should not exceed max_ttl (100s)",
        ttl
    );
}

/// Test TTL bounds are enforced - TTL cannot go below min_ttl
#[tokio::test]
async fn test_adaptive_cache_ttl_min_bound() {
    let config = AdaptiveCacheConfig {
        max_size: 100,
        default_ttl: Duration::from_secs(60),
        min_ttl: Duration::from_secs(30), // High min
        max_ttl: Duration::from_secs(300),
        hot_threshold: 100, // Impossible to be hot
        cold_threshold: 10, // Easy to be cold
        adaptation_rate: 0.5,
        window_size: 10,
        cleanup_interval_secs: 0,
        enable_background_cleanup: false,
    };

    let cache = AdaptiveCache::<String>::new(config);
    let id = Uuid::new_v4();

    // Create entry
    cache
        .record_access(id, false, Some("value".to_string()))
        .await;

    // Access few times to become cold
    for _ in 0..5 {
        cache.record_access(id, true, None).await;
    }

    let ttl = cache.ttl(id).await.expect("Should have TTL");
    assert!(
        ttl >= Duration::from_secs(30),
        "TTL ({:?}) should not go below min_ttl (30s)",
        ttl
    );
}

/// Test access window overflow - old accesses are discarded
#[tokio::test]
async fn test_adaptive_cache_window_overflow() {
    let config = AdaptiveCacheConfig {
        max_size: 100,
        default_ttl: Duration::from_secs(60),
        min_ttl: Duration::from_secs(10),
        max_ttl: Duration::from_secs(300),
        hot_threshold: 3,
        cold_threshold: 1,
        adaptation_rate: 0.25,
        window_size: 5, // Small window
        cleanup_interval_secs: 0,
        enable_background_cleanup: false,
    };

    let cache = AdaptiveCache::<String>::new(config);
    let id = Uuid::new_v4();

    // Create entry
    cache
        .record_access(id, false, Some("value".to_string()))
        .await;

    // Access more times than window size
    for _ in 0..10 {
        cache.record_access(id, true, None).await;
    }

    // Should still be functional and hot
    let access_count = cache.access_count(id).await;
    assert_eq!(access_count, Some(10));

    let hot_count = cache.hot_count().await;
    assert!(hot_count > 0, "Should have hot items");
}

/// Test multiple hot and cold items in same cache
#[tokio::test]
async fn test_adaptive_cache_multiple_hot_cold_items() {
    let config = AdaptiveCacheConfig {
        max_size: 100,
        default_ttl: Duration::from_secs(60),
        min_ttl: Duration::from_secs(10),
        max_ttl: Duration::from_secs(300),
        hot_threshold: 5,
        cold_threshold: 1,
        adaptation_rate: 0.25,
        window_size: 10,
        cleanup_interval_secs: 0,
        enable_background_cleanup: false,
    };

    let cache = AdaptiveCache::<String>::new(config);

    // Create hot item
    let hot_id = Uuid::new_v4();
    cache
        .record_access(hot_id, false, Some("hot".to_string()))
        .await;
    for _ in 0..10 {
        cache.record_access(hot_id, true, None).await;
    }

    // Create cold item
    let cold_id = Uuid::new_v4();
    cache
        .record_access(cold_id, false, Some("cold".to_string()))
        .await;
    // Only 1 access = cold

    // Create neutral item (between hot and cold)
    let neutral_id = Uuid::new_v4();
    cache
        .record_access(neutral_id, false, Some("neutral".to_string()))
        .await;
    for _ in 0..3 {
        cache.record_access(neutral_id, true, None).await;
    }

    let metrics = cache.get_metrics().await;
    assert!(metrics.hot_item_count >= 1, "Should have hot items");
    assert!(metrics.cold_item_count >= 1, "Should have cold items");
    assert_eq!(metrics.base.item_count, 3);
}

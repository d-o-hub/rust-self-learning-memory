//! Cache eviction scenario tests for memory-storage-redb (ACT-035)
//!
//! This module tests LRU eviction behavior including:
//! - Eviction under capacity pressure
//! - LRU ordering and reordering on access
//! - TTL-based expiration
//! - Eviction and hit/miss metrics tracking

// Integration tests are separate crate roots and don't inherit .clippy.toml settings
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use do_memory_storage_redb::{CacheConfig, LRUCache};
use std::time::Duration;
use uuid::Uuid;

// ============================================================================
// LRU Eviction Under Pressure Tests
// ============================================================================

/// Fill cache to max_size, add one more, verify oldest is evicted
#[tokio::test]
async fn test_cache_eviction_under_pressure() {
    let cache = LRUCache::new(CacheConfig {
        max_size: 3,
        default_ttl_secs: 0,
        cleanup_interval_secs: 0,
        enable_background_cleanup: false,
    });

    let ids: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();
    for &id in &ids {
        cache.record_access(id, false, Some(64)).await;
    }

    // All three should be present
    for &id in &ids {
        assert!(
            cache.contains(id).await,
            "Entry should exist before eviction"
        );
    }

    // Add a 4th entry — should evict the oldest (ids[0])
    let new_id = Uuid::new_v4();
    cache.record_access(new_id, false, Some(64)).await;

    assert!(
        !cache.contains(ids[0]).await,
        "Oldest entry should be evicted"
    );
    assert!(cache.contains(ids[1]).await);
    assert!(cache.contains(ids[2]).await);
    assert!(cache.contains(new_id).await);
}

// ============================================================================
// Eviction Ordering Tests
// ============================================================================

/// Add items A, B, C with max_size=2, verify A is evicted first
#[tokio::test]
async fn test_cache_eviction_ordering() {
    let cache = LRUCache::new(CacheConfig {
        max_size: 2,
        default_ttl_secs: 0,
        cleanup_interval_secs: 0,
        enable_background_cleanup: false,
    });

    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let c = Uuid::new_v4();

    cache.record_access(a, false, Some(32)).await;
    cache.record_access(b, false, Some(32)).await;

    // Cache is full (A, B). Adding C should evict A.
    cache.record_access(c, false, Some(32)).await;

    assert!(!cache.contains(a).await, "A should be evicted (oldest)");
    assert!(cache.contains(b).await, "B should remain");
    assert!(cache.contains(c).await, "C should be present");
}

// ============================================================================
// LRU Reordering on Access Tests
// ============================================================================

/// Access oldest item (hit=true), verify it's NOT evicted when cache is full
#[tokio::test]
async fn test_cache_eviction_lru_reordering_on_access() {
    let cache = LRUCache::new(CacheConfig {
        max_size: 2,
        default_ttl_secs: 0,
        cleanup_interval_secs: 0,
        enable_background_cleanup: false,
    });

    let a = Uuid::new_v4();
    let b = Uuid::new_v4();

    cache.record_access(a, false, Some(32)).await;
    cache.record_access(b, false, Some(32)).await;

    // Touch A so it becomes most-recently-used
    cache.record_access(a, true, None).await;

    // Adding C should evict B (now the least recently used)
    let c = Uuid::new_v4();
    cache.record_access(c, false, Some(32)).await;

    assert!(
        cache.contains(a).await,
        "A was accessed, should NOT be evicted"
    );
    assert!(!cache.contains(b).await, "B should be evicted (now oldest)");
    assert!(cache.contains(c).await, "C should be present");
}

// ============================================================================
// Multiple Evictions Tests
// ============================================================================

/// Fill cache with max_size=3, add 3 more items, verify all originals evicted
#[tokio::test]
async fn test_cache_eviction_multiple_evictions() {
    let cache = LRUCache::new(CacheConfig {
        max_size: 3,
        default_ttl_secs: 0,
        cleanup_interval_secs: 0,
        enable_background_cleanup: false,
    });

    let originals: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();
    for &id in &originals {
        cache.record_access(id, false, Some(64)).await;
    }

    let replacements: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();
    for &id in &replacements {
        cache.record_access(id, false, Some(64)).await;
    }

    // All originals should be gone
    for &id in &originals {
        assert!(
            !cache.contains(id).await,
            "Original entry should be evicted"
        );
    }
    // All replacements should be present
    for &id in &replacements {
        assert!(cache.contains(id).await, "Replacement entry should exist");
    }

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.evictions, 3, "Should have exactly 3 evictions");
    assert_eq!(metrics.item_count, 3, "Item count should equal max_size");
}

// ============================================================================
// TTL Expiration Tests
// ============================================================================

/// Add item with 1-second TTL, wait, verify cleanup_expired removes it
#[tokio::test]
async fn test_cache_eviction_ttl_expiration() {
    let cache = LRUCache::new(CacheConfig {
        max_size: 100,
        default_ttl_secs: 1,
        cleanup_interval_secs: 0,
        enable_background_cleanup: false,
    });

    let id = Uuid::new_v4();
    cache.record_access(id, false, Some(64)).await;
    assert!(
        cache.contains(id).await,
        "Entry should exist before expiration"
    );

    // Wait for TTL to expire
    tokio::time::sleep(Duration::from_secs(2)).await;

    let removed = cache.cleanup_expired().await;
    assert_eq!(removed, 1, "Should have removed 1 expired entry");
    assert!(
        !cache.contains(id).await,
        "Entry should be gone after cleanup"
    );

    let metrics = cache.get_metrics().await;
    assert!(
        metrics.expirations > 0,
        "Expiration counter should be incremented"
    );
    assert_eq!(metrics.item_count, 0, "Cache should be empty");
}

// ============================================================================
// Eviction Metrics Tracking Tests
// ============================================================================

/// Verify eviction counter increments correctly
#[tokio::test]
async fn test_cache_eviction_metrics_tracking() {
    let cache = LRUCache::new(CacheConfig {
        max_size: 2,
        default_ttl_secs: 0,
        cleanup_interval_secs: 0,
        enable_background_cleanup: false,
    });

    // Fill cache
    cache.record_access(Uuid::new_v4(), false, Some(32)).await;
    cache.record_access(Uuid::new_v4(), false, Some(32)).await;

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.evictions, 0, "No evictions yet");

    // Trigger 3 evictions one at a time
    for _ in 0..3 {
        cache.record_access(Uuid::new_v4(), false, Some(32)).await;
    }

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.evictions, 3, "Should have 3 evictions");
    assert_eq!(metrics.item_count, 2, "Item count should match max_size");
}

// ============================================================================
// Cache Clear Metrics Tests
// ============================================================================

/// Verify clear() resets item_count
#[tokio::test]
async fn test_cache_eviction_clear_resets_item_count() {
    let cache = LRUCache::new(CacheConfig {
        max_size: 100,
        default_ttl_secs: 0,
        cleanup_interval_secs: 0,
        enable_background_cleanup: false,
    });

    for _ in 0..10 {
        cache.record_access(Uuid::new_v4(), false, Some(64)).await;
    }

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.item_count, 10);

    cache.clear().await;

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.item_count, 0, "item_count should be 0 after clear");
}

// ============================================================================
// Mixed Hit/Miss Pattern Tests
// ============================================================================

/// Record alternating hits and misses, verify metrics
#[tokio::test]
async fn test_cache_eviction_mixed_hit_miss_patterns() {
    let cache = LRUCache::new(CacheConfig {
        max_size: 100,
        default_ttl_secs: 0,
        cleanup_interval_secs: 0,
        enable_background_cleanup: false,
    });

    // Insert 5 entries (all misses)
    let ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();
    for &id in &ids {
        cache.record_access(id, false, Some(64)).await;
    }

    // Alternate hits on existing entries
    for &id in &ids {
        cache.record_access(id, true, None).await;
    }

    // Record misses for new entries
    for _ in 0..3 {
        cache.record_access(Uuid::new_v4(), false, Some(64)).await;
    }

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.hits, 5, "Should have 5 hits");
    assert_eq!(
        metrics.misses, 8,
        "Should have 8 misses (5 initial + 3 new)"
    );
    assert_eq!(metrics.item_count, 8, "Should have 8 items total");

    // Hit rate = 5 / (5 + 8) ≈ 0.3846
    let expected_rate = 5.0 / 13.0;
    assert!(
        (metrics.hit_rate - expected_rate).abs() < 0.01,
        "Hit rate should be ~{:.4}, got {:.4}",
        expected_rate,
        metrics.hit_rate
    );
}

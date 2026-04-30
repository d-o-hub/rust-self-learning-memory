//! Persistence and cache metrics coverage tests for memory-storage-redb (ACT-027)
//!
//! This module adds comprehensive tests for:
//! - Persistence manager operations
//! - Cache persistence with compression
//! - Cache metrics calculations
//! - AdaptiveCacheAdapter integration

// Integration tests are separate crate roots and don't inherit .clippy.toml settings
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::float_cmp)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::uninlined_format_args)]

use do_memory_storage_redb::{
    AdaptiveCacheAdapter, AdaptiveCacheConfig, Cache, CacheMetrics, CachePersistence,
    CacheSnapshot, PersistedCacheEntry, PersistenceConfig, PersistenceManager, PersistenceStats,
    RedbStorage,
};
use std::collections::HashMap;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;
use uuid::Uuid;

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_snapshot_with_entries(count: usize) -> CacheSnapshot {
    let entries: Vec<PersistedCacheEntry> = (0..count)
        .map(|i| PersistedCacheEntry {
            key: format!("key_{}", i),
            value: vec![1, 2, 3, 4],
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            access_count: i as u64 + 1,
            last_accessed: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl_secs: Some(3600),
        })
        .collect();

    CacheSnapshot {
        version: 1,
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        entries,
        metadata: HashMap::new(),
    }
}

// ============================================================================
// Persistence Manager Tests
// ============================================================================

/// Test persistence manager creation and basic operations
#[tokio::test]
async fn test_pm_creation() {
    let temp_dir = TempDir::new().unwrap();
    let config = PersistenceConfig {
        enabled: true,
        persistence_path: temp_dir.path().join("cache.snapshot"),
        ..Default::default()
    };

    let manager = PersistenceManager::new(config);
    assert!(manager.config().enabled);
    assert!(!manager.has_recovery_snapshot());
    assert!(manager.last_snapshot().is_none());
}

/// Test persistence manager force save and recovery
#[tokio::test]
async fn test_pm_save_and_recover() {
    let temp_dir = TempDir::new().unwrap();
    let config = PersistenceConfig {
        enabled: true,
        persistence_path: temp_dir.path().join("cache.snapshot"),
        ..Default::default()
    };

    let manager = PersistenceManager::new(config);

    // Create and save snapshot
    let snapshot = create_test_snapshot_with_entries(10);
    let saved_count = manager.force_save(&snapshot).unwrap();
    assert_eq!(saved_count, 10);
    assert!(manager.has_recovery_snapshot());

    // Recover
    let recovered = manager.recover().unwrap();
    assert!(recovered.is_some());
    let recovered_snapshot = recovered.unwrap();
    assert_eq!(recovered_snapshot.entries.len(), 10);
}

/// Test persistence manager disabled mode
#[tokio::test]
async fn test_pm_disabled() {
    let config = PersistenceConfig::disabled();
    let manager = PersistenceManager::new(config);

    assert!(!manager.config().enabled);

    // Operations should be no-ops when disabled
    let snapshot = create_test_snapshot_with_entries(5);
    let result = manager.force_save(&snapshot);
    assert!(result.is_ok()); // Should succeed but not save

    let recovered = manager.recover().unwrap();
    assert!(recovered.is_none());
}

/// Test persistence manager delete persisted
#[tokio::test]
async fn test_pm_delete() {
    let temp_dir = TempDir::new().unwrap();
    let config = PersistenceConfig {
        enabled: true,
        persistence_path: temp_dir.path().join("cache.snapshot"),
        ..Default::default()
    };

    let manager = PersistenceManager::new(config);

    // Save a snapshot
    let snapshot = create_test_snapshot_with_entries(3);
    manager.force_save(&snapshot).unwrap();
    assert!(manager.has_recovery_snapshot());

    // Delete it
    let deleted = manager.delete_persisted().unwrap();
    assert!(deleted);
    assert!(!manager.has_recovery_snapshot());

    // Delete non-existent
    let deleted_again = manager.delete_persisted().unwrap();
    assert!(!deleted_again);
}

/// Test persistence manager statistics
#[tokio::test]
async fn test_pm_statistics() {
    let temp_dir = TempDir::new().unwrap();
    let config = PersistenceConfig {
        enabled: true,
        persistence_path: temp_dir.path().join("cache.snapshot"),
        compression_enabled: true,
        ..Default::default()
    };

    let manager = PersistenceManager::new(config);

    // Initial stats
    let stats = manager.stats();
    assert_eq!(stats.snapshots_saved, 0);
    assert_eq!(stats.snapshots_loaded, 0);

    // Save a snapshot
    let snapshot = create_test_snapshot_with_entries(5);
    manager.force_save(&snapshot).unwrap();

    let stats = manager.stats();
    assert_eq!(stats.snapshots_saved, 1);
    assert_eq!(stats.total_entries_saved, 5);
    assert!(stats.total_bytes_written > 0);

    // Load it back
    manager.recover().unwrap();

    let stats = manager.stats();
    assert_eq!(stats.snapshots_loaded, 1);
    assert_eq!(stats.total_entries_loaded, 5);
}

// ============================================================================
// Cache Persistence Tests
// ============================================================================

/// Test cache persistence with compression enabled
#[tokio::test]
async fn test_cp_with_compression() {
    let temp_dir = TempDir::new().unwrap();
    let config = PersistenceConfig {
        enabled: true,
        persistence_path: temp_dir.path().join("compressed.snapshot"),
        compression_enabled: true,
        ..Default::default()
    };

    let persistence = CachePersistence::new(config);

    // Create a large snapshot
    let snapshot = create_test_snapshot_with_entries(100);

    // Save
    let saved = persistence.save_snapshot(&snapshot, None).unwrap();
    assert_eq!(saved, 100);

    // Load
    let loaded = persistence.load_snapshot(None).unwrap();
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap().entries.len(), 100);
}

/// Test cache persistence without compression
#[tokio::test]
async fn test_cp_without_compression() {
    let temp_dir = TempDir::new().unwrap();
    let config = PersistenceConfig {
        enabled: true,
        persistence_path: temp_dir.path().join("uncompressed.snapshot"),
        compression_enabled: false,
        ..Default::default()
    };

    let persistence = CachePersistence::new(config);

    let snapshot = create_test_snapshot_with_entries(10);

    // Save
    let saved = persistence.save_snapshot(&snapshot, None).unwrap();
    assert_eq!(saved, 10);

    // Load
    let loaded = persistence.load_snapshot(None).unwrap();
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap().entries.len(), 10);
}

/// Test cache persistence should_save logic
#[tokio::test]
async fn test_cp_should_save() {
    let config = PersistenceConfig {
        enabled: true,
        min_entries_threshold: 10,
        save_interval: Duration::from_secs(1),
        ..Default::default()
    };

    let persistence = CachePersistence::new(config);

    // Below threshold
    assert!(!persistence.should_save(5));

    // At threshold
    assert!(persistence.should_save(10));

    // Above threshold
    assert!(persistence.should_save(100));
}

/// Test cache persistence reset stats
#[tokio::test]
async fn test_cp_reset_stats() {
    let temp_dir = TempDir::new().unwrap();
    let config = PersistenceConfig {
        enabled: true,
        persistence_path: temp_dir.path().join("reset.snapshot"),
        ..Default::default()
    };

    let persistence = CachePersistence::new(config);

    // Save a snapshot
    let snapshot = create_test_snapshot_with_entries(5);
    persistence.save_snapshot(&snapshot, None).unwrap();

    let stats = persistence.stats();
    assert_eq!(stats.snapshots_saved, 1);

    // Reset
    persistence.reset_stats();

    let stats = persistence.stats();
    assert_eq!(stats.snapshots_saved, 0);
    assert_eq!(stats.total_entries_saved, 0);
}

// ============================================================================
// Cache Metrics Tests
// ============================================================================

/// Test cache metrics hit rate calculation
#[tokio::test]
async fn test_cm_hit_rate_calculation() {
    let config = AdaptiveCacheConfig {
        max_size: 100,
        default_ttl: Duration::from_secs(3600),
        enable_background_cleanup: false,
        ..Default::default()
    };

    let cache = AdaptiveCacheAdapter::new(config);
    let id = Uuid::new_v4();

    // Record misses
    cache.record_access(id, false, Some(100)).await;

    // Record hits
    for _ in 0..9 {
        cache.record_access(id, true, None).await;
    }

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.hits, 9);
    assert_eq!(metrics.misses, 1);
    assert!((metrics.hit_rate - 0.9).abs() < 0.001);
}

/// Test cache metrics effectiveness check
#[test]
fn test_cm_is_effective() {
    // 0% hit rate - not effective
    let mut metrics = CacheMetrics {
        hits: 0,
        misses: 10,
        ..CacheMetrics::default()
    };
    metrics.calculate_hit_rate();
    assert!(!metrics.is_effective());

    // 30% hit rate - not effective
    metrics.hits = 3;
    metrics.misses = 7;
    metrics.calculate_hit_rate();
    assert!(!metrics.is_effective());

    // 50% hit rate - effective
    metrics.hits = 5;
    metrics.misses = 5;
    metrics.calculate_hit_rate();
    assert!(metrics.is_effective());

    // 100% hit rate - effective
    metrics.hits = 10;
    metrics.misses = 0;
    metrics.calculate_hit_rate();
    assert!(metrics.is_effective());
}

/// Test cache metrics with evictions
#[tokio::test]
async fn test_cm_with_evictions() {
    let config = AdaptiveCacheConfig {
        max_size: 3, // Small size to trigger evictions
        default_ttl: Duration::from_secs(3600),
        enable_background_cleanup: false,
        ..Default::default()
    };

    let cache = AdaptiveCacheAdapter::new(config);

    // Fill cache beyond capacity
    for _ in 0..5 {
        let id = Uuid::new_v4();
        cache.record_access(id, false, Some(100)).await;
    }

    let metrics = cache.get_metrics().await;
    assert!(metrics.evictions > 0, "Should have evictions");
    assert_eq!(metrics.item_count, 3); // Max size
}

// ============================================================================
// Cache Cleanup Tests
// ============================================================================

/// Test cache cleanup removes expired entries
#[tokio::test]
async fn test_cc_expired() {
    let config = AdaptiveCacheConfig {
        max_size: 100,
        default_ttl: Duration::from_millis(50),
        min_ttl: Duration::from_millis(10),
        max_ttl: Duration::from_millis(100),
        enable_background_cleanup: false,
        ..Default::default()
    };

    let cache = AdaptiveCacheAdapter::new(config);

    // Add entries
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    cache.record_access(id1, false, Some(100)).await;
    cache.record_access(id2, false, Some(100)).await;

    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Cleanup
    let removed = cache.cleanup_expired().await;
    assert!(removed > 0, "Should have removed expired entries");

    let metrics = cache.get_metrics().await;
    assert!(metrics.expirations > 0);
}

/// Test cache clear removes all entries
#[tokio::test]
async fn test_cc_clear() {
    let config = AdaptiveCacheConfig {
        max_size: 100,
        default_ttl: Duration::from_secs(3600),
        enable_background_cleanup: false,
        ..Default::default()
    };

    let cache = AdaptiveCacheAdapter::new(config);

    // Add entries
    for _ in 0..10 {
        let id = Uuid::new_v4();
        cache.record_access(id, false, Some(100)).await;
    }

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.item_count, 10);

    // Clear
    cache.clear().await;

    let metrics = cache.get_metrics().await;
    assert_eq!(metrics.item_count, 0);
}

// ============================================================================
// AdaptiveCacheAdapter Integration Tests
// ============================================================================

/// Test AdaptiveCacheAdapter with RedbStorage
#[tokio::test]
async fn test_aca_with_storage() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.redb");

    let config = AdaptiveCacheConfig {
        max_size: 100,
        default_ttl: Duration::from_secs(60),
        min_ttl: Duration::from_secs(10),
        max_ttl: Duration::from_secs(300),
        hot_threshold: 3,
        cold_threshold: 1,
        adaptation_rate: 0.25,
        window_size: 10,
        cleanup_interval_secs: 0,
        enable_background_cleanup: false,
    };

    let storage = RedbStorage::new_with_adaptive_config(&db_path, config)
        .await
        .expect("Failed to create storage");

    // Basic operations
    let metrics = storage.get_cache_metrics().await;
    assert_eq!(metrics.item_count, 0);
}

/// Test AdaptiveCacheAdapter hit/miss tracking
#[tokio::test]
async fn test_aca_hit_miss_tracking() {
    let config = AdaptiveCacheConfig {
        max_size: 100,
        default_ttl: Duration::from_secs(3600),
        enable_background_cleanup: false,
        ..Default::default()
    };

    let adapter = AdaptiveCacheAdapter::new(config);

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();

    // Misses
    adapter.record_access(id1, false, Some(100)).await;
    adapter.record_access(id2, false, Some(200)).await;

    // Hits
    adapter.record_access(id1, true, None).await;
    adapter.record_access(id1, true, None).await;
    adapter.record_access(id2, true, None).await;

    let metrics = adapter.get_metrics().await;
    assert_eq!(metrics.hits, 3);
    assert_eq!(metrics.misses, 2);
    assert_eq!(metrics.item_count, 2);
}

/// Test AdaptiveCacheAdapter len and is_empty
#[tokio::test]
async fn test_aca_len_is_empty() {
    let config = AdaptiveCacheConfig {
        max_size: 100,
        default_ttl: Duration::from_secs(3600),
        enable_background_cleanup: false,
        ..Default::default()
    };

    let adapter = AdaptiveCacheAdapter::new(config);

    assert!(adapter.is_empty().await);
    assert_eq!(adapter.len().await, 0);

    // Add entries
    for _ in 0..5 {
        let id = Uuid::new_v4();
        adapter.record_access(id, false, Some(100)).await;
    }

    assert!(!adapter.is_empty().await);
    assert_eq!(adapter.len().await, 5);
}

// ============================================================================
// CacheSnapshot Tests
// ============================================================================

/// Test CacheSnapshot creation and operations
#[test]
fn test_cs_creation() {
    let snapshot = CacheSnapshot::new();
    assert_eq!(snapshot.version, 1);
    assert!(snapshot.is_empty());
    assert_eq!(snapshot.len(), 0);
}

/// Test CacheSnapshot with entries
#[test]
fn test_cs_with_entries() {
    let entry = PersistedCacheEntry {
        key: "test_key".to_string(),
        value: vec![1, 2, 3],
        created_at: 1234567890,
        access_count: 5,
        last_accessed: 1234567900,
        ttl_secs: Some(3600),
    };

    let snapshot = CacheSnapshot::new().add_entry(entry);
    assert_eq!(snapshot.len(), 1);
    assert!(!snapshot.is_empty());
    assert!(snapshot.size_bytes() > 0);
}

/// Test CacheSnapshot with metadata
#[test]
fn test_cs_with_metadata() {
    let snapshot = CacheSnapshot::new()
        .with_metadata("version", "1.0")
        .with_metadata("source", "test");

    assert_eq!(snapshot.metadata.get("version"), Some(&"1.0".to_string()));
    assert_eq!(snapshot.metadata.get("source"), Some(&"test".to_string()));
}

// ============================================================================
// PersistenceStats Tests
// ============================================================================

/// Test PersistenceStats average calculations
#[test]
fn test_ps_averages() {
    let stats = PersistenceStats {
        snapshots_saved: 5,
        total_entries_saved: 100,
        total_bytes_written: 5000,
        snapshots_loaded: 3,
        total_entries_loaded: 60,
        total_bytes_read: 3000,
        ..Default::default()
    };

    assert_eq!(stats.avg_bytes_per_entry_written(), 50.0);
    assert_eq!(stats.avg_bytes_per_entry_read(), 50.0);
}

/// Test PersistenceStats success rate calculations
#[test]
fn test_ps_success_rates() {
    // Perfect success rate
    let perfect = PersistenceStats {
        snapshots_saved: 10,
        save_failures: 0,
        snapshots_loaded: 5,
        load_failures: 0,
        ..Default::default()
    };
    assert_eq!(perfect.save_success_rate(), 1.0);
    assert_eq!(perfect.load_success_rate(), 1.0);

    // Partial success rate
    let partial = PersistenceStats {
        snapshots_saved: 9,
        save_failures: 1,
        snapshots_loaded: 4,
        load_failures: 1,
        ..Default::default()
    };
    assert!((partial.save_success_rate() - 0.9).abs() < 0.001);
    assert!((partial.load_success_rate() - 0.8).abs() < 0.001);

    // Empty stats
    let empty = PersistenceStats::default();
    assert_eq!(empty.save_success_rate(), 1.0); // No failures = 100%
    assert_eq!(empty.load_success_rate(), 1.0);
}

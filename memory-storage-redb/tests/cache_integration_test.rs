//! Integration tests for LRU cache with TTL expiration

use chrono::Utc;
use memory_core::{ComplexityLevel, Episode, TaskContext, TaskType};
use memory_storage_redb::{CacheConfig, RedbStorage};
use std::collections::HashMap;
use tempfile::tempdir;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

/// Helper function to create a test episode
fn create_test_episode(id: Uuid) -> Episode {
    Episode {
        episode_id: id,
        task_type: TaskType::Testing,
        task_description: "Test task".to_string(),
        start_time: Utc::now(),
        end_time: None,
        context: TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain: "testing".to_string(),
            tags: vec!["test".to_string()],
        },
        outcome: None,
        steps: vec![],
        patterns: vec![],
        heuristics: vec![],
        reflection: None,
        reward: None,
        applied_patterns: vec![],
        salient_features: None,
        metadata: HashMap::new(),
    }
}

/// Helper to create storage with custom cache config
async fn create_storage_with_config(config: CacheConfig) -> RedbStorage {
    let dir = tempdir().expect("Failed to create temp directory for test");
    let db_path = dir.path().join("test.redb");
    RedbStorage::new_with_cache_config(&db_path, config)
        .await
        .expect("Failed to create storage with config")
}

#[tokio::test]
async fn test_cache_metrics_tracking() {
    let storage = create_storage_with_config(CacheConfig::default()).await;

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let episode1 = create_test_episode(id1);
    let episode2 = create_test_episode(id2);

    // Store two episodes
    storage.store_episode(&episode1).await.unwrap();
    storage.store_episode(&episode2).await.unwrap();

    // Get first episode (hit)
    let result = storage.get_episode(id1).await.unwrap();
    assert!(result.is_some());

    // Get non-existent episode (miss)
    let result = storage.get_episode(Uuid::new_v4()).await.unwrap();
    assert!(result.is_none());

    // Check metrics
    let metrics = storage.get_cache_metrics().await;
    assert_eq!(metrics.hits, 1);
    assert_eq!(metrics.misses, 3); // 2 stores + 1 failed get
    assert_eq!(metrics.item_count, 2);
    assert_eq!(metrics.hit_rate, 0.25); // 1/4
}

#[tokio::test]
async fn test_lru_eviction_on_full_cache() {
    let config = CacheConfig {
        max_size: 3,
        default_ttl_secs: 3600,
        cleanup_interval_secs: 300,
        enable_background_cleanup: false,
    };
    let storage = create_storage_with_config(config).await;

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();
    let id4 = Uuid::new_v4();

    // Fill cache to capacity
    storage
        .store_episode(&create_test_episode(id1))
        .await
        .unwrap();
    storage
        .store_episode(&create_test_episode(id2))
        .await
        .unwrap();
    storage
        .store_episode(&create_test_episode(id3))
        .await
        .unwrap();

    let metrics = storage.get_cache_metrics().await;
    assert_eq!(metrics.item_count, 3);
    assert_eq!(metrics.evictions, 0);

    // Add one more - should evict id1 (least recently used)
    storage
        .store_episode(&create_test_episode(id4))
        .await
        .unwrap();

    let metrics = storage.get_cache_metrics().await;
    assert_eq!(metrics.item_count, 3);
    assert_eq!(metrics.evictions, 1);

    // All episodes should still be retrievable from redb (not affected by cache eviction)
    assert!(storage.get_episode(id1).await.unwrap().is_some());
    assert!(storage.get_episode(id2).await.unwrap().is_some());
    assert!(storage.get_episode(id3).await.unwrap().is_some());
    assert!(storage.get_episode(id4).await.unwrap().is_some());
}

#[tokio::test]
async fn test_lru_order_updates_on_access() {
    let config = CacheConfig {
        max_size: 3,
        default_ttl_secs: 3600,
        cleanup_interval_secs: 300,
        enable_background_cleanup: false,
    };
    let storage = create_storage_with_config(config).await;

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();
    let id4 = Uuid::new_v4();

    // Fill cache
    storage
        .store_episode(&create_test_episode(id1))
        .await
        .unwrap();
    storage
        .store_episode(&create_test_episode(id2))
        .await
        .unwrap();
    storage
        .store_episode(&create_test_episode(id3))
        .await
        .unwrap();

    // Access id1, making it most recently used
    storage.get_episode(id1).await.unwrap();

    // Add id4 - should evict id2 (now least recently used)
    storage
        .store_episode(&create_test_episode(id4))
        .await
        .unwrap();

    let metrics = storage.get_cache_metrics().await;
    assert_eq!(metrics.evictions, 1);

    // Verify all can still be retrieved (from redb)
    assert!(storage.get_episode(id1).await.unwrap().is_some());
    assert!(storage.get_episode(id2).await.unwrap().is_some());
    assert!(storage.get_episode(id3).await.unwrap().is_some());
    assert!(storage.get_episode(id4).await.unwrap().is_some());
}

#[tokio::test]
async fn test_ttl_expiration_lazy_check() {
    let config = CacheConfig {
        max_size: 10,
        default_ttl_secs: 1, // 1 second TTL
        cleanup_interval_secs: 300,
        enable_background_cleanup: false,
    };
    let storage = create_storage_with_config(config).await;

    let id = Uuid::new_v4();
    storage
        .store_episode(&create_test_episode(id))
        .await
        .unwrap();

    // Episode should be in cache
    let metrics = storage.get_cache_metrics().await;
    assert_eq!(metrics.item_count, 1);

    // Wait for expiration
    sleep(Duration::from_secs(2)).await;

    // Access should detect expiration and count as miss
    let result = storage.get_episode(id).await.unwrap();
    assert!(result.is_some()); // Still in redb

    let metrics = storage.get_cache_metrics().await;
    assert_eq!(metrics.expirations, 1);
    assert_eq!(metrics.misses, 2); // 1 from store, 1 from expired access
}

#[tokio::test]
async fn test_manual_cache_cleanup() {
    let config = CacheConfig {
        max_size: 10,
        default_ttl_secs: 1,
        cleanup_interval_secs: 300,
        enable_background_cleanup: false,
    };
    let storage = create_storage_with_config(config).await;

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();

    storage
        .store_episode(&create_test_episode(id1))
        .await
        .unwrap();
    storage
        .store_episode(&create_test_episode(id2))
        .await
        .unwrap();

    let metrics = storage.get_cache_metrics().await;
    assert_eq!(metrics.item_count, 2);

    // Wait for expiration
    sleep(Duration::from_secs(2)).await;

    // Manually trigger cleanup
    let expired_count = storage.cleanup_cache().await;
    assert_eq!(expired_count, 2);

    let metrics = storage.get_cache_metrics().await;
    assert_eq!(metrics.item_count, 0);
    assert_eq!(metrics.expirations, 2);
}

#[tokio::test]
async fn test_background_cleanup_task() {
    let config = CacheConfig {
        max_size: 10,
        default_ttl_secs: 1,
        cleanup_interval_secs: 1,
        enable_background_cleanup: true,
    };
    let storage = create_storage_with_config(config).await;

    let id = Uuid::new_v4();
    storage
        .store_episode(&create_test_episode(id))
        .await
        .unwrap();

    let metrics = storage.get_cache_metrics().await;
    assert_eq!(metrics.item_count, 1);

    // Wait for expiration and background cleanup
    sleep(Duration::from_secs(3)).await;

    let metrics = storage.get_cache_metrics().await;
    assert_eq!(metrics.item_count, 0);
    assert!(metrics.expirations > 0);
}

#[tokio::test]
async fn test_cache_clear() {
    let storage = create_storage_with_config(CacheConfig::default()).await;

    // Add multiple episodes
    for _ in 0..5 {
        let id = Uuid::new_v4();
        storage
            .store_episode(&create_test_episode(id))
            .await
            .unwrap();
    }

    let metrics = storage.get_cache_metrics().await;
    assert_eq!(metrics.item_count, 5);

    // Clear cache
    storage.clear_all().await.unwrap();

    let metrics = storage.get_cache_metrics().await;
    assert_eq!(metrics.item_count, 0);
    assert_eq!(metrics.total_size_bytes, 0);
}

#[tokio::test]
async fn test_cache_delete_removes_tracking() {
    let storage = create_storage_with_config(CacheConfig::default()).await;

    let id = Uuid::new_v4();
    storage
        .store_episode(&create_test_episode(id))
        .await
        .unwrap();

    let metrics = storage.get_cache_metrics().await;
    assert_eq!(metrics.item_count, 1);

    // Delete episode
    storage.delete_episode(id).await.unwrap();

    let metrics = storage.get_cache_metrics().await;
    assert_eq!(metrics.item_count, 0);
}

#[tokio::test]
async fn test_cache_size_tracking() {
    let storage = create_storage_with_config(CacheConfig::default()).await;

    storage
        .store_episode(&create_test_episode(Uuid::new_v4()))
        .await
        .unwrap();
    storage
        .store_episode(&create_test_episode(Uuid::new_v4()))
        .await
        .unwrap();
    storage
        .store_episode(&create_test_episode(Uuid::new_v4()))
        .await
        .unwrap();

    let metrics = storage.get_cache_metrics().await;
    assert_eq!(metrics.item_count, 3);
    assert!(metrics.total_size_bytes > 0);
}

#[tokio::test]
async fn test_zero_ttl_no_expiration() {
    let config = CacheConfig {
        max_size: 10,
        default_ttl_secs: 0, // No TTL
        cleanup_interval_secs: 300,
        enable_background_cleanup: false,
    };
    let storage = create_storage_with_config(config).await;

    let id = Uuid::new_v4();
    storage
        .store_episode(&create_test_episode(id))
        .await
        .unwrap();

    // Wait a bit
    sleep(Duration::from_secs(2)).await;

    // Should still be valid
    let result = storage.get_episode(id).await.unwrap();
    assert!(result.is_some());

    let metrics = storage.get_cache_metrics().await;
    assert_eq!(metrics.expirations, 0);
    assert_eq!(metrics.hits, 1);
}

#[tokio::test]
async fn test_concurrent_access() {
    let storage = create_storage_with_config(CacheConfig::default()).await;
    let storage = std::sync::Arc::new(storage);

    // Create multiple tasks that access the cache concurrently
    let mut handles = vec![];

    for _ in 0..10 {
        let storage_clone = std::sync::Arc::clone(&storage);
        let handle = tokio::spawn(async move {
            let id = Uuid::new_v4();
            storage_clone
                .store_episode(&create_test_episode(id))
                .await
                .unwrap();
            storage_clone.get_episode(id).await.unwrap();
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let metrics = storage.get_cache_metrics().await;
    assert_eq!(metrics.item_count, 10);
    assert_eq!(metrics.hits, 10);
}

#[tokio::test]
async fn test_eviction_frees_space() {
    let config = CacheConfig {
        max_size: 2,
        default_ttl_secs: 3600,
        cleanup_interval_secs: 300,
        enable_background_cleanup: false,
    };
    let storage = create_storage_with_config(config).await;

    // Add 3 episodes, third should evict first
    storage
        .store_episode(&create_test_episode(Uuid::new_v4()))
        .await
        .unwrap();
    let initial_size = storage.get_cache_metrics().await.total_size_bytes;

    storage
        .store_episode(&create_test_episode(Uuid::new_v4()))
        .await
        .unwrap();
    let second_size = storage.get_cache_metrics().await.total_size_bytes;
    assert!(second_size > initial_size);

    storage
        .store_episode(&create_test_episode(Uuid::new_v4()))
        .await
        .unwrap();
    let third_size = storage.get_cache_metrics().await.total_size_bytes;

    // Size should be approximately 2x initial, not 3x (due to eviction)
    assert!(third_size < initial_size * 3);
    assert_eq!(storage.get_cache_metrics().await.item_count, 2);
}

#[tokio::test]
async fn test_hit_rate_calculation() {
    let storage = create_storage_with_config(CacheConfig::default()).await;

    let id = Uuid::new_v4();

    // 1 miss (store)
    storage
        .store_episode(&create_test_episode(id))
        .await
        .unwrap();

    // 4 hits
    for _ in 0..4 {
        storage.get_episode(id).await.unwrap();
    }

    let metrics = storage.get_cache_metrics().await;
    assert_eq!(metrics.hits, 4);
    assert_eq!(metrics.misses, 1);
    assert_eq!(metrics.hit_rate, 0.8); // 4/5
}

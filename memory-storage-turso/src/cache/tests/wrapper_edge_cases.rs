//! Cache wrapper tests - Edge cases and correctness
//!
//! Tests for cache miss/fill, absent values, and error propagation.

use super::{CacheConfig, CachedTursoStorage};
use uuid::Uuid;

use super::create_test_episode;
use super::create_test_turso_storage;

#[tokio::test]
async fn test_cache_miss_followed_by_fill() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

    let episode_id = Uuid::new_v4();
    let episode = create_test_episode(episode_id);

    // Store in storage directly to avoid cache invalidation logic if any
    // actually store_episode_cached calls store_episode then removes from cache.
    cached.storage().store_episode(&episode).await.unwrap();

    // 1. Initial state: Empty cache
    let (ep_size, _, _) = cached.cache_sizes().await;
    assert_eq!(ep_size, 0);

    // 2. First access: Cache miss
    let result1 = cached.get_episode_cached(episode_id).await.unwrap();
    assert!(result1.is_some());
    assert_eq!(cached.stats().episode_misses, 1);
    assert_eq!(cached.stats().episode_hits, 0);

    // 3. Verify cache fill
    let (ep_size, _, _) = cached.cache_sizes().await;
    assert_eq!(ep_size, 1);

    // 4. Second access: Cache hit
    let result2 = cached.get_episode_cached(episode_id).await.unwrap();
    assert!(result2.is_some());
    assert_eq!(cached.stats().episode_misses, 1);
    assert_eq!(cached.stats().episode_hits, 1);

    // Verify data consistency
    assert_eq!(result1.unwrap().episode_id, result2.unwrap().episode_id);
}

#[tokio::test]
async fn test_warm_cache_repeated_reads() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

    let episode_id = Uuid::new_v4();
    let episode = create_test_episode(episode_id);
    cached.store_episode_cached(&episode).await.unwrap();

    // Prime cache
    cached.get_episode_cached(episode_id).await.unwrap();
    assert_eq!(cached.stats().episode_hits, 0);
    assert_eq!(cached.stats().episode_misses, 1);

    // Repeated reads
    for i in 1..=5 {
        let result = cached.get_episode_cached(episode_id).await.unwrap();
        assert!(result.is_some());
        assert_eq!(cached.stats().episode_hits, i as u64);
    }
}

#[tokio::test]
async fn test_absent_values_not_incorrectly_cached_as_hits() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

    let nonexistent_id = Uuid::new_v4();

    // 1. First access: Miss, storage returns None
    let result1 = cached.get_episode_cached(nonexistent_id).await.unwrap();
    assert!(result1.is_none());
    assert_eq!(cached.stats().episode_misses, 1);
    assert_eq!(cached.stats().episode_hits, 0);

    // 2. Second access: Still a miss because we don't cache negative results currently
    // (Checking CachedTursoStorage::get_episode_cached implementation:
    //  if let (Some(ep), Some(cache)) = (&episode, &self.episode_cache) { ... }
    //  It only caches if episode is Some)
    let result2 = cached.get_episode_cached(nonexistent_id).await.unwrap();
    assert!(result2.is_none());
    assert_eq!(cached.stats().episode_misses, 2);
    assert_eq!(cached.stats().episode_hits, 0);
}

#[tokio::test]
async fn test_storage_failure_propagation() {
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

    // Break the storage by dropping the table
    let conn = cached.storage().get_connection().await.unwrap();
    conn.execute("DROP TABLE episodes", ()).await.unwrap();

    // Attempt to get an episode should now fail
    let result = cached.get_episode_cached(Uuid::new_v4()).await;
    assert!(result.is_err());

    // Attempt to store an episode should now fail
    let episode = create_test_episode(Uuid::new_v4());
    let result_store = cached.store_episode_cached(&episode).await;
    assert!(result_store.is_err());
}

#[tokio::test]
async fn test_cache_hit_after_manual_storage_update() {
    // This test demonstrates that the cache might be stale if we bypass it to update storage
    // which is expected behavior for a simple cache wrapper.
    let (storage, _dir) = create_test_turso_storage().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());

    let episode_id = Uuid::new_v4();
    let episode = create_test_episode(episode_id);
    cached.store_episode_cached(&episode).await.unwrap();

    // Populate cache
    cached.get_episode_cached(episode_id).await.unwrap();

    // Manually update storage bypassing cache wrapper
    let mut updated_episode = episode.clone();
    updated_episode.task_description = "Updated manually".to_string();
    cached
        .storage()
        .store_episode(&updated_episode)
        .await
        .unwrap();

    // Next cached read should still return OLD data because it's in the cache
    let result = cached
        .get_episode_cached(episode_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result.task_description, episode.task_description);
    assert_ne!(result.task_description, "Updated manually");

    // Invalidate manually
    cached.clear_caches().await;

    // Now it should hit storage and get NEW data
    let result2 = cached
        .get_episode_cached(episode_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result2.task_description, "Updated manually");
}

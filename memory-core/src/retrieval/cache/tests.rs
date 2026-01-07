//! # Query Cache Tests
//!
//! Test suite for the query cache.

#[cfg(test)]
mod cache_tests {
    use crate::episode::Episode;
    use crate::retrieval::cache::lru::QueryCache;
    use crate::retrieval::cache::types::{CacheKey, DEFAULT_CACHE_TTL};
    use crate::types::{TaskContext, TaskType};
    use chrono;
    use std::collections::HashMap;
    use std::time::Duration;
    use uuid::Uuid;

    fn create_test_episode(id: &str) -> Episode {
        Episode {
            episode_id: Uuid::parse_str(id).unwrap_or_else(|_| Uuid::new_v4()),
            task_type: TaskType::CodeGeneration,
            task_description: "test task".to_string(),
            context: TaskContext::default(),
            start_time: chrono::Utc::now(),
            end_time: None,
            steps: vec![],
            outcome: None,
            reward: None,
            reflection: None,
            patterns: vec![],
            heuristics: vec![],
            applied_patterns: vec![],
            salient_features: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_cache_hit() {
        let cache = QueryCache::new();
        let key = CacheKey::new("test query".to_string());
        let episodes = vec![create_test_episode("ep1")];

        // Cache miss initially
        assert!(cache.get(&key).is_none());

        // Put in cache
        cache.put(key.clone(), episodes.clone());

        // Cache hit
        let result = cache.get(&key);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);

        // Check metrics
        let metrics = cache.metrics();
        assert_eq!(metrics.hits, 1);
        assert_eq!(metrics.misses, 1);
        assert_eq!(metrics.hit_rate(), 0.5);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = QueryCache::with_capacity_and_ttl(100, Duration::from_millis(10));
        let key = CacheKey::new("test query".to_string());
        let episodes = vec![create_test_episode("ep1")];

        cache.put(key.clone(), episodes);

        // Immediate hit
        assert!(cache.get(&key).is_some());

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(15));

        // Should be expired
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_cache_invalidation() {
        let cache = QueryCache::new();
        let key1 = CacheKey::new("query1".to_string());
        let key2 = CacheKey::new("query2".to_string());

        cache.put(key1.clone(), vec![create_test_episode("ep1")]);
        cache.put(key2.clone(), vec![create_test_episode("ep2")]);

        assert_eq!(cache.size(), 2);

        // Invalidate all
        cache.invalidate_all();

        assert_eq!(cache.size(), 0);
        assert!(cache.get(&key1).is_none());
        assert!(cache.get(&key2).is_none());

        let metrics = cache.metrics();
        assert_eq!(metrics.invalidations, 2);
    }

    #[test]
    fn test_lru_eviction() {
        let cache = QueryCache::with_capacity_and_ttl(2, DEFAULT_CACHE_TTL);

        let key1 = CacheKey::new("query1".to_string());
        let key2 = CacheKey::new("query2".to_string());
        let key3 = CacheKey::new("query3".to_string());

        cache.put(key1.clone(), vec![create_test_episode("ep1")]);
        cache.put(key2.clone(), vec![create_test_episode("ep2")]);

        // Cache should have 2 entries
        assert_eq!(cache.size(), 2);

        // Add third entry, should evict oldest (key1)
        cache.put(key3.clone(), vec![create_test_episode("ep3")]);

        assert_eq!(cache.size(), 2);
        assert!(cache.get(&key1).is_none()); // Evicted
        assert!(cache.get(&key2).is_some()); // Still present
        assert!(cache.get(&key3).is_some()); // Newly added

        let metrics = cache.metrics();
        assert_eq!(metrics.evictions, 1);
    }

    #[test]
    fn test_cache_key_with_filters() {
        let key1 = CacheKey::new("test".to_string())
            .with_domain(Some("web".to_string()))
            .with_task_type(Some("api".to_string()))
            .with_limit(5);

        let key2 = CacheKey::new("test".to_string())
            .with_domain(Some("web".to_string()))
            .with_task_type(Some("api".to_string()))
            .with_limit(5);

        let key3 = CacheKey::new("test".to_string())
            .with_domain(Some("data".to_string())) // Different domain
            .with_task_type(Some("api".to_string()))
            .with_limit(5);

        // Same keys should have same hash
        assert_eq!(key1.compute_hash(), key2.compute_hash());

        // Different keys should have different hash
        assert_ne!(key1.compute_hash(), key3.compute_hash());
    }

    #[test]
    fn test_metrics_effectiveness() {
        let cache = QueryCache::new();
        let key = CacheKey::new("test".to_string());
        let episodes = vec![create_test_episode("ep1")];

        cache.put(key.clone(), episodes);

        // Generate hits
        for _ in 0..10 {
            let _ = cache.get(&key);
        }

        let metrics = cache.metrics();
        assert!(metrics.is_effective()); // Should be > 40% hit rate
        assert!(metrics.hit_rate() > 0.9); // Should be ~90% (10 hits, 1 miss)
    }

    #[test]
    fn test_domain_based_invalidation() {
        let cache = QueryCache::new();

        // Create keys with different domains
        let key_web = CacheKey::new("query1".to_string()).with_domain(Some("web-api".to_string()));
        let key_data =
            CacheKey::new("query2".to_string()).with_domain(Some("data-processing".to_string()));
        let key_no_domain = CacheKey::new("query3".to_string());

        // Populate cache
        cache.put(key_web.clone(), vec![create_test_episode("ep1")]);
        cache.put(key_data.clone(), vec![create_test_episode("ep2")]);
        cache.put(key_no_domain.clone(), vec![create_test_episode("ep3")]);

        assert_eq!(cache.size(), 3);

        // Invalidate only web-api domain (lazy invalidation)
        cache.invalidate_domain("web-api");

        // Verify web-api entry was marked invalid (returns None on get)
        assert!(cache.get(&key_web).is_none());

        // Verify other entries remain
        assert!(cache.get(&key_data).is_some());
        assert!(cache.get(&key_no_domain).is_some());

        // Physical size includes invalidated entries, effective size doesn't
        assert_eq!(cache.size(), 3); // Physical: still in cache
        assert_eq!(cache.effective_size(), 2); // Logical: excluding invalidated

        // Check metrics
        let metrics = cache.metrics();
        assert_eq!(metrics.invalidations, 1);
    }

    #[test]
    fn test_domain_invalidation_multiple_entries() {
        let cache = QueryCache::new();

        // Create multiple entries for same domain
        let key1 = CacheKey::new("query1".to_string()).with_domain(Some("web-api".to_string()));
        let key2 = CacheKey::new("query2".to_string()).with_domain(Some("web-api".to_string()));
        let key3 = CacheKey::new("query3".to_string()).with_domain(Some("data".to_string()));

        cache.put(key1.clone(), vec![create_test_episode("ep1")]);
        cache.put(key2.clone(), vec![create_test_episode("ep2")]);
        cache.put(key3.clone(), vec![create_test_episode("ep3")]);

        assert_eq!(cache.size(), 3);

        // Invalidate web-api domain (should mark 2 entries invalid)
        cache.invalidate_domain("web-api");

        assert!(cache.get(&key1).is_none());
        assert!(cache.get(&key2).is_none());
        assert!(cache.get(&key3).is_some());

        // With lazy invalidation, physical size stays 3, effective is 1
        assert_eq!(cache.size(), 3);
        assert_eq!(cache.effective_size(), 1);

        let metrics = cache.metrics();
        assert_eq!(metrics.invalidations, 2);
    }

    #[test]
    fn test_domain_invalidation_nonexistent() {
        let cache = QueryCache::new();

        let key = CacheKey::new("query".to_string()).with_domain(Some("web-api".to_string()));

        cache.put(key.clone(), vec![create_test_episode("ep1")]);

        // Invalidate non-existent domain (should be no-op)
        cache.invalidate_domain("nonexistent-domain");

        // Original entry should still exist
        assert!(cache.get(&key).is_some());
        assert_eq!(cache.size(), 1);

        let metrics = cache.metrics();
        assert_eq!(metrics.invalidations, 0);
    }

    #[test]
    fn test_domain_invalidation_empty_cache() {
        let cache = QueryCache::new();

        // Invalidate on empty cache (should not panic)
        cache.invalidate_domain("any-domain");

        assert_eq!(cache.size(), 0);
        let metrics = cache.metrics();
        assert_eq!(metrics.invalidations, 0);
    }

    #[test]
    fn test_invalidate_all_clears_domain_index() {
        let cache = QueryCache::new();

        let key_web = CacheKey::new("query1".to_string()).with_domain(Some("web-api".to_string()));
        let key_data = CacheKey::new("query2".to_string()).with_domain(Some("data".to_string()));

        cache.put(key_web.clone(), vec![create_test_episode("ep1")]);
        cache.put(key_data.clone(), vec![create_test_episode("ep2")]);

        assert_eq!(cache.size(), 2);

        // Clear all
        cache.invalidate_all();

        assert_eq!(cache.size(), 0);

        // Add new entry with same domain - should work fine
        cache.put(key_web.clone(), vec![create_test_episode("ep3")]);
        assert_eq!(cache.size(), 1);

        // Invalidate domain should work correctly after invalidate_all
        cache.invalidate_domain("web-api");

        // With lazy invalidation, physical size is still 1, but entry is marked invalid
        assert_eq!(cache.size(), 1); // Physical size
        assert_eq!(cache.effective_size(), 0); // Logical size
        assert!(cache.get(&key_web).is_none()); // Verify it's actually invalid
    }
}

//! Integration test for domain-based cache invalidation
//!
//! Validates that domain-based cache invalidation works correctly in realistic
//! multi-domain scenarios with episode completion workflows.
#![allow(
    clippy::expect_used,
    clippy::inefficient_to_string,
    clippy::similar_names,
    clippy::field_reassign_with_default,
    clippy::uninlined_format_args
)]

use memory_core::episode::Episode;
use memory_core::retrieval::{CacheKey, QueryCache};
use memory_core::types::{TaskContext, TaskType};
use std::collections::HashMap;
use uuid::Uuid;

fn create_episode(id: &str, domain: &str) -> Episode {
    let mut context = TaskContext::default();
    context.domain = domain.to_string();

    Episode {
        episode_id: Uuid::parse_str(id).unwrap_or_else(|_| Uuid::new_v4()),
        task_type: TaskType::CodeGeneration,
        task_description: format!("Task in {} domain", domain),
        context,
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
fn test_multi_domain_workflow() {
    let cache = QueryCache::new();

    // Simulate queries across multiple domains
    let web_query =
        CacheKey::new("implement REST API".to_string()).with_domain(Some("web-api".to_string()));
    let data_query = CacheKey::new("process CSV file".to_string())
        .with_domain(Some("data-processing".to_string()));
    let ml_query =
        CacheKey::new("train model".to_string()).with_domain(Some("machine-learning".to_string()));

    // Populate cache with episodes from each domain
    let web_episodes = vec![
        create_episode("00000000-0000-0000-0000-000000000001", "web-api"),
        create_episode("00000000-0000-0000-0000-000000000002", "web-api"),
    ];
    let data_episodes = vec![create_episode(
        "00000000-0000-0000-0000-000000000003",
        "data-processing",
    )];
    let ml_episodes = vec![create_episode(
        "00000000-0000-0000-0000-000000000004",
        "machine-learning",
    )];

    cache.put(web_query.clone(), web_episodes.clone());
    cache.put(data_query.clone(), data_episodes.clone());
    cache.put(ml_query.clone(), ml_episodes.clone());

    // Verify all cached
    assert_eq!(cache.size(), 3);
    assert!(cache.get(&web_query).is_some());
    assert!(cache.get(&data_query).is_some());
    assert!(cache.get(&ml_query).is_some());

    // Simulate episode completion in web-api domain
    // In real usage, this would be called after completing an episode
    cache.invalidate_domain("web-api");

    // Web-api cache is cleared
    assert!(cache.get(&web_query).is_none());

    // Other domains remain cached
    assert!(cache.get(&data_query).is_some());
    assert!(cache.get(&ml_query).is_some());

    // With lazy invalidation, physical size is still 3, but effective size is 2
    assert_eq!(cache.size(), 3); // Physical
    assert_eq!(cache.effective_size(), 2); // Logical

    // Verify metrics
    let metrics = cache.metrics();
    assert_eq!(metrics.invalidations, 1);
    assert!(metrics.hits > 0); // From the successful gets above
}

#[test]
fn test_high_frequency_invalidation() {
    let cache = QueryCache::new();

    // Create queries for two domains
    let domain_a_queries: Vec<_> = (0..10)
        .map(|i| CacheKey::new(format!("query-a-{}", i)).with_domain(Some("domain-a".to_string())))
        .collect();

    let domain_b_queries: Vec<_> = (0..10)
        .map(|i| CacheKey::new(format!("query-b-{}", i)).with_domain(Some("domain-b".to_string())))
        .collect();

    // Populate cache
    for query in &domain_a_queries {
        cache.put(
            query.clone(),
            vec![create_episode(
                "00000000-0000-0000-0000-000000000001",
                "domain-a",
            )],
        );
    }

    for query in &domain_b_queries {
        cache.put(
            query.clone(),
            vec![create_episode(
                "00000000-0000-0000-0000-000000000002",
                "domain-b",
            )],
        );
    }

    assert_eq!(cache.size(), 20);

    // Simulate frequent invalidations in domain-a
    for _ in 0..5 {
        // Re-populate domain-a
        for query in &domain_a_queries {
            cache.put(
                query.clone(),
                vec![create_episode(
                    "00000000-0000-0000-0000-000000000001",
                    "domain-a",
                )],
            );
        }

        // Invalidate domain-a
        cache.invalidate_domain("domain-a");
    }

    // Domain-a should be empty
    for query in &domain_a_queries {
        assert!(cache.get(query).is_none());
    }

    // Domain-b should still have all entries
    for query in &domain_b_queries {
        assert!(cache.get(query).is_some());
    }

    // With lazy invalidation, physical size includes invalidated entries
    // After the last invalidation, domain-a entries are still in cache but marked invalid
    assert_eq!(cache.size(), 20); // Physical (10 domain-a + 10 domain-b)
    assert_eq!(cache.effective_size(), 10); // Logical (only domain-b is valid)
}

#[test]
fn test_cache_hit_rate_improvement() {
    let cache = QueryCache::new();

    // Create queries for 3 domains
    let domains = vec!["web-api", "data-processing", "machine-learning"];

    for domain in &domains {
        for i in 0..5 {
            let key = CacheKey::new(format!("query-{}-{}", domain, i))
                .with_domain(Some(domain.to_string()));
            let episodes = vec![create_episode(
                &format!("00000000-0000-0000-0000-00000000000{}", i),
                domain,
            )];
            cache.put(key, episodes);
        }
    }

    assert_eq!(cache.size(), 15);

    // Simulate queries hitting cache
    for domain in &domains {
        for i in 0..5 {
            let key = CacheKey::new(format!("query-{}-{}", domain, i))
                .with_domain(Some(domain.to_string()));
            let _ = cache.get(&key); // Cache hit
        }
    }

    let metrics_before = cache.metrics();
    let hit_rate_before = metrics_before.hit_rate();

    // Invalidate only one domain
    cache.invalidate_domain("web-api");

    // Query all domains again
    for domain in &domains {
        for i in 0..5 {
            let key = CacheKey::new(format!("query-{}-{}", domain, i))
                .with_domain(Some(domain.to_string()));
            let _ = cache.get(&key); // web-api: miss, others: hit
        }
    }

    let metrics_after = cache.metrics();
    let hit_rate_after = metrics_after.hit_rate();

    // With domain-based invalidation, we still get hits for 2/3 domains
    // Without domain-based invalidation (invalidate_all), we'd get 0 hits
    assert!(hit_rate_after > 0.6); // At least 66% hit rate (2 out of 3 domains)
    println!(
        "Hit rate before: {:.1}%, after: {:.1}%",
        hit_rate_before * 100.0,
        hit_rate_after * 100.0
    );
}

#[test]
fn test_domain_isolation_correctness() {
    let cache = QueryCache::new();

    // Same query text, different domains
    let query_text = "implement feature X".to_string();

    let key_web = CacheKey::new(query_text.clone()).with_domain(Some("web-api".to_string()));
    let key_data =
        CacheKey::new(query_text.clone()).with_domain(Some("data-processing".to_string()));

    // Different results for different domains
    let web_episodes = vec![create_episode(
        "00000000-0000-0000-0000-000000000001",
        "web-api",
    )];
    let data_episodes = vec![create_episode(
        "00000000-0000-0000-0000-000000000002",
        "data-processing",
    )];

    cache.put(key_web.clone(), web_episodes.clone());
    cache.put(key_data.clone(), data_episodes.clone());

    // Verify both are cached
    let cached_web = cache.get(&key_web).expect("web-api should be cached");
    let cached_data = cache.get(&key_data).expect("data should be cached");

    assert_eq!(cached_web.len(), 1);
    assert_eq!(cached_data.len(), 1);
    assert_eq!(cached_web[0].context.domain, "web-api");
    assert_eq!(cached_data[0].context.domain, "data-processing");

    // Invalidate web-api
    cache.invalidate_domain("web-api");

    // Web-api cleared, data-processing remains
    assert!(cache.get(&key_web).is_none());
    let cached_data_after = cache.get(&key_data).expect("data should still be cached");
    assert_eq!(cached_data_after.len(), 1);
}

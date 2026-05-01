//! Tests for scope-before-search shard routing (WG-122).

use super::*;
use chrono::{Duration, Utc};

#[test]
fn test_shard_config_default() {
    let config = ShardConfig::default();
    assert_eq!(config.max_candidates, 100);
    assert_eq!(config.min_candidates, 10);
    assert!((config.tag_weight - 0.4).abs() < 0.01);
    assert!(config.use_temporal_decay);
    assert_eq!(config.stale_days, 30);
}

#[test]
fn test_shard_config_large_dataset() {
    let config = ShardConfig::large_dataset();
    assert_eq!(config.max_candidates, 500);
    assert_eq!(config.min_candidates, 50);
}

#[test]
fn test_shard_config_small_dataset() {
    let config = ShardConfig::small_dataset();
    assert_eq!(config.max_candidates, 50);
    assert_eq!(config.min_candidates, 5);
}

#[test]
fn test_shard_config_recent_focused() {
    let config = ShardConfig::recent_focused();
    assert!((config.timeframe_weight - 0.6).abs() < 0.01);
    assert_eq!(config.stale_days, 7);
}

#[test]
fn test_scope_filter_creation() {
    let filter = ScopeFilter::new();
    assert!(filter.required_tags.is_empty());
    assert!(filter.excluded_tags.is_empty());
    assert!(filter.required_task_types.is_empty());
    assert!(filter.time_range.is_none());
    assert!(filter.min_success_rate.is_none());
    assert!(!filter.has_constraints());
}

#[test]
fn test_scope_filter_from_query() {
    let filter = ScopeFilter::from_query_text("fix bug in recent feature");
    assert!(filter.has_constraints());
    assert!(filter.required_tags.contains("fix"));
    assert!(filter.required_tags.contains("bug"));
    assert!(filter.required_tags.contains("feature"));
    assert!(filter.time_range.is_some());
}

#[test]
fn test_scope_filter_add_constraints() {
    let mut filter = ScopeFilter::new();
    filter.require_tag("security");
    filter.exclude_tag("wasm");
    filter.require_task_type("debugging");
    filter.set_min_success_rate(0.8);

    assert!(filter.required_tags.contains("security"));
    assert!(filter.excluded_tags.contains("wasm"));
    assert!(filter.required_task_types.contains("debugging"));
    assert!((filter.min_success_rate.unwrap() - 0.8).abs() < 0.01);
    assert_eq!(filter.constraint_count(), 4);
}

#[test]
fn test_time_range_recent_days() {
    let now = Utc::now();
    // Create range with end slightly in future to avoid race condition
    let range = TimeRange::new(now - Duration::days(7), now + Duration::seconds(1));
    assert_eq!(range.duration_days(), 7);

    // Now should be in range
    assert!(range.contains(Utc::now()));

    // 10 days ago should not
    let old = Utc::now() - Duration::days(10);
    assert!(!range.contains(old));
}

#[test]
fn test_time_range_today() {
    let now = Utc::now();
    // Create range with end slightly in future to avoid race condition
    let range = TimeRange::new(
        now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc(),
        now + Duration::seconds(1),
    );
    assert!(range.contains(Utc::now()));
}

#[test]
fn test_time_range_contains() {
    let start = Utc::now() - Duration::days(5);
    let end = Utc::now() + Duration::days(5);
    let range = TimeRange::new(start, end);

    assert!(range.contains(Utc::now()));
    assert!(!range.contains(Utc::now() - Duration::days(10)));
}

#[test]
fn test_episode_metadata_creation() {
    let episode_id = Uuid::new_v4();
    let tags: HashSet<String> = ["bug", "fix"].iter().map(|s| (*s).to_string()).collect();
    let meta = EpisodeMetadata::new(
        episode_id,
        tags.clone(),
        "debugging".to_string(),
        Utc::now(),
    );

    assert_eq!(meta.episode_id, episode_id);
    assert_eq!(meta.tags, tags);
    assert_eq!(meta.task_type, "debugging");
    assert!(!meta.is_complete);
}

#[test]
fn test_episode_metadata_helpers() {
    let episode_id = Uuid::new_v4();
    let tags: HashSet<String> = ["bug", "fix"].iter().map(|s| (*s).to_string()).collect();
    let mut meta = EpisodeMetadata::new(episode_id, tags, "debugging".to_string(), Utc::now());

    meta.set_success_rate(0.75);
    meta.mark_complete();

    assert!((meta.success_rate - 0.75).abs() < 0.01);
    assert!(meta.is_complete);
    assert!(meta.has_tag("bug"));
    assert!(meta.matches_task_type("debugging"));
    assert_eq!(meta.age_days(), 0);
}

#[test]
fn test_shard_router_creation() {
    let router = ShardRouter::default_config();
    assert_eq!(router.config().max_candidates, 100);
}

#[test]
fn test_route_empty_episodes() {
    let router = ShardRouter::default_config();
    let filter = ScopeFilter::new();
    let result = router.route(&filter, &[]);

    assert!(result.is_empty());
    assert_eq!(result.original_count, 0);
}

#[test]
fn test_route_no_constraints() {
    let router = ShardRouter::default_config();
    let filter = ScopeFilter::new();

    let ep1 = Uuid::new_v4();
    let ep2 = Uuid::new_v4();
    let episodes = [
        EpisodeMetadata::new(ep1, HashSet::new(), "test".to_string(), Utc::now()),
        EpisodeMetadata::new(ep2, HashSet::new(), "test".to_string(), Utc::now()),
    ];

    let result = router.route(&filter, &episodes);

    // Without constraints, all should pass
    assert_eq!(result.len(), 2);
    assert_eq!(result.original_count, 2);
    assert!(!result.capped);
}

#[test]
fn test_route_tag_filter() {
    let router = ShardRouter::default_config();
    let mut filter = ScopeFilter::new();
    filter.require_tag("bug");

    let ep1 = Uuid::new_v4();
    let tags1: HashSet<String> = ["bug", "fix"].iter().map(|s| (*s).to_string()).collect();
    let ep2 = Uuid::new_v4();
    let tags2: HashSet<String> = ["feature"].iter().map(|s| (*s).to_string()).collect();

    let episodes = [
        EpisodeMetadata::new(ep1, tags1, "debugging".to_string(), Utc::now()),
        EpisodeMetadata::new(ep2, tags2, "implementation".to_string(), Utc::now()),
    ];

    let result = router.route(&filter, &episodes);

    // Only ep1 has the required tag
    assert_eq!(result.len(), 1);
    assert_eq!(result.candidates[0], ep1);
}

#[test]
fn test_route_excluded_tag() {
    let router = ShardRouter::default_config();
    let mut filter = ScopeFilter::new();
    filter.exclude_tag("wasm");

    let ep1 = Uuid::new_v4();
    let tags1: HashSet<String> = ["rust", "core"].iter().map(|s| (*s).to_string()).collect();
    let ep2 = Uuid::new_v4();
    let tags2: HashSet<String> = ["wasm", "sandbox"]
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    let episodes = [
        EpisodeMetadata::new(ep1, tags1, "test".to_string(), Utc::now()),
        EpisodeMetadata::new(ep2, tags2, "test".to_string(), Utc::now()),
    ];

    let result = router.route(&filter, &episodes);

    // ep2 should be excluded
    assert_eq!(result.len(), 1);
    assert_eq!(result.candidates[0], ep1);
}

#[test]
fn test_route_task_type_filter() {
    let router = ShardRouter::default_config();
    let mut filter = ScopeFilter::new();
    filter.require_task_type("debugging");

    let ep1 = Uuid::new_v4();
    let ep2 = Uuid::new_v4();
    let episodes = [
        EpisodeMetadata::new(ep1, HashSet::new(), "debugging".to_string(), Utc::now()),
        EpisodeMetadata::new(
            ep2,
            HashSet::new(),
            "implementation".to_string(),
            Utc::now(),
        ),
    ];

    let result = router.route(&filter, &episodes);

    assert_eq!(result.len(), 1);
    assert_eq!(result.candidates[0], ep1);
}

#[test]
fn test_route_time_range() {
    let router = ShardRouter::default_config();
    let now = Utc::now();
    // Use range with end slightly in future to avoid race condition
    let mut filter = ScopeFilter::new();
    filter.set_time_range(TimeRange::new(
        now - Duration::days(7),
        now + Duration::seconds(1),
    ));

    let ep1 = Uuid::new_v4();
    let ep2 = Uuid::new_v4();
    let old_time = now - Duration::days(30);

    let episodes = [
        EpisodeMetadata::new(ep1, HashSet::new(), "test".to_string(), now),
        EpisodeMetadata::new(ep2, HashSet::new(), "test".to_string(), old_time),
    ];

    let result = router.route(&filter, &episodes);

    // Only recent episode should pass
    assert_eq!(result.len(), 1);
    assert_eq!(result.candidates[0], ep1);
}

#[test]
fn test_route_success_rate_filter() {
    let router = ShardRouter::default_config();
    let mut filter = ScopeFilter::new();
    filter.set_min_success_rate(0.5);

    let ep1 = Uuid::new_v4();
    let mut meta1 = EpisodeMetadata::new(ep1, HashSet::new(), "test".to_string(), Utc::now());
    meta1.set_success_rate(0.8);

    let ep2 = Uuid::new_v4();
    let mut meta2 = EpisodeMetadata::new(ep2, HashSet::new(), "test".to_string(), Utc::now());
    meta2.set_success_rate(0.3);

    let episodes = [meta1, meta2];
    let result = router.route(&filter, &episodes);

    // Only high success rate should pass
    assert_eq!(result.len(), 1);
    assert_eq!(result.candidates[0], ep1);
}

#[test]
fn test_route_max_candidates_cap() {
    let config = ShardConfig {
        max_candidates: 2,
        ..ShardConfig::default()
    };
    let router = ShardRouter::new(config);
    let filter = ScopeFilter::new();

    let episodes: Vec<EpisodeMetadata> = (0..5)
        .map(|_| {
            let id = Uuid::new_v4();
            EpisodeMetadata::new(id, HashSet::new(), "test".to_string(), Utc::now())
        })
        .collect();

    let result = router.route(&filter, &episodes);

    assert!(result.capped);
    assert_eq!(result.len(), 2);
    assert_eq!(result.original_count, 5);
}

#[test]
fn test_routing_result_helpers() {
    let filter = ScopeFilter::new();
    let result = RoutingResult {
        candidates: vec![Uuid::new_v4(), Uuid::new_v4()],
        original_count: 10,
        filtered_count: 2,
        capped: false,
        filter,
        scores: vec![0.8, 0.6],
    };

    assert_eq!(result.len(), 2);
    assert!(!result.is_empty());
    assert!((result.reduction_ratio() - 0.8).abs() < 0.01);
}

#[test]
fn test_routing_result_empty() {
    let filter = ScopeFilter::new();
    let result = RoutingResult::empty(filter.clone());

    assert!(result.is_empty());
    assert_eq!(result.reduction_ratio(), 0.0);
}

#[test]
fn test_estimate_reduction() {
    let router = ShardRouter::default_config();

    let filter = ScopeFilter::new();
    assert!((router.estimate_reduction(&filter, 100) - 0.0).abs() < 1.0);

    let mut constrained = ScopeFilter::new();
    constrained.require_tag("bug");
    constrained.require_task_type("debugging");
    constrained.set_time_range(TimeRange::recent_days(7));

    let estimate = router.estimate_reduction(&constrained, 100);
    assert!(estimate > 0.0);
    assert!(estimate <= 80.0);
}

#[test]
fn test_temporal_decay_very_recent() {
    let router = ShardRouter::default_config();
    let filter = ScopeFilter::new();

    let ep = Uuid::new_v4();
    let meta = EpisodeMetadata::new(ep, HashSet::new(), "test".to_string(), Utc::now());

    let result = router.route(&filter, &[meta]);
    // Very recent should have high score (neutral baseline + full temporal decay)
    assert!(!result.scores.is_empty());
    assert!(result.scores[0] >= 0.4); // 0.5 neutral base * 1.0 decay = 0.5
}

#[test]
fn test_temporal_decay_stale() {
    let router = ShardRouter::default_config();
    let filter = ScopeFilter::new();

    let ep = Uuid::new_v4();
    let old_time = Utc::now() - Duration::days(60);
    let meta = EpisodeMetadata::new(ep, HashSet::new(), "test".to_string(), old_time);

    let result = router.route(&filter, &[meta]);
    // Stale should have lower score
    assert!(!result.scores.is_empty());
    assert!(result.scores[0] < 0.5);
}

#[test]
fn test_combined_filters() {
    let router = ShardRouter::default_config();
    let now = Utc::now();
    // Use range with end slightly in future to avoid race condition
    let mut filter = ScopeFilter::new();
    filter.require_tag("security");
    filter.require_task_type("debugging");
    filter.set_time_range(TimeRange::new(
        now - Duration::days(7),
        now + Duration::seconds(1),
    ));
    filter.set_min_success_rate(0.5);

    let ep1 = Uuid::new_v4();
    let tags1: HashSet<String> = ["security", "fix"]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let mut meta1 = EpisodeMetadata::new(ep1, tags1, "debugging".to_string(), now);
    meta1.set_success_rate(0.8);

    let ep2 = Uuid::new_v4();
    let tags2: HashSet<String> = ["feature"].iter().map(|s| (*s).to_string()).collect();
    let mut meta2 = EpisodeMetadata::new(ep2, tags2, "implementation".to_string(), now);
    meta2.set_success_rate(0.9);

    let result = router.route(&filter, &[meta1, meta2]);

    // Only ep1 matches all criteria
    assert_eq!(result.len(), 1);
    assert_eq!(result.candidates[0], ep1);
}

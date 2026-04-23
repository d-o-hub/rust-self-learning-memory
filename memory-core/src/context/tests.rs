//! Integration and comprehensive tests for context bundle module.

use crate::context::{BundleAccumulator, BundleConfig, BundleStats, ContextItem, ContextItemType};
use crate::episode::Episode;
use crate::pattern::Pattern;
use crate::types::{TaskContext, TaskType};
use chrono::{Duration, Utc};
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// ContextItem Tests
// ============================================================================

/// Helper function to create test episodes for use in edge case tests.
pub(super) fn create_test_episode_with_id(description: &str, days_ago: i64) -> Arc<Episode> {
    let mut episode = Episode::new(
        description.to_string(),
        TaskContext::default(),
        TaskType::Debugging,
    );
    episode.start_time = Utc::now() - Duration::days(days_ago);
    Arc::new(episode)
}

fn create_test_pattern_with_id(name: &str, days_ago: i64) -> Arc<Pattern> {
    use crate::pattern::PatternEffectiveness;
    use chrono::Duration as ChronoDuration;

    // Create a ToolSequence pattern for testing
    let pattern = Pattern::ToolSequence {
        id: Uuid::new_v4(),
        tools: vec![name.to_string(), "test".to_string()],
        context: TaskContext::default(),
        success_rate: 0.8,
        avg_latency: ChronoDuration::milliseconds(100),
        occurrence_count: 10,
        effectiveness: PatternEffectiveness {
            times_retrieved: 5,
            times_applied: 8,
            success_when_applied: 6,
            failure_when_applied: 2,
            avg_reward_delta: 0.2,
            last_used: Utc::now() - ChronoDuration::days(days_ago),
            created_at: Utc::now() - ChronoDuration::days(days_ago),
        },
    };
    Arc::new(pattern)
}

#[test]
fn test_context_item_from_episode() {
    let episode = create_test_episode_with_id("test episode", 0);
    let item = ContextItem::from_episode(episode.clone(), 0.85);

    assert_eq!(item.item_type(), ContextItemType::Episode);
    assert_eq!(item.salience(), 0.85);
    assert!(item.as_episode().is_some());
    assert!(item.as_pattern().is_none());
    assert_eq!(item.id(), episode.episode_id);
}

#[test]
fn test_context_item_from_pattern() {
    let pattern = create_test_pattern_with_id("test pattern", 5);
    let item = ContextItem::from_pattern(pattern.clone(), 0.75);

    assert_eq!(item.item_type(), ContextItemType::Pattern);
    assert_eq!(item.salience(), 0.75);
    assert!(item.as_pattern().is_some());
    assert!(item.as_episode().is_none());
    assert_eq!(item.id(), pattern.id());
}

#[test]
fn test_context_item_salience_clamping() {
    let episode = create_test_episode_with_id("test", 0);

    // Salience should be clamped to 0.0-1.0 range
    let item_high = ContextItem::from_episode(episode.clone(), 1.5);
    assert_eq!(item_high.salience(), 1.0);

    let item_low = ContextItem::from_episode(episode.clone(), -0.5);
    assert_eq!(item_low.salience(), 0.0);
}

#[test]
fn test_context_item_priority_setter() {
    let episode = create_test_episode_with_id("test", 0);
    let mut item = ContextItem::from_episode(episode, 0.5);

    assert_eq!(item.priority(), 0.0); // Default

    item.set_priority(0.75);
    assert_eq!(item.priority(), 0.75);

    // Priority should also clamp
    item.set_priority(2.0);
    assert_eq!(item.priority(), 1.0);
}

#[test]
fn test_context_item_summary() {
    let episode = create_test_episode_with_id("Test episode description", 0);
    let mut item = ContextItem::from_episode(episode, 0.85);
    item.set_priority(0.70);

    let summary = item.summary();
    assert!(summary.contains("[Episode"));
    assert!(summary.contains("salience=0.85"));
    assert!(summary.contains("priority=0.70"));
}

// ============================================================================
// BundleConfig Tests
// ============================================================================

#[test]
fn test_bundle_config_default() {
    let config = BundleConfig::default();

    assert_eq!(config.max_items, 20);
    assert!((config.recency_weight + config.salience_weight - 0.8).abs() < 0.1);
    assert!(config.validate().is_ok());
}

#[test]
fn test_bundle_config_token_efficient() {
    let config = BundleConfig::token_efficient();

    assert_eq!(config.max_items, 10);
    assert_eq!(config.min_salience_threshold, 0.5);
    assert!(config.validate().is_ok());
}

#[test]
fn test_bundle_config_comprehensive() {
    let config = BundleConfig::comprehensive();

    assert_eq!(config.max_items, 50);
    assert_eq!(config.min_salience_threshold, 0.1);
    assert!(config.validate().is_ok());
}

#[test]
fn test_bundle_config_validation_errors() {
    // Zero max_items
    let invalid_zero = BundleConfig {
        max_items: 0,
        ..BundleConfig::default()
    };
    assert!(invalid_zero.validate().is_err());

    // Invalid weight sum (too far from 1.0)
    let invalid_weights = BundleConfig {
        recency_weight: 0.1,
        salience_weight: 0.1,
        ..BundleConfig::default()
    };
    assert!(invalid_weights.validate().is_err());

    // Invalid threshold
    let invalid_threshold = BundleConfig {
        min_salience_threshold: 1.5,
        ..BundleConfig::default()
    };
    assert!(invalid_threshold.validate().is_err());

    // Negative half-life
    let invalid_halflife = BundleConfig {
        recency_half_life_days: -5.0,
        ..BundleConfig::default()
    };
    assert!(invalid_halflife.validate().is_err());
}

// ============================================================================
// BundleAccumulator Tests
// ============================================================================

#[test]
fn test_accumulator_empty() {
    let acc = BundleAccumulator::default_config();

    assert!(acc.is_empty());
    assert_eq!(acc.size(), 0);
    assert!(!acc.is_full());
    assert!(acc.peek_items().is_empty());
}

#[test]
fn test_accumulator_add_accepted() {
    let mut acc = BundleAccumulator::default_config();
    let episode = create_test_episode_with_id("test", 0);
    let item = ContextItem::from_episode(episode, 0.8);

    let result = acc.add(item);

    assert!(result.accepted);
    assert_eq!(result.current_size, 1);
    assert!(result.evicted_id.is_none());
    assert!(result.rejection_reason.is_none());
    assert_eq!(acc.size(), 1);
    assert_eq!(acc.stats().total_added, 1);
}

#[test]
fn test_accumulator_add_rejected_below_threshold() {
    let config = BundleConfig {
        min_salience_threshold: 0.5,
        ..BundleConfig::default()
    };
    let mut acc = BundleAccumulator::new(config);

    let episode = create_test_episode_with_id("test", 0);
    let item = ContextItem::from_episode(episode, 0.3); // Below threshold

    let result = acc.add(item);

    assert!(!result.accepted);
    assert_eq!(acc.stats().total_rejected, 1);
    assert!(result.rejection_reason.is_some());
    assert!(result.rejection_reason.unwrap().contains("threshold"));
}

#[test]
fn test_accumulator_eviction() {
    let config = BundleConfig {
        max_items: 3,
        ..BundleConfig::default()
    };
    let mut acc = BundleAccumulator::new(config);

    // Add three items with different priorities
    let ep1 = create_test_episode_with_id("high priority recent", 0);
    acc.add(ContextItem::from_episode(ep1, 0.9));

    let ep2 = create_test_episode_with_id("medium priority", 15);
    acc.add(ContextItem::from_episode(ep2, 0.5));

    let ep3 = create_test_episode_with_id("low priority old", 30);
    acc.add(ContextItem::from_episode(ep3, 0.3));

    assert_eq!(acc.size(), 3);
    assert!(acc.is_full());

    // Add fourth item - should evict lowest priority (ep3)
    let ep4 = create_test_episode_with_id("new high", 0);
    let result = acc.add(ContextItem::from_episode(ep4, 0.95));

    assert!(result.accepted);
    assert!(result.evicted_id.is_some());
    assert_eq!(acc.stats().total_evicted, 1);
    assert_eq!(acc.size(), 3); // Still at capacity
}

#[test]
fn test_accumulator_to_bundle_sorted() {
    let mut acc = BundleAccumulator::default_config();

    // Add items with different priorities
    let ep_low = create_test_episode_with_id("low", 30);
    acc.add(ContextItem::from_episode(ep_low, 0.3));

    let ep_high = create_test_episode_with_id("high", 0);
    acc.add(ContextItem::from_episode(ep_high, 0.9));

    let ep_med = create_test_episode_with_id("medium", 15);
    acc.add(ContextItem::from_episode(ep_med, 0.5));

    let bundle = acc.to_bundle();

    // Should be sorted by priority (descending)
    assert!(bundle.len() == 3);
    assert!(bundle[0].priority() >= bundle[1].priority());
    assert!(bundle[1].priority() >= bundle[2].priority());
}

#[test]
fn test_accumulator_batch_add() {
    let mut acc = BundleAccumulator::default_config();

    let items: Vec<ContextItem> = (0..5)
        .map(|i| {
            let ep = create_test_episode_with_id(format!("ep{i}").as_str(), i);
            ContextItem::from_episode(ep, 0.5 + i as f32 * 0.1)
        })
        .collect();

    let results = acc.add_batch(items);

    assert_eq!(results.len(), 5);
    assert!(results.iter().all(|r| r.accepted));
    assert_eq!(acc.size(), 5);
}

#[test]
fn test_accumulator_mixed_types() {
    let mut acc = BundleAccumulator::default_config();

    // Add episode
    let ep = create_test_episode_with_id("episode", 0);
    acc.add(ContextItem::from_episode(ep, 0.8));

    // Add pattern
    let pat = create_test_pattern_with_id("pattern", 5);
    acc.add(ContextItem::from_pattern(pat, 0.7));

    assert_eq!(acc.size(), 2);

    let episodes = acc.episodes_only();
    let patterns = acc.patterns_only();

    assert_eq!(episodes.len(), 1);
    assert_eq!(patterns.len(), 1);
}

#[test]
fn test_accumulator_stats() {
    let config = BundleConfig {
        max_items: 2,
        min_salience_threshold: 0.3,
        ..BundleConfig::default()
    };
    let mut acc = BundleAccumulator::new(config);

    // Add accepted items
    acc.add(ContextItem::from_episode(
        create_test_episode_with_id("ep1", 0),
        0.8,
    ));
    acc.add(ContextItem::from_episode(
        create_test_episode_with_id("ep2", 10),
        0.7,
    ));

    // Try to add rejected (below threshold)
    acc.add(ContextItem::from_episode(
        create_test_episode_with_id("ep3", 0),
        0.2,
    ));

    // Add third accepted (causes eviction)
    acc.add(ContextItem::from_episode(
        create_test_episode_with_id("ep4", 0),
        0.9,
    ));

    let _bundle = acc.to_bundle();
    let stats = acc.stats();

    assert_eq!(stats.total_added, 3);
    assert_eq!(stats.total_rejected, 1);
    assert_eq!(stats.total_evicted, 1);
    assert_eq!(stats.current_size, 2);
    assert!(stats.average_salience > 0.0);
    assert!(stats.oldest_timestamp.is_some());
    assert!(stats.newest_timestamp.is_some());
}

#[test]
fn test_accumulator_contains_get_remove() {
    let mut acc = BundleAccumulator::default_config();

    let ep1 = create_test_episode_with_id("ep1", 0);
    let id1 = ep1.episode_id;
    acc.add(ContextItem::from_episode(ep1, 0.8));

    let ep2 = create_test_episode_with_id("ep2", 0);
    let id2 = ep2.episode_id;
    acc.add(ContextItem::from_episode(ep2, 0.7));

    // Contains
    assert!(acc.contains(id1));
    assert!(acc.contains(id2));

    // Get
    let item1 = acc.get(id1);
    assert!(item1.is_some());
    assert_eq!(item1.unwrap().id(), id1);

    // Remove
    assert!(acc.remove(id1));
    assert!(!acc.contains(id1));
    assert_eq!(acc.size(), 1);

    // Remove non-existent
    assert!(!acc.remove(id1));
    assert!(!acc.remove(Uuid::new_v4()));
}

#[test]
fn test_accumulator_clear() {
    let mut acc = BundleAccumulator::default_config();

    acc.add(ContextItem::from_episode(
        create_test_episode_with_id("ep1", 0),
        0.8,
    ));
    acc.add(ContextItem::from_episode(
        create_test_episode_with_id("ep2", 0),
        0.7,
    ));

    assert_eq!(acc.size(), 2);

    acc.clear();

    assert!(acc.is_empty());
    assert_eq!(acc.stats().total_added, 0);
}

#[test]
fn test_from_episodes_convenience() {
    let episodes: Vec<Arc<Episode>> = (0..5)
        .map(|i| create_test_episode_with_id(format!("ep{i}").as_str(), i))
        .collect();

    // Simple salience function (based on reward)
    let bundle = BundleAccumulator::from_episodes(episodes.clone(), |ep| {
        ep.reward.as_ref().map_or(0.5, |r| r.total)
    });

    assert_eq!(bundle.len(), 5);
    assert!(
        bundle
            .iter()
            .all(|item| item.item_type() == ContextItemType::Episode)
    );
}

#[test]
fn test_from_episodes_with_config() {
    let episodes: Vec<Arc<Episode>> = (0..15)
        .map(|i| create_test_episode_with_id(format!("ep{i}").as_str(), i))
        .collect();

    // Use token-efficient config (max 10)
    let bundle = BundleAccumulator::from_episodes_with_config(
        episodes,
        BundleConfig::token_efficient(),
        |_| 0.8,
    );

    // Should be bounded to max_items
    assert!(bundle.len() <= 10);
}

// ============================================================================
// BundleStats Tests
// ============================================================================

#[test]
fn test_bundle_stats_fill_percentage() {
    let stats = BundleStats {
        current_size: 10,
        ..BundleStats::default()
    };

    assert_eq!(stats.fill_percentage(20), 50.0);
    assert_eq!(stats.fill_percentage(10), 100.0);
    assert_eq!(stats.fill_percentage(0), 0.0);
}

#[test]
fn test_bundle_stats_acceptance_rate() {
    let stats = BundleStats {
        total_added: 8,
        total_rejected: 2,
        ..BundleStats::default()
    };

    assert_eq!(stats.acceptance_rate(), 80.0);

    let empty_stats = BundleStats::default();
    assert_eq!(empty_stats.acceptance_rate(), 0.0);
}

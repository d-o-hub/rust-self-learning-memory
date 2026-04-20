//! Edge cases and boundary tests for context bundle module.

use crate::context::{BundleAccumulator, BundleConfig, ContextItem};
use crate::episode::Episode;
use crate::types::{TaskContext, TaskType};
use chrono::{Duration, Utc};
use std::sync::Arc;

use super::tests::create_test_episode_with_id;

#[test]
fn test_accumulator_future_timestamp() {
    let mut acc = BundleAccumulator::default_config();

    // Create episode with future timestamp
    let mut episode = Episode::new(
        "future".to_string(),
        TaskContext::default(),
        TaskType::Other,
    );
    episode.start_time = Utc::now() + Duration::days(5);
    let item = ContextItem::from_episode(Arc::new(episode), 0.8);

    let result = acc.add(item);
    assert!(result.accepted);

    // Future items should have high recency score
    let bundle = acc.to_bundle();
    assert_eq!(bundle.len(), 1);
    assert!(bundle[0].priority() > 0.5);
}

#[test]
fn test_accumulator_single_capacity() {
    let config = BundleConfig {
        max_items: 1,
        ..BundleConfig::default()
    };
    let mut acc = BundleAccumulator::new(config);

    acc.add(ContextItem::from_episode(
        create_test_episode_with_id("ep1", 0),
        0.8,
    ));
    assert!(acc.is_full());

    // Adding another should evict first
    let ep2 = create_test_episode_with_id("ep2", 0);
    let id2 = ep2.episode_id;
    let result = acc.add(ContextItem::from_episode(ep2, 0.9));

    assert!(result.accepted);
    assert!(result.evicted_id.is_some());
    assert!(acc.contains(id2));
}

#[test]
fn test_accumulator_duplicate_id() {
    let mut acc = BundleAccumulator::default_config();

    let ep = create_test_episode_with_id("same", 0);
    let id = ep.episode_id;

    // Add same episode twice (different salience)
    acc.add(ContextItem::from_episode(ep.clone(), 0.8));
    acc.add(ContextItem::from_episode(ep.clone(), 0.9));

    // Both should be accepted (we allow duplicates by ID for flexibility)
    // The bundle will have both items
    assert_eq!(acc.size(), 2);

    // But contains should find the first
    assert!(acc.contains(id));
}

#[test]
fn test_accumulator_zero_threshold() {
    let config = BundleConfig {
        min_salience_threshold: 0.0,
        ..BundleConfig::default()
    };
    let mut acc = BundleAccumulator::new(config);

    // Very low salience should be accepted
    let result = acc.add(ContextItem::from_episode(
        create_test_episode_with_id("low", 0),
        0.01,
    ));

    assert!(result.accepted);
}

#[test]
fn test_accumulator_equal_priorities() {
    let mut acc = BundleAccumulator::default_config();

    // Add items with identical salience and recency
    let ep1 = create_test_episode_with_id("ep1", 0);
    let ep2 = create_test_episode_with_id("ep2", 0);

    acc.add(ContextItem::from_episode(ep1, 0.8));
    acc.add(ContextItem::from_episode(ep2, 0.8));

    let bundle = acc.to_bundle();

    // Both should be present, order is arbitrary but consistent
    assert_eq!(bundle.len(), 2);
}

//! Scoring functions for context bundle items.
//!
//! Provides recency and priority scoring based on configurable weights.

use crate::context::types::{BundleConfig, ContextItem};
use chrono::{DateTime, Utc};

/// Calculate recency score using exponential decay.
///
/// More recent items get higher scores. Uses configurable half-life.
///
/// # Arguments
///
/// * `timestamp` - When the item was created/added to memory
/// * `now` - Current time reference
/// * `half_life_days` - Half-life for decay (items this old get 0.5 score)
///
/// # Returns
///
/// A recency score between 0.0 and 1.0
#[must_use]
pub fn calculate_recency_score(
    timestamp: DateTime<Utc>,
    now: DateTime<Utc>,
    half_life_days: f32,
) -> f32 {
    let age_days = (now - timestamp).num_days() as f32;

    // Handle negative age (future timestamps) - give them high score
    if age_days < 0.0 {
        return 1.0;
    }

    // Exponential decay: score = 0.5^(age / half_life)
    // Items at half_life get 0.5, items at 2*half_life get 0.25, etc.
    0.5_f32.powf(age_days / half_life_days)
}

/// Calculate priority score combining recency and salience.
///
/// # Arguments
///
/// * `item` - The context item to score
/// * `config` - Bundle configuration with weights
/// * `now` - Current time reference
///
/// # Returns
///
/// A priority score between 0.0 and 1.0
#[must_use]
pub fn calculate_priority_score(
    item: &ContextItem,
    config: &BundleConfig,
    now: DateTime<Utc>,
) -> f32 {
    let recency = calculate_recency_score(item.timestamp(), now, config.recency_half_life_days);
    let salience = item.salience();

    // Weighted combination
    let priority = recency * config.recency_weight + salience * config.salience_weight;

    // Normalize to account for potential weight imbalance
    let weight_sum = config.recency_weight + config.salience_weight;
    if weight_sum > 0.0 {
        priority / weight_sum
    } else {
        0.0
    }
}

/// Compare two items by priority score (for sorting).
///
/// Returns `Ordering::Less` if `a` has higher priority than `b`,
/// suitable for sorting in descending order (highest priority first).
#[must_use]
pub fn compare_by_priority(a: &ContextItem, b: &ContextItem) -> std::cmp::Ordering {
    // Higher priority = comes first (Less ordering for descending sort)
    b.priority()
        .partial_cmp(&a.priority())
        .unwrap_or(std::cmp::Ordering::Equal)
}

/// Compare two items by recency (most recent first).
#[must_use]
pub fn compare_by_recency(a: &ContextItem, b: &ContextItem) -> std::cmp::Ordering {
    // More recent = comes first (descending time order)
    b.timestamp().cmp(&a.timestamp())
}

/// Compare two items by salience (highest first).
#[must_use]
pub fn compare_by_salience(a: &ContextItem, b: &ContextItem) -> std::cmp::Ordering {
    b.salience()
        .partial_cmp(&a.salience())
        .unwrap_or(std::cmp::Ordering::Equal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::Episode;
    use crate::types::{TaskContext, TaskType};
    use std::sync::Arc;

    fn create_test_episode(description: &str, days_ago: i64) -> Arc<Episode> {
        let mut episode = Episode::new(
            description.to_string(),
            TaskContext::default(),
            TaskType::Debugging,
        );
        // Set start_time to days_ago
        episode.start_time = Utc::now() - chrono::Duration::days(days_ago);
        Arc::new(episode)
    }

    #[test]
    fn test_recency_score_recent() {
        let now = Utc::now();
        let recent = now - chrono::Duration::days(1);

        let score = calculate_recency_score(recent, now, 30.0);
        assert!(score > 0.9, "Recent item should have high score: {}", score);
    }

    #[test]
    fn test_recency_score_half_life() {
        let now = Utc::now();
        let half_life_old = now - chrono::Duration::days(30);

        let score = calculate_recency_score(half_life_old, now, 30.0);
        assert!(
            (score - 0.5).abs() < 0.01,
            "Item at half_life should have ~0.5 score: {}",
            score
        );
    }

    #[test]
    fn test_recency_score_old() {
        let now = Utc::now();
        let old = now - chrono::Duration::days(90);

        let score = calculate_recency_score(old, now, 30.0);
        assert!(score < 0.2, "Old item should have low score: {}", score);
    }

    #[test]
    fn test_priority_score_balanced_weights() {
        let config = BundleConfig::new(20, 0.5, 0.5);
        let episode = create_test_episode("test", 0);
        let item = ContextItem::from_episode(episode, 0.8);

        let now = Utc::now();
        let priority = calculate_priority_score(&item, &config, now);

        // Recent item (recency=1.0) with salience=0.8, weights=0.5/0.5
        // Priority = (1.0*0.5 + 0.8*0.5) / 1.0 = 0.9
        assert!(
            (priority - 0.9).abs() < 0.01,
            "Expected ~0.9 priority: {}",
            priority
        );
    }

    #[test]
    fn test_priority_score_recency_focused() {
        let config = BundleConfig::new(20, 0.8, 0.2);
        let episode = create_test_episode("test", 30); // 30 days old
        let item = ContextItem::from_episode(episode, 0.9); // High salience

        let now = Utc::now();
        let priority = calculate_priority_score(&item, &config, now);

        // 30 days old with half_life=30 -> recency=0.5
        // Priority = (0.5*0.8 + 0.9*0.2) / 1.0 = 0.4 + 0.18 = 0.58
        // But we use default half_life=30.0
        let recency = calculate_recency_score(item.timestamp(), now, config.recency_half_life_days);
        let expected = (recency * 0.8 + 0.9 * 0.2) / 1.0;
        assert!(
            (priority - expected).abs() < 0.01,
            "Expected {}: {}",
            expected,
            priority
        );
    }

    #[test]
    fn test_compare_by_priority() {
        let ep1 = create_test_episode("high priority", 0);
        let ep2 = create_test_episode("low priority", 30);

        let mut item1 = ContextItem::from_episode(ep1, 0.9);
        let mut item2 = ContextItem::from_episode(ep2, 0.5);

        item1.set_priority(0.9);
        item2.set_priority(0.3);

        // Higher priority item should come first (Less ordering)
        assert_eq!(
            compare_by_priority(&item1, &item2),
            std::cmp::Ordering::Less
        );
    }
}

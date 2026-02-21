//! Eviction logic for capacity management.
//!
//! Implements different eviction policies for determining which episodes
//! to remove when storage capacity is reached.

use super::super::policy::EvictionPolicy;
use super::scoring::calculate_relevance_score;
use crate::episode::Episode;
use std::cmp::Ordering;
use uuid::Uuid;

/// Determine which episodes to evict if needed.
///
/// Returns episode IDs to evict to make room for new episodes.
/// The number of episodes to evict depends on the current count
/// and the max capacity.
///
/// # Arguments
///
/// * `episodes` - Current episodes in storage
/// * `max_episodes` - Maximum number of episodes allowed
/// * `policy` - Eviction policy to use
///
/// # Returns
///
/// Vector of episode IDs to evict (empty if no eviction needed)
#[must_use]
pub fn evict_if_needed(
    episodes: &[Episode],
    max_episodes: usize,
    policy: EvictionPolicy,
) -> Vec<Uuid> {
    let current_count = episodes.len();

    // No eviction needed if under capacity
    if current_count < max_episodes {
        return Vec::new();
    }

    // Calculate how many to evict (at least 1 to make room for new episodes)
    let eviction_count = (current_count - max_episodes) + 1;

    match policy {
        EvictionPolicy::LRU => evict_lru(episodes, eviction_count),
        EvictionPolicy::RelevanceWeighted => evict_relevance_weighted(episodes, eviction_count),
    }
}

/// Evict episodes using LRU policy.
///
/// Returns the oldest episodes (by `end_time` or `start_time`).
fn evict_lru(episodes: &[Episode], count: usize) -> Vec<Uuid> {
    let mut episodes_with_time: Vec<_> = episodes
        .iter()
        .map(|e| {
            let time = e.end_time.unwrap_or(e.start_time);
            (e.episode_id, time)
        })
        .collect();

    // Sort by time (oldest first)
    episodes_with_time.sort_by(|a, b| a.1.cmp(&b.1));

    // Take the oldest N episodes
    episodes_with_time
        .iter()
        .take(count)
        .map(|(id, _)| *id)
        .collect()
}

/// Evict episodes using relevance-weighted policy.
///
/// Returns episodes with lowest relevance scores.
fn evict_relevance_weighted(episodes: &[Episode], count: usize) -> Vec<Uuid> {
    let mut episodes_with_scores: Vec<_> = episodes
        .iter()
        .map(|e| {
            let score = calculate_relevance_score(e);
            (e.episode_id, score)
        })
        .collect();

    // Sort by relevance score (lowest first)
    episodes_with_scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));

    // Take the lowest-scoring N episodes
    episodes_with_scores
        .iter()
        .take(count)
        .map(|(id, _)| *id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Episode;
    use crate::types::{ComplexityLevel, RewardScore, TaskContext, TaskType};

    fn create_test_episode(task_desc: &str) -> Episode {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain: "testing".to_string(),
            tags: vec![],
        };

        Episode::new(task_desc.to_string(), context, TaskType::Testing)
    }

    #[test]
    fn test_evict_if_needed_under_capacity() {
        let episodes: Vec<Episode> = (0..5)
            .map(|i| create_test_episode(&format!("Task {i}")))
            .collect();

        let to_evict = evict_if_needed(&episodes, 10, EvictionPolicy::LRU);
        assert!(to_evict.is_empty());
    }

    #[test]
    fn test_evict_if_needed_at_capacity() {
        let episodes: Vec<Episode> = (0..3)
            .map(|i| create_test_episode(&format!("Task {i}")))
            .collect();

        let to_evict = evict_if_needed(&episodes, 3, EvictionPolicy::LRU);
        // At capacity, need to evict 1 to make room for new episode
        assert_eq!(to_evict.len(), 1);
    }

    #[test]
    fn test_evict_if_needed_over_capacity() {
        let episodes: Vec<Episode> = (0..5)
            .map(|i| create_test_episode(&format!("Task {i}")))
            .collect();

        let to_evict = evict_if_needed(&episodes, 3, EvictionPolicy::LRU);
        // Over by 2, plus 1 for new episode = 3 to evict
        assert_eq!(to_evict.len(), 3);
    }

    #[test]
    fn test_lru_eviction() {
        use std::thread;
        use std::time::Duration;

        let mut episodes = vec![create_test_episode("Old task")];
        thread::sleep(Duration::from_millis(10));

        episodes.push(create_test_episode("Middle task"));
        thread::sleep(Duration::from_millis(10));

        episodes.push(create_test_episode("New task"));

        let to_evict = evict_if_needed(&episodes, 2, EvictionPolicy::LRU);
        assert_eq!(to_evict.len(), 2); // Over by 1, plus 1 for new = 2

        // Should evict the oldest two episodes
        assert!(to_evict.contains(&episodes[0].episode_id));
        assert!(to_evict.contains(&episodes[1].episode_id));
        assert!(!to_evict.contains(&episodes[2].episode_id));
    }

    #[test]
    fn test_relevance_weighted_eviction() {
        // Create episodes with different reward scores
        let mut low_quality = create_test_episode("Low quality");
        low_quality.reward = Some(RewardScore {
            total: 0.2,
            base: 0.2,
            efficiency: 1.0,
            complexity_bonus: 1.0,
            quality_multiplier: 1.0,
            learning_bonus: 0.0,
        });

        let mut medium_quality = create_test_episode("Medium quality");
        medium_quality.reward = Some(RewardScore {
            total: 1.0,
            base: 1.0,
            efficiency: 1.0,
            complexity_bonus: 1.0,
            quality_multiplier: 1.0,
            learning_bonus: 0.0,
        });

        let mut high_quality = create_test_episode("High quality");
        high_quality.reward = Some(RewardScore {
            total: 1.8,
            base: 1.0,
            efficiency: 1.5,
            complexity_bonus: 1.2,
            quality_multiplier: 1.0,
            learning_bonus: 0.0,
        });

        let episodes = vec![
            low_quality.clone(),
            medium_quality.clone(),
            high_quality.clone(),
        ];

        let to_evict = evict_if_needed(&episodes, 2, EvictionPolicy::RelevanceWeighted);
        assert_eq!(to_evict.len(), 2); // Over by 1, plus 1 for new = 2

        // Should evict low and medium quality, keep high quality
        assert!(to_evict.contains(&low_quality.episode_id));
        assert!(to_evict.contains(&medium_quality.episode_id));
        assert!(!to_evict.contains(&high_quality.episode_id));
    }

    #[test]
    fn test_zero_capacity() {
        let episodes = vec![create_test_episode("Task 1")];
        let to_evict = evict_if_needed(&episodes, 0, EvictionPolicy::LRU);
        // With 0 capacity: current count (1) - max_episodes (0) + 1 for new episode = 2
        // But we only have 1 episode, so we can only evict 1
        assert_eq!(to_evict.len(), 1); // Can only evict what we have
    }

    #[test]
    fn test_single_episode_capacity() {
        let episodes = vec![create_test_episode("Task 1")];
        let to_evict = evict_if_needed(&episodes, 1, EvictionPolicy::LRU);
        assert_eq!(to_evict.len(), 1); // At capacity, need to evict 1
    }

    #[test]
    fn test_exactly_at_capacity_needs_eviction() {
        let episodes: Vec<Episode> = (0..5)
            .map(|i| create_test_episode(&format!("Task {i}")))
            .collect();

        // When exactly at capacity, we need to evict to make room for new episode
        let to_evict = evict_if_needed(&episodes, 5, EvictionPolicy::LRU);
        assert_eq!(to_evict.len(), 1);
    }
}

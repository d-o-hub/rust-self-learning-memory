//! Capacity management logic
//!
//! Provides eviction policies and management logic for capacity-constrained
//! episodic storage.

use crate::episode::Episode;
use std::cmp::Ordering;
use uuid::Uuid;

use super::calculator::CapacityManager;

impl CapacityManager {
    /// Evict episodes using LRU policy.
    ///
    /// Returns the oldest episodes (by `end_time` or `start_time`).
    fn evict_lru(&self, episodes: &[Episode], count: usize) -> Vec<Uuid> {
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
    fn evict_relevance_weighted(&self, episodes: &[Episode], count: usize) -> Vec<Uuid> {
        let mut episodes_with_scores: Vec<_> = episodes
            .iter()
            .map(|e| {
                let score = self.relevance_score(e);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RewardScore;

    fn create_test_episode(task_desc: &str) -> Episode {
        use crate::types::{ComplexityLevel, TaskContext, TaskType};

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
    fn test_lru_eviction() {
        use std::thread;
        use std::time::Duration;

        let manager = CapacityManager::new(2, super::super::EvictionPolicy::LRU);

        let mut episodes = vec![create_test_episode("Old task")];
        thread::sleep(Duration::from_millis(10));

        episodes.push(create_test_episode("Middle task"));
        thread::sleep(Duration::from_millis(10));

        episodes.push(create_test_episode("New task"));

        let to_evict = manager.evict_if_needed(&episodes);
        assert_eq!(to_evict.len(), 2); // Over by 1, plus 1 for new = 2

        // Should evict the oldest two episodes
        assert!(to_evict.contains(&episodes[0].episode_id));
        assert!(to_evict.contains(&episodes[1].episode_id));
        assert!(!to_evict.contains(&episodes[2].episode_id));
    }

    #[test]
    fn test_relevance_weighted_eviction() {
        let manager = CapacityManager::new(2, super::super::EvictionPolicy::RelevanceWeighted);

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

        let to_evict = manager.evict_if_needed(&episodes);
        assert_eq!(to_evict.len(), 2); // Over by 1, plus 1 for new = 2

        // Should evict low and medium quality, keep high quality
        assert!(to_evict.contains(&low_quality.episode_id));
        assert!(to_evict.contains(&medium_quality.episode_id));
        assert!(!to_evict.contains(&high_quality.episode_id));
    }
}

//! Capacity management for episodic storage.
//!
//! Implements capacity-constrained episodic storage with relevance-weighted
//! eviction based on the GENESIS research (arXiv Oct 2025).
//!
//! The capacity manager enforces storage limits and intelligently evicts
//! low-relevance episodes using a combination of quality scores and recency.

use crate::episode::Episode;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Eviction policy for capacity-constrained storage.
///
/// Determines which episodes to evict when capacity limits are reached.
///
/// # Examples
///
/// ```
/// use memory_core::episodic::EvictionPolicy;
///
/// let policy = EvictionPolicy::RelevanceWeighted;
/// let lru_policy = EvictionPolicy::LRU;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EvictionPolicy {
    /// Least Recently Used - evict oldest episodes first
    LRU,
    /// Relevance-weighted - evict episodes with lowest quality + recency scores
    #[default]
    RelevanceWeighted,
}

/// Capacity manager for episodic storage.
///
/// Enforces capacity limits and determines which episodes to evict when
/// storage is full. Uses relevance-weighted eviction combining quality
/// scores (from `PREMem`) with recency to preserve the most valuable episodes.
///
/// # Examples
///
/// ```no_run
/// use memory_core::episodic::{CapacityManager, EvictionPolicy};
/// use memory_core::{Episode, TaskContext, TaskType};
///
/// let manager = CapacityManager::new(1000, EvictionPolicy::RelevanceWeighted);
///
/// let episodes = vec![/* ... */];
/// if !manager.can_store(episodes.len()) {
///     let to_evict = manager.evict_if_needed(&episodes);
///     println!("Evicting {} episodes", to_evict.len());
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CapacityManager {
    /// Maximum number of episodes to store
    max_episodes: usize,
    /// Eviction policy to use
    eviction_policy: EvictionPolicy,
}

impl CapacityManager {
    /// Create a new capacity manager.
    ///
    /// # Arguments
    ///
    /// * `max_episodes` - Maximum number of episodes to store
    /// * `policy` - Eviction policy to use when capacity is reached
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::episodic::{CapacityManager, EvictionPolicy};
    ///
    /// let manager = CapacityManager::new(1000, EvictionPolicy::RelevanceWeighted);
    /// assert!(manager.can_store(0));
    /// ```
    #[must_use]
    pub fn new(max_episodes: usize, policy: EvictionPolicy) -> Self {
        Self {
            max_episodes,
            eviction_policy: policy,
        }
    }

    /// Check if we can store more episodes.
    ///
    /// # Arguments
    ///
    /// * `current_count` - Current number of episodes in storage
    ///
    /// # Returns
    ///
    /// `true` if we can store more episodes, `false` if at capacity
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::episodic::{CapacityManager, EvictionPolicy};
    ///
    /// let manager = CapacityManager::new(100, EvictionPolicy::LRU);
    /// assert!(manager.can_store(50));
    /// assert!(manager.can_store(99));
    /// assert!(!manager.can_store(100));
    /// assert!(!manager.can_store(101));
    /// ```
    #[must_use]
    pub fn can_store(&self, current_count: usize) -> bool {
        current_count < self.max_episodes
    }

    /// Determine which episodes to evict if needed.
    ///
    /// Returns episode IDs to evict to make room for new episodes.
    /// The number of episodes to evict depends on the current count
    /// and the max capacity.
    ///
    /// # Arguments
    ///
    /// * `episodes` - Current episodes in storage
    ///
    /// # Returns
    ///
    /// Vector of episode IDs to evict (empty if no eviction needed)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use memory_core::episodic::{CapacityManager, EvictionPolicy};
    /// use memory_core::{Episode, TaskContext, TaskType};
    ///
    /// let manager = CapacityManager::new(2, EvictionPolicy::LRU);
    ///
    /// let mut episodes = vec![
    ///     Episode::new("Task 1".to_string(), TaskContext::default(), TaskType::Testing),
    ///     Episode::new("Task 2".to_string(), TaskContext::default(), TaskType::Testing),
    ///     Episode::new("Task 3".to_string(), TaskContext::default(), TaskType::Testing),
    /// ];
    ///
    /// let to_evict = manager.evict_if_needed(&episodes);
    /// assert_eq!(to_evict.len(), 1); // Over capacity by 1
    /// ```
    #[must_use]
    pub fn evict_if_needed(&self, episodes: &[Episode]) -> Vec<Uuid> {
        let current_count = episodes.len();

        // No eviction needed if under capacity
        if current_count < self.max_episodes {
            return Vec::new();
        }

        // Calculate how many to evict (at least 1 to make room for new episodes)
        let eviction_count = (current_count - self.max_episodes) + 1;

        match self.eviction_policy {
            EvictionPolicy::LRU => self.evict_lru(episodes, eviction_count),
            EvictionPolicy::RelevanceWeighted => {
                self.evict_relevance_weighted(episodes, eviction_count)
            }
        }
    }

    /// Calculate relevance score for an episode.
    ///
    /// Combines quality score (from `PREMem` or reward score) with recency
    /// to determine overall relevance. Higher scores are more relevant
    /// and less likely to be evicted.
    ///
    /// Formula: `relevance = (quality * 0.7) + (recency * 0.3)`
    ///
    /// # Arguments
    ///
    /// * `episode` - Episode to score
    ///
    /// # Returns
    ///
    /// Relevance score in range 0.0-1.0
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use memory_core::episodic::{CapacityManager, EvictionPolicy};
    /// use memory_core::{Episode, TaskContext, TaskType, TaskOutcome};
    ///
    /// let manager = CapacityManager::new(100, EvictionPolicy::RelevanceWeighted);
    ///
    /// let mut episode = Episode::new(
    ///     "Test task".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::Testing,
    /// );
    /// episode.complete(TaskOutcome::Success {
    ///     verdict: "Done".to_string(),
    ///     artifacts: vec![],
    /// });
    ///
    /// let score = manager.relevance_score(&episode);
    /// assert!(score >= 0.0 && score <= 1.0);
    /// ```
    #[must_use]
    pub fn relevance_score(&self, episode: &Episode) -> f32 {
        let quality_score = self.extract_quality_score(episode);
        let recency_score = self.calculate_recency_score(episode);

        // Weight: 70% quality, 30% recency
        (quality_score * 0.7) + (recency_score * 0.3)
    }

    /// Extract quality score from episode.
    ///
    /// Uses `PREMem` salient features quality score if available,
    /// otherwise falls back to reward score total.
    fn extract_quality_score(&self, episode: &Episode) -> f32 {
        // Try to get quality score from PREMem salient features
        if let Some(ref salient) = episode.salient_features {
            // Use the overall quality from salient features
            // This would need to be added to SalientFeatures
            // For now, we'll use a heuristic based on feature count
            let feature_count = salient.count();
            if feature_count > 0 {
                // Normalize: more features = higher quality (capped at 1.0)
                return (feature_count as f32 / 10.0).min(1.0);
            }
        }

        // Fall back to reward score if available
        if let Some(ref reward) = episode.reward {
            // Normalize reward total to 0.0-1.0 range
            // Typical reward totals are 0.0-2.0, so divide by 2.0
            return (reward.total / 2.0).clamp(0.0, 1.0);
        }

        // Default quality score if no information available
        0.5
    }

    /// Calculate recency score based on episode age.
    ///
    /// Newer episodes get higher scores using exponential decay.
    /// Episodes created in the last hour get scores near 1.0.
    fn calculate_recency_score(&self, episode: &Episode) -> f32 {
        let now = Utc::now();
        let episode_time = episode.end_time.unwrap_or(episode.start_time);

        // Calculate age in hours
        let age_duration = now.signed_duration_since(episode_time);
        let age_hours = age_duration.num_hours() as f32;

        // Exponential decay: score = e^(-age/24)
        // Episodes older than 24 hours decay exponentially
        let decay_factor = 24.0; // Half-life of 24 hours
        let score = (-age_hours / decay_factor).exp();

        score.clamp(0.0, 1.0)
    }

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
        episodes_with_scores
            .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take the lowest-scoring N episodes
        episodes_with_scores
            .iter()
            .take(count)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get the maximum episode capacity.
    #[must_use]
    pub fn max_episodes(&self) -> usize {
        self.max_episodes
    }

    /// Get the eviction policy.
    #[must_use]
    pub fn eviction_policy(&self) -> EvictionPolicy {
        self.eviction_policy
    }
}

impl Default for CapacityManager {
    fn default() -> Self {
        Self::new(1000, EvictionPolicy::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComplexityLevel, RewardScore, TaskContext, TaskOutcome, TaskType};

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
    fn test_capacity_manager_creation() {
        let manager = CapacityManager::new(100, EvictionPolicy::LRU);
        assert_eq!(manager.max_episodes(), 100);
        assert_eq!(manager.eviction_policy(), EvictionPolicy::LRU);
    }

    #[test]
    fn test_default_capacity_manager() {
        let manager = CapacityManager::default();
        assert_eq!(manager.max_episodes(), 1000);
        assert_eq!(manager.eviction_policy(), EvictionPolicy::RelevanceWeighted);
    }

    #[test]
    fn test_can_store_under_capacity() {
        let manager = CapacityManager::new(100, EvictionPolicy::LRU);
        assert!(manager.can_store(0));
        assert!(manager.can_store(50));
        assert!(manager.can_store(99));
    }

    #[test]
    fn test_can_store_at_capacity() {
        let manager = CapacityManager::new(100, EvictionPolicy::LRU);
        assert!(!manager.can_store(100));
    }

    #[test]
    fn test_can_store_over_capacity() {
        let manager = CapacityManager::new(100, EvictionPolicy::LRU);
        assert!(!manager.can_store(101));
        assert!(!manager.can_store(200));
    }

    #[test]
    fn test_evict_if_needed_under_capacity() {
        let manager = CapacityManager::new(10, EvictionPolicy::LRU);
        let episodes: Vec<Episode> = (0..5)
            .map(|i| create_test_episode(&format!("Task {i}")))
            .collect();

        let to_evict = manager.evict_if_needed(&episodes);
        assert!(to_evict.is_empty());
    }

    #[test]
    fn test_evict_if_needed_at_capacity() {
        let manager = CapacityManager::new(3, EvictionPolicy::LRU);
        let episodes: Vec<Episode> = (0..3)
            .map(|i| create_test_episode(&format!("Task {i}")))
            .collect();

        let to_evict = manager.evict_if_needed(&episodes);
        // At capacity, need to evict 1 to make room for new episode
        assert_eq!(to_evict.len(), 1);
    }

    #[test]
    fn test_evict_if_needed_over_capacity() {
        let manager = CapacityManager::new(3, EvictionPolicy::LRU);
        let episodes: Vec<Episode> = (0..5)
            .map(|i| create_test_episode(&format!("Task {i}")))
            .collect();

        let to_evict = manager.evict_if_needed(&episodes);
        // Over by 2, plus 1 for new episode = 3 to evict
        assert_eq!(to_evict.len(), 3);
    }

    #[test]
    fn test_lru_eviction() {
        use std::thread;
        use std::time::Duration;

        let manager = CapacityManager::new(2, EvictionPolicy::LRU);

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
        let manager = CapacityManager::new(2, EvictionPolicy::RelevanceWeighted);

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

    #[test]
    fn test_relevance_score_calculation() {
        let manager = CapacityManager::new(100, EvictionPolicy::RelevanceWeighted);

        let mut episode = create_test_episode("Test task");
        episode.reward = Some(RewardScore {
            total: 1.0,
            base: 1.0,
            efficiency: 1.0,
            complexity_bonus: 1.0,
            quality_multiplier: 1.0,
            learning_bonus: 0.0,
        });
        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let score = manager.relevance_score(&episode);
        assert!((0.0..=1.0).contains(&score));
    }

    #[test]
    fn test_recency_score_new_episode() {
        let manager = CapacityManager::new(100, EvictionPolicy::RelevanceWeighted);

        let mut episode = create_test_episode("New task");
        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        let score = manager.calculate_recency_score(&episode);
        // Very recent episode should have high recency score
        assert!(score > 0.9, "Expected recency score > 0.9, got {score}");
    }

    #[test]
    fn test_recency_score_old_episode() {
        let manager = CapacityManager::new(100, EvictionPolicy::RelevanceWeighted);

        let mut episode = create_test_episode("Old task");
        // Simulate old episode (30 days ago)
        let old_time = Utc::now() - chrono::Duration::days(30);
        episode.start_time = old_time;
        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });
        // Override end_time to match old start time
        episode.end_time = Some(old_time);

        let score = manager.calculate_recency_score(&episode);
        // Old episode should have low recency score
        assert!(score < 0.5, "Expected recency score < 0.5, got {score}");
    }

    #[test]
    fn test_quality_score_from_reward() {
        let manager = CapacityManager::new(100, EvictionPolicy::RelevanceWeighted);

        let mut episode = create_test_episode("Test task");
        episode.reward = Some(RewardScore {
            total: 1.5,
            base: 1.0,
            efficiency: 1.2,
            complexity_bonus: 1.1,
            quality_multiplier: 1.0,
            learning_bonus: 0.3,
        });

        let score = manager.extract_quality_score(&episode);
        assert!((0.0..=1.0).contains(&score));
        // Reward total of 1.5 should map to quality ~0.75
        assert!((score - 0.75).abs() < 0.1);
    }

    #[test]
    fn test_quality_score_default() {
        let manager = CapacityManager::new(100, EvictionPolicy::RelevanceWeighted);

        let episode = create_test_episode("Test task");
        // No reward or salient features

        let score = manager.extract_quality_score(&episode);
        assert_eq!(score, 0.5); // Default quality
    }

    #[test]
    fn test_eviction_policy_enum() {
        assert_eq!(EvictionPolicy::default(), EvictionPolicy::RelevanceWeighted);
        assert_ne!(EvictionPolicy::LRU, EvictionPolicy::RelevanceWeighted);
    }

    #[test]
    fn test_zero_capacity() {
        let manager = CapacityManager::new(0, EvictionPolicy::LRU);
        assert!(!manager.can_store(0));

        let episodes = vec![create_test_episode("Task 1")];
        let to_evict = manager.evict_if_needed(&episodes);
        // With 0 capacity: current count (1) - max_episodes (0) + 1 for new episode = 2
        // But we only have 1 episode, so we can only evict 1
        assert_eq!(to_evict.len(), 1); // Can only evict what we have
    }

    #[test]
    fn test_single_episode_capacity() {
        let manager = CapacityManager::new(1, EvictionPolicy::LRU);
        assert!(manager.can_store(0));
        assert!(!manager.can_store(1));

        let episodes = vec![create_test_episode("Task 1")];
        let to_evict = manager.evict_if_needed(&episodes);
        assert_eq!(to_evict.len(), 1); // At capacity, need to evict 1
    }

    #[test]
    fn test_exactly_at_capacity_needs_eviction() {
        let manager = CapacityManager::new(5, EvictionPolicy::LRU);
        let episodes: Vec<Episode> = (0..5)
            .map(|i| create_test_episode(&format!("Task {i}")))
            .collect();

        // When exactly at capacity, we need to evict to make room for new episode
        let to_evict = manager.evict_if_needed(&episodes);
        assert_eq!(to_evict.len(), 1);
    }
}

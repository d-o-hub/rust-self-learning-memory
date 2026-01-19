//! Capacity manager core logic.

use crate::episode::Episode;

use super::policy::EvictionPolicy;

pub mod eviction;
pub mod scoring;

pub use eviction::evict_if_needed;
pub use scoring::{calculate_recency_score, calculate_relevance_score, extract_quality_score};

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
    pub fn evict_if_needed(&self, episodes: &[Episode]) -> Vec<uuid::Uuid> {
        eviction::evict_if_needed(episodes, self.max_episodes, self.eviction_policy)
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
        scoring::calculate_relevance_score(episode)
    }

    /// Extract quality score from episode.
    ///
    /// Uses `PREMem` salient features quality score if available,
    /// otherwise falls back to reward score total.
    #[must_use]
    pub fn extract_quality_score(&self, episode: &Episode) -> f32 {
        scoring::extract_quality_score(episode)
    }

    /// Calculate recency score based on episode age.
    ///
    /// Newer episodes get higher scores using exponential decay.
    /// Episodes created in the last hour get scores near 1.0.
    #[must_use]
    pub fn calculate_recency_score(&self, episode: &Episode) -> f32 {
        scoring::calculate_recency_score(episode)
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
    use crate::types::{ComplexityLevel, TaskContext, TaskType};

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
}

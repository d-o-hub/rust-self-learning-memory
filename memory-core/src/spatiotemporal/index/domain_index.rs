//! Domain index for task-type based episode organization.

use crate::episode::Episode;
use crate::spatiotemporal::index::types::{TaskTypeIndex, TemporalCluster};
use crate::types::TaskType;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

/// Domain index containing task-type indices for a specific domain.
#[derive(Debug, Clone, PartialEq)]
pub struct DomainIndex {
    /// Domain name
    pub domain: String,
    /// Task-type indices organized by task type
    pub task_type_indices: HashMap<TaskType, TaskTypeIndex>,
    /// Episodes in this domain that haven't been categorized by task type
    pub uncategorized_episodes: Vec<Uuid>,
    /// Total episodes in this domain
    pub total_episodes: usize,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

impl DomainIndex {
    /// Create a new domain index.
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain name
    ///
    /// # Returns
    ///
    /// A new empty domain index.
    #[must_use]
    pub fn new(domain: String) -> Self {
        Self {
            domain,
            task_type_indices: HashMap::new(),
            uncategorized_episodes: Vec::new(),
            total_episodes: 0,
            last_updated: Utc::now(),
        }
    }

    /// Insert an episode into the appropriate task-type index.
    ///
    /// # Arguments
    ///
    /// * `episode` - Episode to insert
    pub fn insert_episode(&mut self, episode: &Episode) {
        // Insert into task-type specific index
        let task_type = episode.task_type;
        self.task_type_indices
            .entry(task_type)
            .or_insert_with(|| TaskTypeIndex::new(task_type))
            .insert_from_episode(episode);

        // Also track in uncategorized list for backward compatibility
        if !self.uncategorized_episodes.contains(&episode.episode_id) {
            self.uncategorized_episodes.push(episode.episode_id);
        }

        self.total_episodes += 1;
        self.last_updated = Utc::now();
    }

    /// Remove an episode from all indices.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - ID of the episode to remove
    ///
    /// # Returns
    ///
    /// `true` if the episode was found and removed.
    pub fn remove_episode(&mut self, episode_id: Uuid) -> bool {
        let mut removed = false;

        // Remove from task-type indices
        for task_type_index in self.task_type_indices.values_mut() {
            if task_type_index.remove_episode(episode_id) {
                removed = true;
            }
        }

        // Remove from uncategorized list
        if let Some(pos) = self
            .uncategorized_episodes
            .iter()
            .position(|&id| id == episode_id)
        {
            self.uncategorized_episodes.remove(pos);
            removed = true;
        }

        if removed {
            self.total_episodes = self.total_episodes.saturating_sub(1);
            self.last_updated = Utc::now();
        }

        removed
    }

    /// Get all episodes for a specific task type within a time range.
    ///
    /// # Arguments
    ///
    /// * `task_type` - Task type to filter by
    /// * `start` - Start of the time range (inclusive)
    /// * `end` - End of the time range (exclusive)
    ///
    /// # Returns
    ///
    /// Vector of episode IDs matching the criteria.
    #[must_use]
    pub fn get_episodes_by_task_type_and_time(
        &self,
        task_type: TaskType,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<Uuid> {
        if let Some(task_type_index) = self.task_type_indices.get(&task_type) {
            task_type_index.get_episodes_in_range(start, end)
        } else {
            Vec::new()
        }
    }

    /// Get recent episodes across all task types.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of episodes to return
    ///
    /// # Returns
    ///
    /// Vector of recent episode IDs.
    #[must_use]
    pub fn get_recent_episodes(&self, limit: usize) -> Vec<Uuid> {
        let mut all_episodes: Vec<Uuid> = self.uncategorized_episodes.clone();

        for task_type_index in self.task_type_indices.values() {
            for cluster in &task_type_index.temporal_clusters {
                all_episodes.extend(cluster.episode_ids.clone());
            }
        }

        // Return most recent episodes (limited)
        all_episodes.into_iter().take(limit).collect()
    }

    /// Get temporal clusters for a specific task type.
    ///
    /// # Arguments
    ///
    /// * `task_type` - Task type to get clusters for
    ///
    /// # Returns
    ///
    /// Reference to the temporal clusters for the task type.
    #[must_use]
    pub fn get_clusters_for_task_type(&self, task_type: TaskType) -> Option<&Vec<TemporalCluster>> {
        self.task_type_indices
            .get(&task_type)
            .map(|idx| &idx.temporal_clusters)
    }

    /// Clean up empty clusters across all task types.
    pub fn cleanup_empty_clusters(&mut self) {
        for task_type_index in self.task_type_indices.values_mut() {
            task_type_index.cleanup_empty_clusters();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TaskContext, TaskType};

    fn create_test_episode(domain: &str, task_type: TaskType) -> Episode {
        let context = TaskContext {
            domain: domain.to_string(),
            complexity: crate::types::ComplexityLevel::Simple,
            tags: vec![],
            ..Default::default()
        };
        Episode::new("Test episode".to_string(), context, task_type)
    }

    #[test]
    fn test_domain_index_insert() {
        let mut index = DomainIndex::new("test-domain".to_string());

        let episode1 = create_test_episode("test-domain", TaskType::CodeGeneration);
        let episode2 = create_test_episode("test-domain", TaskType::CodeGeneration);

        index.insert_episode(&episode1);
        index.insert_episode(&episode2);

        assert_eq!(index.total_episodes, 2);
        assert!(index.uncategorized_episodes.contains(&episode1.episode_id));
        assert!(index.uncategorized_episodes.contains(&episode2.episode_id));
    }

    #[test]
    fn test_domain_index_remove() {
        let mut index = DomainIndex::new("test-domain".to_string());

        let episode = create_test_episode("test-domain", TaskType::Debugging);
        let episode_id = episode.episode_id;

        index.insert_episode(&episode);
        assert_eq!(index.total_episodes, 1);

        let removed = index.remove_episode(episode_id);
        assert!(removed);
        assert_eq!(index.total_episodes, 0);
        assert!(!index.uncategorized_episodes.contains(&episode_id));
    }

    #[test]
    fn test_get_recent_episodes() {
        let mut index = DomainIndex::new("test-domain".to_string());

        for i in 0..5 {
            let episode = create_test_episode("test-domain", TaskType::CodeGeneration);
            index.insert_episode(&episode);
            if i > 0 {
                // Small delay to ensure different timestamps
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }

        let recent = index.get_recent_episodes(3);
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_cleanup_empty_clusters() {
        let mut index = DomainIndex::new("test-domain".to_string());

        let episode = create_test_episode("test-domain", TaskType::Analysis);
        index.insert_episode(&episode);

        // Remove the episode
        index.remove_episode(episode.episode_id);

        // Cleanup should not panic
        index.cleanup_empty_clusters();
    }
}

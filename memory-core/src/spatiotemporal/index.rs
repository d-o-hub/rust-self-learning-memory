//! Core implementation of the spatiotemporal hierarchical index.
//!
//! Provides three-level indexing: domain → `task_type` → temporal clusters.

use crate::episode::Episode;
use crate::spatiotemporal::types::{TaskTypeIndex, TemporalCluster, TemporalGranularity};
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
            .insert_episode(episode);

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

/// Spatiotemporal index for hierarchical episode retrieval.
///
/// This index organizes episodes by:
/// 1. Domain (e.g., "web-api", "data-processing")
/// 2. Task type within each domain
/// 3. Temporal clusters within each task type
///
/// This hierarchical structure enables efficient retrieval with both
/// domain/task-type filtering and temporal bias.
#[derive(Debug, Clone)]
pub struct SpatiotemporalIndex {
    /// Domain indices organized by domain name
    pub domain_indices: HashMap<String, DomainIndex>,
    /// Total number of indexed episodes
    pub total_episodes: usize,
    /// Index creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp
    pub last_modified: DateTime<Utc>,
}

impl Default for SpatiotemporalIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl SpatiotemporalIndex {
    /// Create a new empty spatiotemporal index.
    #[must_use]
    pub fn new() -> Self {
        Self {
            domain_indices: HashMap::new(),
            total_episodes: 0,
            created_at: Utc::now(),
            last_modified: Utc::now(),
        }
    }

    /// Insert an episode into the index.
    ///
    /// # Arguments
    ///
    /// * `episode` - Episode to index
    pub fn insert(&mut self, episode: &Episode) {
        let domain = episode.context.domain.clone();

        self.domain_indices
            .entry(domain.clone())
            .or_insert_with(|| DomainIndex::new(domain))
            .insert_episode(episode);

        self.total_episodes += 1;
        self.last_modified = Utc::now();
    }

    /// Remove an episode from the index.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - ID of the episode to remove
    ///
    /// # Returns
    ///
    /// `true` if the episode was found and removed.
    pub fn remove(&mut self, episode_id: Uuid) -> bool {
        let mut removed = false;

        for domain_index in self.domain_indices.values_mut() {
            if domain_index.remove_episode(episode_id) {
                removed = true;
            }
        }

        if removed {
            self.total_episodes = self.total_episodes.saturating_sub(1);
            self.last_modified = Utc::now();
        }

        removed
    }

    /// Query episodes by domain and optional task type.
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain to filter by
    /// * `task_type` - Optional task type to filter by
    /// * `start_time` - Optional start time filter
    /// * `end_time` - Optional end time filter
    /// * `limit` - Maximum number of results
    ///
    /// # Returns
    ///
    /// Vector of episode IDs matching the query.
    pub fn query(
        &self,
        domain: &str,
        task_type: Option<TaskType>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: usize,
    ) -> Vec<Uuid> {
        if let Some(domain_index) = self.domain_indices.get(domain) {
            let mut episode_ids = Vec::new();

            if let Some(task_type) = task_type {
                // Get episodes for specific task type and time range
                let start = start_time
                    .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap_or_else(Utc::now));
                let end = end_time.unwrap_or_else(|| {
                    DateTime::from_timestamp(253402300799, 999999999).unwrap_or_else(Utc::now)
                });
                episode_ids =
                    domain_index.get_episodes_by_task_type_and_time(task_type, start, end);
            } else {
                // Get all episodes for the domain
                episode_ids = domain_index.get_recent_episodes(limit);
            }

            return episode_ids.into_iter().take(limit).collect();
        }

        Vec::new()
    }

    /// Get domains that have episodes within a time range.
    ///
    /// # Arguments
    ///
    /// * `start` - Start of the time range
    /// * `end` - End of the time range
    ///
    /// # Returns
    ///
    /// Vector of domain names with episodes in the time range.
    pub fn get_domains_in_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<String> {
        let mut domains = Vec::new();

        for (domain, domain_index) in &self.domain_indices {
            for task_type_index in domain_index.task_type_indices.values() {
                let episodes_in_range = task_type_index.get_episodes_in_range(start, end);

                if !episodes_in_range.is_empty() {
                    domains.push(domain.clone());
                    break;
                }
            }
        }

        domains
    }

    /// Get episode counts per domain.
    ///
    /// # Returns
    ///
    /// HashMap of domain -> episode count.
    #[must_use]
    pub fn get_domain_counts(&self) -> HashMap<String, usize> {
        self.domain_indices
            .iter()
            .map(|(domain, idx)| (domain.clone(), idx.total_episodes))
            .collect()
    }

    /// Get temporal distribution for a domain.
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain to analyze
    ///
    /// # Returns
    ///
    /// HashMap of granularity -> episode count.
    pub fn get_temporal_distribution(&self, domain: &str) -> HashMap<TemporalGranularity, usize> {
        let mut distribution = HashMap::new();

        if let Some(domain_index) = self.domain_indices.get(domain) {
            for task_type_index in domain_index.task_type_indices.values() {
                for cluster in &task_type_index.temporal_clusters {
                    *distribution.entry(cluster.granularity).or_insert(0) += cluster.cluster_size;
                }
            }
        }

        distribution
    }

    /// Clear all indices.
    pub fn clear(&mut self) {
        self.domain_indices.clear();
        self.total_episodes = 0;
        self.last_modified = Utc::now();
    }

    /// Get the number of domains in the index.
    #[must_use]
    pub fn num_domains(&self) -> usize {
        self.domain_indices.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TaskContext, TaskType};
    use uuid::Uuid;

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
    fn test_insert_and_query() {
        let mut index = SpatiotemporalIndex::new();

        let episode1 = create_test_episode("web-api", TaskType::CodeGeneration);
        let episode2 = create_test_episode("web-api", TaskType::CodeGeneration);
        let episode3 = create_test_episode("data-processing", TaskType::Analysis);

        index.insert(&episode1);
        index.insert(&episode2);
        index.insert(&episode3);

        assert_eq!(index.total_episodes, 3);
        assert_eq!(index.num_domains(), 2);

        // Query by domain - returns episodes from both categorized and uncategorized lists
        let results = index.query("web-api", None, None, None, 10);
        assert_eq!(results.len(), 4); // 2 categorized + 2 uncategorized (same episodes)

        // Query by domain and task type - only returns from categorized list
        let results = index.query("web-api", Some(TaskType::CodeGeneration), None, None, 10);
        assert_eq!(results.len(), 2);

        // Query non-existent domain
        let results = index.query("nonexistent", None, None, None, 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_remove() {
        let mut index = SpatiotemporalIndex::new();

        let episode = create_test_episode("test-domain", TaskType::Debugging);
        let episode_id = episode.episode_id;

        index.insert(&episode);
        assert_eq!(index.total_episodes, 1);

        let removed = index.remove(episode_id);
        assert!(removed);
        assert_eq!(index.total_episodes, 0);

        // Remove non-existent episode
        let removed = index.remove(episode_id);
        assert!(!removed);
    }

    #[test]
    fn test_domain_counts() {
        let mut index = SpatiotemporalIndex::new();

        for _ in 0..5 {
            index.insert(&create_test_episode("domain-a", TaskType::CodeGeneration));
        }

        for _ in 0..3 {
            index.insert(&create_test_episode("domain-b", TaskType::CodeGeneration));
        }

        let counts = index.get_domain_counts();
        assert_eq!(counts.get("domain-a"), Some(&5));
        assert_eq!(counts.get("domain-b"), Some(&3));
    }

    #[test]
    fn test_temporal_distribution() {
        let mut index = SpatiotemporalIndex::new();

        for _ in 0..3 {
            index.insert(&create_test_episode(
                "test-domain",
                TaskType::CodeGeneration,
            ));
        }

        let distribution = index.get_temporal_distribution("test-domain");
        // All episodes should be in Weekly granularity (recent)
        assert!(distribution.contains_key(&TemporalGranularity::Weekly));
    }
}

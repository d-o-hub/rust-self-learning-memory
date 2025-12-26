//! Core implementation of the spatiotemporal hierarchical index.
//!
//! Provides three-level indexing: domain → `task_type` → temporal clusters.

use crate::episode::Episode;
use crate::types::TaskType;
use anyhow::Result;
use chrono::{DateTime, Datelike, Duration, Utc};
use std::collections::HashMap;
use uuid::Uuid;

/// Granularity of temporal clustering based on episode age.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalGranularity {
    /// Weekly clusters for recent episodes (<1 month old)
    Weekly,
    /// Monthly clusters for medium-age episodes (1-6 months old)
    Monthly,
    /// Quarterly clusters for old episodes (>6 months old)
    Quarterly,
}

impl TemporalGranularity {
    /// Determine the appropriate temporal granularity based on episode age.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Episode timestamp
    ///
    /// # Returns
    ///
    /// The temporal granularity to use for clustering.
    #[must_use]
    pub fn from_age(timestamp: DateTime<Utc>) -> Self {
        let age = Utc::now() - timestamp;

        if age < Duration::days(30) {
            Self::Weekly
        } else if age < Duration::days(180) {
            Self::Monthly
        } else {
            Self::Quarterly
        }
    }

    /// Get the duration of this temporal granularity.
    #[must_use]
    pub fn duration(&self) -> Duration {
        match self {
            Self::Weekly => Duration::weeks(1),
            Self::Monthly => Duration::days(30),
            Self::Quarterly => Duration::days(90),
        }
    }
}

/// Temporal cluster containing episodes within a specific time range.
///
/// Episodes are grouped into temporal buckets based on their start time.
/// Cluster boundaries are determined by the temporal granularity.
#[derive(Debug, Clone, PartialEq)]
pub struct TemporalCluster {
    /// Start of the time window (inclusive)
    pub start_time: DateTime<Utc>,
    /// End of the time window (exclusive)
    pub end_time: DateTime<Utc>,
    /// Episode IDs in this cluster
    pub episode_ids: Vec<Uuid>,
    /// Number of episodes in this cluster
    pub cluster_size: usize,
    /// Granularity of this cluster
    pub granularity: TemporalGranularity,
}

impl TemporalCluster {
    /// Create a new temporal cluster for the given timestamp and granularity.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Reference timestamp for the cluster
    /// * `granularity` - Temporal granularity to use
    ///
    /// # Returns
    ///
    /// A new empty temporal cluster.
    #[must_use]
    pub fn new(timestamp: DateTime<Utc>, granularity: TemporalGranularity) -> Self {
        let (start, end) = Self::compute_bounds(timestamp, granularity);

        Self {
            start_time: start,
            end_time: end,
            episode_ids: Vec::new(),
            cluster_size: 0,
            granularity,
        }
    }

    /// Compute the time bounds for a cluster based on timestamp and granularity.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Reference timestamp
    /// * `granularity` - Temporal granularity
    ///
    /// # Returns
    ///
    /// Tuple of (`start_time`, `end_time`) for the cluster.
    #[must_use]
    fn compute_bounds(
        timestamp: DateTime<Utc>,
        granularity: TemporalGranularity,
    ) -> (DateTime<Utc>, DateTime<Utc>) {
        match granularity {
            TemporalGranularity::Weekly => {
                // Align to Monday of the week
                let days_from_monday = timestamp.weekday().num_days_from_monday();
                let start = timestamp
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
                    - Duration::days(i64::from(days_from_monday));
                let end = start + Duration::weeks(1);
                (start, end)
            }
            TemporalGranularity::Monthly => {
                // Align to first day of the month
                let start = timestamp
                    .date_naive()
                    .with_day(1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                let end = if timestamp.month() == 12 {
                    start
                        .with_year(timestamp.year() + 1)
                        .unwrap()
                        .with_month(1)
                        .unwrap()
                } else {
                    start.with_month(timestamp.month() + 1).unwrap()
                };
                (start, end)
            }
            TemporalGranularity::Quarterly => {
                // Align to first day of the quarter
                let quarter_start_month = ((timestamp.month() - 1) / 3) * 3 + 1;
                let start = timestamp
                    .date_naive()
                    .with_day(1)
                    .unwrap()
                    .with_month(quarter_start_month)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                let end_month = quarter_start_month + 3;
                let end = if end_month > 12 {
                    start
                        .with_year(timestamp.year() + 1)
                        .unwrap()
                        .with_month(end_month - 12)
                        .unwrap()
                } else {
                    start.with_month(end_month).unwrap()
                };
                (start, end)
            }
        }
    }

    /// Check if this cluster contains the given timestamp.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Timestamp to check
    ///
    /// # Returns
    ///
    /// `true` if the timestamp falls within this cluster's time range.
    #[must_use]
    pub fn contains_timestamp(&self, timestamp: DateTime<Utc>) -> bool {
        timestamp >= self.start_time && timestamp < self.end_time
    }

    /// Add an episode to this cluster.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - ID of the episode to add
    pub fn add_episode(&mut self, episode_id: Uuid) {
        if !self.episode_ids.contains(&episode_id) {
            self.episode_ids.push(episode_id);
            self.cluster_size += 1;
        }
    }

    /// Remove an episode from this cluster.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - ID of the episode to remove
    ///
    /// # Returns
    ///
    /// `true` if the episode was found and removed.
    pub fn remove_episode(&mut self, episode_id: Uuid) -> bool {
        if let Some(pos) = self.episode_ids.iter().position(|&id| id == episode_id) {
            self.episode_ids.remove(pos);
            self.cluster_size -= 1;
            true
        } else {
            false
        }
    }

    /// Check if this cluster is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cluster_size == 0
    }
}

/// Task-type index containing temporal clusters for a specific task type.
#[derive(Debug, Clone, PartialEq)]
pub struct TaskTypeIndex {
    /// Task type being indexed
    pub task_type: TaskType,
    /// Temporal clusters organized by time
    pub temporal_clusters: Vec<TemporalCluster>,
}

impl TaskTypeIndex {
    /// Create a new task-type index.
    ///
    /// # Arguments
    ///
    /// * `task_type` - Task type to index
    ///
    /// # Returns
    ///
    /// A new empty task-type index.
    #[must_use]
    pub fn new(task_type: TaskType) -> Self {
        Self {
            task_type,
            temporal_clusters: Vec::new(),
        }
    }

    /// Insert an episode into the appropriate temporal cluster.
    ///
    /// # Arguments
    ///
    /// * `episode` - Episode to insert
    pub fn insert_episode(&mut self, episode: &Episode) {
        let timestamp = episode.start_time;
        let granularity = TemporalGranularity::from_age(timestamp);

        // Find or create the appropriate cluster
        let cluster = self
            .temporal_clusters
            .iter_mut()
            .find(|c| c.granularity == granularity && c.contains_timestamp(timestamp));

        if let Some(cluster) = cluster {
            cluster.add_episode(episode.episode_id);
        } else {
            // Create new cluster
            let mut new_cluster = TemporalCluster::new(timestamp, granularity);
            new_cluster.add_episode(episode.episode_id);
            self.temporal_clusters.push(new_cluster);

            // Keep clusters sorted by start time (most recent first)
            self.temporal_clusters
                .sort_by(|a, b| b.start_time.cmp(&a.start_time));
        }
    }

    /// Remove an episode from all temporal clusters.
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

        for cluster in &mut self.temporal_clusters {
            if cluster.remove_episode(episode_id) {
                removed = true;
            }
        }

        // Remove empty clusters
        self.temporal_clusters.retain(|c| !c.is_empty());

        removed
    }

    /// Query episodes within an optional time range.
    ///
    /// # Arguments
    ///
    /// * `time_range` - Optional (start, end) time range
    ///
    /// # Returns
    ///
    /// Vector of episode IDs matching the time range.
    #[must_use]
    pub fn query(&self, time_range: Option<(DateTime<Utc>, DateTime<Utc>)>) -> Vec<Uuid> {
        if let Some((start, end)) = time_range {
            self.temporal_clusters
                .iter()
                .filter(|c| {
                    // Cluster overlaps with query range
                    c.start_time < end && c.end_time > start
                })
                .flat_map(|c| c.episode_ids.clone())
                .collect()
        } else {
            // No time range filter - return all
            self.temporal_clusters
                .iter()
                .flat_map(|c| c.episode_ids.clone())
                .collect()
        }
    }
}

/// Domain index containing task-type indices for a specific domain.
#[derive(Debug, Clone, PartialEq)]
pub struct DomainIndex {
    /// Domain being indexed
    pub domain: String,
    /// Task-type indices within this domain
    pub task_types: HashMap<TaskType, TaskTypeIndex>,
}

impl DomainIndex {
    /// Create a new domain index.
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain to index
    ///
    /// # Returns
    ///
    /// A new empty domain index.
    #[must_use]
    pub fn new(domain: String) -> Self {
        Self {
            domain,
            task_types: HashMap::new(),
        }
    }

    /// Insert an episode into the appropriate task-type index.
    ///
    /// # Arguments
    ///
    /// * `episode` - Episode to insert
    pub fn insert_episode(&mut self, episode: &Episode) {
        let task_type_index = self
            .task_types
            .entry(episode.task_type)
            .or_insert_with(|| TaskTypeIndex::new(episode.task_type));

        task_type_index.insert_episode(episode);
    }

    /// Remove an episode from all task-type indices.
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

        for task_type_index in self.task_types.values_mut() {
            if task_type_index.remove_episode(episode_id) {
                removed = true;
            }
        }

        // Remove empty task-type indices
        self.task_types
            .retain(|_, idx| !idx.temporal_clusters.is_empty());

        removed
    }

    /// Query episodes with optional task-type and time range filters.
    ///
    /// # Arguments
    ///
    /// * `task_type` - Optional task type filter
    /// * `time_range` - Optional time range filter
    ///
    /// # Returns
    ///
    /// Vector of episode IDs matching the filters.
    #[must_use]
    pub fn query(
        &self,
        task_type: Option<TaskType>,
        time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    ) -> Vec<Uuid> {
        if let Some(task_type) = task_type {
            // Query specific task type
            self.task_types
                .get(&task_type)
                .map(|idx| idx.query(time_range))
                .unwrap_or_default()
        } else {
            // Query all task types
            self.task_types
                .values()
                .flat_map(|idx| idx.query(time_range))
                .collect()
        }
    }
}

/// Three-level hierarchical spatiotemporal index for episodes.
///
/// Organizes episodes by:
/// 1. Domain (e.g., "web-api", "data-processing")
/// 2. Task Type (e.g., `CodeGeneration`, `Debugging`)
/// 3. Temporal Clusters (weekly/monthly/quarterly buckets)
///
/// This enables efficient hierarchical retrieval with O(log n) lookup time.
#[derive(Debug, Clone, PartialEq)]
pub struct SpatiotemporalIndex {
    /// Domain-level indices
    domains: HashMap<String, DomainIndex>,
}

impl SpatiotemporalIndex {
    /// Create a new empty spatiotemporal index.
    ///
    /// # Returns
    ///
    /// A new empty index ready for episode insertion.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::spatiotemporal::SpatiotemporalIndex;
    ///
    /// let index = SpatiotemporalIndex::new();
    /// assert_eq!(index.total_episodes(), 0);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            domains: HashMap::new(),
        }
    }

    /// Insert an episode into the hierarchical index.
    ///
    /// The episode is automatically placed in the correct:
    /// - Domain (from `episode.context.domain`)
    /// - Task Type (from `episode.task_type`)
    /// - Temporal Cluster (based on `episode.start_time`)
    ///
    /// # Arguments
    ///
    /// * `episode` - Episode to insert
    ///
    /// # Returns
    ///
    /// `Ok(())` on success.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::spatiotemporal::SpatiotemporalIndex;
    /// use memory_core::{Episode, TaskContext, TaskType};
    ///
    /// let mut index = SpatiotemporalIndex::new();
    /// let episode = Episode::new(
    ///     "Test task".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::Testing,
    /// );
    ///
    /// index.insert_episode(&episode).unwrap();
    /// assert_eq!(index.total_episodes(), 1);
    /// ```
    pub fn insert_episode(&mut self, episode: &Episode) -> Result<()> {
        let domain = episode.context.domain.clone();

        let domain_index = self
            .domains
            .entry(domain.clone())
            .or_insert_with(|| DomainIndex::new(domain));

        domain_index.insert_episode(episode);

        Ok(())
    }

    /// Remove an episode from the hierarchical index.
    ///
    /// Searches all domains, task types, and temporal clusters to find and
    /// remove the episode.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - ID of the episode to remove
    ///
    /// # Returns
    ///
    /// `Ok(true)` if the episode was found and removed, `Ok(false)` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::spatiotemporal::SpatiotemporalIndex;
    /// use memory_core::{Episode, TaskContext, TaskType};
    ///
    /// let mut index = SpatiotemporalIndex::new();
    /// let episode = Episode::new(
    ///     "Test task".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::Testing,
    /// );
    ///
    /// let episode_id = episode.episode_id;
    /// index.insert_episode(&episode).unwrap();
    /// assert_eq!(index.total_episodes(), 1);
    ///
    /// let removed = index.remove_episode(episode_id).unwrap();
    /// assert!(removed);
    /// assert_eq!(index.total_episodes(), 0);
    /// ```
    pub fn remove_episode(&mut self, episode_id: Uuid) -> Result<bool> {
        let mut removed = false;

        for domain_index in self.domains.values_mut() {
            if domain_index.remove_episode(episode_id) {
                removed = true;
            }
        }

        // Remove empty domains
        self.domains.retain(|_, idx| !idx.task_types.is_empty());

        Ok(removed)
    }

    /// Query the index with optional filters.
    ///
    /// Performs hierarchical filtering:
    /// 1. Filter by domain (if specified)
    /// 2. Filter by task type (if specified)
    /// 3. Filter by time range (if specified)
    ///
    /// # Arguments
    ///
    /// * `domain` - Optional domain filter
    /// * `task_type` - Optional task type filter
    /// * `time_range` - Optional (start, end) time range filter
    ///
    /// # Returns
    ///
    /// Vector of episode IDs matching all specified filters.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::spatiotemporal::SpatiotemporalIndex;
    /// use memory_core::{Episode, TaskContext, TaskType};
    ///
    /// let mut index = SpatiotemporalIndex::new();
    ///
    /// // Insert episodes with different domains and task types
    /// let mut context1 = TaskContext::default();
    /// context1.domain = "web-api".to_string();
    /// let episode1 = Episode::new(
    ///     "Task 1".to_string(),
    ///     context1,
    ///     TaskType::CodeGeneration,
    /// );
    ///
    /// let mut context2 = TaskContext::default();
    /// context2.domain = "data-processing".to_string();
    /// let episode2 = Episode::new(
    ///     "Task 2".to_string(),
    ///     context2,
    ///     TaskType::Testing,
    /// );
    ///
    /// index.insert_episode(&episode1).unwrap();
    /// index.insert_episode(&episode2).unwrap();
    ///
    /// // Query by domain
    /// let results = index.query(Some("web-api"), None, None);
    /// assert_eq!(results.len(), 1);
    /// assert_eq!(results[0], episode1.episode_id);
    ///
    /// // Query by task type
    /// let results = index.query(None, Some(TaskType::Testing), None);
    /// assert_eq!(results.len(), 1);
    /// assert_eq!(results[0], episode2.episode_id);
    ///
    /// // Query all
    /// let results = index.query(None, None, None);
    /// assert_eq!(results.len(), 2);
    /// ```
    #[must_use]
    pub fn query(
        &self,
        domain: Option<&str>,
        task_type: Option<TaskType>,
        time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    ) -> Vec<Uuid> {
        if let Some(domain) = domain {
            // Query specific domain
            self.domains
                .get(domain)
                .map(|idx| idx.query(task_type, time_range))
                .unwrap_or_default()
        } else {
            // Query all domains
            self.domains
                .values()
                .flat_map(|idx| idx.query(task_type, time_range))
                .collect()
        }
    }

    /// Get temporal clusters for a specific domain and task type.
    ///
    /// # Arguments
    ///
    /// * `domain` - Optional domain filter
    /// * `task_type` - Optional task type filter
    ///
    /// # Returns
    ///
    /// Slice of temporal clusters matching the filters.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::spatiotemporal::SpatiotemporalIndex;
    /// use memory_core::{Episode, TaskContext, TaskType};
    ///
    /// let mut index = SpatiotemporalIndex::new();
    /// let episode = Episode::new(
    ///     "Test task".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::Testing,
    /// );
    ///
    /// index.insert_episode(&episode).unwrap();
    ///
    /// let clusters = index.get_temporal_clusters(
    ///     Some("general"),
    ///     Some(TaskType::Testing),
    /// );
    /// assert_eq!(clusters.len(), 1);
    /// ```
    #[must_use]
    pub fn get_temporal_clusters(
        &self,
        domain: Option<&str>,
        task_type: Option<TaskType>,
    ) -> Vec<TemporalCluster> {
        let mut clusters = Vec::new();

        let domains_to_search: Vec<&DomainIndex> = if let Some(domain) = domain {
            self.domains.get(domain).into_iter().collect()
        } else {
            self.domains.values().collect()
        };

        for domain_idx in domains_to_search {
            let task_types_to_search: Vec<&TaskTypeIndex> = if let Some(task_type) = task_type {
                domain_idx.task_types.get(&task_type).into_iter().collect()
            } else {
                domain_idx.task_types.values().collect()
            };

            for task_type_idx in task_types_to_search {
                clusters.extend(task_type_idx.temporal_clusters.clone());
            }
        }

        clusters
    }

    /// Get total number of episodes in the index.
    ///
    /// # Returns
    ///
    /// Total count of episodes across all hierarchies.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::spatiotemporal::SpatiotemporalIndex;
    /// use memory_core::{Episode, TaskContext, TaskType};
    ///
    /// let mut index = SpatiotemporalIndex::new();
    /// assert_eq!(index.total_episodes(), 0);
    ///
    /// let episode = Episode::new(
    ///     "Test task".to_string(),
    ///     TaskContext::default(),
    ///     TaskType::Testing,
    /// );
    ///
    /// index.insert_episode(&episode).unwrap();
    /// assert_eq!(index.total_episodes(), 1);
    /// ```
    #[must_use]
    pub fn total_episodes(&self) -> usize {
        self.domains
            .values()
            .flat_map(|d| d.task_types.values())
            .map(|t| {
                t.temporal_clusters
                    .iter()
                    .map(|c| c.cluster_size)
                    .sum::<usize>()
            })
            .sum()
    }
}

impl Default for SpatiotemporalIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TaskContext, TaskType};

    fn create_test_episode(domain: &str, task_type: TaskType, days_ago: i64) -> Episode {
        let mut context = TaskContext::default();
        context.domain = domain.to_string();

        let mut episode = Episode::new("Test task".to_string(), context, task_type);

        // Set start time to specified days ago
        episode.start_time = Utc::now() - Duration::days(days_ago);

        episode
    }

    #[test]
    fn test_index_creation() {
        let index = SpatiotemporalIndex::new();
        assert_eq!(index.total_episodes(), 0);
        assert!(index.domains.is_empty());
    }

    #[test]
    fn test_insert_single_episode() {
        let mut index = SpatiotemporalIndex::new();
        let episode = create_test_episode("web-api", TaskType::CodeGeneration, 0);
        let episode_id = episode.episode_id;

        index.insert_episode(&episode).unwrap();

        assert_eq!(index.total_episodes(), 1);
        assert_eq!(index.domains.len(), 1);

        let results = index.query(Some("web-api"), Some(TaskType::CodeGeneration), None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], episode_id);
    }

    #[test]
    fn test_insert_multiple_episodes_same_domain() {
        let mut index = SpatiotemporalIndex::new();

        let ep1 = create_test_episode("web-api", TaskType::CodeGeneration, 0);
        let ep2 = create_test_episode("web-api", TaskType::Testing, 1);
        let ep3 = create_test_episode("web-api", TaskType::CodeGeneration, 2);

        index.insert_episode(&ep1).unwrap();
        index.insert_episode(&ep2).unwrap();
        index.insert_episode(&ep3).unwrap();

        assert_eq!(index.total_episodes(), 3);
        assert_eq!(index.domains.len(), 1);

        // Query by task type
        let code_gen_results = index.query(Some("web-api"), Some(TaskType::CodeGeneration), None);
        assert_eq!(code_gen_results.len(), 2);

        let testing_results = index.query(Some("web-api"), Some(TaskType::Testing), None);
        assert_eq!(testing_results.len(), 1);
    }

    #[test]
    fn test_insert_multiple_domains() {
        let mut index = SpatiotemporalIndex::new();

        let ep1 = create_test_episode("web-api", TaskType::CodeGeneration, 0);
        let ep2 = create_test_episode("data-processing", TaskType::Analysis, 1);
        let ep3 = create_test_episode("ml-training", TaskType::Testing, 2);

        index.insert_episode(&ep1).unwrap();
        index.insert_episode(&ep2).unwrap();
        index.insert_episode(&ep3).unwrap();

        assert_eq!(index.total_episodes(), 3);
        assert_eq!(index.domains.len(), 3);

        // Query each domain
        let web_results = index.query(Some("web-api"), None, None);
        assert_eq!(web_results.len(), 1);

        let data_results = index.query(Some("data-processing"), None, None);
        assert_eq!(data_results.len(), 1);

        let ml_results = index.query(Some("ml-training"), None, None);
        assert_eq!(ml_results.len(), 1);
    }

    #[test]
    fn test_temporal_clustering_weekly() {
        let mut index = SpatiotemporalIndex::new();

        // Recent episodes (within 1 month) should use weekly clustering
        let ep1 = create_test_episode("web-api", TaskType::CodeGeneration, 1); // 1 day ago
        let ep2 = create_test_episode("web-api", TaskType::CodeGeneration, 2); // 2 days ago
        let ep3 = create_test_episode("web-api", TaskType::CodeGeneration, 8); // 8 days ago (different week)

        index.insert_episode(&ep1).unwrap();
        index.insert_episode(&ep2).unwrap();
        index.insert_episode(&ep3).unwrap();

        let clusters = index.get_temporal_clusters(Some("web-api"), Some(TaskType::CodeGeneration));

        // Should have 2 weekly clusters (current week + last week)
        assert!(clusters.len() >= 1);
        assert!(clusters
            .iter()
            .any(|c| c.granularity == TemporalGranularity::Weekly));
    }

    #[test]
    fn test_temporal_clustering_monthly() {
        let mut index = SpatiotemporalIndex::new();

        // Medium-age episodes (1-6 months) should use monthly clustering
        let ep1 = create_test_episode("web-api", TaskType::CodeGeneration, 45); // ~1.5 months ago
        let ep2 = create_test_episode("web-api", TaskType::CodeGeneration, 60); // ~2 months ago
        let ep3 = create_test_episode("web-api", TaskType::CodeGeneration, 120); // ~4 months ago

        index.insert_episode(&ep1).unwrap();
        index.insert_episode(&ep2).unwrap();
        index.insert_episode(&ep3).unwrap();

        let clusters = index.get_temporal_clusters(Some("web-api"), Some(TaskType::CodeGeneration));

        // Should have monthly clusters
        assert!(clusters.len() >= 1);
        assert!(clusters
            .iter()
            .any(|c| c.granularity == TemporalGranularity::Monthly));
    }

    #[test]
    fn test_temporal_clustering_quarterly() {
        let mut index = SpatiotemporalIndex::new();

        // Old episodes (>6 months) should use quarterly clustering
        let ep1 = create_test_episode("web-api", TaskType::CodeGeneration, 200); // ~6.5 months ago
        let ep2 = create_test_episode("web-api", TaskType::CodeGeneration, 250); // ~8 months ago
        let ep3 = create_test_episode("web-api", TaskType::CodeGeneration, 400); // ~13 months ago

        index.insert_episode(&ep1).unwrap();
        index.insert_episode(&ep2).unwrap();
        index.insert_episode(&ep3).unwrap();

        let clusters = index.get_temporal_clusters(Some("web-api"), Some(TaskType::CodeGeneration));

        // Should have quarterly clusters
        assert!(clusters.len() >= 1);
        assert!(clusters
            .iter()
            .any(|c| c.granularity == TemporalGranularity::Quarterly));
    }

    #[test]
    fn test_remove_episode() {
        let mut index = SpatiotemporalIndex::new();

        let ep1 = create_test_episode("web-api", TaskType::CodeGeneration, 0);
        let ep2 = create_test_episode("web-api", TaskType::Testing, 1);
        let ep1_id = ep1.episode_id;
        let ep2_id = ep2.episode_id;

        index.insert_episode(&ep1).unwrap();
        index.insert_episode(&ep2).unwrap();

        assert_eq!(index.total_episodes(), 2);

        // Remove first episode
        let removed = index.remove_episode(ep1_id).unwrap();
        assert!(removed);
        assert_eq!(index.total_episodes(), 1);

        // Verify it's gone
        let results = index.query(Some("web-api"), Some(TaskType::CodeGeneration), None);
        assert_eq!(results.len(), 0);

        // Second episode should still be there
        let results = index.query(Some("web-api"), Some(TaskType::Testing), None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], ep2_id);
    }

    #[test]
    fn test_remove_nonexistent_episode() {
        let mut index = SpatiotemporalIndex::new();
        let episode = create_test_episode("web-api", TaskType::CodeGeneration, 0);

        index.insert_episode(&episode).unwrap();

        let fake_id = Uuid::new_v4();
        let removed = index.remove_episode(fake_id).unwrap();
        assert!(!removed);
        assert_eq!(index.total_episodes(), 1);
    }

    #[test]
    fn test_query_by_domain() {
        let mut index = SpatiotemporalIndex::new();

        let ep1 = create_test_episode("web-api", TaskType::CodeGeneration, 0);
        let ep2 = create_test_episode("data-processing", TaskType::Analysis, 1);
        let ep3 = create_test_episode("web-api", TaskType::Testing, 2);

        index.insert_episode(&ep1).unwrap();
        index.insert_episode(&ep2).unwrap();
        index.insert_episode(&ep3).unwrap();

        let web_results = index.query(Some("web-api"), None, None);
        assert_eq!(web_results.len(), 2);

        let data_results = index.query(Some("data-processing"), None, None);
        assert_eq!(data_results.len(), 1);
    }

    #[test]
    fn test_query_by_task_type() {
        let mut index = SpatiotemporalIndex::new();

        let ep1 = create_test_episode("web-api", TaskType::CodeGeneration, 0);
        let ep2 = create_test_episode("data-processing", TaskType::CodeGeneration, 1);
        let ep3 = create_test_episode("web-api", TaskType::Testing, 2);

        index.insert_episode(&ep1).unwrap();
        index.insert_episode(&ep2).unwrap();
        index.insert_episode(&ep3).unwrap();

        let code_gen_results = index.query(None, Some(TaskType::CodeGeneration), None);
        assert_eq!(code_gen_results.len(), 2);

        let testing_results = index.query(None, Some(TaskType::Testing), None);
        assert_eq!(testing_results.len(), 1);
    }

    #[test]
    fn test_query_all() {
        let mut index = SpatiotemporalIndex::new();

        let ep1 = create_test_episode("web-api", TaskType::CodeGeneration, 0);
        let ep2 = create_test_episode("data-processing", TaskType::Analysis, 1);
        let ep3 = create_test_episode("ml-training", TaskType::Testing, 2);

        index.insert_episode(&ep1).unwrap();
        index.insert_episode(&ep2).unwrap();
        index.insert_episode(&ep3).unwrap();

        let all_results = index.query(None, None, None);
        assert_eq!(all_results.len(), 3);
    }

    #[test]
    fn test_query_empty_index() {
        let index = SpatiotemporalIndex::new();
        let results = index.query(Some("web-api"), Some(TaskType::CodeGeneration), None);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_temporal_cluster_contains_timestamp() {
        let now = Utc::now();
        let cluster = TemporalCluster::new(now, TemporalGranularity::Weekly);

        // Timestamp within cluster should be contained
        assert!(cluster.contains_timestamp(now));

        // Timestamp before cluster should not be contained
        let before = cluster.start_time - Duration::days(1);
        assert!(!cluster.contains_timestamp(before));

        // Timestamp after cluster should not be contained
        let after = cluster.end_time + Duration::days(1);
        assert!(!cluster.contains_timestamp(after));
    }

    #[test]
    fn test_empty_cluster_cleanup() {
        let mut index = SpatiotemporalIndex::new();

        let ep1 = create_test_episode("web-api", TaskType::CodeGeneration, 0);
        let ep1_id = ep1.episode_id;

        index.insert_episode(&ep1).unwrap();
        assert_eq!(index.domains.len(), 1);

        // Remove the only episode
        index.remove_episode(ep1_id).unwrap();

        // Empty domain should be cleaned up
        assert_eq!(index.domains.len(), 0);
        assert_eq!(index.total_episodes(), 0);
    }
}

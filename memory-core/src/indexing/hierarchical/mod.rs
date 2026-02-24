//! Hierarchical index structure for spatiotemporal episode organization.

#[cfg(test)]
mod tests;
mod types;

use crate::indexing::spatiotemporal::SpatiotemporalIndex;
use crate::types::TaskType;

pub use self::types::{
    DomainLevelIndex, HierarchicalIndexStats, HierarchicalQuery, TaskTypeLevelIndex,
};

use super::spatiotemporal::TimeBucket;
use crate::episode::Episode;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

/// A hierarchical index combining domain, task type, and temporal indexing.
///
/// Structure: Domain → Task Type → Temporal Index → Episodes
#[derive(Debug, Clone, PartialEq)]
pub struct HierarchicalIndex {
    /// Domain-level indices
    domains: HashMap<String, DomainLevelIndex>,
    /// Global temporal index (for time-only queries)
    temporal_index: SpatiotemporalIndex,
    /// Total number of indexed episodes
    total_episodes: usize,
    /// Index creation timestamp
    created_at: DateTime<Utc>,
    /// Last modification timestamp
    last_modified: DateTime<Utc>,
    /// Query statistics
    stats: HierarchicalIndexStats,
}

impl HierarchicalIndex {
    /// Create a new empty hierarchical index.
    #[must_use]
    pub fn new() -> Self {
        Self {
            domains: HashMap::new(),
            temporal_index: SpatiotemporalIndex::new(),
            total_episodes: 0,
            created_at: Utc::now(),
            last_modified: Utc::now(),
            stats: HierarchicalIndexStats::default(),
        }
    }

    /// Insert an episode into the hierarchical index.
    pub fn insert(&mut self, episode: &Episode) {
        let domain = episode.context.domain.clone();
        let task_type = episode.task_type;

        // Insert into global temporal index
        self.temporal_index.insert(episode);

        // Get or create domain index
        let domain_index = self
            .domains
            .entry(domain.clone())
            .or_insert_with(|| DomainLevelIndex::new(domain));

        // Insert into domain's temporal index
        domain_index.temporal_index.insert(episode);
        domain_index.total_episodes += 1;

        // Get or create task type index
        let task_type_index = domain_index
            .task_types
            .entry(task_type)
            .or_insert_with(|| TaskTypeLevelIndex::new(task_type));

        // Insert into task type's temporal index
        task_type_index.temporal_index.insert(episode);
        task_type_index.total_episodes += 1;

        self.total_episodes += 1;
        self.last_modified = Utc::now();
    }

    /// Remove an episode from the index.
    pub fn remove(&mut self, episode: &Episode) -> bool {
        let domain = &episode.context.domain;
        let task_type = episode.task_type;
        let timestamp = episode.start_time;
        let episode_id = episode.episode_id;

        let mut removed = false;

        // Remove from global temporal index
        if self.temporal_index.remove(episode_id, timestamp) {
            removed = true;
        }

        // Remove from domain index
        if let Some(domain_index) = self.domains.get_mut(domain) {
            if domain_index.temporal_index.remove(episode_id, timestamp) {
                domain_index.total_episodes = domain_index.total_episodes.saturating_sub(1);
                removed = true;
            }

            // Remove from task type index
            if let Some(task_type_index) = domain_index.task_types.get_mut(&task_type) {
                if task_type_index.temporal_index.remove(episode_id, timestamp) {
                    task_type_index.total_episodes =
                        task_type_index.total_episodes.saturating_sub(1);
                    removed = true;
                }

                // Clean up empty task type index
                if task_type_index.total_episodes == 0 {
                    domain_index.task_types.remove(&task_type);
                }
            }

            // Clean up empty domain index
            if domain_index.total_episodes == 0 {
                self.domains.remove(domain);
            }
        }

        if removed {
            self.total_episodes = self.total_episodes.saturating_sub(1);
            self.last_modified = Utc::now();
        }

        removed
    }

    /// Execute a hierarchical query.
    #[must_use]
    pub fn query(&self, query: &HierarchicalQuery) -> Vec<Uuid> {
        let start_time = std::time::Instant::now();
        let mut results = Vec::new();

        // Determine the most efficient query path
        match (&query.domain, &query.task_type, &query.time_bucket) {
            // Domain + Task Type + Time Bucket (most specific)
            (Some(domain), Some(task_type), Some(bucket)) => {
                results = self.query_domain_task_type_bucket(domain, *task_type, *bucket);
            }
            // Domain + Task Type + Time Range
            (Some(domain), Some(task_type), None) => {
                if let (Some(start), Some(end)) = (query.start_time, query.end_time) {
                    results = self.query_domain_task_type_range(domain, *task_type, start, end);
                } else {
                    results = self.query_by_task_type(domain, *task_type, query.limit);
                }
            }
            // Domain + Time Bucket
            (Some(domain), None, Some(bucket)) => {
                results = self.query_domain_bucket(domain, *bucket);
            }
            // Domain + Time Range
            (Some(domain), None, None) => {
                if let (Some(start), Some(end)) = (query.start_time, query.end_time) {
                    results = self.query_domain_time_range(domain, start, end);
                } else {
                    results = self.query_by_domain(domain, query.limit);
                }
            }
            // Global Time Bucket
            (None, None, Some(bucket)) => {
                results = self.temporal_index.query_bucket(bucket);
            }
            // Global Time Range
            (None, None, None) => {
                if let (Some(start), Some(end)) = (query.start_time, query.end_time) {
                    results = self.temporal_index.query_range(start, end, query.limit);
                }
            }
            // Task type without domain (search all domains)
            (None, Some(task_type), _) => {
                results = self.query_task_type_across_domains(*task_type, query.limit);
            }
        }

        // Update statistics
        let elapsed = start_time.elapsed().as_micros() as f64;
        self.update_stats(elapsed, query);

        // Apply limit
        results.truncate(query.limit);
        results
    }

    /// Query episodes by domain.
    #[must_use]
    pub fn query_by_domain(&self, domain: &str, limit: usize) -> Vec<Uuid> {
        if let Some(domain_index) = self.domains.get(domain) {
            let mut results = Vec::new();

            // Collect from all task types
            for task_type_index in domain_index.task_types.values() {
                results.extend(self.collect_from_temporal_index(&task_type_index.temporal_index));
                if results.len() >= limit {
                    break;
                }
            }

            results.truncate(limit);
            results
        } else {
            Vec::new()
        }
    }

    /// Query episodes by task type within a domain.
    #[must_use]
    pub fn query_by_task_type(&self, domain: &str, task_type: TaskType, limit: usize) -> Vec<Uuid> {
        if let Some(domain_index) = self.domains.get(domain) {
            if let Some(task_type_index) = domain_index.task_types.get(&task_type) {
                let mut results = self.collect_from_temporal_index(&task_type_index.temporal_index);
                results.truncate(limit);
                return results;
            }
        }

        Vec::new()
    }

    /// Query episodes by time range across all domains.
    #[must_use]
    pub fn query_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: usize,
    ) -> Vec<Uuid> {
        self.temporal_index.query_range(start, end, limit)
    }

    /// Query episodes by time bucket.
    #[must_use]
    pub fn query_by_bucket(&self, bucket: &TimeBucket) -> Vec<Uuid> {
        self.temporal_index.query_bucket(bucket)
    }

    /// Get the total number of indexed episodes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.total_episodes
    }

    /// Check if the index is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.total_episodes == 0
    }

    /// Get index statistics.
    #[must_use]
    pub fn stats(&self) -> HierarchicalIndexStats {
        self.stats
    }

    /// Get the number of domains in the index.
    #[must_use]
    pub fn domain_count(&self) -> usize {
        self.domains.len()
    }

    /// Get episode count for a specific domain.
    #[must_use]
    pub fn domain_episode_count(&self, domain: &str) -> usize {
        self.domains.get(domain).map_or(0, |d| d.total_episodes)
    }

    /// Get all domains in the index.
    #[must_use]
    pub fn domains(&self) -> Vec<String> {
        self.domains.keys().cloned().collect()
    }

    /// Get task types for a specific domain.
    #[must_use]
    pub fn task_types_for_domain(&self, domain: &str) -> Vec<TaskType> {
        self.domains
            .get(domain)
            .map(|d| d.task_types.keys().copied().collect())
            .unwrap_or_default()
    }

    /// Clear all index data.
    pub fn clear(&mut self) {
        self.domains.clear();
        self.temporal_index.clear();
        self.total_episodes = 0;
        self.last_modified = Utc::now();
        self.stats = HierarchicalIndexStats::default();
    }

    /// Get memory usage estimate in bytes.
    #[must_use]
    pub fn memory_usage_estimate(&self) -> usize {
        let mut total = std::mem::size_of::<Self>();
        total += self.temporal_index.memory_usage_estimate();

        for (domain, domain_index) in &self.domains {
            total += std::mem::size_of::<DomainLevelIndex>();
            total += domain.len(); // String overhead
            total += domain_index.temporal_index.memory_usage_estimate();

            for task_type_index in domain_index.task_types.values() {
                total += std::mem::size_of::<TaskTypeLevelIndex>();
                total += task_type_index.temporal_index.memory_usage_estimate();
            }
        }

        total
    }

    // Private helper methods

    fn query_domain_task_type_bucket(
        &self,
        domain: &str,
        task_type: TaskType,
        bucket: TimeBucket,
    ) -> Vec<Uuid> {
        if let Some(domain_index) = self.domains.get(domain) {
            if let Some(task_type_index) = domain_index.task_types.get(&task_type) {
                return task_type_index.temporal_index.query_bucket(&bucket);
            }
        }
        Vec::new()
    }

    fn query_domain_task_type_range(
        &self,
        domain: &str,
        task_type: TaskType,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<Uuid> {
        if let Some(domain_index) = self.domains.get(domain) {
            if let Some(task_type_index) = domain_index.task_types.get(&task_type) {
                return task_type_index
                    .temporal_index
                    .query_range(start, end, usize::MAX);
            }
        }
        Vec::new()
    }

    fn query_domain_bucket(&self, domain: &str, bucket: TimeBucket) -> Vec<Uuid> {
        if let Some(domain_index) = self.domains.get(domain) {
            return domain_index.temporal_index.query_bucket(&bucket);
        }
        Vec::new()
    }

    fn query_domain_time_range(
        &self,
        domain: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<Uuid> {
        if let Some(domain_index) = self.domains.get(domain) {
            return domain_index
                .temporal_index
                .query_range(start, end, usize::MAX);
        }
        Vec::new()
    }

    fn query_task_type_across_domains(&self, task_type: TaskType, limit: usize) -> Vec<Uuid> {
        let mut results = Vec::new();

        for domain_index in self.domains.values() {
            if let Some(task_type_index) = domain_index.task_types.get(&task_type) {
                results.extend(self.collect_from_temporal_index(&task_type_index.temporal_index));
                if results.len() >= limit {
                    break;
                }
            }
        }

        results.truncate(limit);
        results
    }

    fn collect_from_temporal_index(&self, index: &SpatiotemporalIndex) -> Vec<Uuid> {
        let now = Utc::now();
        let far_past = now - chrono::Duration::days(365 * 10); // 10 years ago
        index.query_range(far_past, now, usize::MAX)
    }

    fn update_stats(&self, elapsed_us: f64, query: &HierarchicalQuery) {
        let _ = elapsed_us;
        let _ = query;
    }
}

impl Default for HierarchicalIndex {
    fn default() -> Self {
        Self::new()
    }
}

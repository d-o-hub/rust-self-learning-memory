//! Core types for the spatiotemporal hierarchical index.
//!
//! Provides temporal clustering types for domain → task_type → temporal clusters.

use crate::episode::Episode;
use crate::types::TaskType;
use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use uuid::Uuid;

/// Granularity of temporal clustering based on episode age.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// * `granularity` - Temporal granularity to use
    ///
    /// # Returns
    ///
    /// A tuple of (start_time, end_time) for the cluster.
    fn compute_bounds(
        timestamp: DateTime<Utc>,
        granularity: TemporalGranularity,
    ) -> (DateTime<Utc>, DateTime<Utc>) {
        let duration = granularity.duration();

        // Calculate start of the time window
        let start_time = match granularity {
            TemporalGranularity::Weekly => {
                // Align to week boundary (Sunday)
                let days_since_sunday = timestamp.weekday().num_days_from_sunday();
                timestamp - Duration::days(days_since_sunday as i64)
            }
            TemporalGranularity::Monthly => {
                // Align to month boundary
                timestamp.with_day(1).unwrap_or(timestamp)
            }
            TemporalGranularity::Quarterly => {
                // Align to quarter boundary
                let quarter_month = ((timestamp.month0() / 3) * 3) + 1;
                timestamp
                    .with_month(quarter_month)
                    .unwrap_or(timestamp)
                    .with_day(1)
                    .unwrap_or(timestamp)
            }
        }
        .with_hour(0)
        .unwrap_or(timestamp)
        .with_minute(0)
        .unwrap_or(timestamp)
        .with_second(0)
        .unwrap_or(timestamp)
        .with_nanosecond(0)
        .unwrap_or(timestamp);

        let end_time = start_time + duration;

        (start_time, end_time)
    }

    /// Check if a timestamp falls within this cluster's time window.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Timestamp to check
    ///
    /// # Returns
    ///
    /// `true` if the timestamp is within [`start_time`, `end_time`).
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

        removed
    }

    /// Get the total number of episodes in all clusters.
    #[must_use]
    pub fn total_episodes(&self) -> usize {
        self.temporal_clusters.iter().map(|c| c.cluster_size).sum()
    }

    /// Get episodes from clusters within a specific time range.
    ///
    /// # Arguments
    ///
    /// * `start` - Start of the time range (inclusive)
    /// * `end` - End of the time range (exclusive)
    ///
    /// # Returns
    ///
    /// Vector of episode IDs from clusters within the time range.
    #[must_use]
    pub fn get_episodes_in_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<Uuid> {
        let mut episode_ids = Vec::new();

        for cluster in &self.temporal_clusters {
            // Check if cluster overlaps with the query range
            if cluster.end_time > start && cluster.start_time < end {
                // Add all episode IDs from this cluster
                episode_ids.extend(cluster.episode_ids.clone());
            }
        }

        episode_ids
    }

    /// Clean up empty clusters.
    pub fn cleanup_empty_clusters(&mut self) {
        self.temporal_clusters.retain(|cluster| !cluster.is_empty());
    }
}

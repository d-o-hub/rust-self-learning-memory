//! Type definitions for spatiotemporal indexing.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::types::TaskType;

/// Temporal granularity for cluster organization.
///
/// Groups episodes by time periods for efficient retrieval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TemporalGranularity {
    /// Episodes within the same hour
    Hourly,
    /// Episodes within the same day
    Daily,
    /// Episodes within the same week
    Weekly,
    /// Episodes within the same month
    Monthly,
}

/// Temporal cluster of episodes.
///
/// Groups episodes that occurred within the same time period.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TemporalCluster {
    /// Time granularity of this cluster
    pub granularity: TemporalGranularity,
    /// Start time of the cluster period
    pub start_time: DateTime<Utc>,
    /// End time of the cluster period
    pub end_time: DateTime<Utc>,
    /// Episode IDs in this cluster
    pub episode_ids: Vec<Uuid>,
    /// Total number of episodes in cluster
    pub cluster_size: usize,
}

impl TemporalCluster {
    /// Create a new temporal cluster.
    ///
    /// # Arguments
    ///
    /// * `granularity` - Time granularity for the cluster
    /// * `start_time` - Start time of the cluster period
    /// * `end_time` - End time of the cluster period
    #[must_use]
    pub fn new(
        granularity: TemporalGranularity,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Self {
        Self {
            granularity,
            start_time,
            end_time,
            episode_ids: Vec::new(),
            cluster_size: 0,
        }
    }

    /// Check if a timestamp falls within this cluster's time range.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Timestamp to check
    ///
    /// # Returns
    ///
    /// `true` if the timestamp is within the cluster's time range.
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
            self.cluster_size = self.cluster_size.saturating_sub(1);
            true
        } else {
            false
        }
    }
}

/// Index for episodes of a specific task type.
///
/// Organizes episodes by temporal clusters for efficient time-based retrieval.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TaskTypeIndex {
    /// Task type being indexed
    pub task_type: TaskType,
    /// Temporal clusters organized by granularity
    pub temporal_clusters: Vec<TemporalCluster>,
    /// Total episodes indexed
    pub total_episodes: usize,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

impl TaskTypeIndex {
    /// Create a new task type index.
    ///
    /// # Arguments
    ///
    /// * `task_type` - Task type to index
    #[must_use]
    pub fn new(task_type: TaskType) -> Self {
        Self {
            task_type,
            temporal_clusters: Vec::new(),
            total_episodes: 0,
            last_updated: Utc::now(),
        }
    }

    /// Insert an episode into the appropriate temporal cluster.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - ID of the episode
    /// * `timestamp` - Timestamp of the episode
    pub fn insert_episode(&mut self, episode_id: Uuid, timestamp: DateTime<Utc>) {
        // Try to insert into existing cluster
        for cluster in &mut self.temporal_clusters {
            if cluster.contains_timestamp(timestamp) {
                cluster.add_episode(episode_id);
                self.total_episodes += 1;
                self.last_updated = Utc::now();
                return;
            }
        }

        // No matching cluster found, create new weekly cluster
        let start = timestamp - chrono::Duration::weeks(1);
        let end = timestamp + chrono::Duration::weeks(1);
        let mut cluster = TemporalCluster::new(TemporalGranularity::Weekly, start, end);
        cluster.add_episode(episode_id);
        self.temporal_clusters.push(cluster);
        self.total_episodes += 1;
        self.last_updated = Utc::now();
    }

    /// Insert an episode from Episode struct.
    ///
    /// # Arguments
    ///
    /// * `episode` - Episode to insert
    pub fn insert_from_episode(&mut self, episode: &crate::Episode) {
        self.insert_episode(episode.episode_id, episode.start_time);
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

        if removed {
            self.total_episodes = self.total_episodes.saturating_sub(1);
            self.last_updated = Utc::now();
        }

        removed
    }

    /// Get episodes within a time range.
    ///
    /// # Arguments
    ///
    /// * `start` - Start of the time range
    /// * `end` - End of the time range
    ///
    /// # Returns
    ///
    /// Vector of episode IDs in the time range.
    #[must_use]
    pub fn get_episodes_in_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<Uuid> {
        let mut episodes = Vec::new();

        for cluster in &self.temporal_clusters {
            // Check if cluster overlaps with the query range
            if cluster.start_time < end && cluster.end_time > start {
                // Filter episodes within the range
                for episode_id in &cluster.episode_ids {
                    // We can't filter individual episodes by timestamp here since
                    // we only have IDs, so we include all episodes from overlapping clusters
                    episodes.push(*episode_id);
                }
            }
        }

        episodes
    }

    /// Clean up empty clusters.
    pub fn cleanup_empty_clusters(&mut self) {
        self.temporal_clusters
            .retain(|cluster| !cluster.episode_ids.is_empty());
    }
}

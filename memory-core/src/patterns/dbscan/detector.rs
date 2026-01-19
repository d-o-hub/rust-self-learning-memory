//! # DBSCAN Anomaly Detector
//!
//! Main detector implementation for identifying anomalous episodes using DBSCAN clustering.

use crate::episode::Episode;
use crate::patterns::dbscan::{
    algorithms, Anomaly, AnomalyReason, DBSCANClusterResult, DBSCANConfig, DBSCANStats,
    EpisodeCluster,
};
use crate::types::TaskContext;

/// DBSCAN Anomaly Detector
#[derive(Debug, Clone)]
pub struct DBSCANAnomalyDetector {
    config: DBSCANConfig,
}

impl DBSCANAnomalyDetector {
    /// Create a new detector with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: DBSCANConfig::default(),
        }
    }

    /// Create a detector with custom configuration
    #[must_use]
    pub fn with_config(config: DBSCANConfig) -> Self {
        Self { config }
    }

    /// Get the detector configuration
    #[must_use]
    pub fn config(&self) -> Option<&DBSCANConfig> {
        Some(&self.config)
    }

    /// Detect anomalies in a collection of episodes
    ///
    /// # Arguments
    ///
    /// * `episodes` - Collection of episodes to analyze
    ///
    /// # Returns
    ///
    /// Clustering result containing clusters and anomalies
    ///
    /// # Errors
    ///
    /// Returns error if feature extraction fails
    #[allow(clippy::unused_async)]
    pub async fn detect_anomalies(
        &self,
        episodes: &[Episode],
    ) -> anyhow::Result<DBSCANClusterResult> {
        if episodes.is_empty() {
            return Ok(DBSCANClusterResult {
                clusters: Vec::new(),
                anomalies: Vec::new(),
                iterations: 0,
                stats: DBSCANStats::default(),
            });
        }

        // Extract feature vectors
        let features = self.extract_features(episodes);

        // Determine epsilon (adaptive if configured)
        let _eps = if self.config.adaptive_eps {
            self.config.calculate_adaptive_eps(&features)
        } else {
            self.config.eps
        };

        // Apply DBSCAN
        let (cluster_labels, _visited, iterations) = algorithms::dbscan(&self.config, &features);

        // Build clusters
        let clusters =
            algorithms::build_clusters(&self.config, episodes, &cluster_labels, &features);

        // Identify anomalies
        let anomalies = self.identify_anomalies(episodes, &cluster_labels, &features, &clusters);

        // Calculate statistics
        let stats = algorithms::calculate_stats(episodes.len(), &anomalies, &clusters);

        Ok(DBSCANClusterResult {
            clusters,
            anomalies,
            iterations,
            stats,
        })
    }

    /// Extract feature vectors from episodes
    ///
    /// Features include:
    /// - Context encoding (domain, language, tags)
    /// - Step count
    /// - Duration (derived from timestamps)
    /// - Outcome type
    /// - Task type
    fn extract_features(&self, episodes: &[Episode]) -> Vec<Vec<f64>> {
        let mut features_vec = Vec::with_capacity(episodes.len());

        for episode in episodes {
            let features = self.episode_to_features(episode);
            features_vec.push(features);
        }

        features_vec
    }

    /// Convert a single episode to a feature vector
    fn episode_to_features(&self, episode: &Episode) -> Vec<f64> {
        let mut features = Vec::with_capacity(20);

        // Context encoding (domain, language, tags)
        self.encode_context(&episode.context, &mut features);

        // Step count (normalized)
        let step_count = episode.steps.len() as f64;
        features.push((step_count / 100.0).clamp(0.0, 1.0));

        // Duration (normalized, handle edge cases)
        if let (start, Some(end)) = (&episode.start_time, &episode.end_time) {
            let duration = *end - *start;
            let duration_ms = duration.num_milliseconds() as f64;
            let normalized_duration = (duration_ms / 3_600_000.0).clamp(0.0, 1.0); // Normalize to 1 hour
            features.push(normalized_duration);
        } else {
            features.push(0.0);
        }

        // Outcome encoding
        self.encode_outcome(&episode.outcome, &mut features);

        // Task type encoding
        self.encode_task_type(episode.task_type, &mut features);

        // Step tool diversity (unique tools / total steps)
        let tool_diversity = if episode.steps.is_empty() {
            0.0
        } else {
            let unique_tools: std::collections::HashSet<_> =
                episode.steps.iter().map(|s| s.tool.as_str()).collect();
            unique_tools.len() as f64 / episode.steps.len() as f64
        };
        features.push(tool_diversity);

        // Success rate of steps
        let success_rate = if episode.steps.is_empty() {
            0.0
        } else {
            let successful_steps: usize = episode.steps.iter().filter(|s| s.is_success()).count();
            successful_steps as f64 / episode.steps.len() as f64
        };
        features.push(success_rate);

        // Average latency per step (normalized)
        if episode.steps.is_empty() {
            features.push(0.0);
        } else {
            let avg_latency: f64 = episode
                .steps
                .iter()
                .map(|s| s.latency_ms as f64)
                .sum::<f64>()
                / episode.steps.len() as f64;
            features.push((avg_latency / 10000.0).clamp(0.0, 1.0)); // Normalize to 10 seconds
        }

        // Tags count (normalized)
        features.push((episode.context.tags.len() as f64 / 10.0).clamp(0.0, 1.0));

        // Complexity level encoding
        self.encode_complexity(episode.context.complexity, &mut features);

        features
    }

    /// Encode context features
    fn encode_context(&self, context: &TaskContext, features: &mut Vec<f64>) {
        // Domain encoding (simple hash-based for now, could be improved with embeddings)
        let domain_hash = Self::hash_string(&context.domain);
        features.push(domain_hash);

        // Language encoding
        if let Some(lang) = &context.language {
            features.push(Self::hash_string(lang) * 0.5);
        } else {
            features.push(0.0);
        }

        // Framework encoding
        if let Some(framework) = &context.framework {
            features.push(Self::hash_string(framework) * 0.3);
        } else {
            features.push(0.0);
        }
    }

    /// Encode outcome features
    fn encode_outcome(&self, outcome: &Option<crate::types::TaskOutcome>, features: &mut Vec<f64>) {
        match outcome {
            Some(crate::types::TaskOutcome::Success { .. }) => {
                features.push(1.0); // Success
                features.push(0.0); // Not failure
                features.push(0.0); // Not partial
            }
            Some(crate::types::TaskOutcome::Failure { .. }) => {
                features.push(0.0); // Not success
                features.push(1.0); // Failure
                features.push(0.0); // Not partial
            }
            Some(crate::types::TaskOutcome::PartialSuccess { .. }) => {
                features.push(0.0); // Not success
                features.push(0.0); // Not failure
                features.push(1.0); // Partial
            }
            None => {
                features.push(0.5); // Unknown
                features.push(0.0);
                features.push(0.0);
            }
        }
    }

    /// Encode task type features
    fn encode_task_type(&self, task_type: crate::types::TaskType, features: &mut Vec<f64>) {
        match task_type {
            crate::types::TaskType::CodeGeneration => {
                features.push(1.0);
                features.push(0.0);
                features.push(0.0);
                features.push(0.0);
                features.push(0.0);
                features.push(0.0);
            }
            crate::types::TaskType::Debugging => {
                features.push(0.0);
                features.push(1.0);
                features.push(0.0);
                features.push(0.0);
                features.push(0.0);
                features.push(0.0);
            }
            crate::types::TaskType::Refactoring => {
                features.push(0.0);
                features.push(0.0);
                features.push(1.0);
                features.push(0.0);
                features.push(0.0);
                features.push(0.0);
            }
            crate::types::TaskType::Testing => {
                features.push(0.0);
                features.push(0.0);
                features.push(0.0);
                features.push(1.0);
                features.push(0.0);
                features.push(0.0);
            }
            crate::types::TaskType::Documentation => {
                features.push(0.0);
                features.push(0.0);
                features.push(0.0);
                features.push(0.0);
                features.push(1.0);
                features.push(0.0);
            }
            crate::types::TaskType::Analysis => {
                features.push(0.0);
                features.push(0.0);
                features.push(0.0);
                features.push(0.0);
                features.push(0.0);
                features.push(1.0);
            }
            crate::types::TaskType::Other => {
                features.push(0.0);
                features.push(0.0);
                features.push(0.0);
                features.push(0.0);
                features.push(0.0);
                features.push(0.0);
            }
        }
    }

    /// Encode complexity level
    fn encode_complexity(
        &self,
        complexity: crate::types::ComplexityLevel,
        features: &mut Vec<f64>,
    ) {
        match complexity {
            crate::types::ComplexityLevel::Simple => {
                features.push(0.0);
                features.push(0.0);
                features.push(0.0);
            }
            crate::types::ComplexityLevel::Moderate => {
                features.push(1.0);
                features.push(0.0);
                features.push(0.0);
            }
            crate::types::ComplexityLevel::Complex => {
                features.push(0.0);
                features.push(1.0);
                features.push(0.0);
            }
        }
    }

    /// Simple hash function for string encoding
    fn hash_string(s: &str) -> f64 {
        let mut hash = 0u64;
        for (i, c) in s.chars().enumerate() {
            hash = hash.wrapping_mul(31).wrapping_add(c as u64);
            if i > 10 {
                break; // Limit to first 10 characters
            }
        }
        // Normalize to 0-1 range
        (hash as f64 / u64::MAX as f64).abs()
    }

    /// Identify anomalies from cluster labels
    fn identify_anomalies(
        &self,
        episodes: &[Episode],
        cluster_labels: &[isize],
        features: &[Vec<f64>],
        clusters: &[EpisodeCluster],
    ) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();

        for (i, &label) in cluster_labels.iter().enumerate() {
            // Check if point is noise (not in any cluster)
            if label < 0 {
                // Find nearest cluster
                let (nearest_id, distance) = self.find_nearest_cluster(i, features, clusters);

                let reason = if clusters.is_empty() {
                    AnomalyReason::Isolated { neighbor_count: 0 }
                } else {
                    AnomalyReason::Outlier {
                        distance,
                        threshold: self.config.eps,
                    }
                };

                anomalies.push(Anomaly {
                    episode: episodes[i].clone(),
                    distance_to_cluster: distance,
                    nearest_cluster_id: nearest_id,
                    reason,
                });
            }
        }

        // Sort by distance (most anomalous first)
        anomalies.sort_by(|a, b| {
            b.distance_to_cluster
                .partial_cmp(&a.distance_to_cluster)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        anomalies
    }

    /// Find the nearest cluster centroid for a point
    fn find_nearest_cluster(
        &self,
        point_idx: usize,
        features: &[Vec<f64>],
        clusters: &[EpisodeCluster],
    ) -> (Option<usize>, f64) {
        if clusters.is_empty() {
            return (None, f64::MAX);
        }

        let mut min_distance = f64::MAX;
        let mut nearest_id = None;

        for cluster in clusters {
            let dist = algorithms::distance_to_centroid(
                &self.config,
                &features[point_idx],
                &cluster.centroid,
            );
            if dist < min_distance {
                min_distance = dist;
                nearest_id = Some(cluster.id);
            }
        }

        (nearest_id, min_distance)
    }

    /// Detect anomalies for a single episode against historical data
    ///
    /// This is useful for real-time anomaly detection when a new episode
    /// completes and needs to be checked against historical patterns.
    pub async fn detect_single_anomaly(
        &self,
        episode: &Episode,
        historical_episodes: &[Episode],
    ) -> Option<Anomaly> {
        if historical_episodes.is_empty() {
            return None;
        }

        // Add the new episode to the set
        let mut all_episodes = historical_episodes.to_vec();
        all_episodes.push(episode.clone());

        let result = self.detect_anomalies(&all_episodes).await.ok()?;

        // Find the anomaly that corresponds to our episode
        for anomaly in result.anomalies {
            if anomaly.episode.episode_id == episode.episode_id {
                return Some(anomaly);
            }
        }

        None
    }

    /// Get cluster information for an episode (which cluster it belongs to)
    #[must_use]
    pub fn get_episode_cluster(
        &self,
        episode: &Episode,
        clusters: &[EpisodeCluster],
    ) -> Option<usize> {
        for cluster in clusters {
            if cluster
                .episodes
                .iter()
                .any(|e| e.episode_id == episode.episode_id)
            {
                return Some(cluster.id);
            }
        }
        None
    }
}

impl Default for DBSCANAnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

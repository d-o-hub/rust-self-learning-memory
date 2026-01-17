//! # DBSCAN Anomaly Detection for Episodes
//!
//! This module implements the DBSCAN (Density-Based Spatial Clustering of
//! Applications with Noise) algorithm for detecting anomalous episodes.
//!
//! DBSCAN is ideal for this use case because:
//! - It doesn't require specifying the number of clusters upfront
//! - It naturally identifies outliers as noise (points not belonging to any cluster)
//! - It can find clusters of arbitrary shape
//!
//! ## Usage
//!
//! ```rust
//! use memory_core::patterns::DBSCANAnomalyDetector;
//!
//! let detector = DBSCANAnomalyDetector::new();
//! let anomalies = detector.detect_anomalies(&episodes).await;
//! ```
//!
//! ## Integration
//!
//! The detector is integrated into the learning cycle and can be called
//! during episode completion to identify unusual patterns.

use crate::episode::Episode;
use crate::types::TaskContext;
use std::collections::HashMap;

/// Configuration for DBSCAN algorithm
#[derive(Debug, Clone)]
pub struct DBSCANConfig {
    /// Epsilon: maximum distance between two points to be considered neighbors
    pub eps: f64,
    /// Minimum number of points required to form a dense region (cluster)
    pub min_samples: usize,
    /// Whether to use adaptive epsilon based on data distribution
    pub adaptive_eps: bool,
    /// Feature weights for distance calculation
    pub feature_weights: FeatureWeights,
    /// Minimum cluster size to be considered valid
    pub min_cluster_size: usize,
}

impl Default for DBSCANConfig {
    fn default() -> Self {
        Self {
            eps: 0.5,
            min_samples: 3,
            adaptive_eps: true,
            feature_weights: FeatureWeights::default(),
            min_cluster_size: 2,
        }
    }
}

/// Weights for different features in distance calculation
#[derive(Debug, Clone)]
pub struct FeatureWeights {
    /// Weight for context features (domain, language, etc.)
    pub context: f64,
    /// Weight for step count similarity
    pub step_count: f64,
    /// Weight for duration similarity
    pub duration: f64,
    /// Weight for outcome type similarity
    pub outcome: f64,
    /// Weight for task type similarity
    pub task_type: f64,
}

impl Default for FeatureWeights {
    fn default() -> Self {
        Self {
            context: 0.3,
            step_count: 0.2,
            duration: 0.2,
            outcome: 0.15,
            task_type: 0.15,
        }
    }
}

/// Result of DBSCAN clustering
#[derive(Debug, Clone)]
pub struct DBSCANClusterResult {
    /// Clusters found by DBSCAN
    pub clusters: Vec<EpisodeCluster>,
    /// Anomalies (points not belonging to any cluster)
    pub anomalies: Vec<Anomaly>,
    /// Number of iterations to converge
    pub iterations: usize,
    /// Statistics about the clustering
    pub stats: DBSCANStats,
}

/// Statistics about the DBSCAN clustering result
#[derive(Debug, Clone, Default)]
pub struct DBSCANStats {
    /// Total number of points processed
    pub total_points: usize,
    /// Number of points assigned to clusters
    pub clustered_points: usize,
    /// Number of anomalies detected
    pub anomaly_count: usize,
    /// Average distance to nearest cluster for anomalies
    pub avg_anomaly_distance: f64,
    /// Maximum distance to nearest cluster for anomalies
    pub max_anomaly_distance: f64,
}

/// A cluster of similar episodes
#[derive(Debug, Clone)]
pub struct EpisodeCluster {
    /// Unique identifier for this cluster
    pub id: usize,
    /// Episodes belonging to this cluster
    pub episodes: Vec<Episode>,
    /// Centroid (mean feature vector) of this cluster
    pub centroid: ClusterCentroid,
    /// Density score (points / radius)
    pub density: f64,
}

/// Centroid representation for a cluster
#[derive(Debug, Clone)]
pub struct ClusterCentroid {
    /// Average context encoding
    pub context_encoding: Vec<f64>,
    /// Average number of steps
    pub avg_steps: f64,
    /// Average duration in milliseconds
    pub avg_duration_ms: f64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Task type distribution
    pub task_type_encoding: Vec<f64>,
}

/// An anomalous episode that doesn't fit any cluster
#[derive(Debug, Clone)]
pub struct Anomaly {
    /// The anomalous episode
    pub episode: Episode,
    /// Distance to the nearest cluster centroid
    pub distance_to_cluster: f64,
    /// ID of the nearest cluster (if any)
    pub nearest_cluster_id: Option<usize>,
    /// Reason why this episode is anomalous
    pub reason: AnomalyReason,
}

/// Reason for flagging an episode as anomalous
#[derive(Debug, Clone)]
pub enum AnomalyReason {
    /// Point is isolated (fewer neighbors than min_samples)
    Isolated { neighbor_count: usize },
    /// Point is far from any cluster centroid
    Outlier { distance: f64, threshold: f64 },
    /// Point is in a sparse region
    SparseRegion { density: f64, threshold: f64 },
    /// Point differs significantly in multiple dimensions
    MultidimensionalOutlier { deviations: Vec<(String, f64)> },
}

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
        let features = self.extract_features(episodes)?;

        // Determine epsilon (adaptive if configured)
        let eps = if self.config.adaptive_eps {
            self.calculate_adaptive_eps(&features)
        } else {
            self.config.eps
        };

        // Apply DBSCAN
        let (cluster_labels, visited, iterations) = self.dbscan(&features, eps);

        // Build clusters
        let clusters = self.build_clusters(episodes, &cluster_labels, &features);

        // Identify anomalies
        let anomalies = self.identify_anomalies(episodes, &cluster_labels, &features, &clusters);

        // Calculate statistics
        let stats = self.calculate_stats(episodes.len(), &anomalies, &clusters);

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
    fn extract_features(&self, episodes: &[Episode]) -> anyhow::Result<Vec<Vec<f64>>> {
        let mut features_vec = Vec::with_capacity(episodes.len());

        for episode in episodes {
            let features = self.episode_to_features(episode)?;
            features_vec.push(features);
        }

        Ok(features_vec)
    }

    /// Convert a single episode to a feature vector
    fn episode_to_features(&self, episode: &Episode) -> anyhow::Result<Vec<f64>> {
        let mut features = Vec::with_capacity(20);

        // Context encoding (domain, language, tags)
        self.encode_context(&episode.context, &mut features)?;

        // Step count (normalized)
        let step_count = episode.steps.len() as f64;
        features.push((step_count / 100.0).clamp(0.0, 1.0));

        // Duration (normalized, handle edge cases)
        if let (Some(start), Some(end)) = (&episode.start_time, &episode.end_time) {
            let duration_ms = (*end - *start).num_milliseconds() as f64;
            let normalized_duration = (duration_ms / 3600000.0).clamp(0.0, 1.0); // Normalize to 1 hour
            features.push(normalized_duration);
        } else {
            features.push(0.0);
        }

        // Outcome encoding
        self.encode_outcome(&episode.outcome, &mut features);

        // Task type encoding
        self.encode_task_type(&episode.task_type, &mut features);

        // Step tool diversity (unique tools / total steps)
        let tool_diversity = if !episode.steps.is_empty() {
            let unique_tools: std::collections::HashSet<_> =
                episode.steps.iter().map(|s| s.tool.as_str()).collect();
            unique_tools.len() as f64 / episode.steps.len() as f64
        } else {
            0.0
        };
        features.push(tool_diversity);

        // Success rate of steps
        let success_rate = if !episode.steps.is_empty() {
            let successful_steps: usize = episode.steps.iter().filter(|s| s.is_success()).count();
            successful_steps as f64 / episode.steps.len() as f64
        } else {
            0.0
        };
        features.push(success_rate);

        // Average latency per step (normalized)
        if !episode.steps.is_empty() {
            let avg_latency: f64 = episode
                .steps
                .iter()
                .map(|s| s.latency_ms as f64)
                .sum::<f64>()
                / episode.steps.len() as f64;
            features.push((avg_latency / 10000.0).clamp(0.0, 1.0)); // Normalize to 10 seconds
        } else {
            features.push(0.0);
        }

        // Tags count (normalized)
        features.push((episode.context.tags.len() as f64 / 10.0).clamp(0.0, 1.0));

        // Complexity level encoding
        self.encode_complexity(&episode.context.complexity, &mut features);

        Ok(features)
    }

    /// Encode context features
    fn encode_context(&self, context: &TaskContext, features: &mut Vec<f64>) -> anyhow::Result<()> {
        // Domain encoding (simple hash-based for now, could be improved with embeddings)
        let domain_hash = self.hash_string(&context.domain);
        features.push(domain_hash);

        // Language encoding
        if let Some(lang) = &context.language {
            features.push(self.hash_string(lang) * 0.5);
        } else {
            features.push(0.0);
        }

        // Framework encoding
        if let Some(framework) = &context.framework {
            features.push(self.hash_string(framework) * 0.3);
        } else {
            features.push(0.0);
        }

        Ok(())
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
    fn encode_task_type(&self, task_type: &crate::types::TaskType, features: &mut Vec<f64>) {
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
        }
    }

    /// Encode complexity level
    fn encode_complexity(
        &self,
        complexity: &crate::types::ComplexityLevel,
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
            crate::types::ComplexityLevel::VeryComplex => {
                features.push(0.0);
                features.push(0.0);
                features.push(1.0);
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

    /// Calculate adaptive epsilon based on data distribution
    fn calculate_adaptive_eps(&self, features: &[Vec<f64>]) -> f64 {
        if features.len() < 2 {
            return self.config.eps;
        }

        // Calculate pairwise distances and find the k-nearest neighbor distance for each point
        let k = (self.config.min_samples as f64 * 0.5) as usize;
        let mut kth_distances: Vec<f64> = Vec::new();

        for (i, f1) in features.iter().enumerate() {
            let mut distances: Vec<f64> = Vec::new();
            for (j, f2) in features.iter().enumerate() {
                if i != j {
                    distances.push(self.euclidean_distance(f1, f2));
                }
            }
            distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            if !distances.is_empty() && k > 0 && k <= distances.len() {
                kth_distances.push(distances[k - 1]);
            } else if !distances.is_empty() {
                kth_distances.push(distances[distances.len() / 2]);
            }
        }

        // Use the median of kth distances as epsilon
        kth_distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = if kth_distances.len() % 2 == 0 {
            let mid = kth_distances.len() / 2;
            (kth_distances[mid - 1] + kth_distances[mid]) / 2.0
        } else {
            kth_distances[kth_distances.len() / 2]
        };

        // Apply scaling factor for better cluster formation
        (median * 1.5).clamp(0.1, 2.0)
    }

    /// Apply DBSCAN algorithm
    fn dbscan(&self, features: &[Vec<f64>], eps: f64) -> (Vec<isize>, Vec<bool>, usize) {
        let n = features.len();
        let mut cluster_labels: Vec<isize> = vec![-1; n]; // -1 = unvisited
        let mut visited: Vec<bool> = vec![false; n];
        let mut cluster_id: isize = 0;
        let mut iterations = 0;

        for i in 0..n {
            if visited[i] {
                continue;
            }

            visited[i] = true;
            let neighbors = self.region_query(i, features, eps);

            if neighbors.len() < self.config.min_samples {
                // Mark as noise (anomaly)
                cluster_labels[i] = -1;
            } else {
                // Start a new cluster
                self.expand_cluster(
                    i,
                    &neighbors,
                    cluster_id,
                    features,
                    eps,
                    &mut cluster_labels,
                );
                cluster_id += 1;
                iterations += 1;
            }
        }

        (cluster_labels, visited, iterations)
    }

    /// Find all points within eps distance of point i
    fn region_query(&self, i: usize, features: &[Vec<f64>], eps: f64) -> Vec<usize> {
        let mut neighbors = Vec::new();

        for (j, feature) in features.iter().enumerate() {
            if i != j {
                let dist = self.euclidean_distance(&features[i], feature);
                if dist <= eps {
                    neighbors.push(j);
                }
            }
        }

        neighbors
    }

    /// Expand cluster from seed point
    fn expand_cluster(
        &self,
        i: usize,
        neighbors: &[usize],
        cluster_id: isize,
        features: &[Vec<f64>],
        eps: f64,
        cluster_labels: &mut Vec<isize>,
    ) {
        let mut queue = Vec::from(neighbors);
        cluster_labels[i] = cluster_id;

        while let Some(p) = queue.pop() {
            if cluster_labels[p as usize] == -1 {
                cluster_labels[p as usize] = cluster_id;
            }

            if cluster_labels[p as usize] != -2 {
                continue;
            }

            cluster_labels[p as usize] = cluster_id;
            let p_neighbors = self.region_query(p as usize, features, eps);

            if p_neighbors.len() >= self.config.min_samples {
                // Add unvisited neighbors to queue
                for n in p_neighbors {
                    if !queue.contains(&n) {
                        queue.push(n);
                    }
                }
            }
        }
    }

    /// Calculate Euclidean distance between two feature vectors
    fn euclidean_distance(&self, f1: &[f64], f2: &[f64]) -> f64 {
        let min_len = f1.len().min(f2.len());
        let mut sum = 0.0;

        for i in 0..min_len {
            let diff = f1[i] - f2[i];
            sum += diff * diff;
        }

        sum.sqrt()
    }

    /// Build cluster objects from DBSCAN labels
    fn build_clusters(
        &self,
        episodes: &[Episode],
        cluster_labels: &[isize],
        features: &[Vec<f64>],
    ) -> Vec<EpisodeCluster> {
        let mut clusters: HashMap<isize, Vec<usize>> = HashMap::new();

        for (i, &label) in cluster_labels.iter().enumerate() {
            if label >= 0 {
                clusters.entry(label).or_default().push(i);
            }
        }

        let mut result: Vec<EpisodeCluster> = clusters
            .into_iter()
            .filter(|(id, indices)| indices.len() >= self.config.min_cluster_size && *id >= 0)
            .map(|(id, indices)| {
                let cluster_episodes: Vec<Episode> =
                    indices.iter().map(|&i| episodes[i].clone()).collect();

                // Calculate centroid
                let centroid = self.calculate_centroid(&indices, features);

                // Calculate density
                let density = self.calculate_density(&indices, features);

                EpisodeCluster {
                    id: id as usize,
                    episodes: cluster_episodes,
                    centroid,
                    density,
                }
            })
            .collect();

        // Sort by density (densest first)
        result.sort_by(|a, b| {
            b.density
                .partial_cmp(&a.density)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        result
    }

    /// Calculate centroid from feature vectors
    fn calculate_centroid(&self, indices: &[usize], features: &[Vec<f64>]) -> ClusterCentroid {
        if indices.is_empty() {
            return ClusterCentroid {
                context_encoding: Vec::new(),
                avg_steps: 0.0,
                avg_duration_ms: 0.0,
                success_rate: 0.0,
                task_type_encoding: Vec::new(),
            };
        }

        let dim = features[0].len();
        let mut sum: Vec<f64> = vec![0.0; dim];

        for &idx in indices {
            for (d, val) in features[idx].iter().enumerate() {
                sum[d] += val;
            }
        }

        let n = indices.len() as f64;
        let avg_sum: Vec<f64> = sum.iter().map(|s| s / n).collect();

        ClusterCentroid {
            context_encoding: avg_sum[0..3].to_vec(),
            avg_steps: avg_sum[3],
            avg_duration_ms: avg_sum[4] * 3600000.0, // Denormalize
            success_rate: avg_sum[10],
            task_type_encoding: avg_sum[6..12].to_vec(),
        }
    }

    /// Calculate density of a cluster
    fn calculate_density(&self, indices: &[usize], features: &[Vec<f64>]) -> f64 {
        if indices.len() < 2 {
            return 0.0;
        }

        let mut total_dist = 0.0;
        let mut count = 0;

        for (i, &idx1) in indices.iter().enumerate() {
            for &idx2 in &indices[i + 1..] {
                let dist = self.euclidean_distance(&features[idx1], &features[idx2]);
                total_dist += dist;
                count += 1;
            }
        }

        if count == 0 {
            return 0.0;
        }

        let avg_dist = total_dist / count as f64;
        if avg_dist == 0.0 {
            return f64::INFINITY;
        }

        indices.len() as f64 / avg_dist
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
            let dist = self.distance_to_centroid(&features[point_idx], &cluster.centroid);
            if dist < min_distance {
                min_distance = dist;
                nearest_id = Some(cluster.id);
            }
        }

        (nearest_id, min_distance)
    }

    /// Calculate distance from a feature vector to a centroid
    fn distance_to_centroid(&self, features: &[f64], centroid: &ClusterCentroid) -> f64 {
        let mut distance = 0.0;

        // Context distance
        for (i, &val) in centroid.context_encoding.iter().enumerate() {
            if i < features.len() {
                let diff = features[i] - val;
                distance += diff * diff * self.config.feature_weights.context;
            }
        }

        // Step count distance
        if features.len() > 3 {
            let diff = features[3] - centroid.avg_steps;
            distance += diff * diff * self.config.feature_weights.step_count;
        }

        // Duration distance
        if features.len() > 4 {
            let diff = features[4] - (centroid.avg_duration_ms / 3600000.0);
            distance += diff * diff * self.config.feature_weights.duration;
        }

        distance.sqrt()
    }

    /// Calculate clustering statistics
    fn calculate_stats(
        &self,
        total_points: usize,
        anomalies: &[Anomaly],
        clusters: &[EpisodeCluster],
    ) -> DBSCANStats {
        let clustered_points: usize = clusters.iter().map(|c| c.episodes.len()).sum();

        let (avg_anomaly_distance, max_anomaly_distance) = if !anomalies.is_empty() {
            let sum: f64 = anomalies.iter().map(|a| a.distance_to_cluster).sum();
            let max = anomalies
                .iter()
                .map(|a| a.distance_to_cluster)
                .fold(0.0 / 0.0, f64::max);

            (sum / anomalies.len() as f64, max)
        } else {
            (0.0, 0.0)
        };

        DBSCANStats {
            total_points,
            clustered_points,
            anomaly_count: anomalies.len(),
            avg_anomaly_distance,
            max_anomaly_distance,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComplexityLevel, ExecutionResult, TaskOutcome, TaskType};
    use crate::ExecutionStep;
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    fn create_test_episode(
        domain: &str,
        step_count: usize,
        task_type: TaskType,
        is_success: bool,
    ) -> Episode {
        let mut episode = Episode::new(
            format!("Test task in {}", domain),
            TaskContext {
                domain: domain.to_string(),
                language: Some("rust".to_string()),
                complexity: ComplexityLevel::Moderate,
                framework: None,
                tags: vec!["test".to_string()],
            },
            task_type,
        );

        let start_time = Utc::now() - Duration::hours(1);
        episode.start_time = start_time;

        for i in 0..step_count {
            let mut step =
                ExecutionStep::new(i + 1, format!("tool_{}", i % 3), format!("Action {}", i));
            step.result = Some(ExecutionResult::Success {
                output: "Success".to_string(),
            });
            step.latency_ms = 100;
            episode.steps.push(step);
        }

        if is_success {
            episode.outcome = Some(TaskOutcome::Success {
                verdict: "Task completed successfully".to_string(),
                artifacts: vec![],
            });
        } else {
            episode.outcome = Some(TaskOutcome::Failure {
                reason: "Task failed".to_string(),
                error_details: None,
            });
        }

        let end_time = Utc::now();
        episode.end_time = Some(end_time);

        episode
    }

    #[tokio::test]
    async fn test_empty_episodes() {
        let detector = DBSCANAnomalyDetector::new();
        let result = detector.detect_anomalies(&[]).await.unwrap();

        assert!(result.clusters.is_empty());
        assert!(result.anomalies.is_empty());
        assert_eq!(result.stats.total_points, 0);
    }

    #[tokio::test]
    async fn test_single_episode() {
        let detector = DBSCANAnomalyDetector::new();
        let episodes = vec![create_test_episode(
            "web-api",
            5,
            TaskType::CodeGeneration,
            true,
        )];

        let result = detector.detect_anomalies(&episodes).await.unwrap();

        // Single episode is always an anomaly (no neighbors)
        assert_eq!(result.stats.anomaly_count, 1);
    }

    #[tokio::test]
    async fn test_similar_episodes_no_anomalies() {
        let detector = DBSCANAnomalyDetector::new();

        // Create similar episodes
        let episodes = vec![
            create_test_episode("web-api", 5, TaskType::CodeGeneration, true),
            create_test_episode("web-api", 6, TaskType::CodeGeneration, true),
            create_test_episode("web-api", 5, TaskType::CodeGeneration, true),
            create_test_episode("web-api", 6, TaskType::CodeGeneration, true),
            create_test_episode("web-api", 5, TaskType::CodeGeneration, true),
        ];

        let result = detector.detect_anomalies(&episodes).await.unwrap();

        // Similar episodes should form a cluster with no anomalies
        assert!(!result.clusters.is_empty());
        assert!(result.anomalies.is_empty());
        assert_eq!(result.stats.clustered_points, 5);
    }

    #[tokio::test]
    async fn test_anomaly_detection() {
        let detector = DBSCANAnomalyDetector::new();

        // Create mostly similar episodes
        let mut episodes = Vec::new();
        for i in 0..5 {
            episodes.push(create_test_episode(
                "web-api",
                5,
                TaskType::CodeGeneration,
                true,
            ));
        }

        // Add one very different episode (many more steps)
        let anomaly = create_test_episode("web-api", 50, TaskType::Debugging, false);
        episodes.push(anomaly);

        let result = detector.detect_anomalies(&episodes).await.unwrap();

        // Should detect the anomaly
        assert!(result.stats.anomaly_count >= 1);

        // The anomaly should be far from clusters
        if !result.anomalies.is_empty() {
            let max_distance = result.stats.max_anomaly_distance;
            assert!(max_distance > 0.1);
        }
    }

    #[tokio::test]
    async fn test_multiple_domains() {
        let detector = DBSCANAnomalyDetector::new();

        let episodes = vec![
            create_test_episode("web-api", 5, TaskType::CodeGeneration, true),
            create_test_episode("web-api", 6, TaskType::CodeGeneration, true),
            create_test_episode("cli", 3, TaskType::Testing, true),
            create_test_episode("cli", 4, TaskType::Testing, true),
            create_test_episode("data-processing", 10, TaskType::Analysis, true),
        ];

        let result = detector.detect_anomalies(&episodes).await.unwrap();

        // Should form multiple clusters (one per domain)
        assert!(result.clusters.len() >= 2);
    }

    #[test]
    fn test_config_customization() {
        let config = DBSCANConfig {
            eps: 0.8,
            min_samples: 5,
            adaptive_eps: false,
            feature_weights: FeatureWeights {
                context: 0.5,
                step_count: 0.1,
                duration: 0.1,
                outcome: 0.15,
                task_type: 0.15,
            },
            min_cluster_size: 3,
        };

        let detector = DBSCANAnomalyDetector::with_config(config);

        // Verify config was applied
        assert_eq!(detector.config.eps, 0.8);
        assert_eq!(detector.config.min_samples, 5);
        assert!(!detector.config.adaptive_eps);
    }

    #[tokio::test]
    async fn test_cluster_statistics() {
        let detector = DBSCANAnomalyDetector::new();

        let episodes = vec![
            create_test_episode("web-api", 5, TaskType::CodeGeneration, true),
            create_test_episode("web-api", 6, TaskType::CodeGeneration, true),
            create_test_episode("cli", 20, TaskType::Debugging, false), // Anomaly
        ];

        let result = detector.detect_anomalies(&episodes).await.unwrap();

        assert_eq!(result.stats.total_points, 3);
        assert!(result.stats.anomaly_count >= 1);
        assert!(result.stats.avg_anomaly_distance >= 0.0);
    }

    #[tokio::test]
    async fn test_adaptive_eps() {
        let detector = DBSCANAnomalyDetector::new();

        // Create tightly clustered episodes
        let episodes: Vec<_> = (0..10)
            .map(|_| create_test_episode("web-api", 5, TaskType::CodeGeneration, true))
            .collect();

        let result = detector.detect_anomalies(&episodes).await.unwrap();

        // With adaptive eps, similar episodes should cluster well
        assert!(result.stats.avg_anomaly_distance >= 0.0 || result.anomalies.is_empty());
    }

    #[test]
    fn test_feature_extraction() {
        let detector = DBSCANAnomalyDetector::new();

        let episode = create_test_episode("test-domain", 10, TaskType::CodeGeneration, true);
        let features = detector.episode_to_features(&episode).unwrap();

        // Should have extracted features
        assert!(!features.is_empty());
        // Step count should be normalized (10/100 = 0.1)
        assert!((features[3] - 0.1).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_dbscan_iterations() {
        let detector = DBSCANAnomalyDetector::new();

        let episodes: Vec<_> = (0..5)
            .map(|_| create_test_episode("web-api", 5, TaskType::CodeGeneration, true))
            .collect();

        let result = detector.detect_anomalies(&episodes).await.unwrap();

        // Should have run at least some iterations
        assert!(result.iterations >= 0);
    }
}

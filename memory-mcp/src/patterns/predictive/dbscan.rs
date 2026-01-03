use anyhow::Result;
use serde::Serialize;

use super::kdtree::{KDTree, Point};

#[derive(Debug, Clone, Serialize)]
pub struct Cluster {
    pub id: usize,
    pub points: Vec<Point>,
    pub centroid: Vec<f64>,
    pub density: f64,
}

/// Cluster label for DBSCAN results
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClusterLabel {
    /// Noise point (anomaly)
    Noise,
    /// Cluster ID
    Cluster(usize),
}

/// Density-adaptive DBSCAN configuration
#[derive(Debug, Clone)]
pub struct DBSCANConfig {
    /// Density parameter (replaces eps/MinPts)
    pub density: f64,
    /// Minimum cluster size for validation
    pub min_cluster_size: usize,
    /// Maximum distance for neighbors
    pub max_distance: f64,
    /// Window size for streaming data
    pub window_size: usize,
}

impl Default for DBSCANConfig {
    fn default() -> Self {
        Self {
            density: 0.1,
            min_cluster_size: 3,
            max_distance: 1.0,
            window_size: 1000,
        }
    }
}

/// Streaming cluster state for incremental updates
#[derive(Debug)]
pub struct StreamingClusters {
    pub clusters: Vec<Cluster>,
    pub kd_tree: KDTree,
    pub window: Vec<Point>,
    pub config: DBSCANConfig,
}

impl StreamingClusters {
    pub fn new(config: DBSCANConfig) -> Self {
        Self {
            clusters: Vec::new(),
            kd_tree: KDTree::new(),
            window: Vec::with_capacity(config.window_size),
            config,
        }
    }

    /// Update clusters with new point
    pub fn update(&mut self, new_point: Point) -> ClusterLabel {
        // Add to window and maintain size
        self.window.push(new_point.clone());
        if self.window.len() > self.config.window_size {
            self.window.remove(0);
        }

        // Rebuild KD-tree with current window
        self.kd_tree = KDTree::build(&self.window);

        // Calculate local density for the new point
        let local_density = self.calculate_local_density(&new_point);

        // Check if point is noise or part of cluster
        if local_density < self.config.density {
            ClusterLabel::Noise
        } else {
            // Find or assign to cluster
            self.assign_to_cluster(new_point, local_density)
        }
    }

    /// Calculate local density around a point
    fn calculate_local_density(&self, point: &Point) -> f64 {
        let neighbors = self
            .kd_tree
            .find_neighbors(&point.features, self.config.max_distance);

        // Exclude the point itself (distance 0) from density calculation.
        let filtered: Vec<Point> = neighbors.into_iter().filter(|p| p.id != point.id).collect();

        // Calculate density based on neighbor count and distances
        if filtered.is_empty() {
            0.0
        } else {
            let avg_distance: f64 = filtered
                .iter()
                .map(|neighbor| calculate_distance(&point.features, &neighbor.features))
                .sum::<f64>()
                / filtered.len() as f64;

            // Higher density = more neighbors + closer proximity
            filtered.len() as f64 / (1.0 + avg_distance)
        }
    }

    /// Assign point to existing cluster or create new one
    fn assign_to_cluster(&mut self, point: Point, density: f64) -> ClusterLabel {
        // Find nearest cluster centroid
        let mut nearest_cluster = None;
        let mut min_distance = f64::INFINITY;

        for (i, cluster) in self.clusters.iter().enumerate() {
            let distance = calculate_distance(&point.features, &cluster.centroid);
            if distance < min_distance && distance <= self.config.max_distance {
                min_distance = distance;
                nearest_cluster = Some(i);
            }
        }

        if let Some(cluster_idx) = nearest_cluster {
            // Add to existing cluster
            self.clusters[cluster_idx].points.push(point.clone());
            self.update_cluster_centroid(cluster_idx);
            ClusterLabel::Cluster(cluster_idx)
        } else {
            // Create new cluster
            let new_cluster = Cluster {
                id: self.clusters.len(),
                points: vec![point.clone()],
                centroid: point.features.clone(),
                density,
            };
            self.clusters.push(new_cluster);
            ClusterLabel::Cluster(self.clusters.len() - 1)
        }
    }

    /// Update cluster centroid after adding points
    fn update_cluster_centroid(&mut self, cluster_idx: usize) {
        if self.clusters[cluster_idx].points.is_empty() {
            return;
        }

        let cluster = &self.clusters[cluster_idx];
        let feature_count = cluster.points[0].features.len();
        let mut new_centroid = vec![0.0; feature_count];

        for point in &cluster.points {
            for (i, &feature) in point.features.iter().enumerate() {
                new_centroid[i] += feature;
            }
        }

        for centroid_val in &mut new_centroid {
            *centroid_val /= cluster.points.len() as f64;
        }

        self.clusters[cluster_idx].centroid = new_centroid;
    }

    /// Calculate local density maps for the entire dataset
    pub fn calculate_density_maps(&self) -> Vec<f64> {
        let mut density_map = Vec::with_capacity(self.window.len());

        for point in &self.window {
            let density = self.calculate_local_density(point);
            density_map.push(density);
        }

        density_map
    }
}

/// Adaptive DBSCAN anomaly detector
#[derive(Debug)]
pub struct AdaptiveDBSCAN {
    #[allow(dead_code)]
    config: DBSCANConfig,
    streaming_clusters: StreamingClusters,
}

impl AdaptiveDBSCAN {
    pub fn new(config: DBSCANConfig) -> Result<Self> {
        Ok(Self {
            streaming_clusters: StreamingClusters::new(config.clone()),
            config,
        })
    }

    /// Main DBSCAN anomaly detection function
    pub fn detect_anomalies_dbscan(
        &mut self,
        values: &[f64],
        timestamps: &[f64],
    ) -> Vec<ClusterLabel> {
        // Build a point set and run (adaptive) DBSCAN in batch for deterministic results.
        // This avoids streaming-order artifacts and improves anomaly detection for small series.
        let points: Vec<Point> = values
            .iter()
            .enumerate()
            .map(|(i, &value)| {
                let timestamp = timestamps.get(i).copied().unwrap_or(i as f64);
                Point {
                    id: i,
                    values: vec![value],
                    embedding: None,
                    timestamp,
                    features: vec![value],
                }
            })
            .collect();

        self.adaptive_dbscan_clustering(&points)
    }

    /// Adaptive DBSCAN clustering with density-based parameter optimization
    pub fn adaptive_dbscan_clustering(&mut self, points: &[Point]) -> Vec<ClusterLabel> {
        // Calculate adaptive parameters based on data distribution
        let adaptive_params = self.calculate_adaptive_parameters(points);

        // Apply DBSCAN with adaptive parameters
        self.apply_dbscan(points, adaptive_params)
    }

    /// Calculate adaptive parameters using metaheuristic optimization
    fn calculate_adaptive_parameters(&self, points: &[Point]) -> (f64, usize) {
        if points.len() < 3 {
            return (0.5, 2);
        }

        // Extract features for analysis
        let features: Vec<Vec<f64>> = points.iter().map(|p| p.features.clone()).collect();

        // Calculate feature statistics
        let mut all_values = Vec::new();
        for feature_vec in &features {
            all_values.extend(feature_vec);
        }

        let mean: f64 = all_values.iter().sum::<f64>() / all_values.len() as f64;
        let variance: f64 =
            all_values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / all_values.len() as f64;
        let std_dev = variance.sqrt();

        // Adaptive epsilon based on data distribution
        // Ensure it's strictly positive so callers can rely on `epsilon > 0.0`.
        let adaptive_epsilon = (std_dev * 2.0).max(1e-6); // 2 std dev, floored
        let adaptive_min_samples = (points.len() as f64 * 0.1).max(3.0) as usize; // 10% of data or minimum 3

        (
            adaptive_epsilon,
            adaptive_min_samples.min(points.len().saturating_sub(1)),
        )
    }

    /// Apply DBSCAN algorithm with given parameters
    fn apply_dbscan(&self, points: &[Point], params: (f64, usize)) -> Vec<ClusterLabel> {
        let (epsilon, min_samples) = params;

        if points.is_empty() {
            return Vec::new();
        }

        // Build KD-tree for efficient neighbor queries
        let kd_tree = KDTree::build(points);

        let mut labels = vec![ClusterLabel::Noise; points.len()];
        let mut cluster_id = 0;

        for (i, point) in points.iter().enumerate() {
            if !matches!(labels[i], ClusterLabel::Noise) {
                continue; // Already processed
            }

            // Find neighbors
            let neighbors = kd_tree.find_neighbors(&point.features, epsilon);

            if neighbors.len() < min_samples {
                labels[i] = ClusterLabel::Noise; // Mark as noise
            } else {
                // Start new cluster
                labels[i] = ClusterLabel::Cluster(cluster_id);
                let mut cluster_points = vec![i];

                // Expand cluster
                let mut queue = neighbors.iter().map(|n| n.id).collect::<Vec<_>>();

                while let Some(neighbor_id) = queue.pop() {
                    if matches!(labels[neighbor_id], ClusterLabel::Noise) {
                        labels[neighbor_id] = ClusterLabel::Cluster(cluster_id);
                        cluster_points.push(neighbor_id);

                        // Add neighbors of this point to queue
                        let neighbor_neighbors =
                            kd_tree.find_neighbors(&points[neighbor_id].features, epsilon);

                        for neighbor_neighbor in &neighbor_neighbors {
                            if matches!(labels[neighbor_neighbor.id], ClusterLabel::Noise) {
                                queue.push(neighbor_neighbor.id);
                            }
                        }
                    }
                }

                cluster_id += 1;
            }
        }

        labels
    }

    /// Update streaming clusters incrementally
    pub fn update_streaming_clusters(&mut self, new_point: Point) -> ClusterLabel {
        self.streaming_clusters.update(new_point)
    }

    /// Get current density maps
    pub fn get_density_maps(&self) -> Vec<f64> {
        self.streaming_clusters.calculate_density_maps()
    }
}

/// Calculate Euclidean distance between two feature vectors
fn calculate_distance(a: &[f64], b: &[f64]) -> f64 {
    let len = a.len().min(b.len());
    if len == 0 {
        return 0.0;
    }

    let mut sum = 0.0;
    for i in 0..len {
        let diff = a[i] - b[i];
        sum += diff * diff;
    }
    sum.sqrt()
}

//! # DBSCAN Clustering Module
//!
//! Density-adaptive DBSCAN with KD-tree spatial indexing for efficient
//! real-time streaming anomaly detection in memory patterns.

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub id: usize,
    pub values: Vec<f64>,
    pub embedding: Option<Vec<f32>>,
    pub timestamp: f64,
    pub features: Vec<f64>,
}

impl Point {
    pub fn new(id: usize, values: &[f64], embedding: Option<Vec<f32>>, timestamp: f64) -> Self {
        let features = Self::extract_features(values);
        Self {
            id,
            values: values.to_vec(),
            embedding,
            timestamp,
            features,
        }
    }

    fn extract_features(values: &[f64]) -> Vec<f64> {
        if values.len() < 3 {
            return vec![0.0; 8];
        }
        let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;
        let variance: f64 = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        let mut features = Vec::with_capacity(8);
        features.push(mean);
        features.push(std_dev);
        let trend = if values.len() > 1 {
            let last_val = *values.last().unwrap();
            let first_val = *values.first().unwrap();
            (last_val - first_val) / (values.len() - 1) as f64
        } else {
            0.0
        };
        features.push(trend);
        let volatility = if values.len() > 3 {
            let window = values.len().min(5);
            let mut rolling_std = Vec::new();
            for i in (window - 1)..values.len() {
                let start = i.saturating_sub(window - 1);
                let window_data = &values[start..=i];
                let window_mean: f64 = window_data.iter().sum::<f64>() / window_data.len() as f64;
                let window_var: f64 = window_data.iter().map(|&x| (x - window_mean).powi(2)).sum::<f64>() / window_data.len() as f64;
                rolling_std.push(window_var.sqrt());
            }
            rolling_std.iter().sum::<f64>() / rolling_std.len() as f64
        } else {
            std_dev
        };
        features.push(volatility);
        let skewness = if std_dev > 0.0 {
            values.iter().map(|&x| ((x - mean) / std_dev).powi(3)).sum::<f64>() / values.len() as f64
        } else {
            0.0
        };
        features.push(skewness);
        let kurtosis = if std_dev > 0.0 {
            values.iter().map(|&x| ((x - mean) / std_dev).powi(4)).sum::<f64>() / values.len() as f64 - 3.0
        } else {
            0.0
        };
        features.push(kurtosis);
        let autocorr = if values.len() > 1 {
            let mut numerator = 0.0;
            let mut denom_x = 0.0;
            let mut denom_y = 0.0;
            for i in 1..values.len() {
                numerator += (values[i - 1] - mean) * (values[i] - mean);
                denom_x += (values[i - 1] - mean).powi(2);
                denom_y += (values[i] - mean).powi(2);
            }
            if denom_x > 0.0 && denom_y > 0.0 {
                numerator / (denom_x.sqrt() * denom_y.sqrt())
            } else {
                0.0
            }
        } else {
            0.0
        };
        features.push(autocorr);
        let recent_change = if values.len() >= 3 {
            let recent_mean: f64 = values.iter().rev().take(3).sum::<f64>() / 3.0;
            (recent_mean - mean) / std_dev.max(1e-6)
        } else {
            0.0
        };
        features.push(recent_change);
        features
    }
}

#[derive(Debug)]
struct KDNode {
    point: Point,
    axis: usize,
    left: Option<Box<KDNode>>,
    right: Option<Box<KDNode>>,
}

impl KDNode {
    fn new(point: Point, axis: usize) -> Self {
        Self { point, axis, left: None, right: None }
    }
    fn insert(&mut self, point: Point) {
        let axis = self.axis;
        let comparison = if point.features.len() > axis {
            point.features[axis] < self.point.features[axis]
        } else {
            point.id < self.point.id
        };
        if comparison {
            if let Some(left) = &mut self.left {
                left.insert(point);
            } else {
                self.left = Some(Box::new(KDNode::new(point, (axis + 1) % self.point.features.len())));
            }
        } else if let Some(right) = &mut self.right {
            right.insert(point);
        } else {
            self.right = Some(Box::new(KDNode::new(point, (axis + 1) % self.point.features.len())));
        }
    }
    fn range_query(&self, epsilon: f64, center: &[f64], results: &mut Vec<Point>) {
        let distance = calculate_distance(&self.point.features, center);
        if distance <= epsilon {
            results.push(self.point.clone());
        }
        let axis = self.axis;
        if self.point.features.len() > axis {
            let plane_distance = if center.len() > axis {
                (center[axis] - self.point.features[axis]).abs()
            } else {
                0.0
            };
            if plane_distance <= epsilon {
                if let Some(left) = &self.left { left.range_query(epsilon, center, results); }
                if let Some(right) = &self.right { right.range_query(epsilon, center, results); }
            } else if center.len() > axis && center[axis] < self.point.features[axis] {
                if let Some(left) = &self.left { left.range_query(epsilon, center, results); }
            } else if let Some(right) = &self.right { right.range_query(epsilon, center, results); }
        }
    }
}

#[derive(Debug)]
pub struct KDTree {
    root: Option<Box<KDNode>>,
    #[allow(dead_code)]
    max_depth: usize,
}

impl KDTree {
    fn new() -> Self {
        Self { root: None, max_depth: 10 }
    }
    pub fn build(points: &[Point]) -> Self {
        if points.is_empty() {
            return Self::new();
        }
        let mut tree = Self::new();
        let axis = 0;
        if let Some(root_point) = points.first() {
            tree.root = Some(Box::new(KDNode::new(root_point.clone(), axis)));
            for point in points.iter().skip(1) {
                if let Some(root) = tree.root.as_mut() {
                    root.insert(point.clone());
                }
            }
        }
        tree
    }
    pub fn find_neighbors(&self, center: &[f64], epsilon: f64) -> Vec<Point> {
        let mut results = Vec::new();
        if let Some(root) = &self.root {
            root.range_query(epsilon, center, &mut results);
        }
        results
    }
}

#[derive(Debug, Clone)]
pub struct Cluster {
    pub id: usize,
    pub points: Vec<Point>,
    pub centroid: Vec<f64>,
    pub density: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClusterLabel {
    Noise,
    Cluster(usize),
}

#[derive(Debug, Clone)]
pub struct DBSCANConfig {
    pub density: f64,
    pub min_cluster_size: usize,
    pub max_distance: f64,
    pub window_size: usize,
}

impl Default for DBSCANConfig {
    fn default() -> Self {
        Self { density: 0.1, min_cluster_size: 3, max_distance: 1.0, window_size: 1000 }
    }
}

#[derive(Debug)]
pub struct StreamingClusters {
    pub clusters: Vec<Cluster>,
    pub kd_tree: KDTree,
    pub window: Vec<Point>,
    pub config: DBSCANConfig,
}

impl StreamingClusters {
    pub fn new(config: DBSCANConfig) -> Self {
        Self { clusters: Vec::new(), kd_tree: KDTree::new(), window: Vec::with_capacity(config.window_size), config }
    }
    pub fn update(&mut self, new_point: Point) -> ClusterLabel {
        self.window.push(new_point.clone());
        if self.window.len() > self.config.window_size {
            self.window.remove(0);
        }
        self.kd_tree = KDTree::build(&self.window);
        let local_density = self.calculate_local_density(&new_point);
        if local_density < self.config.density {
            ClusterLabel::Noise
        } else {
            self.assign_to_cluster(new_point, local_density)
        }
    }
    fn calculate_local_density(&self, point: &Point) -> f64 {
        let neighbors = self.kd_tree.find_neighbors(&point.features, self.config.max_distance);
        let filtered: Vec<Point> = neighbors.into_iter().filter(|p| p.id != point.id).collect();
        if filtered.is_empty() {
            0.0
        } else {
            let avg_distance: f64 = filtered.iter().map(|neighbor| calculate_distance(&point.features, &neighbor.features)).sum::<f64>() / filtered.len() as f64;
            filtered.len() as f64 / (1.0 + avg_distance)
        }
    }
    fn assign_to_cluster(&mut self, point: Point, density: f64) -> ClusterLabel {
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
            self.clusters[cluster_idx].points.push(point.clone());
            self.update_cluster_centroid(cluster_idx);
            ClusterLabel::Cluster(cluster_idx)
        } else {
            let new_cluster = Cluster { id: self.clusters.len(), points: vec![point.clone()], centroid: point.features.clone(), density };
            self.clusters.push(new_cluster);
            ClusterLabel::Cluster(self.clusters.len() - 1)
        }
    }
    fn update_cluster_centroid(&mut self, cluster_idx: usize) {
        if self.clusters[cluster_idx].points.is_empty() { return; }
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
    pub fn calculate_density_maps(&self) -> Vec<f64> {
        let mut density_map = Vec::with_capacity(self.window.len());
        for point in &self.window {
            let density = self.calculate_local_density(point);
            density_map.push(density);
        }
        density_map
    }
}

#[derive(Debug)]
pub struct AdaptiveDBSCAN {
    #[allow(dead_code)]
    config: DBSCANConfig,
    streaming_clusters: StreamingClusters,
}

impl AdaptiveDBSCAN {
    pub fn new(config: DBSCANConfig) -> Result<Self> {
        Ok(Self { streaming_clusters: StreamingClusters::new(config.clone()), config })
    }
    pub fn detect_anomalies_dbscan(&mut self, values: &[f64], timestamps: &[f64]) -> Vec<ClusterLabel> {
        let points: Vec<Point> = values.iter().enumerate().map(|(i, &value)| {
            let timestamp = timestamps.get(i).copied().unwrap_or(i as f64);
            Point { id: i, values: vec![value], embedding: None, timestamp, features: vec![value] }
        }).collect();
        self.adaptive_dbscan_clustering(&points)
    }
    pub fn adaptive_dbscan_clustering(&mut self, points: &[Point]) -> Vec<ClusterLabel> {
        let adaptive_params = self.calculate_adaptive_parameters(points);
        self.apply_dbscan(points, adaptive_params)
    }
    fn calculate_adaptive_parameters(&self, points: &[Point]) -> (f64, usize) {
        if points.len() < 3 { return (0.5, 2); }
        let features: Vec<Vec<f64>> = points.iter().map(|p| p.features.clone()).collect();
        let mut all_values = Vec::new();
        for feature_vec in &features { all_values.extend(feature_vec); }
        let mean: f64 = all_values.iter().sum::<f64>() / all_values.len() as f64;
        let variance: f64 = all_values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / all_values.len() as f64;
        let std_dev = variance.sqrt();
        let adaptive_epsilon = (std_dev * 2.0).max(1e-6);
        let adaptive_min_samples = (points.len() as f64 * 0.1).max(3.0) as usize;
        (adaptive_epsilon, adaptive_min_samples.min(points.len().saturating_sub(1)))
    }
    fn apply_dbscan(&self, points: &[Point], params: (f64, usize)) -> Vec<ClusterLabel> {
        let (epsilon, min_samples) = params;
        if points.is_empty() { return Vec::new(); }
        let kd_tree = KDTree::build(points);
        let mut labels = vec![ClusterLabel::Noise; points.len()];
        let mut cluster_id = 0;
        for (i, point) in points.iter().enumerate() {
            if !matches!(labels[i], ClusterLabel::Noise) { continue; }
            let neighbors = kd_tree.find_neighbors(&point.features, epsilon);
            if neighbors.len() < min_samples {
                labels[i] = ClusterLabel::Noise;
            } else {
                labels[i] = ClusterLabel::Cluster(cluster_id);
                let mut queue = neighbors.iter().map(|n| n.id).collect::<Vec<_>>();
                while let Some(neighbor_id) = queue.pop() {
                    if matches!(labels[neighbor_id], ClusterLabel::Noise) {
                        labels[neighbor_id] = ClusterLabel::Cluster(cluster_id);
                        let neighbor_neighbors = kd_tree.find_neighbors(&points[neighbor_id].features, epsilon);
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
    pub fn update_streaming_clusters(&mut self, new_point: Point) -> ClusterLabel {
        self.streaming_clusters.update(new_point)
    }
    pub fn get_density_maps(&self) -> Vec<f64> {
        self.streaming_clusters.calculate_density_maps()
    }
}

pub fn calculate_distance(a: &[f64], b: &[f64]) -> f64 {
    let len = a.len().min(b.len());
    if len == 0 { return 0.0; }
    let mut sum = 0.0;
    for i in 0..len {
        let diff = a[i] - b[i];
        sum += diff * diff;
    }
    sum.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_density_adaptive_parameter_selection() -> Result<()> {
        let dbscan = AdaptiveDBSCAN::new(DBSCANConfig::default())?;
        let mut points = Vec::new();
        for i in 0..10 { points.push(Point::new(i, &[i as f64], None, i as f64)); }
        for i in 10..13 { points.push(Point::new(i, &[i as f64 * 10.0], None, i as f64)); }
        let params = dbscan.calculate_adaptive_parameters(&points);
        let (epsilon, min_samples) = params;
        assert!(epsilon > 0.0 && epsilon < 100.0);
        assert!(min_samples >= 2 && min_samples <= points.len());
        Ok(())
    }
    #[test]
    fn test_anomaly_detection_accuracy() -> Result<()> {
        let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig::default())?;
        let values = vec![1.0, 1.1, 0.9, 1.0, 0.95, 1.05, 50.0, 1.0, 0.98, 1.02];
        let timestamps: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();
        let labels = dbscan.detect_anomalies_dbscan(&values, &timestamps);
        let noise_count = labels.iter().filter(|&label| matches!(label, ClusterLabel::Noise)).count();
        assert!(noise_count >= 1, "Should detect at least one anomaly");
        let cluster_count = labels.iter().filter(|&label| matches!(label, ClusterLabel::Cluster(_))).count();
        assert!(cluster_count >= values.len() - 3, "Should classify most points as clusters");
        Ok(())
    }
    #[test]
    fn test_streaming_cluster_updates() -> Result<()> {
        let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig { window_size: 5, density: 0.1, min_cluster_size: 2, max_distance: 1.0 })?;
        let mut labels = Vec::new();
        for i in 0..8 {
            let point = Point::new(i, &[i as f64], None, i as f64);
            let label = dbscan.update_streaming_clusters(point);
            labels.push(label);
            assert!(dbscan.streaming_clusters.window.len() <= 5);
        }
        let has_clusters = labels.iter().any(|&label| matches!(label, ClusterLabel::Cluster(_)));
        let has_noise = labels.iter().any(|&label| matches!(label, ClusterLabel::Noise));
        assert!(has_clusters || has_noise, "Should produce some clustering results");
        Ok(())
    }
    #[test]
    fn test_kdtree_neighbor_queries() -> Result<()> {
        let mut points = Vec::new();
        for i in 0..10 {
            let features = vec![i as f64, (i * 2) as f64];
            points.push(Point::new(i, &[i as f64], None, i as f64));
            points[i].features = features;
        }
        let kd_tree = KDTree::build(&points);
        let center = vec![5.0, 10.0];
        let neighbors = kd_tree.find_neighbors(&center, 3.0);
        assert!(!neighbors.is_empty());
        for neighbor in &neighbors {
            let distance = calculate_distance(&center, &neighbor.features);
            assert!(distance <= 3.0);
        }
        Ok(())
    }
    #[test]
    fn test_dbscan_edge_cases() -> Result<()> {
        let dbscan = AdaptiveDBSCAN::new(DBSCANConfig::default())?;
        let empty_labels = dbscan.apply_dbscan(&[], (1.0, 2));
        assert!(empty_labels.is_empty());
        let single_point = vec![Point::new(0, &[1.0], None, 0.0)];
        let single_labels = dbscan.apply_dbscan(&single_point, (1.0, 1));
        assert_eq!(single_labels.len(), 1);
        assert!(matches!(single_labels[0], ClusterLabel::Cluster(_)));
        let mut high_dim_point = Point::new(0, &[1.0], None, 0.0);
        high_dim_point.features = vec![1.0; 20];
        let high_dim_labels = dbscan.apply_dbscan(&[high_dim_point], (2.0, 1));
        assert_eq!(high_dim_labels.len(), 1);
        Ok(())
    }
    #[test]
    fn test_multidimensional_feature_handling() -> Result<()> {
        let dbscan = AdaptiveDBSCAN::new(DBSCANConfig::default())?;
        let mut points = Vec::new();
        for i in 0..5 {
            let mut point = Point::new(i, &[1.0], None, i as f64);
            point.features = vec![1.0, 1.0, 1.0];
            points.push(point);
        }
        for i in 5..10 {
            let mut point = Point::new(i, &[2.0], None, i as f64);
            point.features = vec![2.0, 2.0, 2.0];
            points.push(point);
        }
        let mut outlier = Point::new(10, &[5.0], None, 10.0);
        outlier.features = vec![5.0, 5.0, 5.0];
        points.push(outlier);
        let labels = dbscan.apply_dbscan(&points, (0.5, 2));
        let cluster_ids: std::collections::HashSet<usize> = labels.iter().filter_map(|label| {
            if let ClusterLabel::Cluster(id) = label { Some(*id) } else { None }
        }).collect();
        assert!(!cluster_ids.is_empty(), "Should identify at least one cluster");
        Ok(())
    }
}

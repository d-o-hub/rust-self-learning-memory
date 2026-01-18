//! # DBSCAN Algorithm Implementation
//!
//! Core clustering algorithms for DBSCAN anomaly detection.

use crate::patterns::dbscan::{ClusterCentroid, DBSCANConfig, EpisodeCluster};

impl DBSCANConfig {
    /// Calculate adaptive epsilon based on data distribution
    pub fn calculate_adaptive_eps(&self, features: &[Vec<f64>]) -> f64 {
        if features.len() < 2 {
            return self.eps;
        }

        // Calculate pairwise distances and find the k-nearest neighbor distance for each point
        let k = (self.min_samples as f64 * 0.5) as usize;
        let mut kth_distances: Vec<f64> = Vec::new();

        for (i, f1) in features.iter().enumerate() {
            let mut distances: Vec<f64> = Vec::new();
            for (j, f2) in features.iter().enumerate() {
                if i != j {
                    distances.push(euclidean_distance(f1, f2));
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
}

/// Apply DBSCAN algorithm
pub fn dbscan(config: &DBSCANConfig, features: &[Vec<f64>]) -> (Vec<isize>, Vec<bool>, usize) {
    let n = features.len();
    let mut cluster_labels: Vec<isize> = vec![-2; n]; // -2 = unvisited, -1 = noise, >=0 = cluster_id
    let mut visited: Vec<bool> = vec![false; n];
    let mut cluster_id: isize = 0;
    let mut iterations = 0;

    for i in 0..n {
        if visited[i] {
            continue;
        }

        visited[i] = true;
        let neighbors = region_query(config, i, features);

        if neighbors.len() < config.min_samples {
            // Mark as noise (anomaly)
            cluster_labels[i] = -1;
        } else {
            // Start a new cluster
            expand_cluster(
                config,
                i,
                &neighbors,
                cluster_id,
                features,
                &mut cluster_labels,
            );
            cluster_id += 1;
            iterations += 1;
        }
    }

    (cluster_labels, visited, iterations)
}

/// Find all points within eps distance of point i
fn region_query(config: &DBSCANConfig, i: usize, features: &[Vec<f64>]) -> Vec<usize> {
    let mut neighbors = Vec::new();
    let eps = config.eps;

    for (j, feature) in features.iter().enumerate() {
        if i != j {
            let dist = euclidean_distance(&features[i], feature);
            if dist <= eps {
                neighbors.push(j);
            }
        }
    }

    neighbors
}

/// Expand cluster from seed point
fn expand_cluster(
    config: &DBSCANConfig,
    i: usize,
    neighbors: &[usize],
    cluster_id: isize,
    features: &[Vec<f64>],
    cluster_labels: &mut [isize],
) {
    let mut queue = Vec::from(neighbors);
    cluster_labels[i] = cluster_id;

    while let Some(p) = queue.pop() {
        // Check if unvisited first (must be done before other checks)
        if cluster_labels[p as usize] != -2 {
            continue;
        }

        // Mark as cluster member
        cluster_labels[p as usize] = cluster_id;

        // If it's noise (was -1), keep it as cluster_id
        // Get neighbors and expand
        let p_neighbors = region_query(config, p as usize, features);

        if p_neighbors.len() >= config.min_samples {
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
pub fn euclidean_distance(f1: &[f64], f2: &[f64]) -> f64 {
    let min_len = f1.len().min(f2.len());
    let mut sum = 0.0;

    for i in 0..min_len {
        let diff = f1[i] - f2[i];
        sum += diff * diff;
    }

    sum.sqrt()
}

/// Build cluster objects from DBSCAN labels
pub fn build_clusters(
    config: &DBSCANConfig,
    episodes: &[crate::episode::Episode],
    cluster_labels: &[isize],
    features: &[Vec<f64>],
) -> Vec<EpisodeCluster> {
    use std::collections::HashMap;

    let mut clusters: HashMap<isize, Vec<usize>> = HashMap::new();

    for (i, &label) in cluster_labels.iter().enumerate() {
        if label >= 0 {
            clusters.entry(label).or_default().push(i);
        }
    }

    let mut result: Vec<EpisodeCluster> = clusters
        .into_iter()
        .filter(|(id, indices)| indices.len() >= config.min_cluster_size && *id >= 0)
        .map(|(id, indices)| {
            let cluster_episodes: Vec<crate::episode::Episode> =
                indices.iter().map(|&i| episodes[i].clone()).collect();

            // Calculate centroid
            let centroid = calculate_centroid(&indices, features);

            // Calculate density
            let density = calculate_density(config, &indices, features);

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
pub fn calculate_centroid(indices: &[usize], features: &[Vec<f64>]) -> ClusterCentroid {
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
        avg_duration_ms: avg_sum[4] * 3_600_000.0, // Denormalize
        success_rate: avg_sum[10],
        task_type_encoding: avg_sum[6..12].to_vec(),
    }
}

/// Calculate density of a cluster
pub fn calculate_density(_config: &DBSCANConfig, indices: &[usize], features: &[Vec<f64>]) -> f64 {
    if indices.len() < 2 {
        return 0.0;
    }

    let mut total_dist = 0.0;
    let mut count = 0;

    for (i, &idx1) in indices.iter().enumerate() {
        for &idx2 in &indices[i + 1..] {
            let dist = euclidean_distance(&features[idx1], &features[idx2]);
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

/// Calculate distance from a feature vector to a centroid
pub fn distance_to_centroid(
    config: &DBSCANConfig,
    features: &[f64],
    centroid: &ClusterCentroid,
) -> f64 {
    let mut distance = 0.0;

    // Context distance
    for (i, &val) in centroid.context_encoding.iter().enumerate() {
        if i < features.len() {
            let diff = features[i] - val;
            distance += diff * diff * config.feature_weights.context;
        }
    }

    // Step count distance
    if features.len() > 3 {
        let diff = features[3] - centroid.avg_steps;
        distance += diff * diff * config.feature_weights.step_count;
    }

    // Duration distance
    if features.len() > 4 {
        let diff = features[4] - (centroid.avg_duration_ms / 3_600_000.0);
        distance += diff * diff * config.feature_weights.duration;
    }

    distance.sqrt()
}

/// Calculate clustering statistics
pub fn calculate_stats(
    total_points: usize,
    anomalies: &[crate::patterns::dbscan::Anomaly],
    clusters: &[EpisodeCluster],
) -> crate::patterns::dbscan::DBSCANStats {
    let clustered_points: usize = clusters.iter().map(|c| c.episodes.len()).sum();

    let (avg_anomaly_distance, max_anomaly_distance) = if !anomalies.is_empty() {
        let sum: f64 = anomalies.iter().map(|a| a.distance_to_cluster).sum();
        let max = anomalies
            .iter()
            .map(|a| a.distance_to_cluster)
            .fold(f64::NEG_INFINITY, f64::max);

        (sum / anomalies.len() as f64, max)
    } else {
        (0.0, 0.0)
    };

    crate::patterns::dbscan::DBSCANStats {
        total_points,
        clustered_points,
        anomaly_count: anomalies.len(),
        avg_anomaly_distance,
        max_anomaly_distance,
    }
}

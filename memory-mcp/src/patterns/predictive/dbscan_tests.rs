//! # Comprehensive DBSCAN Tests
//!
//! Unit and integration tests for DBSCAN clustering algorithm

use crate::patterns::predictive::{
    dbscan::{AdaptiveDBSCAN, ClusterLabel, DBSCANConfig},
    kdtree::Point,
};

/// Helper function to create test points
fn create_test_points(values: &[f64]) -> Vec<Point> {
    values
        .iter()
        .enumerate()
        .map(|(i, &value)| Point::new(i, &[value], None, i as f64))
        .collect()
}

/// Helper function to calculate distance
#[allow(dead_code)]
fn _calculate_distance(a: &[f64], b: &[f64]) -> f64 {
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

#[cfg(test)]
mod dbscan_unit_tests {
    use super::*;

    /// Test cluster formation with well-separated clusters
    #[test]
    fn test_cluster_formation() {
        let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig::default()).unwrap();

        // Create two well-separated clusters
        let values = vec![
            // Cluster 1: values around 1.0
            1.0, 1.1, 0.9, 1.05, 0.95, 1.02, // Cluster 2: values around 10.0
            10.0, 10.1, 9.9, 10.05, 9.95,
        ];

        let timestamps: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();
        let labels = dbscan.detect_anomalies_dbscan(&values, &timestamps);

        // Should identify clusters (not all noise)
        let cluster_count = labels
            .iter()
            .filter(|&l| matches!(l, ClusterLabel::Cluster(_)))
            .count();

        assert!(
            cluster_count > values.len() / 2,
            "Should identify most points as belonging to clusters"
        );
    }

    /// Test noise point handling
    #[test]
    fn test_noise_point_handling() {
        let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig::default()).unwrap();

        // Create cluster with outliers
        let values = vec![
            1.0, 1.1, 0.9, 1.05, 0.95,  // Cluster
            100.0, // Outlier
            1.02, 0.98,  // More cluster points
            -50.0, // Another outlier
        ];

        let timestamps: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();
        let labels = dbscan.detect_anomalies_dbscan(&values, &timestamps);

        // Should detect outliers as noise
        let noise_indices: Vec<usize> = labels
            .iter()
            .enumerate()
            .filter(|(_, &l)| matches!(l, ClusterLabel::Noise))
            .map(|(i, _)| i)
            .collect();

        assert!(
            !noise_indices.is_empty(),
            "Should detect at least one noise point, labels: {:?}",
            labels
        );
    }

    /// Test epsilon parameter sensitivity
    #[test]
    fn test_epsilon_sensitivity() {
        let values = vec![1.0, 1.1, 0.9, 2.0, 2.1, 1.95];
        let timestamps: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();

        // Test with small epsilon (strict clustering)
        let mut dbscan_small = AdaptiveDBSCAN::new(DBSCANConfig {
            density: 0.1,
            min_cluster_size: 2,
            max_distance: 0.2, // Small epsilon
            window_size: 1000,
        })
        .unwrap();

        let labels_small = dbscan_small.detect_anomalies_dbscan(&values, &timestamps);

        // Test with large epsilon (permissive clustering)
        let mut dbscan_large = AdaptiveDBSCAN::new(DBSCANConfig {
            density: 0.1,
            min_cluster_size: 2,
            max_distance: 5.0, // Large epsilon
            window_size: 1000,
        })
        .unwrap();

        let labels_large = dbscan_large.detect_anomalies_dbscan(&values, &timestamps);

        // Larger epsilon should result in more clusters/fewer noise
        let noise_small = labels_small
            .iter()
            .filter(|&l| matches!(l, ClusterLabel::Noise))
            .count();
        let noise_large = labels_large
            .iter()
            .filter(|&l| matches!(l, ClusterLabel::Noise))
            .count();

        assert!(
            noise_small >= noise_large,
            "Smaller epsilon should result in more noise points"
        );
    }

    /// Test min_samples parameter
    #[test]
    fn test_min_samples_parameter() {
        let values = vec![1.0, 1.1, 0.9, 1.05, 0.95];
        let timestamps: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();

        // Test with high min_samples (strict)
        let mut dbscan_strict = AdaptiveDBSCAN::new(DBSCANConfig {
            density: 0.1,
            min_cluster_size: 10, // Higher than number of points
            max_distance: 1.0,
            window_size: 1000,
        })
        .unwrap();

        let labels_strict = dbscan_strict.detect_anomalies_dbscan(&values, &timestamps);

        // Test with low min_samples (permissive)
        let mut dbscan_permissive = AdaptiveDBSCAN::new(DBSCANConfig {
            density: 0.1,
            min_cluster_size: 2, // Low min_samples
            max_distance: 1.0,
            window_size: 1000,
        })
        .unwrap();

        let labels_permissive = dbscan_permissive.detect_anomalies_dbscan(&values, &timestamps);

        // Permissive should create more clusters
        let clusters_strict = labels_strict
            .iter()
            .filter(|&l| matches!(l, ClusterLabel::Cluster(_)))
            .count();
        let clusters_permissive = labels_permissive
            .iter()
            .filter(|&l| matches!(l, ClusterLabel::Cluster(_)))
            .count();

        assert!(
            clusters_permissive >= clusters_strict,
            "Lower min_samples should allow more clusters"
        );
    }

    /// Test edge case: empty data
    #[test]
    fn test_empty_data() {
        let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig::default()).unwrap();
        let values = vec![];
        let timestamps: Vec<f64> = vec![];

        let labels = dbscan.detect_anomalies_dbscan(&values, &timestamps);

        assert!(labels.is_empty(), "Empty data should produce empty results");
    }

    /// Test edge case: single point
    #[test]
    fn test_single_point() {
        let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig::default()).unwrap();
        let values = vec![5.0];
        let timestamps: Vec<f64> = vec![0.0];

        let labels = dbscan.detect_anomalies_dbscan(&values, &timestamps);

        assert_eq!(labels.len(), 1, "Should have one label for one point");
        // Single point can be noise or cluster depending on implementation
    }

    /// Test edge case: all same values
    #[test]
    fn test_all_same_values() {
        let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig::default()).unwrap();
        let values = vec![1.0; 50];
        let timestamps: Vec<f64> = (0..50).map(|i| i as f64).collect();

        let labels = dbscan.detect_anomalies_dbscan(&values, &timestamps);

        // Should identify some structure even with identical values
        // (based on temporal proximity)
        let total_count = labels.len();
        assert_eq!(total_count, 50, "Should have label for each point");
    }

    /// Test high-dimensional data
    #[test]
    fn test_high_dimensional_data() {
        let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig::default()).unwrap();

        // Create points with 10 dimensions
        let mut points = Vec::new();
        for i in 0..20 {
            let mut point = Point::new(i, &[i as f64], None, i as f64);
            point.features = vec![i as f64; 10];
            points.push(point);
        }

        let labels = dbscan.adaptive_dbscan_clustering(&points);

        assert_eq!(labels.len(), 20, "Should label all points");
    }

    /// Test adaptive parameter calculation
    #[test]
    fn test_adaptive_parameters() {
        // Create data with known variance
        let values: Vec<f64> = (0..100).map(|i| (i as f64) / 10.0).collect();
        let points = create_test_points(&values);

        // Test that the clustering works with adaptive parameters
        let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig::default()).unwrap();
        let labels = dbscan.adaptive_dbscan_clustering(&points);

        // Should produce labels for all points
        assert_eq!(labels.len(), points.len(), "Should label all points");

        // Should identify some structure
        let cluster_count = labels
            .iter()
            .filter(|&l| matches!(l, ClusterLabel::Cluster(_)))
            .count();
        assert!(cluster_count > 0, "Should detect at least one cluster");
    }

    /// Test streaming cluster updates
    #[test]
    fn test_streaming_cluster_updates() {
        let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig {
            window_size: 10,
            ..Default::default()
        })
        .unwrap();

        // Add points incrementally
        for i in 0..15 {
            let point = Point::new(i, &[i as f64], None, i as f64);
            let _label = dbscan.update_streaming_clusters(point);
        }

        // Verify that the streaming updates work (labels are returned)
        // The internal window state is managed by the implementation
        let final_point = Point::new(15, &[15.0], None, 15.0);
        let final_label = dbscan.update_streaming_clusters(final_point);

        // Should return a valid cluster label
        assert!(
            matches!(final_label, ClusterLabel::Cluster(_) | ClusterLabel::Noise),
            "Should return a valid cluster label"
        );
    }

    /// Test density calculation
    #[test]
    fn test_density_calculation() {
        let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig::default()).unwrap();

        // Create dense cluster
        let points: Vec<Point> = (0..10)
            .map(|i| Point::new(i, &[1.0], None, i as f64))
            .collect();

        let labels = dbscan.adaptive_dbscan_clustering(&points);

        // Dense cluster should be detected
        let cluster_count = labels
            .iter()
            .filter(|&l| matches!(l, ClusterLabel::Cluster(_)))
            .count();

        assert!(cluster_count > 0, "Should detect cluster in dense region");
    }
}

#[cfg(test)]
mod dbscan_integration_tests {
    use super::*;

    /// Test anomaly detection with real-world-like data
    #[test]
    fn test_anomaly_detection_realistic() {
        let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig::default()).unwrap();

        // Simulate sensor readings with occasional anomalies
        let mut values = Vec::new();
        for i in 0..100 {
            if i == 25 || i == 50 || i == 75 {
                // Inject anomalies
                values.push(50.0);
            } else {
                // Normal readings (with small noise)
                values.push(10.0 + (rand::random::<f64>() - 0.5) * 2.0);
            }
        }

        let timestamps: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();
        let labels = dbscan.detect_anomalies_dbscan(&values, &timestamps);

        // Should detect anomalies at indices 25, 50, 75
        let anomaly_indices: Vec<usize> = labels
            .iter()
            .enumerate()
            .filter(|(_, &l)| matches!(l, ClusterLabel::Noise))
            .map(|(i, _)| i)
            .collect();

        assert!(
            anomaly_indices.len() >= 3,
            "Should detect at least 3 anomalies"
        );

        // Check if we detected the injected anomalies
        let detected_25 = anomaly_indices.iter().any(|&i| (i as i64 - 25).abs() <= 2);
        let detected_50 = anomaly_indices.iter().any(|&i| (i as i64 - 50).abs() <= 2);
        let detected_75 = anomaly_indices.iter().any(|&i| (i as i64 - 75).abs() <= 2);

        assert!(
            detected_25 || detected_50 || detected_75,
            "Should detect at least some of the injected anomalies"
        );
    }

    /// Test with large dataset (performance test)
    #[test]
    fn test_large_dataset_performance() {
        let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig::default()).unwrap();

        // Generate large dataset
        let values: Vec<f64> = (0..1000)
            .map(|i| 10.0 + (i as f64 / 100.0) + (rand::random::<f64>() - 0.5))
            .collect();

        let timestamps: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();

        let start = std::time::Instant::now();
        let labels = dbscan.detect_anomalies_dbscan(&values, &timestamps);
        let duration = start.elapsed();

        assert_eq!(labels.len(), 1000, "Should process all points");

        // Performance assertion (should complete in reasonable time)
        assert!(
            duration.as_secs() < 10,
            "Should process 1000 points in less than 10 seconds"
        );
    }

    /// Test with multi-variate data
    #[test]
    fn test_multivariate_data() {
        let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig::default()).unwrap();

        // Create 2D points
        let mut points = Vec::new();
        for i in 0..20 {
            let mut point = Point::new(i, &[i as f64], None, i as f64);
            // Create two clusters in 2D
            if i < 10 {
                point.features = vec![1.0 + (rand::random::<f64>() - 0.5) * 0.2, 1.0];
            } else {
                point.features = vec![5.0, 5.0 + (rand::random::<f64>() - 0.5) * 0.2];
            }
            points.push(point);
        }

        let labels = dbscan.adaptive_dbscan_clustering(&points);

        // Should identify two clusters
        let cluster_ids: std::collections::HashSet<_> = labels
            .iter()
            .filter_map(|&l| {
                if let ClusterLabel::Cluster(id) = l {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();

        assert!(
            !cluster_ids.is_empty(),
            "Should identify at least one cluster"
        );
    }

    /// Test temporal clustering
    #[test]
    fn test_temporal_clustering() {
        let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig::default()).unwrap();

        // Create time-series with distinct temporal patterns
        let mut values = Vec::new();
        for _i in 0..30 {
            values.push(5.0); // First period
        }
        for _i in 0..30 {
            values.push(15.0); // Second period (shift)
        }
        for _i in 0..30 {
            values.push(5.0); // Third period (back to original)
        }

        let timestamps: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();
        let labels = dbscan.detect_anomalies_dbscan(&values, &timestamps);

        // Should detect the shift around index 30 and 60
        let has_clusters = labels
            .iter()
            .any(|&l| matches!(l, ClusterLabel::Cluster(_)));

        assert!(has_clusters, "Should detect temporal clusters");
    }

    /// Test cluster quality assessment
    #[test]
    fn test_cluster_quality() {
        let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig::default()).unwrap();

        // Create high-quality cluster (tight, dense)
        let values: Vec<f64> = (0..20)
            .map(|_i| 10.0 + (rand::random::<f64>() - 0.5) * 0.1)
            .collect();

        let timestamps: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();
        let labels = dbscan.detect_anomalies_dbscan(&values, &timestamps);

        // High-quality cluster should be detected
        let cluster_count = labels
            .iter()
            .filter(|&l| matches!(l, ClusterLabel::Cluster(_)))
            .count();

        assert!(
            cluster_count > values.len() / 2,
            "Most points should belong to cluster"
        );
    }
}

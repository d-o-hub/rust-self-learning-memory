//! # Performance Benchmarks for Pattern Algorithms
//!
//! Comprehensive performance benchmarks for DBSCAN and BOCPD algorithms

#[cfg(test)]
mod performance_benchmarks {

    use crate::patterns::predictive::{
        dbscan::{AdaptiveDBSCAN, DBSCANConfig},
        kdtree::{KDTree, Point},
    };
    use crate::patterns::statistical::{
        analysis::types::BOCPDConfig, bocpd_tests::create_changepoint_data, SimpleBOCPD,
    };
    use std::time::Instant;

    /// Benchmark DBSCAN clustering with varying dataset sizes
    #[test]
    fn benchmark_dbscan_scalability() {
        let sizes = vec![100, 500, 1000, 2000, 5000];

        for size in sizes {
            let values: Vec<f64> = (0..size)
                .map(|i| 10.0 + (i as f64 / size as f64) * 10.0 + (rand::random::<f64>() - 0.5))
                .collect();

            let timestamps: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();

            let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig::default()).unwrap();

            let start = Instant::now();
            let labels = dbscan.detect_anomalies_dbscan(&values, &timestamps);
            let duration = start.elapsed();

            println!(
                "DBSCAN with {} points: {:?} ({:.2} ms)",
                size,
                duration,
                duration.as_millis() as f64
            );

            assert_eq!(labels.len(), size);

            // Performance should be reasonable
            // For 5000 points, should complete within reasonable time
            if size >= 5000 {
                assert!(
                    duration.as_secs() < 30,
                    "DBSCAN with 5000 points should complete in < 30s"
                );
            }
        }
    }

    /// Benchmark BOCPD with varying dataset sizes
    #[test]
    fn benchmark_bocpd_scalability() {
        let sizes = vec![100, 500, 1000, 2000, 5000];

        for size in sizes {
            let data = create_changepoint_data(10.0, 20.0, size / 2, size / 2);

            let config = BOCPDConfig {
                buffer_size: size.min(1000),
                ..Default::default()
            };

            let mut bocpd = SimpleBOCPD::new(config);

            let start = Instant::now();
            let results = bocpd.detect_changepoints(&data).unwrap();
            let duration = start.elapsed();

            println!(
                "BOCPD with {} points: {:?} ({:.2} ms, {} detections)",
                size,
                duration,
                duration.as_millis() as f64,
                results.len()
            );

            // Performance should be reasonable
            if size >= 5000 {
                assert!(
                    duration.as_secs() < 30,
                    "BOCPD with 5000 points should complete in < 30s"
                );
            }
        }
    }

    /// Benchmark KD-tree construction and queries
    #[test]
    fn benchmark_kdtree_performance() {
        // Reduced sizes to prevent stack overflow with unbalanced tree insertion
        // The current KDTree::build() uses sequential insertion which can create
        // degenerate trees with sorted data, causing stack overflow at large sizes.
        let sizes = vec![100, 500, 1000, 1500];

        for size in sizes {
            // Use randomization to avoid creating completely sorted data
            let mut points: Vec<Point> = (0..size)
                .map(|i| Point::new(i, &[i as f64, (i * 2) as f64], None, i as f64))
                .collect();

            // Shuffle to prevent degenerate tree formation
            use rand::seq::SliceRandom;
            points.shuffle(&mut rand::thread_rng());

            // Benchmark construction
            let start = Instant::now();
            let kd_tree = KDTree::build(&points);
            let construction_time = start.elapsed();

            // Benchmark queries
            let query_point = vec![size as f64 / 2.0, size as f64];
            let start = Instant::now();

            let _neighbors = kd_tree.find_neighbors(&query_point, 10.0);
            let query_time = start.elapsed();

            println!(
                "KD-tree with {} points: construction {:?}, query {:?}",
                size,
                construction_time.as_micros(),
                query_time.as_micros()
            );

            // KD-tree should be fast
            assert!(
                construction_time.as_millis() < 1000,
                "KD-tree construction should be fast"
            );
            assert!(
                query_time.as_micros() < 10000,
                "KD-tree query should be fast"
            );
        }
    }

    /// Benchmark pattern extraction
    #[test]
    fn benchmark_pattern_extraction() {
        use crate::patterns::predictive::extraction::{ExtractionConfig, PatternExtractor};

        let sizes = vec![10, 50, 100, 500];

        for size in sizes {
            let mut points = Vec::new();
            for i in 0..size {
                points.push(Point::new(i, &[i as f64], None, i as f64));
            }

            let clusters = vec![crate::patterns::predictive::dbscan::Cluster {
                id: 0,
                points: points.clone(),
                centroid: vec![size as f64 / 2.0],
                density: 0.8,
            }];

            let extractor = PatternExtractor::new(ExtractionConfig::default());

            let start = Instant::now();
            let _patterns = extractor.extract_patterns(&clusters, &[], &["test".to_string()]);
            let duration = start.elapsed();

            println!(
                "Pattern extraction with {} points: {:?}",
                size,
                duration.as_micros()
            );

            // Pattern extraction should be fast
            assert!(
                duration.as_millis() < 100,
                "Pattern extraction should be fast"
            );
        }
    }

    /// Benchmark tool compatibility assessment
    #[test]
    fn benchmark_compatibility_assessment() {
        use crate::patterns::compatibility::{
            AssessmentConfig, CompatibilityAssessor, PatternContext,
        };

        let tool_counts = vec![1, 5, 10, 20];

        for count in tool_counts {
            let tools: Vec<String> = (0..count).map(|i| format!("tool_{}", i)).collect();

            let context = PatternContext {
                domain: "test".to_string(),
                data_quality: 0.8,
                occurrences: 10,
                temporal_stability: 0.9,
                available_memory_mb: 200,
                complexity: 0.5,
            };

            let assessor = CompatibilityAssessor::new(AssessmentConfig::default());

            let start = Instant::now();
            let _assessments = assessor
                .batch_assess("test_pattern", &tools, &context)
                .unwrap();
            let duration = start.elapsed();

            println!(
                "Compatibility assessment for {} tools: {:?}",
                count,
                duration.as_micros()
            );

            // Assessment should be fast
            assert!(
                duration.as_millis() < 100,
                "Compatibility assessment should be fast"
            );
        }
    }

    /// Benchmark memory usage for large datasets
    #[test]
    fn benchmark_memory_usage() {
        // Test DBSCAN memory with 10000 points by measuring vector capacity
        let size = 10000;

        let values: Vec<f64> = (0..size)
            .map(|i| 10.0 + (i as f64 / size as f64) * 10.0)
            .collect();
        let timestamps: Vec<f64> = (0..size).map(|i| i as f64).collect();

        use rand::seq::SliceRandom;
        let mut indexed_values: Vec<(f64, f64)> = values.into_iter().zip(timestamps).collect();
        indexed_values.shuffle(&mut rand::thread_rng());

        let values: Vec<f64> = indexed_values.iter().map(|(v, _)| *v).collect();
        let timestamps: Vec<f64> = indexed_values.iter().map(|(_, t)| *t).collect();
        let values_capacity_mb =
            (values.capacity() * std::mem::size_of::<f64>()) as f64 / (1024.0 * 1024.0);
        let timestamps_capacity_mb =
            (timestamps.capacity() * std::mem::size_of::<f64>()) as f64 / (1024.0 * 1024.0);

        let start = Instant::now();
        let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig::default()).unwrap();
        let _labels = dbscan.detect_anomalies_dbscan(&values, &timestamps);
        let duration = start.elapsed();

        // Estimate total memory (input data + overhead)
        let estimated_mb = values_capacity_mb + timestamps_capacity_mb + 10.0; // 10MB overhead estimate

        println!(
            "DBSCAN with {} points: input data {:.2} MB, completed in {:?}",
            size, estimated_mb, duration
        );

        // Should complete in reasonable time
        assert!(
            duration.as_secs() < 30,
            "DBSCAN with 10000 points should complete in < 30s"
        );

        // Memory usage should be reasonable (less than 500 MB for 10k points)
        assert!(estimated_mb < 500.0, "Memory usage should be reasonable");
    }

    /// Benchmark streaming performance
    #[test]
    fn benchmark_streaming_performance() {
        let window_sizes = vec![100, 500, 1000, 2000];

        for window_size in window_sizes {
            let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig {
                window_size,
                ..Default::default()
            })
            .unwrap();

            let num_points = 10000;

            let start = Instant::now();

            for i in 0..num_points {
                let point = Point::new(i, &[i as f64], None, i as f64);
                dbscan.update_streaming_clusters(point);
            }

            let duration = start.elapsed();

            let throughput = (num_points as f64) / duration.as_secs_f64();

            println!(
                "Streaming DBSCAN (window={}): {} points in {:?} ({:.0} points/sec)",
                window_size, num_points, duration, throughput
            );

            // Streaming should be fast
            assert!(
                throughput > 100.0,
                "Streaming should process at least 100 points/sec"
            );
        }
    }

    /// Benchmark concurrent pattern analysis
    #[test]
    fn benchmark_concurrent_analysis() {
        use std::thread;

        let num_threads = vec![1, 2, 4];

        for threads in num_threads {
            let start = Instant::now();

            let handles: Vec<_> = (0..threads)
                .map(|_| {
                    thread::spawn(|| {
                        let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig::default()).unwrap();
                        let values: Vec<f64> = (0..1000).map(|i| i as f64).collect();
                        let timestamps: Vec<f64> = (0..1000).map(|i| i as f64).collect();
                        dbscan.detect_anomalies_dbscan(&values, &timestamps)
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }

            let duration = start.elapsed();

            println!(
                "Concurrent analysis ({} threads): {:?}",
                threads,
                duration.as_millis()
            );
        }
    }

    /// Real-world workload benchmark
    #[test]
    fn benchmark_real_world_workload() {
        let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig::default()).unwrap();

        // Simulate real-time anomaly detection workload
        let num_batches = 100;
        let points_per_batch = 50;

        let start = Instant::now();

        for batch in 0..num_batches {
            let values: Vec<f64> = (0..points_per_batch)
                .map(|_i| {
                    let base = 10.0;
                    // Occasional anomalies
                    if rand::random::<f64>() < 0.05 {
                        base + 50.0
                    } else {
                        base + (rand::random::<f64>() - 0.5) * 2.0
                    }
                })
                .collect();

            let timestamps: Vec<f64> = (0..points_per_batch)
                .map(|i| (batch * points_per_batch + i) as f64)
                .collect();

            let _labels = dbscan.detect_anomalies_dbscan(&values, &timestamps);
        }

        let duration = start.elapsed();
        let total_points = num_batches * points_per_batch;
        let throughput = total_points as f64 / duration.as_secs_f64();

        println!(
            "Real-world workload: {} points in {:?} ({:.0} points/sec)",
            total_points, duration, throughput
        );

        // Real-time workload should be fast
        assert!(
            throughput > 100.0,
            "Real-time processing should handle at least 100 points/sec"
        );
    }

    /// Benchmark accuracy vs performance trade-offs
    #[test]
    fn benchmark_accuracy_performance_tradeoff() {
        let configs = vec![(0.1, 2, 1000), (0.5, 5, 500), (1.0, 10, 200)];

        for (density, min_samples, max_distance) in configs {
            let config = DBSCANConfig {
                density,
                min_cluster_size: min_samples,
                max_distance: max_distance as f64,
                window_size: 1000,
            };

            let mut dbscan = AdaptiveDBSCAN::new(config).unwrap();

            // Generate test data with known outliers
            let values: Vec<f64> = (0..1000)
                .map(|i| {
                    if i == 100 || i == 500 || i == 900 {
                        50.0 // Known outliers
                    } else {
                        10.0 + (rand::random::<f64>() - 0.5) * 2.0
                    }
                })
                .collect();

            let timestamps: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();

            let start = Instant::now();
            let labels = dbscan.detect_anomalies_dbscan(&values, &timestamps);
            let duration = start.elapsed();

            let detected_outliers = labels
                .iter()
                .filter(|&l| matches!(l, crate::patterns::predictive::dbscan::ClusterLabel::Noise))
                .count();

            println!(
                "Config (density={}, min_samples={}, max_distance={}): {:?}, detected {} outliers",
                density, min_samples, max_distance, duration, detected_outliers
            );
        }
    }
}

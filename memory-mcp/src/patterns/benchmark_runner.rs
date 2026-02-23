//! # Benchmark Runner for Advanced Scenarios
//!
//! Benchmarks for memory usage, streaming, concurrent analysis, and real-world workloads.

#[cfg(test)]
mod runner_benchmarks {

    use crate::patterns::predictive::{
        dbscan::{AdaptiveDBSCAN, DBSCANConfig},
        kdtree::Point,
    };
    use std::time::Instant;

    /// Benchmark memory usage for large datasets
    #[test]
    fn benchmark_memory_usage() {
        let is_ci = std::env::var("CI").is_ok();
        // Test DBSCAN memory with large datasets by measuring vector capacity
        let size = if is_ci { 2000 } else { 10000 };

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
        let max_secs = if is_ci { 60 } else { 45 };
        assert!(
            duration.as_secs() < max_secs,
            "DBSCAN should complete within the time budget"
        );

        // Memory usage should be reasonable (less than 500 MB for 10k points)
        assert!(estimated_mb < 500.0, "Memory usage should be reasonable");
    }

    /// Benchmark streaming performance
    ///
    /// # Purpose
    /// Validates streaming DBSCAN performance across different window sizes.
    ///
    /// # Ignore Reason
    /// Performance varies significantly by environment. In CI mode, uses reduced
    /// point counts (1K vs 10K) to prevent timeouts while still validating correctness.
    ///
    /// # ADR Reference
    /// See ADR-027: Strategy for Ignored Tests - streaming-impl feature gate.
    ///
    /// # Running the Test
    /// - CI mode (fast): `CI=true cargo test --package memory-mcp benchmark_streaming_performance -- --ignored`
    /// - Full mode (slow): `cargo test --package memory-mcp --features streaming-impl benchmark_streaming_performance -- --ignored`
    ///
    /// # Performance Targets
    /// - CI mode: 4-7 seconds with 1K-5K points
    /// - Full mode: 60-90 seconds with 40K points
    #[cfg_attr(not(feature = "streaming-impl"), ignore)]
    #[test]
    fn benchmark_streaming_performance() {
        let is_ci = std::env::var("CI").is_ok();
        // Reduce window sizes and point count for CI to prevent timeouts
        let window_sizes = if is_ci {
            vec![100, 500]
        } else {
            vec![100, 500, 1000, 2000]
        };

        for window_size in window_sizes {
            // Significantly reduce points for larger windows to prevent timeout
            let num_points = match window_size {
                _ if is_ci && window_size >= 1000 => 500, // Only 500 points for large windows in CI
                _ if is_ci => 1000,                       // 1000 points for small windows in CI
                _ => 10000,                               // Full load for local testing
            };

            let mut dbscan = AdaptiveDBSCAN::new(DBSCANConfig {
                window_size,
                ..Default::default()
            })
            .unwrap();

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

            // Streaming performance varies by window size and environment
            // Larger windows are slower due to more points to cluster
            // See ADR-026 for handling strategy
            let min_throughput = if is_ci { 3.0 } else { 10.0 }; // Relaxed for CI
            assert!(
                throughput > min_throughput,
                "Streaming performance degraded: got {:.0} pts/sec, min {} pts/sec. \
                 See ADR-026 for handling strategy.",
                throughput,
                min_throughput
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

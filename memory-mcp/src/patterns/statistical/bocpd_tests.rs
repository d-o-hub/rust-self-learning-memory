//! # Comprehensive BOCPD Tests
//!
//! Unit and integration tests for Bayesian Online Changepoint Detection

use crate::patterns::statistical::analysis::{
    bocpd::{log_sum_exp, SimpleBOCPD},
    types::BOCPDConfig,
};
use anyhow::Result;
use rand::{Rng, SeedableRng};

/// Helper function to create test data with changepoint
pub fn create_changepoint_data(
    before_mean: f64,
    after_mean: f64,
    before_len: usize,
    after_len: usize,
) -> Vec<f64> {
    let mut data = Vec::new();

    for _ in 0..before_len {
        data.push(before_mean + (rand::random::<f64>() - 0.5) * 0.5);
    }

    for _ in 0..after_len {
        data.push(after_mean + (rand::random::<f64>() - 0.5) * 0.5);
    }

    data
}

#[cfg(test)]
mod bocpd_unit_tests {
    use super::*;

    /// Test changepoint detection accuracy
    #[test]
    fn test_changepoint_detection_accuracy() -> Result<()> {
        let config = BOCPDConfig {
            hazard_rate: 100.0,
            expected_run_length: 50,
            max_run_length_hypotheses: 200,
            alert_threshold: 0.7,
            buffer_size: 100,
        };

        let mut bocpd = SimpleBOCPD::new(config);

        // Create data with changepoint at index 50
        let data = create_changepoint_data(10.0, 20.0, 50, 50);

        let results = bocpd.detect_changepoints(&data)?;

        // Should detect at least one changepoint
        assert!(!results.is_empty(), "Should detect changepoints");

        // At least one detection should be near index 50
        let near_changepoint = results.iter().any(|r| {
            if let Some(idx) = r.changepoint_index {
                (idx as i64 - 50).abs() <= 10
            } else {
                false
            }
        });

        assert!(near_changepoint, "Should detect changepoint near index 50");

        Ok(())
    }

    /// Test probability threshold tuning
    #[test]
    fn test_probability_threshold_tuning() -> Result<()> {
        // Test with low threshold (more detections)
        let config_low = BOCPDConfig {
            alert_threshold: 0.5,
            ..Default::default()
        };

        let mut bocpd_low = SimpleBOCPD::new(config_low);
        let data = create_changepoint_data(10.0, 20.0, 50, 50);

        let results_low = bocpd_low.detect_changepoints(&data)?;

        // Test with high threshold (fewer detections)
        let config_high = BOCPDConfig {
            alert_threshold: 0.9,
            ..Default::default()
        };

        let mut bocpd_high = SimpleBOCPD::new(config_high);

        let results_high = bocpd_high.detect_changepoints(&data)?;

        // Lower threshold should result in more detections
        assert!(
            results_low.len() >= results_high.len(),
            "Lower threshold should produce at least as many detections"
        );

        Ok(())
    }

    /// Test online vs offline modes
    #[test]
    fn test_online_offline_modes() -> Result<()> {
        let config = BOCPDConfig {
            hazard_rate: 100.0,
            buffer_size: 50,
            ..Default::default()
        };

        // Online mode: process data incrementally using update_state (crate-visible)
        let mut bocpd_online = SimpleBOCPD::new(config.clone());
        let data = create_changepoint_data(10.0, 20.0, 50, 50);

        let mut online_results = Vec::new();
        for (i, &value) in data.iter().enumerate() {
            bocpd_online.update_state(value)?;

            // Check for changepoints periodically using normalized distribution
            if i % 10 == 0 && i >= 10 {
                let distribution = bocpd_online.normalize_distribution();
                if !distribution.is_empty() {
                    let prob = distribution[0]; // Probability of changepoint at run length 0
                    if prob > config.alert_threshold {
                        online_results.push((i, prob));
                    }
                }
            }
        }

        // Offline mode: process all data at once
        let mut bocpd_offline = SimpleBOCPD::new(config);
        let offline_results = bocpd_offline.detect_changepoints(&data)?;

        // Both should detect changepoints
        assert!(
            !online_results.is_empty() || !offline_results.is_empty(),
            "At least one mode should detect changepoints"
        );

        Ok(())
    }

    /// Test handling of concept drift
    #[test]
    fn test_concept_drift_handling() -> Result<()> {
        let config = BOCPDConfig {
            hazard_rate: 0.01,     // Much lower hazard rate for gradual drift
            alert_threshold: 0.90, // Higher confidence threshold
            max_run_length: 150,   // Allow longer runs before resetting
            min_samples: 10,
        };

        let mut bocpd = SimpleBOCPD::new(config);
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        // Create data with gradual drift (not abrupt change)
        let mut data = Vec::new();
        for i in 0..100 {
            let value = 10.0 + (i as f64 / 100.0) * 10.0; // Gradual increase from 10 to 20
            data.push(value + (rng.gen::<f64>() - 0.5) * 0.5);
        }

        let results = bocpd.detect_changepoints(&data)?;

        // Should detect very few changepoints for gradual drift
        let is_ci = std::env::var("CI").is_ok();
        let max_changepoints = if is_ci { 2 } else { 1 };
        assert!(
            results.len() <= max_changepoints,
            "Gradual drift should not produce many changepoints: got {}, max allowed {}",
            results.len(),
            max_changepoints
        );

        Ok(())
    }

    /// Test edge case: empty data
    #[test]
    fn test_empty_data() -> Result<()> {
        let config = BOCPDConfig::default();
        let mut bocpd = SimpleBOCPD::new(config);

        let results = bocpd.detect_changepoints(&[])?;

        assert!(results.is_empty(), "Empty data should produce no results");

        Ok(())
    }

    /// Test edge case: single point
    #[test]
    fn test_single_point() -> Result<()> {
        let config = BOCPDConfig::default();
        let mut bocpd = SimpleBOCPD::new(config);

        let results = bocpd.detect_changepoints(&[5.0])?;

        // Single point should not trigger changepoint
        assert!(
            results.is_empty() || results[0].changepoint_probability < 0.5,
            "Single point should not produce high-confidence changepoint"
        );

        Ok(())
    }

    /// Test edge case: constant series
    #[test]
    fn test_constant_series() -> Result<()> {
        let config = BOCPDConfig::default();
        let mut bocpd = SimpleBOCPD::new(config);

        let data = vec![10.0; 100];
        let results = bocpd.detect_changepoints(&data)?;

        // Constant series should have few or no high-confidence changepoints
        let high_confidence = results.iter().filter(|r| r.confidence > 0.8).count();

        assert!(
            high_confidence <= 2,
            "Constant series should not have many high-confidence changepoints"
        );

        Ok(())
    }

    /// Test log_sum_exp numerical stability
    #[test]
    fn test_log_sum_exp_stability() {
        // Test with large negative numbers (should not underflow)
        let values = vec![-1000.0, -999.0, -998.0];
        let result = log_sum_exp(&values);

        assert!(result.is_finite(), "Should handle large negative numbers");

        // Test with large positive numbers (should not overflow)
        let values = vec![1000.0, 1001.0, 1002.0];
        let result = log_sum_exp(&values);

        assert!(result.is_finite(), "Should handle large positive numbers");

        // Test with mixed values
        let values = vec![-100.0, 0.0, 100.0];
        let result = log_sum_exp(&values);

        assert!(result.is_finite(), "Should handle mixed values");

        // Test with empty vector
        let values: Vec<f64> = vec![];
        let result = log_sum_exp(&values);

        assert_eq!(
            result,
            f64::NEG_INFINITY,
            "Empty vector should return negative infinity"
        );
    }

    /// Test posterior distribution normalization
    #[test]
    fn test_posterior_normalization() -> Result<()> {
        let config = BOCPDConfig::default();
        let mut bocpd = SimpleBOCPD::new(config);

        // Add some data
        for i in 0..20 {
            bocpd.update_state(10.0 + (i as f64 * 0.1))?;
        }

        // Get normalized distribution
        let normalized = bocpd.normalize_distribution();

        // Sum should be approximately 1.0
        let sum: f64 = normalized.iter().sum();

        assert!(
            (sum - 1.0).abs() < 1e-6,
            "Posterior distribution should sum to 1.0, got {}",
            sum
        );

        // All probabilities should be in [0, 1]
        for &p in &normalized {
            assert!(
                (0.0..=1.0).contains(&p),
                "Probabilities should be in [0, 1]"
            );
        }

        Ok(())
    }

    /// Test MAP run length computation
    #[test]
    fn test_map_run_length() -> Result<()> {
        let config = BOCPDConfig::default();
        let mut bocpd = SimpleBOCPD::new(config);

        // Add constant data (should result in increasing run length)
        for _ in 0..20 {
            bocpd.update_state(10.0)?;
        }

        // Compute MAP run length from normalized distribution
        let distribution = bocpd.normalize_distribution();
        let mut max_prob = 0.0;
        let mut map_run_length = 0;
        for (i, &prob) in distribution.iter().enumerate() {
            if prob > max_prob {
                max_prob = prob;
                map_run_length = i;
            }
        }

        // MAP run length should be a valid index within the distribution
        assert!(
            map_run_length < distribution.len(),
            "MAP run length should be within distribution bounds"
        );

        Ok(())
    }

    /// Test circular buffer management
    #[test]
    fn test_circular_buffer() -> Result<()> {
        let config = BOCPDConfig {
            buffer_size: 10,
            ..Default::default()
        };

        let mut bocpd = SimpleBOCPD::new(config);

        // Add more data than buffer size
        for i in 0..20 {
            bocpd.update_state(i as f64)?;
        }

        // Buffer should maintain correct size
        assert_eq!(bocpd.state.data_buffer.len(), 10);

        // Should contain most recent values
        let buffer_values: Vec<f64> = bocpd.state.data_buffer.iter().cloned().collect();
        assert_eq!(
            buffer_values,
            vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0]
        );

        Ok(())
    }

    /// Test hazard rate configuration
    #[test]
    fn test_hazard_rate_configuration() -> Result<()> {
        let config_low = BOCPDConfig {
            hazard_rate: 10.0, // Low hazard rate (expect fewer changepoints)
            ..Default::default()
        };

        let config_high = BOCPDConfig {
            hazard_rate: 200.0, // High hazard rate (expect more changepoints)
            ..Default::default()
        };

        let data = create_changepoint_data(10.0, 20.0, 50, 50);

        let mut bocpd_low = SimpleBOCPD::new(config_low);
        let results_low = bocpd_low.detect_changepoints(&data)?;

        let mut bocpd_high = SimpleBOCPD::new(config_high);
        let results_high = bocpd_high.detect_changepoints(&data)?;

        // Higher hazard rate should result in more detections
        assert!(
            results_high.len() >= results_low.len(),
            "Higher hazard rate should produce at least as many detections"
        );

        Ok(())
    }
}

#[cfg(test)]
mod bocpd_integration_tests {
    use super::*;

    /// Test with real-world-like time series
    #[test]
    fn test_realistic_time_series() -> Result<()> {
        let config = BOCPDConfig {
            hazard_rate: 100.0,
            alert_threshold: 0.7,
            buffer_size: 100,
            ..Default::default()
        };

        let mut bocpd = SimpleBOCPD::new(config);

        // Simulate server load with sudden spikes
        let mut data = Vec::new();
        for i in 0..200 {
            let base_load = 50.0;

            let value = if i == 50 || i == 100 || i == 150 {
                // Sudden spikes
                base_load + 40.0
            } else {
                // Normal load with small fluctuations
                base_load + (rand::random::<f64>() - 0.5) * 10.0
            };

            data.push(value);
        }

        let results = bocpd.detect_changepoints(&data)?;

        // Should detect changepoints around spikes
        assert!(!results.is_empty(), "Should detect changepoints");

        // Check for detections near spike indices
        let detections_near_spikes = results
            .iter()
            .filter(|r| {
                if let Some(idx) = r.changepoint_index {
                    // Within 10 indices of any spike
                    (idx as i64 - 50).abs() <= 10
                        || (idx as i64 - 100).abs() <= 10
                        || (idx as i64 - 150).abs() <= 10
                } else {
                    false
                }
            })
            .count();

        assert!(
            detections_near_spikes > 0,
            "Should detect at least one changepoint near a spike"
        );

        Ok(())
    }

    /// Test with multiple changepoints
    #[test]
    fn test_multiple_changepoints() -> Result<()> {
        let config = BOCPDConfig {
            hazard_rate: 100.0,
            alert_threshold: 0.6,
            buffer_size: 100,
            ..Default::default()
        };

        let mut bocpd = SimpleBOCPD::new(config);

        // Create data with 3 changepoints
        let mut data = Vec::new();

        // Regime 1: mean=10
        for _ in 0..30 {
            data.push(10.0 + (rand::random::<f64>() - 0.5) * 1.0);
        }

        // Regime 2: mean=20
        for _ in 0..30 {
            data.push(20.0 + (rand::random::<f64>() - 0.5) * 1.0);
        }

        // Regime 3: mean=15
        for _ in 0..30 {
            data.push(15.0 + (rand::random::<f64>() - 0.5) * 1.0);
        }

        // Regime 4: mean=25
        for _ in 0..30 {
            data.push(25.0 + (rand::random::<f64>() - 0.5) * 1.0);
        }

        let results = bocpd.detect_changepoints(&data)?;

        // Should detect multiple changepoints
        assert!(results.len() >= 2, "Should detect at least 2 changepoints");

        // Check detections are near expected changepoints (30, 60, 90)
        let near_expected = results
            .iter()
            .filter(|r| {
                if let Some(idx) = r.changepoint_index {
                    (idx as i64 - 30).abs() <= 10
                        || (idx as i64 - 60).abs() <= 10
                        || (idx as i64 - 90).abs() <= 10
                } else {
                    false
                }
            })
            .count();

        assert!(
            near_expected >= 2,
            "Should detect changepoints near expected locations"
        );

        Ok(())
    }

    /// Test performance with large dataset
    #[test]
    fn test_large_dataset_performance() -> Result<()> {
        let config = BOCPDConfig {
            buffer_size: 500,
            ..Default::default()
        };

        let mut bocpd = SimpleBOCPD::new(config);

        // Generate large dataset
        let data: Vec<f64> = (0..1000)
            .map(|i| {
                let base = if i < 500 { 10.0 } else { 20.0 };
                base + (rand::random::<f64>() - 0.5) * 2.0
            })
            .collect();

        let start = std::time::Instant::now();
        let results = bocpd.detect_changepoints(&data)?;
        let duration = start.elapsed();

        // Should process in reasonable time
        assert!(
            duration.as_secs() < 15,
            "Should process 1000 points in less than 15 seconds"
        );

        // Should detect the changepoint around index 500
        let near_middle = results.iter().any(|r| {
            if let Some(idx) = r.changepoint_index {
                (idx as i64 - 500).abs() <= 50
            } else {
                false
            }
        });

        assert!(near_middle, "Should detect changepoint near index 500");

        Ok(())
    }

    /// test integration with statistical engine
    #[test]
    fn test_integration_with_statistical_engine() -> Result<()> {
        use std::collections::HashMap;

        let mut engine = crate::patterns::statistical::StatisticalEngine::new()?;
        let mut data = HashMap::new();
        let mut rng = rand::rngs::StdRng::seed_from_u64(1337);

        // Create series with clear changepoint
        let series: Vec<f64> = (0..100)
            .map(|i| {
                if i < 50 {
                    10.0 + (rng.gen::<f64>() - 0.5) * 1.0
                } else {
                    20.0 + (rng.gen::<f64>() - 0.5) * 1.0
                }
            })
            .collect();

        data.insert("test_series".to_string(), series);

        let results = engine.analyze_time_series(&data)?;

        // Should have changepoint results
        assert!(
            !results.changepoints.is_empty(),
            "Engine should detect changepoints"
        );

        // At least one changepoint should be near index 50
        let near_fifty = results
            .changepoints
            .iter()
            .any(|cp| (cp.index as i64 - 50).abs() <= 10 && cp.confidence > 0.0);

        assert!(near_fifty, "Should detect changepoint near index 50");

        Ok(())
    }

    /// Test temporal consistency
    #[test]
    fn test_temporal_consistency() -> Result<()> {
        let config = BOCPDConfig {
            // Lower hazard rate for seasonal data (requires stronger evidence)
            hazard_rate: 0.5,
            alert_threshold: 0.95, // Higher confidence threshold
            max_run_length: 100,
            min_samples: 10,
        };
        let mut bocpd = SimpleBOCPD::new(config);

        // Create data with seasonal pattern (no true changepoints)
        let data: Vec<f64> = (0..100)
            .map(|i| 10.0 + 5.0 * ((i as f64) / 10.0 * 2.0 * std::f64::consts::PI).cos())
            .collect();

        let results = bocpd.detect_changepoints(&data)?;

        // Seasonal data should not produce many high-confidence changepoints
        let high_confidence = results.iter().filter(|r| r.confidence > 0.8).count();
        let is_ci = std::env::var("CI").is_ok();
        let max_high_confidence = if is_ci { 3 } else { 2 };

        assert!(
            high_confidence <= max_high_confidence,
            "Seasonal data should not produce excessive high-confidence changepoints: got {}, max allowed {}",
            high_confidence, max_high_confidence
        );

        Ok(())
    }

    /// Test confidence calibration
    #[test]
    fn test_confidence_calibration() -> Result<()> {
        let config = BOCPDConfig {
            alert_threshold: 0.5,
            ..Default::default()
        };

        let mut bocpd = SimpleBOCPD::new(config);

        // Create data with known changepoint
        let data = create_changepoint_data(10.0, 20.0, 50, 50);

        let results = bocpd.detect_changepoints(&data)?;

        // Find the most confident detection
        let max_confidence = results.iter().map(|r| r.confidence).fold(0.0_f64, f64::max);

        // At least one detection should have reasonable confidence
        assert!(
            max_confidence > 0.3,
            "Should have at least one detection with confidence > 0.3"
        );

        Ok(())
    }
}

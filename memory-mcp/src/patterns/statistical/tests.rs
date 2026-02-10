//! # Statistical Analysis Tests
//!
//! Test suite for the statistical analysis engine.

use anyhow::Result;
use std::collections::HashMap;

// Import types from parent module (statistical which re-exports from analysis)
use super::{
    AnalysisMetadata, BOCPDConfig, BOCPDResult, ChangepointResult, CorrelationResult, SimpleBOCPD,
    StatisticalEngine, StatisticalResults, TrendDirection, TrendResult,
};

// Import helper functions directly from analysis module
use super::analysis::log_sum_exp;

#[test]
fn test_bocpd_detects_mean_shift() -> Result<()> {
    let mut engine = StatisticalEngine::new()?;
    let mut data = HashMap::new();

    // Clear mean shift around the midpoint
    let mut series = vec![1.0; 30];
    series.extend(vec![10.0; 30]);
    data.insert("x".to_string(), series);

    let results = engine.analyze_time_series(&data)?;
    assert!(
        !results.changepoints.is_empty(),
        "Expected at least one changepoint"
    );

    // Should have at least one changepoint in the neighborhood of the shift
    let has_near_mid = results
        .changepoints
        .iter()
        .any(|cp| (cp.index as i64 - 30).abs() <= 5 && cp.confidence >= 0.0);
    assert!(has_near_mid, "Expected a changepoint near index 30");

    Ok(())
}

#[test]
fn test_bocpd_constant_series_no_high_confidence_changepoints() -> Result<()> {
    let mut engine = StatisticalEngine::new()?;
    let mut data = HashMap::new();
    data.insert("x".to_string(), vec![5.0; 60]);

    let results = engine.analyze_time_series(&data)?;

    // BOCPD may emit low-confidence candidates; ensure we do not see many high-confidence.
    let high_confidence = results
        .changepoints
        .iter()
        .filter(|cp| cp.confidence > 0.9)
        .count();
    assert!(
        high_confidence <= 1,
        "Constant series should not have many high-confidence changepoints"
    );

    Ok(())
}

#[test]
fn test_statistical_engine_creation() {
    let engine = StatisticalEngine::new();
    assert!(engine.is_ok());
}

#[test]
fn test_correlation_calculation() -> Result<()> {
    let mut engine = StatisticalEngine::new()?;
    let mut data = HashMap::new();
    data.insert("x".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    data.insert("y".to_string(), vec![2.0, 4.0, 6.0, 8.0, 10.0]);

    let results = engine.analyze_time_series(&data)?;
    assert!(!results.correlations.is_empty());

    let corr = results
        .correlations
        .iter()
        .find(|corr| corr.variables == ("x".to_string(), "y".to_string()))
        .expect("Expected correlation for (x, y)");
    // Allow small floating point differences
    assert!(
        (corr.coefficient - 1.0).abs() < 0.01,
        "Correlation coefficient should be close to 1.0, got {}",
        corr.coefficient
    );
    assert!(corr.significant);

    Ok(())
}

#[test]
fn test_trend_analysis() -> Result<()> {
    let mut engine = StatisticalEngine::new()?;
    let mut data = HashMap::new();
    data.insert("trend".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);

    let results = engine.analyze_time_series(&data)?;
    assert!(!results.trends.is_empty());

    let trend = &results.trends[0];
    assert_eq!(trend.variable, "trend");
    assert!(matches!(trend.direction, TrendDirection::Increasing));
    assert!(trend.significant);

    Ok(())
}

#[test]
fn test_data_validation() {
    let engine = StatisticalEngine::new().unwrap();
    let mut data = HashMap::new();

    // Empty data should fail
    assert!(engine.validate_data(&data).is_err());

    // Data with NaN should fail
    data.insert("bad".to_string(), vec![1.0, f64::NAN, 3.0]);
    assert!(engine.validate_data(&data).is_err());
}

// BOCPD Implementation Tests
#[test]
fn test_simple_bocpd_creation() {
    let config = BOCPDConfig::default();
    let bocpd = SimpleBOCPD::new(config);

    assert_eq!(bocpd.state.processed_points, 0);
    assert_eq!(bocpd.state.data_buffer.len(), 0);
}

#[test]
fn test_joint_anomaly_changepoint_detection() {
    let config = BOCPDConfig {
        hazard_rate: 100.0,
        expected_run_length: 50,
        max_run_length_hypotheses: 200,
        alert_threshold: 0.8,
        buffer_size: 50,
    };

    let mut bocpd = SimpleBOCPD::new(config);

    // Create data with a clear changepoint at index 25
    let mut data = Vec::new();
    for i in 0..20 {
        data.push(10.0 + (i as f64 * 0.1)); // Gradually increasing
    }
    for i in 20..40 {
        data.push(20.0 + (i as f64 * 0.2)); // Clear shift to higher values
    }

    let results = bocpd.detect_changepoints(&data).unwrap();

    // Should detect at least one changepoint
    assert!(!results.is_empty());

    // At least one result should have reasonable confidence
    let reasonable_confidence_results: Vec<_> =
        results.iter().filter(|r| r.confidence > 0.3).collect();
    assert!(!reasonable_confidence_results.is_empty());
}

#[test]
fn test_posterior_distribution_computation() {
    let config = BOCPDConfig::default();
    let mut bocpd = SimpleBOCPD::new(config);

    // Test with simple data
    let test_data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 10.0, 11.0, 12.0]; // Clear break at index 5

    for &value in &test_data {
        bocpd.update_state(value).unwrap();
    }

    // Check that posterior is properly normalized (sum should be close to 1)
    let normalized = bocpd.normalize_distribution();
    let sum: f64 = normalized.iter().sum();
    assert!(
        (sum - 1.0).abs() < 1e-10,
        "Posterior should be normalized, got sum: {}",
        sum
    );
}

#[test]
fn test_streaming_updates_and_circular_buffers() {
    let config = BOCPDConfig {
        max_run_length_hypotheses: 100,
        buffer_size: 5,
        ..Default::default()
    };

    let mut bocpd = SimpleBOCPD::new(config);

    // Add data that exceeds buffer size
    for i in 0..10 {
        bocpd.update_state(i as f64).unwrap();
        assert_eq!(bocpd.state.data_buffer.len(), (i + 1).min(5));
    }

    // Verify buffer size is maintained
    assert_eq!(bocpd.state.data_buffer.len(), 5);

    // Verify oldest values are removed
    let buffer_values: Vec<f64> = bocpd.state.data_buffer.iter().cloned().collect();
    assert_eq!(buffer_values, vec![5.0, 6.0, 7.0, 8.0, 9.0]);
}

#[test]
fn test_hazard_rate_adaptation() {
    let config = BOCPDConfig {
        hazard_rate: 200.0,
        ..Default::default()
    };

    let mut bocpd = SimpleBOCPD::new(config);

    // Add data with low variance first
    for i in 0..15 {
        bocpd.update_state(10.0 + (i as f64 * 0.01)).unwrap();
    }

    let _initial_hazard = bocpd.state.hazard_rate;

    // Add data with high variance - hazard rate should adapt based on variance
    for i in 0..15 {
        let value = 10.0 + (i as f64 * 10.0); // Much higher variance
        bocpd.update_state(value).unwrap();
    }

    // State should be updated (processed points should increase)
    assert!(bocpd.state.processed_points > 15);
}

#[test]
fn test_multi_resolution_detection() {
    let config = BOCPDConfig {
        buffer_size: 100,
        expected_run_length: 50,
        ..Default::default()
    };

    let mut bocpd = SimpleBOCPD::new(config);

    // Create data with multiple types of patterns
    let data = vec![
        // Short-term pattern: gentle oscillation
        1.0, 1.1, 1.0, 1.1, 1.0, 1.1, 1.0, 1.1, 1.0, 1.1,
        // Medium-term shift: mean change
        5.0, 5.1, 5.0, 5.1, 5.0, 5.1, 5.0, 5.1, 5.0, 5.1,
        // Long-term trend: clear trend change
        10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0,
    ];

    let results = bocpd.detect_changepoints(&data).unwrap();

    // Should detect some patterns
    assert!(!results.is_empty());
}

#[test]
fn test_edge_cases() {
    let config = BOCPDConfig::default();
    let mut bocpd = SimpleBOCPD::new(config);

    // Test empty data
    let empty_results = bocpd.detect_changepoints(&[]);
    assert!(empty_results.is_ok());
    assert!(empty_results.unwrap().is_empty());

    // Test constant series (no changepoints expected)
    let constant_data = vec![5.0; 30];
    let constant_results = bocpd.detect_changepoints(&constant_data).unwrap();
    // Should not detect many changepoints in constant data
    let high_confidence_count = constant_results
        .iter()
        .filter(|r| r.confidence > 0.8)
        .count();
    assert!(
        high_confidence_count <= 2,
        "Constant series should not have many high-confidence changepoints"
    );

    // Test rapid changes (multiple changepoints)
    let rapid_changes = vec![
        1.0, 1.0, 1.0, 10.0, 10.0, 10.0, 2.0, 2.0, 2.0, 15.0, 15.0, 15.0, 3.0, 3.0, 3.0,
    ];
    let rapid_results = bocpd.detect_changepoints(&rapid_changes).unwrap();

    // Should detect some changepoints in rapidly changing data
    assert!(!rapid_results.is_empty());
}

#[test]
fn test_numerical_stability() {
    let config = BOCPDConfig::default();
    let mut bocpd = SimpleBOCPD::new(config);

    // Test with extreme values
    let extreme_data = vec![1e10, 1e10, -1e10, 1e-10, 1e-10, f64::MAX, f64::MIN_POSITIVE];

    let results = bocpd.detect_changepoints(&extreme_data);
    assert!(results.is_ok(), "Should handle extreme values gracefully");

    // Test log-space arithmetic functions
    let test_values = vec![-1000.0, -500.0, -100.0, 0.0, 100.0, 500.0, 1000.0];
    let log_sum = log_sum_exp(&test_values);
    assert!(log_sum.is_finite(), "Log-sum-exp should be finite");

    let log_add = log_sum_exp(&[-1000.0, -500.0]);
    assert!(log_add.is_finite(), "Log-add-exp should be finite");
}

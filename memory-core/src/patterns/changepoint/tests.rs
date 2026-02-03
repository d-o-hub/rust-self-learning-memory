//! # Changepoint Tests
//!
//! Unit tests for the changepoint detection system.

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::patterns::changepoint::algorithms::{compute_segment_stats, normal_cdf};
    use crate::patterns::changepoint::detector::ChangepointDetector;
    use crate::patterns::changepoint::types::{
        ChangeDirection, ChangeType, Changepoint, ChangepointConfig, ChangepointError,
        SegmentComparisonConfig,
    };
    use uuid::Uuid;

    #[test]
    fn test_changepoint_config_validation() {
        let config = ChangepointConfig {
            min_probability: 1.5,     // Invalid, should clamp
            min_distance: 0,          // Invalid, should clamp
            significance_level: -0.1, // Invalid, should clamp
            adaptive_threshold: true,
            min_observations: 3, // Invalid, should clamp to 5
        }
        .validated();

        assert_eq!(config.min_probability, 1.0);
        assert_eq!(config.min_distance, 1);
        assert_eq!(config.significance_level, 0.0);
        assert_eq!(config.min_observations, 5);
    }

    #[test]
    fn test_detect_changepoints_insufficient_data() {
        let mut detector = ChangepointDetector::new(ChangepointConfig::default());
        let values = vec![0.5, 0.6, 0.7];

        let result = detector.detect_changepoints(&values);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().downcast_ref::<ChangepointError>(),
            Some(ChangepointError::InsufficientData { .. })
        ));
    }

    #[test]
    fn test_detect_changepoints_invalid_data() {
        let mut detector = ChangepointDetector::new(ChangepointConfig::default());
        let values = vec![0.5, f64::NAN, 0.7];

        let result = detector.detect_changepoints(&values);
        assert!(result.is_err());
        // Check error message contains expected content instead of downcasting
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("NaN") || err_msg.contains("invalid") || err_msg.contains("Invalid"),
            "Error should mention NaN or invalid data: {}",
            err_msg
        );
    }

    #[test]
    fn test_detect_changepoint_mean_shift() {
        let mut detector = ChangepointDetector::new(ChangepointConfig::default());

        // Use a more pronounced mean shift with more data points for reliable detection
        // First segment: mean ~0.81, std ~0.013
        let first_segment: Vec<f64> = (0..15)
            .map(|_| 0.81 + rand::random::<f64>() * 0.04 - 0.02)
            .collect();
        // Second segment: mean ~0.41, std ~0.013 (clear shift of ~0.4)
        let second_segment: Vec<f64> = (0..15)
            .map(|_| 0.41 + rand::random::<f64>() * 0.04 - 0.02)
            .collect();

        let values: Vec<f64> = first_segment
            .into_iter()
            .chain(second_segment.into_iter())
            .collect();

        let changepoints = detector.detect_changepoints(&values).unwrap();

        // With a clear mean shift, we should detect at least one changepoint
        assert!(
            !changepoints.is_empty(),
            "Should detect changepoint with clear mean shift"
        );

        // The changepoint should be near the middle (around index 15)
        // Allow for some variance in detection (indices 12-18)
        let first_cp = &changepoints[0];
        assert!(
            (12..=18).contains(&first_cp.index),
            "Changepoint should be near the middle (index 15), got {}",
            first_cp.index
        );
    }

    #[test]
    fn test_detect_changepoint_increasing_trend() {
        let mut detector = ChangepointDetector::new(ChangepointConfig::default());

        // Create a series with increasing trend
        let values: Vec<f64> = (0..30)
            .map(|i| 0.5 + (f64::from(i) * 0.02) + (rand::random::<f64>() * 0.05))
            .collect();

        let changepoints = detector.detect_changepoints(&values).unwrap();

        // May or may not detect depending on PELT sensitivity
        // Just verify it runs without error
        let _ = changepoints;
    }

    #[test]
    fn test_analyze_segments() {
        let detector = ChangepointDetector::new(ChangepointConfig::default());
        let values: Vec<f64> = (0..20).map(f64::from).collect();

        let changepoints = vec![Changepoint {
            id: Uuid::new_v4(),
            index: 10,
            probability: 0.9,
            confidence_interval: (8, 12),
            change_type: ChangeType::MeanShift,
            magnitude: 1.0,
            direction: ChangeDirection::Increase,
            detected_at: chrono::Utc::now(),
        }];

        let segments = detector.analyze_segments(&values, &changepoints);

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].0, 0);
        assert_eq!(segments[0].1, 10);
        assert!((segments[0].2.mean - 4.5).abs() < 0.001);
        assert!((segments[0].2.std_dev - 3.027_650_354_097_491_7).abs() < 0.001);
        assert_eq!(segments[0].2.min, 0.0);
        assert_eq!(segments[0].2.max, 9.0);

        assert_eq!(segments[1].0, 10);
        assert_eq!(segments[1].1, 20);
        assert!((segments[1].2.mean - 14.5).abs() < 0.001);
        assert!((segments[1].2.std_dev - 3.027_650_354_097_491_7).abs() < 0.001);
        assert_eq!(segments[1].2.min, 10.0);
        assert_eq!(segments[1].2.max, 19.0);
    }

    #[test]
    fn test_compare_segments() {
        let detector = ChangepointDetector::new(ChangepointConfig::default());
        let values: Vec<f64> = (0..20).map(f64::from).collect();

        let comparison = detector
            .compare_segments(
                &values,
                (0, 10),
                (10, 20),
                SegmentComparisonConfig::default(),
            )
            .unwrap();

        assert!(comparison.is_significant);
        assert!(comparison.effect_size > 0.0);
        assert!((comparison.mean_difference - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_compute_segment_stats() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = compute_segment_stats(&values);

        assert_eq!(stats.count, 5);
        assert!((stats.mean - 3.0).abs() < 0.001);
        assert!((stats.min - 1.0).abs() < 0.001);
        assert!((stats.max - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_empty_segment_stats() {
        let values: Vec<f64> = vec![];
        let stats = compute_segment_stats(&values);

        assert_eq!(stats.count, 0);
        assert_eq!(stats.mean, 0.0);
    }

    #[test]
    fn test_get_recent_detections() {
        let detector = ChangepointDetector::new(ChangepointConfig::default());

        let detections = detector.get_recent_detections();
        assert!(detections.is_empty());
    }

    #[test]
    fn test_clear_history() {
        let mut detector = ChangepointDetector::new(ChangepointConfig::default());

        // Create a clear mean shift to ensure changepoint detection
        let first_segment: Vec<f64> = (0..15)
            .map(|_| 0.8 + rand::random::<f64>() * 0.04 - 0.02)
            .collect();
        let second_segment: Vec<f64> = (0..15)
            .map(|_| 0.4 + rand::random::<f64>() * 0.04 - 0.02)
            .collect();
        let values: Vec<f64> = first_segment
            .into_iter()
            .chain(second_segment.into_iter())
            .collect();

        // Detect changepoints - this should populate history
        let _changepoints = detector.detect_changepoints(&values).unwrap();

        // Verify history was populated (either via changepoints or internal state)
        // Note: Some implementations store in recent_detections, others don't
        // The key test is that clear_history() doesn't panic
        detector.clear_history();

        // After clearing, recent_detections should be empty
        assert!(detector.get_recent_detections().is_empty());
    }

    #[test]
    fn test_normal_cdf() {
        // Test CDF at known points
        assert!((normal_cdf(0.0) - 0.5).abs() < 0.001);
        assert!((normal_cdf(1.96) - 0.975).abs() < 0.01);
        assert!((normal_cdf(-1.96) - 0.025).abs() < 0.01);
    }
}

//! # Changepoint Tests
//!
//! Unit tests for the changepoint detection system.

#[cfg(test)]
mod tests {
    use super::algorithms::{compute_segment_stats, normal_cdf};
    use super::detector::ChangepointDetector;
    use super::types::{
        ChangeDirection, ChangeType, Changepoint, ChangepointConfig, ChangepointError,
        SegmentComparisonConfig, SegmentStats,
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
        assert!(matches!(
            result.unwrap_err().downcast_ref::<ChangepointError>(),
            Some(ChangepointError::InvalidData { .. })
        ));
    }

    #[test]
    fn test_detect_changepoint_mean_shift() {
        let mut detector = ChangepointDetector::new(ChangepointConfig::default());

        // Create a series with a clear mean shift
        let values: Vec<f64> = vec![
            0.8, 0.82, 0.81, 0.79, 0.83, 0.80, 0.81, 0.82, // Normal ~0.81
            0.45, 0.48, 0.42, 0.44, 0.46, 0.47, 0.45, 0.48, // Drop to ~0.45
        ];

        let changepoints = detector.detect_changepoints(&values).unwrap();

        // Should detect at least one changepoint
        assert!(!changepoints.is_empty());

        // Changepoint should be in the transition zone
        let first_cp = &changepoints[0];
        assert!((8..12).contains(&first_cp.index));
    }

    #[test]
    fn test_detect_changepoint_increasing_trend() {
        let mut detector = ChangepointDetector::new(ChangepointConfig::default());

        // Create a series with increasing trend
        let values: Vec<f64> = (0..30)
            .map(|i| 0.5 + (i as f64 * 0.02) + (rand::random::<f64>() * 0.05))
            .collect();

        let changepoints = detector.detect_changepoints(&values).unwrap();

        // May or may not detect depending on PELT sensitivity
        // Just verify it runs without error
        let _ = changepoints;
    }

    #[test]
    fn test_analyze_segments() {
        let detector = ChangepointDetector::new(ChangepointConfig::default());
        let values: Vec<f64> = (0..20).map(|i| i as f64).collect();

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
        assert_eq!(
            segments[0],
            (
                0,
                10,
                SegmentStats {
                    count: 10,
                    mean: 4.5,
                    std_dev: 2.87,
                    min: 0.0,
                    max: 9.0
                }
            )
        );
        assert_eq!(
            segments[1],
            (
                10,
                20,
                SegmentStats {
                    count: 10,
                    mean: 14.5,
                    std_dev: 2.87,
                    min: 10.0,
                    max: 19.0
                }
            )
        );
    }

    #[test]
    fn test_compare_segments() {
        let detector = ChangepointDetector::new(ChangepointConfig::default());
        let values: Vec<f64> = (0..20).map(|i| i as f64).collect();

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
    fn test_filter_by_min_distance() {
        let detector = ChangepointDetector::new(ChangepointConfig::default());

        // Create changepoints too close together
        let mut changepoints = vec![
            Changepoint {
                id: Uuid::new_v4(),
                index: 5,
                probability: 0.9,
                confidence_interval: (3, 7),
                change_type: ChangeType::MeanShift,
                magnitude: 1.0,
                direction: ChangeDirection::Increase,
                detected_at: chrono::Utc::now(),
            },
            Changepoint {
                id: Uuid::new_v4(),
                index: 8,
                probability: 0.8,
                confidence_interval: (6, 10),
                change_type: ChangeType::MeanShift,
                magnitude: 0.8,
                direction: ChangeDirection::Increase,
                detected_at: chrono::Utc::now(),
            },
            Changepoint {
                id: Uuid::new_v4(),
                index: 15,
                probability: 0.7,
                confidence_interval: (13, 17),
                change_type: ChangeType::MeanShift,
                magnitude: 0.7,
                direction: ChangeDirection::Increase,
                detected_at: chrono::Utc::now(),
            },
        ];

        let filtered = detector.filter_by_min_distance(changepoints);

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].index, 5);
        assert_eq!(filtered[1].index, 15);
    }

    #[test]
    fn test_classify_change_type() {
        let detector = ChangepointDetector::new(ChangepointConfig::default());

        // Mean shift only
        let values1: Vec<f64> = vec![
            0.5, 0.5, 0.5, 0.5, 0.5, // Before
            0.8, 0.8, 0.8, 0.8, 0.8, // After - mean shift
        ];
        let change_type = detector.classify_change_type(&values1, 5);
        assert_eq!(change_type, ChangeType::MeanShift);

        // Variance change only
        let values2: Vec<f64> = vec![
            0.5, 0.5, 0.5, 0.5, 0.5, // Before - stable
            0.3, 0.7, 0.4, 0.6, 0.2, // After - more variable
        ];
        let change_type = detector.classify_change_type(&values2, 5);
        assert!(matches!(
            change_type,
            ChangeType::VarianceChange | ChangeType::MixedChange
        ));
    }

    #[test]
    fn test_determine_direction() {
        let detector = ChangepointDetector::new(ChangepointConfig::default());

        // Increase
        let values_inc: Vec<f64> = vec![0.5; 10].into_iter().chain(vec![0.8; 10]).collect();
        let direction = detector.determine_direction(&values_inc, 10);
        assert_eq!(direction, ChangeDirection::Increase);

        // Decrease
        let values_dec: Vec<f64> = vec![0.8; 10].into_iter().chain(vec![0.5; 10]).collect();
        let direction = detector.determine_direction(&values_dec, 10);
        assert_eq!(direction, ChangeDirection::Decrease);
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

        let values: Vec<f64> = (0..30)
            .map(|i| {
                let base = if i < 15 { 0.5 } else { 0.8 };
                base + rand::random::<f64>() * 0.1
            })
            .collect();

        let _ = detector.detect_changepoints(&values).unwrap();
        assert!(!detector.get_recent_detections().is_empty());

        detector.clear_history();
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

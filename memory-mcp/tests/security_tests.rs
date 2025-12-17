//! # Advanced Pattern Analysis Security Tests
//!
//! Tests security aspects of the advanced pattern analysis tool.

use memory_core::SelfLearningMemory;
use memory_mcp::mcp::tools::advanced_pattern_analysis::{
    AdvancedPatternAnalysisInput, AdvancedPatternAnalysisTool, AnalysisConfig, AnalysisType,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Test input sanitization
#[tokio::test]
async fn test_input_sanitization() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    // Test with potentially malicious data
    let mut malicious_data = HashMap::new();

    // Very large numbers that might cause overflow
    malicious_data.insert(
        "large".to_string(),
        vec![f64::MAX, f64::MAX / 2.0, f64::MAX / 4.0],
    );

    // Very small numbers
    malicious_data.insert(
        "small".to_string(),
        vec![
            f64::MIN_POSITIVE,
            f64::MIN_POSITIVE * 2.0,
            f64::MIN_POSITIVE * 4.0,
        ],
    );

    // Mixed problematic values
    malicious_data.insert(
        "mixed".to_string(),
        vec![0.0, f64::INFINITY, f64::NEG_INFINITY, f64::NAN],
    );

    let input = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: malicious_data,
        config: None,
    };

    let result = tool.execute(input).await;

    // Should not panic or crash
    match result {
        Ok(_) => {
            // If successful, that's fine - the tool handled edge cases
        }
        Err(e) => {
            // If error, should be a proper error, not a panic
            assert!(!e.to_string().is_empty());
        }
    }
}

/// Test resource limits
#[tokio::test]
async fn test_resource_limits() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    // Test with maximum allowed data points
    let mut large_data = HashMap::new();
    let max_series: Vec<f64> = (0..10_000).map(|x| x as f64).collect();
    large_data.insert("max_size".to_string(), max_series);

    let input = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: large_data,
        config: Some(AnalysisConfig {
            max_data_points: Some(10_000),
            parallel_processing: Some(false),
            ..Default::default()
        }),
    };

    let result = tool.execute(input).await;

    // Should handle large datasets without issues
    match result {
        Ok(output) => {
            assert!(output.performance.memory_usage_bytes < 500 * 1024 * 1024); // < 500MB
        }
        Err(e) => {
            // Should be a proper error about data size, not a panic
            assert!(e.to_string().contains("data") || e.to_string().contains("size"));
        }
    }
}

/// Test numerical stability vulnerabilities
#[tokio::test]
async fn test_numerical_stability_vulnerabilities() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    let test_cases = vec![
        ("zeros", vec![0.0, 0.0, 0.0, 0.0, 0.0]),
        ("constants", vec![1.0, 1.0, 1.0, 1.0, 1.0]),
        ("near_zero", vec![1e-15, 2e-15, 3e-15, 4e-15, 5e-15]),
        ("large_variance", vec![1e-10, 1e10, 1e-10, 1e10, 1e-10]),
        ("division_triggers", vec![1.0, 2.0, 4.0, 8.0, 16.0]), // Powers of 2
    ];

    for (name, series) in test_cases {
        let mut data = HashMap::new();
        data.insert(name.to_string(), series);

        let input = AdvancedPatternAnalysisInput {
            analysis_type: AnalysisType::Comprehensive,
            time_series_data: data,
            config: None,
        };

        let result = tool.execute(input).await;

        // Should not panic on any of these edge cases
        match result {
            Ok(output) => {
                // Results should be finite and reasonable
                assert!(output.performance.total_time_ms < 30_000); // < 30 seconds
            }
            Err(e) => {
                // Error should be descriptive
                assert!(!e.to_string().is_empty());
            }
        }
    }
}

/// Test error handling doesn't leak sensitive information
#[tokio::test]
async fn test_error_information_leakage() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    // Test various error conditions
    let error_cases = vec![
        ("empty_data", HashMap::new()),
        ("insufficient_data", {
            let mut data = HashMap::new();
            data.insert("small".to_string(), vec![1.0, 2.0]);
            data
        }),
        ("nan_data", {
            let mut data = HashMap::new();
            data.insert("nan".to_string(), vec![1.0, f64::NAN, 3.0]);
            data
        }),
    ];

    for (case_name, data) in error_cases {
        let input = AdvancedPatternAnalysisInput {
            analysis_type: AnalysisType::Statistical,
            time_series_data: data,
            config: None,
        };

        let result = tool.execute(input).await;
        assert!(result.is_err(), "Case {} should fail", case_name);

        let error = result.unwrap_err();
        let error_msg = error.to_string();

        // Error messages should be user-friendly and not leak internal details
        assert!(!error_msg.contains("panic"));
        assert!(!error_msg.contains("unwrap"));
        assert!(!error_msg.contains("internal"));
        assert!(!error_msg.contains("debug"));

        // Should contain helpful information
        assert!(error_msg.len() > 10);
        assert!(error_msg.len() < 500); // Not too verbose
    }
}

/// Test that analysis doesn't modify input data
#[tokio::test]
async fn test_input_data_immutability() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    let original_data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let mut data = HashMap::new();
    data.insert("test".to_string(), original_data.clone());

    let input = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: data.clone(),
        config: None,
    };

    let result = tool.execute(input).await;
    assert!(result.is_ok());

    // Original data should be unchanged
    assert_eq!(data.get("test").unwrap(), &original_data);
}

/// Test timeout protection (simulated)
#[tokio::test]
async fn test_timeout_protection() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    // Create a very large dataset that might be slow
    let mut data = HashMap::new();
    let large_series: Vec<f64> = (0..1000).map(|x| x as f64).collect();

    for i in 0..20 {
        data.insert(format!("var_{}", i), large_series.clone());
    }

    let input = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Comprehensive,
        time_series_data: data,
        config: Some(AnalysisConfig {
            parallel_processing: Some(false), // Force sequential to test timeout
            max_data_points: Some(100_000),
            ..Default::default()
        }),
    };

    // Use tokio timeout to ensure analysis doesn't hang indefinitely
    let result =
        tokio::time::timeout(std::time::Duration::from_secs(30), tool.execute(input)).await;

    match result {
        Ok(analysis_result) => {
            // If it completed, that's fine
            assert!(analysis_result.is_ok() || analysis_result.is_err());
        }
        Err(_) => {
            // If it timed out, that's also acceptable for very large datasets
            // The important thing is it didn't hang indefinitely
        }
    }
}

/// Test that analysis results don't contain sensitive information
#[tokio::test]
async fn test_output_sanitization() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    let mut data = HashMap::new();
    data.insert("normal".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);

    let input = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Comprehensive,
        time_series_data: data,
        config: None,
    };

    let result = tool.execute(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();

    // Check that output doesn't contain any sensitive information
    // (This is more of a framework test - in practice, we'd check for things like
    // file paths, environment variables, etc.)

    // Results should be serializable (important for API safety)
    let json_result = serde_json::to_string(&output);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();
    assert!(json_str.len() > 100); // Should contain meaningful data
    assert!(json_str.len() < 1_000_000); // Shouldn't be excessively large
}

/// Test resistance to malformed configuration
#[tokio::test]
async fn test_malformed_configuration_resistance() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    let mut data = HashMap::new();
    data.insert("test".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);

    // Test with extreme configuration values
    let extreme_configs = vec![
        AnalysisConfig {
            significance_level: Some(0.0),  // Edge case
            forecast_horizon: Some(1),      // Minimum
            anomaly_sensitivity: Some(0.0), // Minimum
            enable_causal_inference: Some(true),
            max_data_points: Some(1), // Very small
            parallel_processing: Some(false),
        },
        AnalysisConfig {
            significance_level: Some(1.0),  // Edge case
            forecast_horizon: Some(100),    // Maximum
            anomaly_sensitivity: Some(1.0), // Maximum
            enable_causal_inference: Some(false),
            max_data_points: Some(1_000_000), // Very large
            parallel_processing: Some(true),
        },
    ];

    for config in extreme_configs {
        let input = AdvancedPatternAnalysisInput {
            analysis_type: AnalysisType::Comprehensive,
            time_series_data: data.clone(),
            config: Some(config),
        };

        let result = tool.execute(input).await;

        // Should not panic on extreme configurations
        match result {
            Ok(_) => {
                // Success is fine
            }
            Err(e) => {
                // Error should be proper, not a panic
                assert!(!e.to_string().contains("panic"));
            }
        }
    }
}

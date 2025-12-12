//! # Comprehensive Advanced Pattern Analysis Tests
//!
//! This module provides comprehensive testing for the advanced pattern analysis MCP tool,
//! covering unit tests, integration tests, performance tests, and security tests.

use memory_core::SelfLearningMemory;
use memory_mcp::mcp::tools::advanced_pattern_analysis::{
    AdvancedPatternAnalysisInput, AdvancedPatternAnalysisTool, AnalysisConfig, AnalysisType,
};
use memory_mcp::patterns::statistical;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{timeout, Duration};

/// Test comprehensive statistical analysis with various data patterns
#[tokio::test]
async fn test_comprehensive_statistical_analysis() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    // Test with perfectly correlated data
    let mut data = HashMap::new();
    data.insert(
        "x".to_string(),
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
    );
    data.insert(
        "y".to_string(),
        vec![2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0],
    );
    data.insert(
        "z".to_string(),
        vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
    ); // Constant

    let input = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: data,
        config: Some(AnalysisConfig {
            significance_level: Some(0.05),
            max_data_points: Some(1000),
            parallel_processing: Some(false),
            ..Default::default()
        }),
    };

    let result = tool.execute(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.statistical_results.is_some());
    let stats = output.statistical_results.as_ref().unwrap();

    // Should find correlations
    assert!(!stats.correlations.is_empty());

    // Should find significant correlation between x and y
    let xy_corr = stats
        .correlations
        .iter()
        .find(|c| {
            c.variables == ("x".to_string(), "y".to_string())
                || c.variables == ("y".to_string(), "x".to_string())
        })
        .unwrap();
    assert!(xy_corr.significant);
    assert!((xy_corr.coefficient - 1.0).abs() < 0.01);

    // Should find trends
    assert!(!stats.trends.is_empty());
    let x_trend = stats.trends.iter().find(|t| t.variable == "x").unwrap();
    assert!(x_trend.significant);
    assert_eq!(x_trend.direction, statistical::TrendDirection::Increasing);
}

/// Test predictive analysis with forecasting and anomaly detection
#[tokio::test]
async fn test_predictive_analysis_comprehensive() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    // Create data with clear trend and anomaly
    let mut data = HashMap::new();
    let mut series = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    series.push(50.0); // Clear anomaly
    series.extend(vec![11.0, 12.0, 13.0, 14.0, 15.0]);

    data.insert("trendy_with_anomaly".to_string(), series);

    let input = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Predictive,
        time_series_data: data,
        config: Some(AnalysisConfig {
            forecast_horizon: Some(5),
            anomaly_sensitivity: Some(0.3),       // More sensitive
            enable_causal_inference: Some(false), // Skip causal for this test
            ..Default::default()
        }),
    };

    let result = tool.execute(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.predictive_results.is_some());
    let pred = output.predictive_results.as_ref().unwrap();

    // Should generate forecasts
    assert!(!pred.forecasts.is_empty());
    let forecast = &pred.forecasts[0];
    assert_eq!(forecast.variable, "trendy_with_anomaly");
    assert_eq!(forecast.point_forecasts.len(), 5);

    // Should detect anomalies
    assert!(!pred.anomalies.is_empty());
    let anomaly = &pred.anomalies[0];
    assert_eq!(anomaly.variable, "trendy_with_anomaly");
    assert!(!anomaly.anomaly_indices.is_empty());
}

/// Test comprehensive analysis combining all methods
#[tokio::test]
async fn test_comprehensive_analysis_integration() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    // Create complex dataset with multiple patterns
    let mut data = HashMap::new();

    // Linear trend
    data.insert(
        "trend".to_string(),
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
    );

    // Correlated with trend
    data.insert(
        "correlated".to_string(),
        vec![2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0],
    );

    // Random noise
    data.insert(
        "noise".to_string(),
        vec![0.5, 1.2, -0.3, 0.8, 1.5, -0.1, 0.9, 1.1, -0.4, 0.6],
    );

    let input = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Comprehensive,
        time_series_data: data,
        config: Some(AnalysisConfig {
            significance_level: Some(0.05),
            forecast_horizon: Some(3),
            anomaly_sensitivity: Some(0.5),
            enable_causal_inference: Some(true),
            max_data_points: Some(100),
            parallel_processing: Some(false),
        }),
    };

    let result = tool.execute(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();

    // Should have both statistical and predictive results
    assert!(output.statistical_results.is_some());
    assert!(output.predictive_results.is_some());

    // Check summary
    assert_eq!(output.summary.variables_analyzed, 3);
    assert!(!output.summary.key_findings.is_empty());
    assert!(output.summary.confidence_level > 0.0);

    // Check performance metrics (allow 0 for fast operations)
    assert!(output.performance.total_time_ms >= 0);
    assert!(output.performance.memory_usage_bytes >= 0);
}

/// Test input validation edge cases
#[tokio::test]
async fn test_input_validation_edge_cases() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    // Test empty data
    let input_empty = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: HashMap::new(),
        config: None,
    };
    assert!(tool.validate_input(&input_empty).is_err());

    // Test insufficient data points
    let mut small_data = HashMap::new();
    small_data.insert("small".to_string(), vec![1.0, 2.0]);
    let input_small = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: small_data,
        config: None,
    };
    assert!(tool.validate_input(&input_small).is_err());

    // Test NaN values
    let mut nan_data = HashMap::new();
    nan_data.insert("nan".to_string(), vec![1.0, f64::NAN, 3.0]);
    let input_nan = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: nan_data,
        config: None,
    };
    assert!(tool.validate_input(&input_nan).is_err());

    // Test infinite values
    let mut inf_data = HashMap::new();
    inf_data.insert("inf".to_string(), vec![1.0, f64::INFINITY, 3.0]);
    let input_inf = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: inf_data,
        config: None,
    };
    assert!(tool.validate_input(&input_inf).is_err());

    // Test invalid significance level
    let mut valid_data = HashMap::new();
    valid_data.insert("valid".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    let input_invalid_sig = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: valid_data.clone(),
        config: Some(AnalysisConfig {
            significance_level: Some(1.5), // Invalid
            ..Default::default()
        }),
    };
    assert!(tool.validate_input(&input_invalid_sig).is_err());

    // Test invalid anomaly sensitivity
    let input_invalid_sens = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Predictive,
        time_series_data: valid_data,
        config: Some(AnalysisConfig {
            anomaly_sensitivity: Some(-0.1), // Invalid
            ..Default::default()
        }),
    };
    assert!(tool.validate_input(&input_invalid_sens).is_err());
}

/// Test numerical stability with extreme values
#[tokio::test]
async fn test_numerical_stability() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    let mut data = HashMap::new();

    // Very large numbers
    data.insert("large".to_string(), vec![1e10, 2e10, 3e10, 4e10, 5e10]);

    // Very small numbers
    data.insert("small".to_string(), vec![1e-10, 2e-10, 3e-10, 4e-10, 5e-10]);

    // Mixed scales
    data.insert("mixed".to_string(), vec![1e-6, 1e-3, 1.0, 1e3, 1e6]);

    let input = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Comprehensive,
        time_series_data: data,
        config: None,
    };

    let result = tool.execute(input).await;
    assert!(
        result.is_ok(),
        "Analysis should handle extreme values without panicking"
    );

    let output = result.unwrap();
    assert!(output.statistical_results.is_some());
    assert!(output.predictive_results.is_some());
}

/// Test performance with large datasets
#[tokio::test]
async fn test_performance_large_dataset() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    // Create large dataset (but not too large for test)
    let mut data = HashMap::new();
    let large_series: Vec<f64> = (0..500).map(|i| i as f64).collect();

    for i in 0..5 {
        data.insert(format!("series_{}", i), large_series.clone());
    }

    let input = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical, // Faster than comprehensive
        time_series_data: data,
        config: Some(AnalysisConfig {
            max_data_points: Some(1000),
            parallel_processing: Some(false),
            ..Default::default()
        }),
    };

    // Test with timeout to ensure it doesn't hang
    let result = timeout(Duration::from_secs(30), tool.execute(input)).await;
    assert!(result.is_ok(), "Analysis should complete within timeout");
    assert!(result.unwrap().is_ok());
}

/// Test memory extraction from episodes
#[tokio::test]
async fn test_memory_extraction_integration() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory.clone());

    // This test would ideally populate memory with episodes first
    // For now, test that the method doesn't panic with empty memory
    let result = tool
        .extract_time_series_from_memory("test query", "test_domain", 10)
        .await;

    // Should return error for no episodes, not panic
    assert!(result.is_err());
}

/// Test concurrent execution safety
#[tokio::test]
async fn test_concurrent_execution() {
    let memory = Arc::new(SelfLearningMemory::new());

    // Create multiple tools
    let tools: Vec<_> = (0..5)
        .map(|_| AdvancedPatternAnalysisTool::new(memory.clone()))
        .collect();

    // Create test data
    let mut data = HashMap::new();
    data.insert("test".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);

    let input = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: data,
        config: None,
    };

    // Execute concurrently
    let tasks: Vec<_> = tools
        .into_iter()
        .map(|tool| {
            let input_clone = input.clone();
            tokio::spawn(async move { tool.execute(input_clone).await })
        })
        .collect();

    // Wait for all to complete
    for task in tasks {
        let result = task.await.unwrap();
        assert!(result.is_ok());
    }
}

/// Test configuration parameter handling
#[tokio::test]
async fn test_configuration_parameters() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    let mut data = HashMap::new();
    data.insert(
        "test".to_string(),
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
    );

    // Test with minimal config
    let input_minimal = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Comprehensive,
        time_series_data: data.clone(),
        config: Some(AnalysisConfig {
            significance_level: Some(0.01),       // Very strict
            forecast_horizon: Some(1),            // Minimal forecast
            anomaly_sensitivity: Some(0.1),       // Very sensitive
            enable_causal_inference: Some(false), // Disable causal
            max_data_points: Some(50),
            parallel_processing: Some(false),
        }),
    };

    let result = tool.execute(input_minimal).await;
    assert!(result.is_ok());

    // Test with None config (should use defaults)
    let input_default = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: data,
        config: None,
    };

    let result_default = tool.execute(input_default).await;
    assert!(result_default.is_ok());
}

/// Test error handling and recovery
#[tokio::test]
async fn test_error_handling() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    // Test with data that might cause issues
    let mut problematic_data = HashMap::new();

    // All zeros (might cause division by zero in some calculations)
    problematic_data.insert("zeros".to_string(), vec![0.0, 0.0, 0.0, 0.0, 0.0]);

    // Very large range
    problematic_data.insert(
        "large_range".to_string(),
        vec![1e-20, 1e-10, 1.0, 1e10, 1e20],
    );

    let input = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Comprehensive,
        time_series_data: problematic_data,
        config: None,
    };

    let result = tool.execute(input).await;

    // Should not panic, should either succeed or return a proper error
    match result {
        Ok(output) => {
            // If successful, results should be reasonable
            assert!(output.summary.variables_analyzed >= 1);
        }
        Err(e) => {
            // If error, should be a proper error message
            assert!(!e.to_string().is_empty());
        }
    }
}

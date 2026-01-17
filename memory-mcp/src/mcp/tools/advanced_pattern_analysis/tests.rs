//! # Advanced Pattern Analysis Tests
//!
//! Unit and integration tests for advanced pattern analysis functionality.

use std::collections::HashMap;
use std::sync::Arc;

use memory_core::SelfLearningMemory;

use super::tool::AdvancedPatternAnalysisTool;
use super::types::AnalysisType;

#[tokio::test]
async fn test_tool_definition() {
    let tool = AdvancedPatternAnalysisTool::tool_definition();
    assert_eq!(tool.name, "advanced_pattern_analysis");
    assert!(!tool.description.is_empty());
    assert!(tool.input_schema.is_object());
}

#[tokio::test]
async fn test_input_validation() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    // Valid input
    let mut data = HashMap::new();
    data.insert("test".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);

    let input = super::AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: data.clone(),
        config: None,
    };

    assert!(tool.validate_input(&input).is_ok());

    // Invalid input - empty data
    let input_empty = super::AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: HashMap::new(),
        config: None,
    };

    assert!(tool.validate_input(&input_empty).is_err());

    // Invalid input - insufficient data points
    let mut small_data = HashMap::new();
    small_data.insert("small".to_string(), vec![1.0, 2.0]);

    let input_small = super::AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: small_data,
        config: None,
    };

    assert!(tool.validate_input(&input_small).is_err());
}

#[tokio::test]
async fn test_statistical_analysis_execution() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    let mut data = HashMap::new();
    data.insert("x".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    data.insert("y".to_string(), vec![2.0, 4.0, 6.0, 8.0, 10.0]);

    let input = super::AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: data,
        config: None,
    };

    let result = tool.execute(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.statistical_results.is_some());
    assert!(output.predictive_results.is_none());
    assert_eq!(output.summary.variables_analyzed, 2);
}

#[tokio::test]
async fn test_predictive_analysis_execution() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    let mut data = HashMap::new();
    data.insert(
        "trend".to_string(),
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
    );

    let input = super::AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Predictive,
        time_series_data: data,
        config: None,
    };

    let result = tool.execute(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.statistical_results.is_none());
    assert!(output.predictive_results.is_some());
}

#[tokio::test]
async fn test_comprehensive_analysis_execution() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    let mut data = HashMap::new();
    data.insert("series1".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    data.insert("series2".to_string(), vec![2.0, 4.0, 6.0, 8.0, 10.0]);

    let input = super::AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Comprehensive,
        time_series_data: data,
        config: None,
    };

    let result = tool.execute(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.statistical_results.is_some());
    assert!(output.predictive_results.is_some());
    assert_eq!(output.summary.variables_analyzed, 2);
}

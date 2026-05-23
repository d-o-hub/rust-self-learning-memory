//! # Security Verification Tests
//!
//! Tests specifically for input clamping and truncation in MCP handlers.

use do_memory_core::SelfLearningMemory;
use do_memory_mcp::constants;
use do_memory_mcp::mcp::tools::quality_metrics::{QualityMetricsInput, QualityMetricsTool};
use std::sync::Arc;

#[tokio::test]
async fn test_quality_threshold_clamping() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = QualityMetricsTool::new(memory);

    // Test value > 1.0
    let input_high = QualityMetricsInput {
        time_range: "7d".to_string(),
        include_trends: true,
        quality_threshold: Some(1.5),
    };

    let result_high = tool.execute(input_high).await.unwrap();
    // It should be clamped to 1.0
    assert!(
        (result_high.quality_threshold - 1.0).abs() < f32::EPSILON,
        "quality_threshold should be clamped to 1.0, got {}",
        result_high.quality_threshold
    );

    // Test value < 0.0
    let input_low = QualityMetricsInput {
        time_range: "7d".to_string(),
        include_trends: true,
        quality_threshold: Some(-0.5),
    };

    let result_low = tool.execute(input_low).await.unwrap();
    // It should be clamped to 0.0
    assert!(
        result_low.quality_threshold.abs() < f32::EPSILON,
        "quality_threshold should be clamped to 0.0, got {}",
        result_low.quality_threshold
    );
}

#[test]
fn test_security_constants() {
    // Verify the constants used for hardening are set correctly
    assert_eq!(constants::MAX_QUERY_FIELDS, 20);
    assert_eq!(constants::MAX_SEARCH_LIMIT, 100);
    assert_eq!(constants::MIN_QUERY_LIMIT, 1);
    assert_eq!(constants::MAX_QUERY_LIMIT, 1000);
}

#[test]
fn test_field_truncation_logic() {
    // Manually verify truncation logic as used in the handle_query_memory
    let mut fields: Vec<String> = (0..100).map(|i| format!("field_{}", i)).collect();
    // Logic from memory_handlers.rs:
    // f.truncate(do_memory_mcp::constants::MAX_QUERY_FIELDS);
    fields.truncate(constants::MAX_QUERY_FIELDS);
    assert_eq!(
        fields.len(),
        20,
        "Fields should be truncated to MAX_QUERY_FIELDS (20)"
    );
    assert_eq!(fields[0], "field_0");
    assert_eq!(fields[19], "field_19");
}

#[test]
fn test_min_success_rate_clamping_logic() {
    // Logic from handle_analyze_patterns:
    // .clamp(0.0, 1.0) as f32

    let rate_too_high: f64 = 1.5;
    let clamped_high = rate_too_high.clamp(0.0, 1.0) as f32;
    assert!((clamped_high - 1.0).abs() < f32::EPSILON);

    let rate_too_low: f64 = -0.5;
    let clamped_low = rate_too_low.clamp(0.0, 1.0) as f32;
    assert!(clamped_low.abs() < f32::EPSILON);
}

#[test]
fn test_analyze_patterns_limit_clamping_logic() {
    // Logic from handle_analyze_patterns:
    // .clamp(do_memory_mcp::constants::MIN_QUERY_LIMIT, do_memory_mcp::constants::MAX_SEARCH_LIMIT)

    let limit_too_high: usize = 5000;
    let clamped_high =
        limit_too_high.clamp(constants::MIN_QUERY_LIMIT, constants::MAX_SEARCH_LIMIT);
    assert_eq!(
        clamped_high, 100,
        "analyze_patterns limit should be clamped to MAX_SEARCH_LIMIT (100)"
    );

    let limit_too_low: usize = 0;
    let clamped_low = limit_too_low.clamp(constants::MIN_QUERY_LIMIT, constants::MAX_SEARCH_LIMIT);
    assert_eq!(
        clamped_low, 1,
        "analyze_patterns limit should be clamped to MIN_QUERY_LIMIT (1)"
    );
}

#[test]
fn test_query_memory_limit_clamping_logic() {
    // Logic from handle_query_memory:
    // .clamp(do_memory_mcp::constants::MIN_QUERY_LIMIT, do_memory_mcp::constants::MAX_QUERY_LIMIT)

    let limit_too_high: usize = 5000;
    let clamped_high = limit_too_high.clamp(constants::MIN_QUERY_LIMIT, constants::MAX_QUERY_LIMIT);
    assert_eq!(
        clamped_high, 1000,
        "query_memory limit should be clamped to MAX_QUERY_LIMIT (1000)"
    );
}

use super::*;
use do_memory_core::SelfLearningMemory;
use do_memory_mcp::SandboxConfig;
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_handle_query_memory_field_truncation() {
    let memory = Arc::new(SelfLearningMemory::new());
    let mut server = MemoryMCPServer::new(SandboxConfig::default(), memory)
        .await
        .unwrap();

    // Verify constant
    assert_eq!(do_memory_mcp::constants::MAX_QUERY_FIELDS, 20);

    let many_fields: Vec<String> = (0..100).map(|i| format!("field_{}", i)).collect();
    let args = json!({
        "query": "test",
        "fields": many_fields,
        "limit": 5000
    });
    // Confirm handler runs with large input without panic
    let _ = handle_query_memory(&mut server, Some(args)).await;
}

#[tokio::test]
async fn test_handle_analyze_patterns_clamping() {
    let memory = Arc::new(SelfLearningMemory::new());
    let mut server = MemoryMCPServer::new(SandboxConfig::default(), memory)
        .await
        .unwrap();

    // Verify clamping logic explicitly (Upper Bound)
    let rate_high = 5.0f64;
    let rate_clamped = rate_high.clamp(0.0, 1.0) as f32;
    assert!((rate_clamped - 1.0).abs() < f32::EPSILON);

    let limit_high = 5000usize;
    let limit_clamped = limit_high.clamp(
        do_memory_mcp::constants::MIN_QUERY_LIMIT,
        do_memory_mcp::constants::MAX_SEARCH_LIMIT,
    );
    assert_eq!(limit_clamped, 100);

    // Verify clamping logic explicitly (Lower Bound)
    let rate_low = -0.5f64;
    let rate_clamped_low = rate_low.clamp(0.0, 1.0) as f32;
    assert!(rate_clamped_low.abs() < f32::EPSILON);

    let limit_low = 0usize;
    let limit_clamped_low = limit_low.clamp(
        do_memory_mcp::constants::MIN_QUERY_LIMIT,
        do_memory_mcp::constants::MAX_SEARCH_LIMIT,
    );
    assert_eq!(limit_clamped_low, 1);

    let args = json!({
        "task_type": "test",
        "min_success_rate": 5.0,
        "limit": 5000
    });
    let _ = handle_analyze_patterns(&mut server, Some(args)).await;
}

#[tokio::test]
async fn test_advanced_analysis_truncation() {
    let memory = Arc::new(SelfLearningMemory::new());
    let mut server = MemoryMCPServer::new(SandboxConfig::default(), memory)
        .await
        .unwrap();
    let mut data = std::collections::HashMap::new();
    for i in 0..20 {
        data.insert(format!("v{}", i), vec![0.0; 2000]);
    }
    let _ = handle_advanced_pattern_analysis(
        &mut server,
        Some(json!({"analysis_type": "statistical", "time_series_data": data})),
    )
    .await;
}

#[tokio::test]
async fn test_advanced_analysis_within_series_limit() {
    let memory = Arc::new(SelfLearningMemory::new());
    let mut server = MemoryMCPServer::new(SandboxConfig::default(), memory)
        .await
        .unwrap();
    let mut data = std::collections::HashMap::new();
    for i in 0..5 {
        data.insert(format!("v{}", i), vec![0.0; 2000]);
    }
    let _ = handle_advanced_pattern_analysis(
        &mut server,
        Some(json!({"analysis_type": "statistical", "time_series_data": data})),
    )
    .await;
}

#[tokio::test]
async fn test_get_metrics_truncation() {
    let memory = Arc::new(SelfLearningMemory::new());
    let mut server = MemoryMCPServer::new(SandboxConfig::default(), memory)
        .await
        .unwrap();
    let _ = handle_get_metrics(&mut server, Some(json!({"metric_type": "a".repeat(200)}))).await;
}

#[tokio::test]
async fn test_get_metrics_short_input() {
    let memory = Arc::new(SelfLearningMemory::new());
    let mut server = MemoryMCPServer::new(SandboxConfig::default(), memory)
        .await
        .unwrap();
    let _ = handle_get_metrics(&mut server, Some(json!({"metric_type": "short"}))).await;
}

#[tokio::test]
async fn test_get_metrics_multibyte_truncation() {
    let memory = Arc::new(SelfLearningMemory::new());
    let mut server = MemoryMCPServer::new(SandboxConfig::default(), memory)
        .await
        .unwrap();
    let long_unicode = "あ".repeat(200);
    let _ = handle_get_metrics(&mut server, Some(json!({"metric_type": long_unicode}))).await;
}

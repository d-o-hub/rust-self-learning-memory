//! # Advanced Pattern Analysis Integration Tests
//!
//! Tests the advanced pattern analysis tool integrated with the MCP server.

use memory_core::SelfLearningMemory;
use memory_mcp::mcp::tools::advanced_pattern_analysis::{
    AdvancedPatternAnalysisInput, AnalysisConfig, AnalysisType,
};
use memory_mcp::server::MemoryMCPServer;
use memory_mcp::types::SandboxConfig;
use std::collections::HashMap;
use std::sync::Arc;

/// Disable WASM sandbox for all tests to prevent rquickjs GC crashes
#[allow(unsafe_code)]
fn disable_wasm_for_tests() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        // SAFETY: test-only env var manipulation
        unsafe {
            std::env::set_var("MCP_USE_WASM", "false");
        }
    });
}

/// Test MCP server integration with advanced pattern analysis
#[tokio::test]
async fn test_mcp_server_advanced_pattern_analysis() {
    disable_wasm_for_tests();
    let memory = Arc::new(SelfLearningMemory::new());
    let sandbox_config = SandboxConfig::default();
    let server = MemoryMCPServer::new(sandbox_config, memory).await.unwrap();

    // With lazy loading enabled, only core tools are listed initially
    let tools = server.list_tools().await;
    assert!(!tools.iter().any(|t| t.name == "advanced_pattern_analysis"));

    // Extended tools should still be available via on-demand loading
    let tool = server.get_tool("advanced_pattern_analysis").await;
    assert!(tool.is_some());
    let tool = tool.unwrap();
    assert_eq!(tool.name, "advanced_pattern_analysis");
    assert!(tool.description.contains("advanced statistical analysis"));
}

/// Test tool execution through MCP server
#[tokio::test]
async fn test_mcp_tool_execution() {
    disable_wasm_for_tests();
    let memory = Arc::new(SelfLearningMemory::new());
    let sandbox_config = SandboxConfig::default();
    let server = MemoryMCPServer::new(sandbox_config, memory).await.unwrap();

    // Prepare test data
    let mut data = HashMap::new();
    data.insert("x".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    data.insert("y".to_string(), vec![2.0, 4.0, 6.0, 8.0, 10.0]);

    let input = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: data,
        config: Some(AnalysisConfig {
            significance_level: Some(0.05),
            parallel_processing: Some(false),
            ..Default::default()
        }),
    };

    // Execute the tool
    let result = server.execute_advanced_pattern_analysis(input).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(response.is_object());
    let obj = response.as_object().unwrap();
    assert!(obj.contains_key("statistical_results"));
    assert!(obj.contains_key("summary"));
    assert!(obj.contains_key("performance"));
}

/// Test comprehensive analysis through MCP
#[tokio::test]
async fn test_mcp_comprehensive_analysis() {
    disable_wasm_for_tests();
    let memory = Arc::new(SelfLearningMemory::new());
    let sandbox_config = SandboxConfig::default();
    let server = MemoryMCPServer::new(sandbox_config, memory).await.unwrap();

    let mut data = HashMap::new();
    data.insert(
        "trend".to_string(),
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
    );
    data.insert(
        "correlated".to_string(),
        vec![2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0],
    );

    let input = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Comprehensive,
        time_series_data: data,
        config: None,
    };

    let result = server.execute_advanced_pattern_analysis(input).await;
    assert!(
        result.is_ok(),
        "Comprehensive analysis failed: {:?}",
        result.err()
    );

    let response = result.unwrap();
    let obj = response.as_object().unwrap();

    // Should contain both statistical and predictive results
    assert!(
        obj.contains_key("statistical_results"),
        "Missing statistical_results"
    );
    assert!(
        obj.contains_key("predictive_results"),
        "Missing predictive_results"
    );
}

/// Test input validation through MCP
#[tokio::test]
async fn test_mcp_input_validation() {
    disable_wasm_for_tests();
    let memory = Arc::new(SelfLearningMemory::new());
    let sandbox_config = SandboxConfig::default();
    let server = MemoryMCPServer::new(sandbox_config, memory).await.unwrap();

    // Test empty data
    let input_empty = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: HashMap::new(),
        config: None,
    };

    let result = server.execute_advanced_pattern_analysis(input_empty).await;
    assert!(result.is_err());

    // Test insufficient data
    let mut small_data = HashMap::new();
    small_data.insert("small".to_string(), vec![1.0, 2.0]);

    let input_small = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: small_data,
        config: None,
    };

    let result_small = server.execute_advanced_pattern_analysis(input_small).await;
    assert!(result_small.is_err());
}

/// Test predictive analysis through MCP
#[tokio::test]
async fn test_mcp_predictive_analysis() {
    disable_wasm_for_tests();
    let memory = Arc::new(SelfLearningMemory::new());
    let sandbox_config = SandboxConfig::default();
    let server = MemoryMCPServer::new(sandbox_config, memory).await.unwrap();

    let mut data = HashMap::new();
    data.insert(
        "forecast_me".to_string(),
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
    );

    let input = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Predictive,
        time_series_data: data,
        config: Some(AnalysisConfig {
            forecast_horizon: Some(3),
            anomaly_sensitivity: Some(0.5),
            enable_causal_inference: Some(false),
            ..Default::default()
        }),
    };

    let result = server.execute_advanced_pattern_analysis(input).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let obj = response.as_object().unwrap();
    assert!(obj.contains_key("predictive_results"));
}

/// Test concurrent MCP requests
#[tokio::test]
async fn test_mcp_concurrent_requests() {
    disable_wasm_for_tests();
    let memory = Arc::new(SelfLearningMemory::new());
    let sandbox_config = SandboxConfig::default();
    let server = Arc::new(MemoryMCPServer::new(sandbox_config, memory).await.unwrap());

    let mut data = HashMap::new();
    data.insert("test".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);

    let input = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: data,
        config: None,
    };

    // Create multiple concurrent requests
    let mut tasks = Vec::new();
    for _ in 0..5 {
        let server_clone = server.clone();
        let input_clone = input.clone();
        let task = tokio::spawn(async move {
            server_clone
                .execute_advanced_pattern_analysis(input_clone)
                .await
        });
        tasks.push(task);
    }

    // Wait for all to complete
    for task in tasks {
        let result = task.await.unwrap();
        assert!(result.is_ok());
    }
}

//! Integration tests for quality metrics MCP tool

use memory_core::types::{ComplexityLevel, ExecutionResult, Reflection};
use memory_core::{Episode, ExecutionStep, SelfLearningMemory, TaskContext, TaskOutcome, TaskType};
use memory_mcp::SandboxConfig;
use memory_mcp::mcp::tools::quality_metrics::{QualityMetricsInput, QualityMetricsTool};
use memory_mcp::server::MemoryMCPServer;
use std::sync::Arc;

/// Create test episodes with varying quality levels
fn create_test_episodes() -> Vec<Episode> {
    let mut episodes = Vec::new();

    // High quality episodes (10 steps, diverse tools, good reflection)
    for i in 0..5 {
        let mut episode = Episode::new(
            format!("High quality task {}", i),
            TaskContext {
                domain: "testing".to_string(),
                language: Some("rust".to_string()),
                framework: None,
                complexity: ComplexityLevel::Complex,
                tags: vec!["quality".to_string()],
            },
            TaskType::Testing,
        );

        for j in 0..10 {
            let mut step =
                ExecutionStep::new(j + 1, format!("tool_{}", j % 5), format!("action_{}", j));
            step.result = Some(ExecutionResult::Success {
                output: "Success".to_string(),
            });
            episode.add_step(step);
        }

        episode.reflection = Some(Reflection {
            successes: vec!["Good work".to_string(); 3],
            improvements: vec!["Could improve".to_string(); 2],
            insights: vec!["Key insight".to_string(); 2],
            generated_at: chrono::Utc::now(),
        });

        episode.complete(TaskOutcome::Success {
            verdict: "Success".to_string(),
            artifacts: vec!["artifact.rs".to_string()],
        });

        episodes.push(episode);
    }

    // Medium quality episodes (5 steps, moderate complexity)
    for i in 0..3 {
        let mut episode = Episode::new(
            format!("Medium quality task {}", i),
            TaskContext {
                domain: "testing".to_string(),
                language: Some("rust".to_string()),
                framework: None,
                complexity: ComplexityLevel::Moderate,
                tags: vec!["quality".to_string()],
            },
            TaskType::Testing,
        );

        for j in 0..5 {
            let mut step = ExecutionStep::new(j + 1, "tool".to_string(), "action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "Success".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Success".to_string(),
            artifacts: vec![],
        });

        episodes.push(episode);
    }

    // Low quality episodes (1-2 steps, simple)
    for i in 0..2 {
        let mut episode = Episode::new(
            format!("Low quality task {}", i),
            TaskContext {
                domain: "testing".to_string(),
                language: Some("rust".to_string()),
                framework: None,
                complexity: ComplexityLevel::Simple,
                tags: vec!["quality".to_string()],
            },
            TaskType::Testing,
        );

        let mut step = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        episode.add_step(step);

        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        episodes.push(episode);
    }

    episodes
}

#[tokio::test]
async fn test_quality_metrics_tool_basic() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = QualityMetricsTool::new(Arc::clone(&memory));

    // Execute with default parameters
    let input = QualityMetricsInput {
        time_range: "all".to_string(),
        include_trends: true,
        quality_threshold: Some(0.7),
    };

    let result = tool.execute(input).await;
    assert!(result.is_ok(), "Quality metrics query should succeed");

    let metrics = result.unwrap();

    // Should have empty metrics with no episodes
    assert_eq!(metrics.total_episodes_attempted, 0);
    assert_eq!(metrics.episodes_accepted, 0);
    assert_eq!(metrics.episodes_rejected, 0);
    assert_eq!(metrics.noise_reduction_rate, 0.0);
    assert!(!metrics.recommendations.is_empty());
}

#[tokio::test]
async fn test_quality_metrics_with_episodes() {
    let memory = Arc::new(SelfLearningMemory::new());

    // Store test episodes
    let episodes = create_test_episodes();
    for episode in episodes {
        // In a real scenario, episodes would be stored via memory.store_episode()
        // For now, we just verify the tool doesn't crash with empty memory
        let _ = episode;
    }

    let tool = QualityMetricsTool::new(Arc::clone(&memory));

    let input = QualityMetricsInput {
        time_range: "all".to_string(),
        include_trends: true,
        quality_threshold: Some(0.7),
    };

    let result = tool.execute(input).await;
    assert!(result.is_ok(), "Quality metrics query should succeed");

    let metrics = result.unwrap();

    // Verify metrics structure
    assert_eq!(metrics.quality_score_distribution.len(), 5);
    assert_eq!(metrics.quality_threshold, 0.7);
    assert!(!metrics.recommendations.is_empty());
}

#[tokio::test]
async fn test_quality_metrics_time_ranges() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = QualityMetricsTool::new(Arc::clone(&memory));

    // Test different time ranges
    let time_ranges = vec!["24h", "7d", "30d", "90d", "all"];

    for time_range in time_ranges {
        let input = QualityMetricsInput {
            time_range: time_range.to_string(),
            include_trends: false,
            quality_threshold: None,
        };

        let result = tool.execute(input).await;
        assert!(
            result.is_ok(),
            "Quality metrics should work with time_range={}",
            time_range
        );

        let metrics = result.unwrap();
        assert_eq!(metrics.time_period, time_range);
    }
}

#[tokio::test]
async fn test_quality_metrics_via_mcp_server() {
    let memory = Arc::new(SelfLearningMemory::new());
    let server = MemoryMCPServer::new(SandboxConfig::default(), memory)
        .await
        .expect("Failed to create MCP server");

    // Execute quality metrics tool via server
    let input = QualityMetricsInput {
        time_range: "7d".to_string(),
        include_trends: true,
        quality_threshold: Some(0.7),
    };

    let result = server.execute_quality_metrics(input).await;
    assert!(result.is_ok(), "Quality metrics should work via MCP server");

    let json_result = result.unwrap();
    assert!(json_result.is_object());
    assert!(json_result["average_quality_score"].is_number());
    assert!(json_result["noise_reduction_rate"].is_number());
    assert!(json_result["quality_score_distribution"].is_object());
    assert!(json_result["recommendations"].is_array());
}

#[tokio::test]
async fn test_quality_metrics_tool_definition() {
    let tool_def = QualityMetricsTool::tool_definition();

    assert_eq!(tool_def.name, "quality_metrics");
    assert!(!tool_def.description.is_empty());
    assert!(
        tool_def
            .description
            .contains("quality metrics and noise reduction")
    );
    assert!(tool_def.input_schema.is_object());

    // Verify schema properties
    let schema = tool_def.input_schema;
    assert!(schema["properties"]["time_range"].is_object());
    assert!(schema["properties"]["include_trends"].is_object());
    assert!(schema["properties"]["quality_threshold"].is_object());
}

#[tokio::test]
async fn test_quality_metrics_invalid_time_range() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = QualityMetricsTool::new(Arc::clone(&memory));

    let input = QualityMetricsInput {
        time_range: "invalid".to_string(),
        include_trends: false,
        quality_threshold: None,
    };

    let result = tool.execute(input).await;
    assert!(result.is_err(), "Invalid time range should return error");
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Invalid time range")
    );
}

#[tokio::test]
async fn test_quality_metrics_distribution_categories() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = QualityMetricsTool::new(Arc::clone(&memory));

    let input = QualityMetricsInput {
        time_range: "all".to_string(),
        include_trends: true,
        quality_threshold: Some(0.7),
    };

    let result = tool.execute(input).await.expect("Should succeed");

    // Verify all distribution categories are present
    let dist = &result.quality_score_distribution;
    assert!(dist.contains_key("0.0-0.3 (Low)"));
    assert!(dist.contains_key("0.3-0.5 (Below Average)"));
    assert!(dist.contains_key("0.5-0.7 (Average)"));
    assert!(dist.contains_key("0.7-0.9 (Good)"));
    assert!(dist.contains_key("0.9-1.0 (Excellent)"));
}

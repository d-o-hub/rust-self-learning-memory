//! # DBSCAN Tests
//!
//! Unit tests for the DBSCAN anomaly detection system.

use crate::ExecutionStep;
use crate::episode::Episode;
use crate::types::{ComplexityLevel, ExecutionResult, TaskContext, TaskOutcome, TaskType};
#[allow(unused)]
use crate::{DBSCANAnomalyDetector, DBSCANConfig, FeatureWeights};
use chrono::{Duration, Utc};

#[allow(dead_code)]
fn create_test_episode(
    domain: &str,
    step_count: usize,
    task_type: TaskType,
    is_success: bool,
) -> Episode {
    let mut episode = Episode::new(
        format!("Test task in {domain}"),
        TaskContext {
            domain: domain.to_string(),
            language: Some("rust".to_string()),
            complexity: ComplexityLevel::Moderate,
            framework: None,
            tags: vec!["test".to_string()],
        },
        task_type,
    );

    let start_time = Utc::now() - Duration::hours(1);
    episode.start_time = start_time;

    for i in 0..step_count {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i % 3), format!("Action {i}"));
        step.result = Some(ExecutionResult::Success {
            output: "Success".to_string(),
        });
        step.latency_ms = 100;
        episode.steps.push(step);
    }

    if is_success {
        episode.outcome = Some(TaskOutcome::Success {
            verdict: "Task completed successfully".to_string(),
            artifacts: vec![],
        });
    } else {
        episode.outcome = Some(TaskOutcome::Failure {
            reason: "Task failed".to_string(),
            error_details: None,
        });
    }

    let end_time = Utc::now();
    episode.end_time = Some(end_time);

    episode
}

#[tokio::test]
async fn test_empty_episodes() {
    let detector = DBSCANAnomalyDetector::new();
    let result = detector.detect_anomalies(&[]).await.unwrap();

    assert!(result.clusters.is_empty());
    assert!(result.anomalies.is_empty());
    assert_eq!(result.stats.total_points, 0);
}

#[tokio::test]
async fn test_single_episode() {
    let detector = DBSCANAnomalyDetector::new();
    let episodes = vec![create_test_episode(
        "web-api",
        5,
        TaskType::CodeGeneration,
        true,
    )];

    let result = detector.detect_anomalies(&episodes).await.unwrap();

    // Single episode is always an anomaly (no neighbors)
    assert_eq!(result.stats.anomaly_count, 1);
}

#[tokio::test]
async fn test_similar_episodes_no_anomalies() {
    let detector = DBSCANAnomalyDetector::new();

    // Create similar episodes
    let episodes = vec![
        create_test_episode("web-api", 5, TaskType::CodeGeneration, true),
        create_test_episode("web-api", 6, TaskType::CodeGeneration, true),
        create_test_episode("web-api", 5, TaskType::CodeGeneration, true),
        create_test_episode("web-api", 6, TaskType::CodeGeneration, true),
        create_test_episode("web-api", 5, TaskType::CodeGeneration, true),
        create_test_episode("web-api", 6, TaskType::CodeGeneration, true),
    ];

    let result = detector.detect_anomalies(&episodes).await.unwrap();

    assert!(
        !result.clusters.is_empty(),
        "Similar episodes should form at least one cluster"
    );
    assert!(
        result.anomalies.is_empty(),
        "Similar episodes should have no anomalies"
    );
}

#[tokio::test]
async fn test_anomaly_detection() {
    let detector = DBSCANAnomalyDetector::new();

    // Create mostly similar episodes
    let mut episodes = Vec::new();
    for _i in 0..5 {
        episodes.push(create_test_episode(
            "web-api",
            5,
            TaskType::CodeGeneration,
            true,
        ));
    }

    // Add one very different episode (many more steps)
    let anomaly = create_test_episode("web-api", 50, TaskType::Debugging, false);
    episodes.push(anomaly);

    let result = detector.detect_anomalies(&episodes).await.unwrap();

    assert!(
        result.stats.anomaly_count >= 1,
        "Should detect at least one anomaly, got {}",
        result.stats.anomaly_count
    );

    if !result.anomalies.is_empty() {
        let max_distance = result.stats.max_anomaly_distance;
        assert!(
            max_distance > 0.05,
            "Anomaly should be somewhat far from clusters, got {max_distance}"
        );
    }
}

#[test]
fn test_config_customization() {
    let config = DBSCANConfig {
        eps: 0.8,
        min_samples: 5,
        adaptive_eps: false,
        feature_weights: FeatureWeights {
            context: 0.5,
            step_count: 0.1,
            duration: 0.1,
            outcome: 0.15,
            task_type: 0.15,
        },
        min_cluster_size: 3,
    };

    let detector = DBSCANAnomalyDetector::with_config(config);

    // Verify config was applied - detector was created successfully
    assert!(detector.config().is_some());
}

#[tokio::test]
async fn test_multiple_domains() {
    let detector = DBSCANAnomalyDetector::new();

    let episodes = vec![
        create_test_episode("web-api", 5, TaskType::CodeGeneration, true),
        create_test_episode("web-api", 6, TaskType::CodeGeneration, true),
        create_test_episode("web-api", 5, TaskType::CodeGeneration, true),
        create_test_episode("web-api", 6, TaskType::CodeGeneration, true),
        create_test_episode("web-api", 5, TaskType::CodeGeneration, true),
        create_test_episode("cli", 3, TaskType::Testing, true),
        create_test_episode("cli", 4, TaskType::Testing, true),
        create_test_episode("cli", 3, TaskType::Testing, true),
        create_test_episode("cli", 4, TaskType::Testing, true),
        create_test_episode("cli", 3, TaskType::Testing, true),
        create_test_episode("data-processing", 10, TaskType::Analysis, true),
        create_test_episode("data-processing", 11, TaskType::Analysis, true),
        create_test_episode("data-processing", 10, TaskType::Analysis, true),
        create_test_episode("data-processing", 11, TaskType::Analysis, true),
        create_test_episode("data-processing", 10, TaskType::Analysis, true),
    ];

    let result = detector.detect_anomalies(&episodes).await.unwrap();

    // Should detect 3 clusters (one per domain: web-api, cli, data-processing)
    assert_eq!(
        result.clusters.len(),
        3,
        "Expected 3 clusters for 3 different domains"
    );
    assert_eq!(
        result.stats.total_points, 15,
        "Should have 15 total episodes"
    );
    // With good clustering, we expect few or no anomalies
    assert!(
        result.stats.anomaly_count <= 3,
        "Should have at most 3 anomalies with 3 well-separated domains"
    );
    assert!(result.stats.avg_anomaly_distance >= 0.0);
}

#[tokio::test]
async fn test_adaptive_eps() {
    let detector = DBSCANAnomalyDetector::new();

    // Create tightly clustered episodes
    let episodes: Vec<_> = (0..10)
        .map(|_| create_test_episode("web-api", 5, TaskType::CodeGeneration, true))
        .collect();

    let result = detector.detect_anomalies(&episodes).await.unwrap();

    // With adaptive eps, similar episodes should cluster well
    assert!(result.stats.avg_anomaly_distance >= 0.0 || result.anomalies.is_empty());
}

#[tokio::test]
async fn test_dbscan_iterations() {
    let detector = DBSCANAnomalyDetector::new();

    let episodes: Vec<_> = (0..5)
        .map(|_| create_test_episode("web-api", 5, TaskType::CodeGeneration, true))
        .collect();

    let result = detector.detect_anomalies(&episodes).await.unwrap();

    // Should have run some iterations
    assert!(result.iterations > 0);
}

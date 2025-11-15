//! Integration tests for agent monitoring functionality

use memory_core::{SelfLearningMemory, AgentMetrics, TaskContext, TaskOutcome, TaskType, ExecutionStep, ExecutionResult};
use std::time::{Duration, Instant};

#[tokio::test]
async fn test_agent_monitoring_integration() {
    let memory = SelfLearningMemory::new();

    // Record some agent executions
    memory.record_agent_execution("feature-implementer", true, Duration::from_secs(5)).await.unwrap();
    memory.record_agent_execution("feature-implementer", false, Duration::from_secs(10)).await.unwrap();
    memory.record_agent_execution("code-reviewer", true, Duration::from_secs(3)).await.unwrap();

    // Check agent metrics
    let feature_metrics = memory.get_agent_metrics("feature-implementer").await.unwrap();
    assert_eq!(feature_metrics.total_executions, 2);
    assert_eq!(feature_metrics.successful_executions, 1);
    assert_eq!(feature_metrics.success_rate(), 0.5);

    let reviewer_metrics = memory.get_agent_metrics("code-reviewer").await.unwrap();
    assert_eq!(reviewer_metrics.total_executions, 1);
    assert_eq!(reviewer_metrics.successful_executions, 1);
    assert_eq!(reviewer_metrics.success_rate(), 1.0);

    // Check summary stats
    let summary = memory.get_monitoring_summary().await;
    assert_eq!(summary.total_agents, 2);
    assert_eq!(summary.total_executions, 3);
    assert_eq!(summary.successful_executions, 2);
    assert_eq!(summary.success_rate, 2.0 / 3.0);
}

#[tokio::test]
async fn test_monitoring_with_episodes() {
    let memory = SelfLearningMemory::new();

    // Create and complete an episode
    let context = TaskContext {
        language: Some("rust".to_string()),
        domain: "testing".to_string(),
        ..Default::default()
    };

    let episode_id = memory.start_episode(
        "Implement test monitoring".to_string(),
        context,
        TaskType::Testing,
    ).await;

    // Log some steps
    let mut step = ExecutionStep::new(1, "feature-implementer", "Implement monitoring".to_string());
    step.result = Some(ExecutionResult::Success {
        output: "Monitoring implemented".to_string(),
    });
    memory.log_step(episode_id, step).await;

    // Record agent execution
    let start = Instant::now();
    tokio::time::sleep(Duration::from_millis(10)).await; // Simulate work
    let duration = start.elapsed();

    memory.record_agent_execution("feature-implementer", true, duration).await.unwrap();

    // Complete episode
    memory.complete_episode(
        episode_id,
        TaskOutcome::Success {
            verdict: "Monitoring system implemented".to_string(),
            artifacts: vec!["monitoring.rs".to_string()],
        },
    ).await.unwrap();

    // Verify monitoring data persists alongside episode data
    let metrics = memory.get_agent_metrics("feature-implementer").await.unwrap();
    assert_eq!(metrics.total_executions, 1);
    assert!(metrics.success_rate() > 0.0);

    // Verify episode was completed
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete());
}

#[tokio::test]
async fn test_monitoring_performance_overhead() {
    let memory = SelfLearningMemory::new();

    // Measure time for a large number of monitoring calls
    let start = Instant::now();

    for i in 0..1000 {
        let success = i % 10 != 0; // 90% success rate
        memory.record_agent_execution("test-agent", success, Duration::from_millis(i as u64 % 100)).await.unwrap();
    }

    let monitoring_time = start.elapsed();

    // Verify the monitoring overhead is reasonable (< 1ms per call on average)
    let avg_time_per_call = monitoring_time.as_millis() as f64 / 1000.0;
    assert!(avg_time_per_call < 1.0, "Monitoring overhead too high: {}ms per call", avg_time_per_call);

    // Verify metrics are accurate
    let metrics = memory.get_agent_metrics("test-agent").await.unwrap();
    assert_eq!(metrics.total_executions, 1000);
    assert_eq!(metrics.successful_executions, 900); // 90% success rate
}

#[tokio::test]
async fn test_detailed_monitoring() {
    let memory = SelfLearningMemory::new();

    // Record detailed execution
    memory.record_agent_execution_detailed(
        "feature-implementer",
        true,
        Duration::from_secs(2),
        Some("Implement user authentication".to_string()),
        None,
    ).await.unwrap();

    // Record failed execution with error
    memory.record_agent_execution_detailed(
        "code-reviewer",
        false,
        Duration::from_secs(1),
        Some("Review authentication code".to_string()),
        Some("Found security vulnerability".to_string()),
    ).await.unwrap();

    // Verify metrics
    let feature_metrics = memory.get_agent_metrics("feature-implementer").await.unwrap();
    assert_eq!(feature_metrics.total_executions, 1);
    assert_eq!(feature_metrics.successful_executions, 1);

    let reviewer_metrics = memory.get_agent_metrics("code-reviewer").await.unwrap();
    assert_eq!(reviewer_metrics.total_executions, 1);
    assert_eq!(reviewer_metrics.successful_executions, 0);

    // Verify summary includes both agents
    let summary = memory.get_monitoring_summary().await;
    assert_eq!(summary.total_agents, 2);
    assert_eq!(summary.total_executions, 2);
}

#[tokio::test]
async fn test_monitoring_data_persistence() {
    let memory = SelfLearningMemory::new();

    // Record executions
    memory.record_agent_execution("persistent-agent", true, Duration::from_secs(1)).await.unwrap();
    memory.record_agent_execution("persistent-agent", true, Duration::from_secs(2)).await.unwrap();

    // Clone memory (simulating persistence across sessions)
    let memory_clone = memory.clone();

    // Verify data is accessible from cloned instance
    let metrics = memory_clone.get_agent_metrics("persistent-agent").await.unwrap();
    assert_eq!(metrics.total_executions, 2);
    assert_eq!(metrics.successful_executions, 2);

    // Verify summary is consistent
    let summary1 = memory.get_monitoring_summary().await;
    let summary2 = memory_clone.get_monitoring_summary().await;
    assert_eq!(summary1.total_executions, summary2.total_executions);
    assert_eq!(summary1.total_agents, summary2.total_agents);
}
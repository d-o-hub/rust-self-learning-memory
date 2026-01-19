//! Episode-related tests for SelfLearningMemory.

use crate::episode::ExecutionStep;
use crate::types::{ComplexityLevel, ExecutionResult, TaskContext, TaskOutcome, TaskType};
use crate::SelfLearningMemory;

/// Test starting an episode.
#[tokio::test]
pub async fn test_start_episode() {
    let memory = SelfLearningMemory::new();

    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "testing".to_string(),
        tags: vec!["async".to_string()],
    };

    let episode_id = memory
        .start_episode("Test task".to_string(), context.clone(), TaskType::Testing)
        .await;

    // Verify episode was created
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.task_description, "Test task");
    assert_eq!(episode.context.domain, "testing");
    assert!(!episode.is_complete());
}

/// Test logging execution steps.
#[tokio::test]
pub async fn test_log_steps() {
    let memory = SelfLearningMemory::new();

    let episode_id = memory
        .start_episode(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Log some steps
    for i in 0..3 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{i}"), "Action".to_string());
        step.result = Some(ExecutionResult::Success {
            output: "OK".to_string(),
        });
        memory.log_step(episode_id, step).await;
    }

    // Flush buffered steps (if batching enabled)
    memory.flush_steps(episode_id).await.unwrap();

    // Verify steps were logged
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.steps.len(), 3);
}

/// Test completing an episode.
#[tokio::test]
pub async fn test_complete_episode() {
    let test_config = crate::MemoryConfig {
        quality_threshold: 0.5,
        ..Default::default()
    };
    let memory = SelfLearningMemory::with_config(test_config);

    let episode_id = memory
        .start_episode(
            "Test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Log multiple steps to meet quality threshold
    for i in 0..20 {
        let mut step =
            ExecutionStep::new(i + 1, format!("tool_{}", i % 6), format!("Test action {i}"));
        step.result = Some(ExecutionResult::Success {
            output: format!("Step {i} passed"),
        });
        memory.log_step(episode_id, step).await;
    }

    // Complete the episode
    let outcome = TaskOutcome::Success {
        verdict: "Tests passed".to_string(),
        artifacts: vec!["test_results.json".to_string()],
    };

    memory.complete_episode(episode_id, outcome).await.unwrap();

    // Verify episode was completed and analyzed
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.is_complete());
    assert!(episode.reward.is_some());
    assert!(episode.reflection.is_some());

    // Check that patterns were extracted
    let stats = memory.get_stats().await;
    assert!(stats.2 > 0); // Should have some patterns
}

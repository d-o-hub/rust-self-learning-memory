//! Lazy loading tests for `SelfLearningMemory`.

use crate::episode::ExecutionStep;
use crate::types::{ExecutionResult, TaskContext, TaskOutcome, TaskType};
use crate::SelfLearningMemory;

/// Test `get_all_episodes` with lazy loading.
#[tokio::test]
#[ignore = "Slow test - complete_episode with pattern extraction takes too long in CI"]
pub async fn test_get_all_episodes_lazy_loading() {
    let test_config = crate::MemoryConfig {
        quality_threshold: 0.5,
        ..Default::default()
    };
    let memory = SelfLearningMemory::with_config(test_config);

    // Create a few episodes
    let episode_id1 = memory
        .start_episode(
            "Test task 1".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    let _episode_id2 = memory
        .start_episode(
            "Test task 2".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        )
        .await;

    // Add steps to meet quality threshold
    for i in 0..20 {
        let mut step =
            ExecutionStep::new(i + 1, format!("tool_{}", i % 6), format!("Test step {i}"));
        step.result = Some(ExecutionResult::Success {
            output: "Success".to_string(),
        });
        memory.log_step(episode_id1, step).await;
    }

    // Complete one episode
    memory
        .complete_episode(
            episode_id1,
            TaskOutcome::Success {
                verdict: "Task completed".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Test get_all_episodes
    let all_episodes = memory.get_all_episodes().await.unwrap();
    assert_eq!(all_episodes.len(), 2, "Should return all episodes");

    // Test list_episodes with filters
    let all_episodes_list = memory.list_episodes(None, None, None).await.unwrap();
    assert_eq!(all_episodes_list.len(), 2, "Should list all episodes");

    let completed_episodes = memory.list_episodes(None, None, Some(true)).await.unwrap();
    assert_eq!(
        completed_episodes.len(),
        1,
        "Should return only completed episodes"
    );

    let limited_episodes = memory.list_episodes(Some(1), None, None).await.unwrap();
    assert_eq!(limited_episodes.len(), 1, "Should respect limit");

    // Test that episodes are sorted by start_time (newest first)
    let mut episodes_by_time = all_episodes_list.clone();
    episodes_by_time.sort_by(|a, b| b.start_time.cmp(&a.start_time));
    assert_eq!(
        all_episodes_list, episodes_by_time,
        "Episodes should be sorted by start_time (newest first)"
    );
}

/// Test `get_episode` with lazy loading.
#[tokio::test]
pub async fn test_get_episode_lazy_loading() {
    let memory = SelfLearningMemory::new();

    // Create an episode
    let episode_id = memory
        .start_episode(
            "Test lazy loading".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Get episode should work from in-memory
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.task_description, "Test lazy loading");

    // Note: In test environment without storage backends,
    // lazy loading fallback won't work since episodes aren't persisted
    // This test mainly verifies the method doesn't panic
    // and works correctly when episode is in memory

    // Verify episode is in in-memory cache
    {
        let episodes = memory.episodes_fallback.read().await;
        assert!(
            episodes.contains_key(&episode_id),
            "Episode should be in memory cache"
        );
    }

    // The existing get_episode method with lazy loading should work
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.task_description, "Test lazy loading");
}

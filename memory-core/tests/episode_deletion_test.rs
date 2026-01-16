//! Integration tests for episode deletion functionality

use memory_core::{ExecutionResult, ExecutionStep, SelfLearningMemory, TaskContext, TaskType};
use uuid::Uuid;

#[tokio::test]
async fn test_delete_episode_success() {
    let memory = SelfLearningMemory::new();

    // Create an episode
    let episode_id = memory
        .start_episode(
            "Test task for deletion".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Verify episode exists
    let episode = memory.get_episode(episode_id).await;
    assert!(episode.is_ok());

    // Delete the episode
    let result = memory.delete_episode(episode_id).await;
    assert!(result.is_ok());

    // Verify episode is gone
    let episode = memory.get_episode(episode_id).await;
    assert!(episode.is_err());
}

#[tokio::test]
async fn test_delete_nonexistent_episode() {
    let memory = SelfLearningMemory::new();

    // Try to delete non-existent episode
    let fake_id = Uuid::new_v4();
    let result = memory.delete_episode(fake_id).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_episode_with_steps() {
    let memory = SelfLearningMemory::new();

    // Create an episode with steps
    let episode_id = memory
        .start_episode(
            "Episode with steps".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Add some steps
    for i in 1..=5 {
        let mut step = ExecutionStep::new(i, format!("tool_{i}"), format!("Action {i}"));
        step.result = Some(ExecutionResult::Success {
            output: format!("Output {i}"),
        });
        memory.log_step(episode_id, step).await;
    }

    // Flush steps if batching is enabled
    let _ = memory.flush_steps(episode_id).await;

    // Verify episode has steps
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert_eq!(episode.steps.len(), 5);

    // Delete the episode
    let result = memory.delete_episode(episode_id).await;
    assert!(result.is_ok());

    // Verify episode is gone
    let episode = memory.get_episode(episode_id).await;
    assert!(episode.is_err());
}

#[tokio::test]
async fn test_delete_episode_idempotency() {
    let memory = SelfLearningMemory::new();

    // Create and delete an episode
    let episode_id = memory
        .start_episode(
            "Test idempotency".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    memory.delete_episode(episode_id).await.unwrap();

    // Try to delete again - should fail
    let result = memory.delete_episode(episode_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_multiple_episodes() {
    let memory = SelfLearningMemory::new();

    // Create multiple episodes
    let mut episode_ids = Vec::new();
    for i in 0..5 {
        let id = memory
            .start_episode(
                format!("Episode {i}"),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;
        episode_ids.push(id);
    }

    // Delete all episodes
    for id in &episode_ids {
        let result = memory.delete_episode(*id).await;
        assert!(result.is_ok());
    }

    // Verify all are gone
    for id in &episode_ids {
        let result = memory.get_episode(*id).await;
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn test_delete_episode_does_not_affect_others() {
    let memory = SelfLearningMemory::new();

    // Create multiple episodes
    let id1 = memory
        .start_episode(
            "Episode 1".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    let id2 = memory
        .start_episode(
            "Episode 2".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    let id3 = memory
        .start_episode(
            "Episode 3".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Delete only the middle one
    memory.delete_episode(id2).await.unwrap();

    // Verify id1 and id3 still exist
    assert!(memory.get_episode(id1).await.is_ok());
    assert!(memory.get_episode(id2).await.is_err());
    assert!(memory.get_episode(id3).await.is_ok());
}

#[tokio::test]
async fn test_archive_episode_functionality() {
    let memory = SelfLearningMemory::new();

    let episode_id = memory
        .start_episode(
            "Test archive".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Archive should succeed (episode doesn't need to be completed)
    let result = memory.archive_episode(episode_id).await;
    assert!(result.is_ok());

    // Episode should still exist but be marked as archived
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(episode.metadata.contains_key("archived_at"));
}

#[tokio::test]
async fn test_restore_episode_functionality() {
    let memory = SelfLearningMemory::new();

    let episode_id = memory
        .start_episode(
            "Test restore".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Archive then restore (episode doesn't need to be completed)
    memory.archive_episode(episode_id).await.unwrap();

    // Restore should succeed
    let result = memory.restore_episode(episode_id).await;
    assert!(result.is_ok());

    // Episode should no longer be marked as archived
    let episode = memory.get_episode(episode_id).await.unwrap();
    assert!(!episode.metadata.contains_key("archived_at"));
}

#[tokio::test]
async fn test_restore_nonexistent_episode() {
    let memory = SelfLearningMemory::new();

    let episode_id = Uuid::new_v4();

    // Restore should return not found error for non-existent episode
    let result = memory.restore_episode(episode_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_restore_non_archived_episode() {
    let memory = SelfLearningMemory::new();

    let episode_id = memory
        .start_episode(
            "Test restore without archive".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // Restore should fail since episode is not archived
    let result = memory.restore_episode(episode_id).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not archived"));
}

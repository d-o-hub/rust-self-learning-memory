//! Integration tests for episode deletion functionality

use do_memory_core::{
    BatchConfig, Error, ExecutionResult, ExecutionStep, MemoryConfig, SelfLearningMemory,
    TaskContext, TaskOutcome, TaskType,
};
use uuid::Uuid;

/// Batching on so steps sit in the per-episode buffer instead of storage.
fn batching_memory() -> SelfLearningMemory {
    SelfLearningMemory::with_config(MemoryConfig {
        batch_config: Some(BatchConfig::manual_only()),
        ..Default::default()
    })
}

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

/// Deleting an episode must also discard steps that never reached storage.
/// A surviving buffer would let a later flush write a deleted episode back.
#[tokio::test]
async fn test_delete_episode_with_unflushed_buffered_steps_stays_deleted() {
    let memory = batching_memory();
    let episode_id = memory
        .start_episode(
            "Episode deleted before flush".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    for i in 1..=3 {
        memory
            .log_step(
                episode_id,
                ExecutionStep::new(i, "tool".to_string(), format!("Buffered {i}")),
            )
            .await;
    }
    // Steps stay buffered: flush_steps is deliberately not called.
    memory.delete_episode(episode_id).await.unwrap();

    let flush = memory.flush_steps(episode_id).await;
    assert!(flush.is_ok(), "flushing a deleted episode must not error");
    assert!(
        memory.get_episode(episode_id).await.is_err(),
        "flush must not resurrect a deleted episode from buffered steps"
    );

    let completion = memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: Vec::new(),
            },
        )
        .await;
    assert!(
        matches!(completion, Err(Error::NotFound(id)) if id == episode_id),
        "completing a deleted episode must fail with NotFound, got {completion:?}"
    );
    assert!(memory.get_episode(episode_id).await.is_err());
}

/// Late steps for a deleted episode are dropped in both buffered and
/// immediate-write modes instead of recreating the record.
#[tokio::test]
async fn test_log_step_after_delete_does_not_recreate_episode() {
    for memory in [
        batching_memory(),
        SelfLearningMemory::with_config(MemoryConfig {
            batch_config: None,
            ..Default::default()
        }),
    ] {
        let episode_id = memory
            .start_episode(
                "Late step after delete".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;
        memory.delete_episode(episode_id).await.unwrap();

        memory
            .log_step(
                episode_id,
                ExecutionStep::new(1, "tool".to_string(), "Late".to_string()),
            )
            .await;
        memory.flush_steps(episode_id).await.unwrap();

        assert!(
            memory.get_episode(episode_id).await.is_err(),
            "a step logged after deletion must not recreate the episode"
        );
    }
}

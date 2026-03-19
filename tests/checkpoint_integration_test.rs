use memory_core::memory::SelfLearningMemory;
use memory_core::memory::checkpoint::{checkpoint_episode, get_handoff_pack, resume_from_handoff};
use memory_core::{ExecutionStep, MemoryConfig, TaskContext, TaskType};

#[tokio::test]
async fn test_checkpoint_handoff_flow() {
    // Disable batching to ensure steps are persisted immediately for the test
    let mut config = MemoryConfig::default();
    config.batch_config = None;
    let memory = SelfLearningMemory::with_config(config);

    // 1. Start episode and log some steps
    let episode_id = memory
        .start_episode(
            "Checkpoint test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    memory
        .log_step(
            episode_id,
            ExecutionStep::new(1, "tool1".to_string(), "action1".to_string()),
        )
        .await;

    // 2. Create checkpoint
    let checkpoint = checkpoint_episode(&memory, episode_id, "Testing handoff".to_string())
        .await
        .expect("Create checkpoint");

    // 3. Get handoff pack
    let handoff = get_handoff_pack(&memory, checkpoint.checkpoint_id)
        .await
        .expect("Get handoff pack");

    assert_eq!(handoff.episode_id, episode_id);
    assert_eq!(handoff.steps_completed.len(), 1);

    // 4. Resume from handoff
    let new_episode_id = resume_from_handoff(&memory, handoff)
        .await
        .expect("Resume from handoff");

    assert_ne!(new_episode_id, episode_id);

    // 5. Verify new episode has context
    let new_episode = memory
        .get_episode(new_episode_id)
        .await
        .expect("Get new episode");
    assert!(
        new_episode
            .task_description
            .contains("Checkpoint test task")
    );
}

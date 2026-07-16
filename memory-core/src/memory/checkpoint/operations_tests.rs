use super::*;
use crate::types::{TaskContext, TaskType};

#[tokio::test]
async fn test_checkpoint_episode_not_found() {
    let memory = SelfLearningMemory::new();
    let result = checkpoint_episode(&memory, Uuid::new_v4(), "test".to_string()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_handoff_pack_not_found() {
    let memory = SelfLearningMemory::new();
    let result = get_handoff_pack(&memory, Uuid::new_v4()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_checkpoint_completed_episode() {
    use crate::episode::ExecutionStep;
    use crate::memory::MemoryConfig;
    use crate::types::ExecutionResult;

    let test_config = MemoryConfig {
        quality_threshold: 0.3,
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

    let mut step = ExecutionStep::new(1, "test_tool".to_string(), "test action".to_string());
    step.result = Some(ExecutionResult::Success {
        output: "test output".to_string(),
    });
    memory.log_step(episode_id, step).await;

    memory
        .complete_episode(
            episode_id,
            crate::types::TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    let result = checkpoint_episode(&memory, episode_id, "test".to_string()).await;
    assert!(result.is_err());
}

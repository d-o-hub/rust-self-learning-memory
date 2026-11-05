//! Resource-limit enforcement through the production episode API.

use do_memory_core::memory::step_buffer::BatchConfig;
use do_memory_core::types::{MAX_ARTIFACT_SIZE, MAX_EPISODE_SIZE, MAX_OBSERVATION_LEN};
use do_memory_core::{
    Error, ExecutionResult, ExecutionStep, MemoryConfig, SelfLearningMemory, TaskContext,
    TaskOutcome, TaskType,
};

#[tokio::test]
async fn test_log_step_oversized_observation_rejected_before_buffering() -> anyhow::Result<()> {
    for batch_config in [None, Some(BatchConfig::manual_only())] {
        let memory = SelfLearningMemory::with_config(MemoryConfig {
            batch_config,
            ..Default::default()
        });
        let episode_id = memory
            .start_episode(
                "Validate observation byte limits".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;
        // Multibyte output must be limited by bytes, not character count.
        let at_limit = "é".repeat(MAX_OBSERVATION_LEN / 2);
        let mut oversized = ExecutionStep::new(1, "tool".to_string(), "Rejected".to_string());
        oversized.result = Some(ExecutionResult::Success {
            output: format!("{at_limit}x"),
        });

        memory.log_step(episode_id, oversized).await;
        memory.flush_steps(episode_id).await?;
        assert_eq!(
            memory.get_episode(episode_id).await?.steps,
            Vec::<ExecutionStep>::new()
        );

        let mut valid = ExecutionStep::new(2, "tool".to_string(), "Accepted".to_string());
        valid.result = Some(ExecutionResult::Success {
            output: at_limit.clone(),
        });
        memory.log_step(episode_id, valid).await;
        memory.flush_steps(episode_id).await?;

        let episode = memory.get_episode(episode_id).await?;
        assert_eq!(episode.steps.len(), 1);
        assert_eq!(episode.steps[0].step_number, 2);
        assert_eq!(
            episode.steps[0].result,
            Some(ExecutionResult::Success { output: at_limit })
        );
    }
    Ok(())
}

#[tokio::test]
async fn test_log_step_oversized_parameters_rejected_without_losing_valid_steps()
-> anyhow::Result<()> {
    let memory = SelfLearningMemory::with_config(MemoryConfig {
        batch_config: Some(BatchConfig::manual_only()),
        ..Default::default()
    });
    let episode_id = memory
        .start_episode(
            "Validate serialized parameter limits".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;
    let mut oversized = ExecutionStep::new(2, "tool".to_string(), "Rejected".to_string());
    oversized.set_parameters(serde_json::json!({ "payload": "x".repeat(MAX_ARTIFACT_SIZE) }));

    memory
        .log_step(
            episode_id,
            ExecutionStep::new(1, "tool".to_string(), "Before".to_string()),
        )
        .await;
    memory.log_step(episode_id, oversized).await;
    memory
        .log_step(
            episode_id,
            ExecutionStep::new(3, "tool".to_string(), "After".to_string()),
        )
        .await;
    memory.flush_steps(episode_id).await?;

    let episode = memory.get_episode(episode_id).await?;
    let step_numbers: Vec<_> = episode.steps.iter().map(|step| step.step_number).collect();
    assert_eq!(step_numbers, vec![1, 3]);
    Ok(())
}

#[tokio::test]
async fn test_complete_episode_oversized_episode_remains_incomplete() -> anyhow::Result<()> {
    let memory = SelfLearningMemory::new();
    let episode_id = memory
        .start_episode(
            "Validate completion size limit".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;
    let mut episode = memory.get_episode(episode_id).await?;
    episode
        .metadata
        .insert("payload".to_string(), "x".repeat(MAX_EPISODE_SIZE));
    memory.update_episode_full(&episode).await?;

    let result = memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: Vec::new(),
            },
        )
        .await;

    assert!(matches!(result, Err(Error::InvalidInput(_))));
    let retained = memory.get_episode(episode_id).await?;
    assert!(!retained.is_complete());
    assert!(retained.outcome.is_none());
    assert!(retained.end_time.is_none());
    assert_eq!(retained.metadata, episode.metadata);
    Ok(())
}

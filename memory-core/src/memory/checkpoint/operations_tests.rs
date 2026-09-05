use super::*;
use crate::memory::checkpoint::resume_from_compact;
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

async fn start_episode_with_steps(
    memory: &SelfLearningMemory,
    task: &str,
    steps: usize,
) -> uuid::Uuid {
    use crate::types::ExecutionResult;

    let episode_id = memory
        .start_episode(task.to_string(), TaskContext::default(), TaskType::Testing)
        .await;
    for i in 1..=steps {
        let mut step = crate::episode::ExecutionStep::new(
            i,
            "test_tool".to_string(),
            format!("test action {i}"),
        );
        step.result = Some(ExecutionResult::Success {
            output: format!("test output {i}"),
        });
        memory.log_step(episode_id, step).await;
    }
    episode_id
}

#[tokio::test]
async fn test_compact_handoff_pack_default_budget() {
    use crate::memory::MemoryConfig;

    let memory = SelfLearningMemory::with_config(MemoryConfig {
        quality_threshold: 0.0,
        batch_config: None,
        ..Default::default()
    });
    let episode_id = start_episode_with_steps(&memory, "Compact the thing", 5).await;
    let checkpoint = checkpoint_episode(&memory, episode_id, "pause".to_string())
        .await
        .unwrap();

    let pack =
        get_compact_handoff_pack(&memory, checkpoint.checkpoint_id, HandoffBudget::default())
            .await
            .unwrap();

    assert_eq!(pack.episode_id, episode_id);
    assert_eq!(pack.current_goal, "Compact the thing");
    assert_eq!(pack.status, "in_progress");
    assert_eq!(pack.steps_done, checkpoint.step_number);
    assert!(pack.payload_bytes() <= HandoffBudget::default().max_bytes);
    assert_eq!(pack.evidence_excerpts.len(), 5);
    assert_eq!(pack.omitted.omitted_steps, 0);
    assert!(!pack.verified_findings.is_empty());
}

#[tokio::test]
async fn test_compact_handoff_tiny_budget_reports_omissions() {
    use crate::memory::MemoryConfig;

    let memory = SelfLearningMemory::with_config(MemoryConfig {
        quality_threshold: 0.0,
        batch_config: None,
        ..Default::default()
    });
    let episode_id = start_episode_with_steps(&memory, "Long task", 20).await;
    let checkpoint = checkpoint_episode(&memory, episode_id, "pause".to_string())
        .await
        .unwrap();
    let budget = HandoffBudget {
        max_bytes: 700,
        max_evidence_excerpts: 2,
        ..HandoffBudget::default()
    };

    let pack = get_compact_handoff_pack(&memory, checkpoint.checkpoint_id, budget.clone())
        .await
        .unwrap();

    assert!(pack.payload_bytes() <= budget.max_bytes);
    assert!(pack.evidence_excerpts.len() <= 2);
    assert!(pack.omitted.omitted_steps >= 18);
    assert_eq!(
        pack.omitted.full_available_via,
        "get_handoff_pack(checkpoint_id)"
    );
}

#[tokio::test]
async fn test_resume_from_compact_preserves_context() {
    use crate::memory::MemoryConfig;

    let memory = SelfLearningMemory::with_config(MemoryConfig {
        quality_threshold: 0.0,
        batch_config: None,
        ..Default::default()
    });
    let episode_id = start_episode_with_steps(&memory, "Resume me", 3).await;
    let checkpoint = checkpoint_episode(&memory, episode_id, "switch".to_string())
        .await
        .unwrap();
    let pack =
        get_compact_handoff_pack(&memory, checkpoint.checkpoint_id, HandoffBudget::default())
            .await
            .unwrap();

    let new_id = resume_from_compact(&memory, pack).await.unwrap();
    let resumed = memory.get_episode(new_id).await.unwrap();

    assert_eq!(resumed.task_description, "Resume me");
    assert_eq!(
        resumed.metadata.get("handoff_format").map(String::as_str),
        Some("compact")
    );
    assert_eq!(
        resumed
            .metadata
            .get("resumed_from_checkpoint")
            .map(String::as_str),
        Some(checkpoint.checkpoint_id.to_string()).as_deref()
    );
    assert!(resumed.metadata.contains_key("compact_pending_actions"));
}

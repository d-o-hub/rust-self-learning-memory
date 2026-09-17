use super::*;
use chrono::Utc;
use do_memory_core::{MemoryConfig, TaskContext, TaskType};

async fn setup_test_episode() -> (SelfLearningMemory, Config, Uuid) {
    let memory = SelfLearningMemory::with_config(MemoryConfig {
        quality_threshold: 0.0,
        batch_config: None,
        ..MemoryConfig::default()
    });
    let config = Config::default();
    let episode_id = memory
        .start_episode(
            "CLI checkpoint task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;
    for i in 1..=3 {
        let step = do_memory_core::ExecutionStep::new(i, "tool".to_string(), format!("action {i}"));
        memory.log_step(episode_id, step).await;
    }
    (memory, config, episode_id)
}

#[tokio::test]
async fn test_cli_checkpoint_and_list() {
    let (memory, config, episode_id) = setup_test_episode().await;

    let res = checkpoint(
        episode_id.to_string(),
        "pause reason".to_string(),
        Some("note context".to_string()),
        &memory,
        &config,
        OutputFormat::Json,
        false,
    )
    .await;
    assert!(res.is_ok());

    let list_res = list_checkpoints(
        episode_id.to_string(),
        &memory,
        &config,
        OutputFormat::Human,
    )
    .await;
    assert!(list_res.is_ok());

    let list_json =
        list_checkpoints(episode_id.to_string(), &memory, &config, OutputFormat::Json).await;
    assert!(list_json.is_ok());

    let list_yaml =
        list_checkpoints(episode_id.to_string(), &memory, &config, OutputFormat::Yaml).await;
    assert!(list_yaml.is_ok());
}

#[tokio::test]
async fn test_cli_handoff_and_resume_compact_and_full() {
    let (memory, config, episode_id) = setup_test_episode().await;

    let cp = checkpoint_episode(&memory, episode_id, "pause".to_string())
        .await
        .unwrap();

    // Test handoff compact and full
    assert!(
        handoff(
            cp.checkpoint_id.to_string(),
            false,
            Some(2048),
            &memory,
            &config,
            OutputFormat::Human,
            false,
        )
        .await
        .is_ok()
    );

    assert!(
        handoff(
            cp.checkpoint_id.to_string(),
            true,
            None,
            &memory,
            &config,
            OutputFormat::Human,
            false,
        )
        .await
        .is_ok()
    );

    // Test resume compact and full
    assert!(
        resume(
            cp.checkpoint_id.to_string(),
            false,
            &memory,
            &config,
            OutputFormat::Human,
            false,
        )
        .await
        .is_ok()
    );

    assert!(
        resume(
            cp.checkpoint_id.to_string(),
            true,
            &memory,
            &config,
            OutputFormat::Human,
            false,
        )
        .await
        .is_ok()
    );
}

#[test]
fn test_checkpoint_result_formats() {
    let cp_res = CheckpointResult {
        checkpoint_id: Uuid::new_v4().to_string(),
        episode_id: Uuid::new_v4().to_string(),
        label: "test label".to_string(),
        step_number: 1,
        timestamp: Utc::now().to_rfc3339(),
        is_abstention: true,
    };
    assert!(cp_res.write(OutputFormat::Human).is_ok());
    assert!(cp_res.write(OutputFormat::Json).is_ok());
    assert!(cp_res.write(OutputFormat::Yaml).is_ok());
}

#[test]
fn test_handoff_result_formats() {
    let h_res = HandoffResult {
        checkpoint_id: Uuid::new_v4().to_string(),
        episode_id: Uuid::new_v4().to_string(),
        current_goal: "goal".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        steps_completed_count: 2,
        what_worked: vec!["worked".to_string()],
        what_failed: vec!["failed".to_string()],
        salient_facts: vec!["fact".to_string()],
        suggested_next_steps: vec!["next".to_string()],
        pattern_count: 1,
        heuristic_count: 1,
    };
    assert!(h_res.write(OutputFormat::Human).is_ok());
    assert!(h_res.write(OutputFormat::Json).is_ok());
    assert!(h_res.write(OutputFormat::Yaml).is_ok());
}

#[test]
fn test_resume_result_formats() {
    let r_res = ResumeResult {
        new_episode_id: Uuid::new_v4().to_string(),
        checkpoint_id: Uuid::new_v4().to_string(),
    };
    assert!(r_res.write(OutputFormat::Human).is_ok());
    assert!(r_res.write(OutputFormat::Json).is_ok());
    assert!(r_res.write(OutputFormat::Yaml).is_ok());
}

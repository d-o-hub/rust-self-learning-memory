use super::*;
use crate::episode::Episode;
use crate::types::{TaskContext, TaskOutcome, TaskType};

fn create_test_episode_with_steps(step_count: usize) -> Episode {
    let mut episode = Episode::new(
        "Test task".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );
    for i in 1..=step_count {
        episode.add_step(crate::episode::ExecutionStep::new(
            i,
            "tool".to_string(),
            "action".to_string(),
        ));
    }
    episode
}

#[test]
fn test_abstention_auto_checkpoint_created() {
    let mut episode = create_test_episode_with_steps(5);
    episode.outcome = Some(TaskOutcome::Abstained {
        reason: "Tool not found".to_string(),
        stopped_at_step: 3,
        infeasibility_signals: vec!["404_tool".to_string()],
    });
    maybe_create_abstention_checkpoint(&mut episode);
    assert_eq!(episode.checkpoints.len(), 1);
    assert!(episode.checkpoints[0].is_abstention_checkpoint);
    assert_eq!(episode.checkpoints[0].step_number, 3);
    assert!(episode.checkpoints[0].label.starts_with("[ABSTAIN]"));
}

#[test]
fn test_no_checkpoint_for_zero_step_abstention() {
    let mut episode = create_test_episode_with_steps(0);
    episode.outcome = Some(TaskOutcome::Abstained {
        reason: "Immediate rejection".to_string(),
        stopped_at_step: 0,
        infeasibility_signals: vec![],
    });
    maybe_create_abstention_checkpoint(&mut episode);
    // No steps = no useful state to checkpoint
    assert_eq!(episode.checkpoints.len(), 0);
}

#[test]
fn test_non_abstention_outcome_no_auto_checkpoint() {
    let mut episode = create_test_episode_with_steps(5);
    episode.outcome = Some(TaskOutcome::Failure {
        reason: "Error".to_string(),
        error_details: None,
    });
    maybe_create_abstention_checkpoint(&mut episode);
    assert_eq!(episode.checkpoints.len(), 0);
}

#[test]
fn test_abstention_checkpoint_backward_compat_deserialization() {
    // Old JSON without is_abstention_checkpoint and episode_id fields should deserialize with defaults
    // and aliases should handle timestamp/label
    let json = r#"{
            "checkpoint_id": "00000000-0000-0000-0000-000000000000",
            "step_number": 3,
            "created_at": "2026-01-01T00:00:00Z",
            "reason": "old checkpoint",
            "salient_features_snapshot": null,
            "note": "old note"
        }"#;
    let meta: CheckpointMeta = serde_json::from_str(json).unwrap();
    assert!(!meta.is_abstention_checkpoint);
    assert_eq!(meta.label, "old checkpoint");
    assert_eq!(meta.step_number, 3);
}

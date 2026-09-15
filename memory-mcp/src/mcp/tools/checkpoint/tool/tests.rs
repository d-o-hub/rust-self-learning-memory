use super::*;

#[tokio::test]
async fn test_checkpoint_episode_invalid_uuid() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = CheckpointTools::new(memory);

    let input = CheckpointEpisodeInput {
        episode_id: "not-a-uuid".to_string(),
        reason: "test".to_string(),
        note: None,
    };

    let result = tools.checkpoint_episode(input).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_handoff_pack_invalid_uuid() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = CheckpointTools::new(memory);

    let input = GetHandoffPackInput {
        checkpoint_id: "not-a-uuid".to_string(),
        mode: None,
        max_bytes: None,
    };

    let result = tools.get_handoff_pack(input).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_checkpoint_reason_truncation() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = CheckpointTools::new(memory);

    // Reason longer than MAX_CHECKPOINT_REASON_LEN
    let long_reason = "x".repeat(constants::MAX_CHECKPOINT_REASON_LEN + 100);
    let input = CheckpointEpisodeInput {
        episode_id: Uuid::new_v4().to_string(),
        reason: long_reason,
        note: Some("test".to_string()),
    };

    // Should fail because episode doesn't exist (not due to reason length)
    let result = tools.checkpoint_episode(input).await;
    assert!(result.is_err() || !result.as_ref().unwrap().success);
}

#[tokio::test]
async fn test_checkpoint_reason_truncation_utf8_safe() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = CheckpointTools::new(memory);

    // Multi-byte UTF-8 at the truncation boundary (e.g., CJK characters)
    // 'あ' is 3 bytes in UTF-8; place it so truncation falls mid-character
    let prefix = "x".repeat(constants::MAX_CHECKPOINT_REASON_LEN - 1);
    let reason = format!("{}あああ", prefix);
    assert!(reason.len() > constants::MAX_CHECKPOINT_REASON_LEN);

    let input = CheckpointEpisodeInput {
        episode_id: Uuid::new_v4().to_string(),
        reason,
        note: None,
    };

    // Must not panic from multi-byte truncation
    let result = tools.checkpoint_episode(input).await;
    // Episode won't exist, but truncation must succeed without panic
    assert!(result.is_err() || !result.as_ref().unwrap().success);
}

#[tokio::test]
async fn test_checkpoint_reason_truncation_emoji_safe() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = CheckpointTools::new(memory);

    // Emoji at truncation boundary (🚀 is 4 bytes in UTF-8)
    let prefix = "x".repeat(constants::MAX_CHECKPOINT_REASON_LEN.saturating_sub(3));
    let reason = format!("{}🚀🚀🚀", prefix);
    assert!(reason.len() > constants::MAX_CHECKPOINT_REASON_LEN);

    let input = CheckpointEpisodeInput {
        episode_id: Uuid::new_v4().to_string(),
        reason,
        note: None,
    };

    // Must not panic from emoji truncation
    let result = tools.checkpoint_episode(input).await;
    assert!(result.is_err() || !result.as_ref().unwrap().success);
}

#[tokio::test]
async fn test_checkpoint_note_truncation() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = CheckpointTools::new(memory);

    // Note longer than MAX_CHECKPOINT_NOTE_LEN
    let long_note = "x".repeat(constants::MAX_CHECKPOINT_NOTE_LEN + 100);
    let input = CheckpointEpisodeInput {
        episode_id: Uuid::new_v4().to_string(),
        reason: "test".to_string(),
        note: Some(long_note),
    };

    let result = tools.checkpoint_episode(input).await;
    // Should not panic from length
    assert!(result.is_err() || !result.as_ref().unwrap().success);
}

async fn checkpoint_fixture() -> (Arc<SelfLearningMemory>, String) {
    use do_memory_core::memory::checkpoint::checkpoint_episode as core_checkpoint;
    use do_memory_core::{MemoryConfig, TaskContext, TaskType};

    // Step buffering would hide steps from the checkpoint; disable it.
    let memory = Arc::new(SelfLearningMemory::with_config(MemoryConfig {
        quality_threshold: 0.0,
        batch_config: None,
        ..MemoryConfig::default()
    }));
    let episode_id = memory
        .start_episode(
            "MCP handoff task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;
    for i in 1..=3 {
        let step = do_memory_core::episode::ExecutionStep::new(
            i,
            "tool".to_string(),
            format!("action {i}"),
        );
        memory.log_step(episode_id, step).await;
    }
    let checkpoint_id = core_checkpoint(&memory, episode_id, "test".to_string())
        .await
        .expect("checkpoint must succeed")
        .checkpoint_id
        .to_string();
    (memory, checkpoint_id)
}

#[tokio::test]
async fn test_get_handoff_pack_defaults_to_compact() {
    let (memory, checkpoint_id) = checkpoint_fixture().await;
    let tools = CheckpointTools::new(memory);

    let input = GetHandoffPackInput {
        checkpoint_id,
        mode: None,
        max_bytes: None,
    };
    let output = tools.get_handoff_pack(input).await.unwrap();

    assert!(output.success);
    assert!(output.handoff_pack.is_none());
    let compact = output.compact_handoff.expect("compact pack expected");
    assert!(compact.payload_bytes() <= 8192);
    assert_eq!(compact.evidence_excerpts.len(), 3);
}

#[tokio::test]
async fn test_get_handoff_pack_full_mode() {
    let (memory, checkpoint_id) = checkpoint_fixture().await;
    let tools = CheckpointTools::new(memory);

    let input = GetHandoffPackInput {
        checkpoint_id,
        mode: Some("full".to_string()),
        max_bytes: None,
    };
    let output = tools.get_handoff_pack(input).await.unwrap();

    assert!(output.success);
    assert!(output.compact_handoff.is_none());
    assert!(output.handoff_pack.is_some());
}

#[tokio::test]
async fn test_get_handoff_pack_invalid_mode_rejected() {
    let (memory, checkpoint_id) = checkpoint_fixture().await;
    let tools = CheckpointTools::new(memory);

    let input = GetHandoffPackInput {
        checkpoint_id,
        mode: Some("bogus".to_string()),
        max_bytes: None,
    };
    let output = tools.get_handoff_pack(input).await.unwrap();

    assert!(!output.success);
    assert!(output.message.contains("Invalid mode"));
}

#[tokio::test]
async fn test_get_handoff_pack_max_bytes_floor_rejected() {
    let (memory, checkpoint_id) = checkpoint_fixture().await;
    let tools = CheckpointTools::new(memory);

    let input = GetHandoffPackInput {
        checkpoint_id,
        mode: None,
        max_bytes: Some(100),
    };
    let output = tools.get_handoff_pack(input).await.unwrap();

    assert!(!output.success);
    assert!(output.message.contains("minimum"));
}

#[tokio::test]
async fn test_resume_from_compact_round_trip() {
    let (memory, checkpoint_id) = checkpoint_fixture().await;
    let tools = CheckpointTools::new(Arc::clone(&memory));

    let input = GetHandoffPackInput {
        checkpoint_id,
        mode: None,
        max_bytes: None,
    };
    let pack = tools.get_handoff_pack(input).await.unwrap();
    let compact = pack.compact_handoff.expect("compact pack expected");

    let resume = tools
        .resume_from_compact(ResumeFromCompactInput {
            compact_handoff: compact,
        })
        .await
        .unwrap();

    assert!(resume.success);
    let new_id: Uuid = resume
        .new_episode_id
        .expect("new episode expected")
        .parse()
        .expect("valid UUID");
    let resumed = memory.get_episode(new_id).await.unwrap();
    assert_eq!(resumed.task_description, "MCP handoff task");
    assert_eq!(
        resumed.metadata.get("handoff_format").map(String::as_str),
        Some("compact")
    );
}

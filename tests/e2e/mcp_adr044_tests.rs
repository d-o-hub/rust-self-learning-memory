//! Integration tests for ADR-044 features (Playbooks, Feedback, Checkpoints) via MCP

use memory_core::types::{MemoryConfig, TaskContext, TaskOutcome, TaskType};
use memory_core::{ExecutionStep, SelfLearningMemory};
use memory_storage_redb::RedbStorage;
use std::sync::Arc;
use tempfile::tempdir;
use uuid::Uuid;

/// Test helper to create a memory instance with storage
async fn setup_test_memory() -> (Arc<SelfLearningMemory>, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let turso_path = dir.path().join("test_turso.redb");
    let cache_path = dir.path().join("test_cache.redb");

    let turso_storage = RedbStorage::new(&turso_path)
        .await
        .expect("Failed to create turso storage");
    let cache_storage = RedbStorage::new(&cache_path)
        .await
        .expect("Failed to create cache storage");

    let config = MemoryConfig {
        quality_threshold: 0.1,
        ..Default::default()
    };

    let memory = Arc::new(SelfLearningMemory::with_storage(
        config,
        Arc::new(turso_storage),
        Arc::new(cache_storage),
    ));

    (memory, dir)
}

#[tokio::test]
async fn test_checkpoint_and_handoff_flow() {
    let (memory, _dir) = setup_test_memory().await;

    // 1. Create episode
    let episode_id = memory
        .start_episode(
            "Test handoff".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // 2. Add a step
    memory
        .log_step(
            episode_id,
            ExecutionStep::new(1, "tool".to_string(), "action".to_string()),
        )
        .await;
    memory.flush_steps(episode_id).await.unwrap();

    // 3. Create checkpoint
    let checkpoint = memory_core::memory::checkpoint::checkpoint_episode(
        &memory,
        episode_id,
        "Testing".to_string(),
    )
    .await
    .unwrap();

    // 4. Get handoff pack
    let handoff =
        memory_core::memory::checkpoint::get_handoff_pack(&memory, checkpoint.checkpoint_id)
            .await
            .unwrap();
    assert_eq!(handoff.episode_id, episode_id);
    assert!(!handoff.steps_completed.is_empty());

    // 5. Resume from handoff
    let new_episode_id = memory_core::memory::checkpoint::resume_from_handoff(&memory, handoff)
        .await
        .unwrap();
    assert_ne!(new_episode_id, episode_id);

    let new_episode = memory.get_episode(new_episode_id).await.unwrap();
    assert!(new_episode.metadata.contains_key("resumed_from_checkpoint"));
}

#[tokio::test]
async fn test_playbook_generation() {
    let (memory, _dir) = setup_test_memory().await;

    // Playbook generation is currently template based and works even with empty memory
    let playbooks = memory
        .retrieve_playbooks(
            "How to write tests",
            "testing",
            TaskType::Testing,
            TaskContext::default(),
            1,
            5,
        )
        .await;

    // With empty memory, it should return an empty list or some default result
    // The current implementation might return a default playbook structure
    assert!(playbooks.len() <= 1);
}

#[tokio::test]
async fn test_recommendation_feedback_loop() {
    let (memory, _dir) = setup_test_memory().await;

    let episode_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    // 1. Record session
    let session = memory_core::memory::attribution::RecommendationSession {
        session_id,
        episode_id,
        timestamp: chrono::Utc::now(),
        recommended_pattern_ids: vec!["p-1".to_string()],
        recommended_playbook_ids: vec![],
    };
    memory.record_recommendation_session(session).await;

    // 2. Record feedback
    let feedback = memory_core::memory::attribution::RecommendationFeedback {
        session_id,
        applied_pattern_ids: vec!["p-1".to_string()],
        consulted_episode_ids: vec![],
        outcome: TaskOutcome::Success {
            verdict: "It worked".to_string(),
            artifacts: vec![],
        },
        agent_rating: Some(0.9),
    };
    memory
        .record_recommendation_feedback(feedback)
        .await
        .unwrap();

    // 3. Check stats
    let stats = memory.get_recommendation_stats().await;
    assert_eq!(stats.total_feedback, 1);
    assert_eq!(stats.patterns_applied, 1);
}

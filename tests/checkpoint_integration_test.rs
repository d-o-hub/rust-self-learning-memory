#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Checkpoint Integration Tests
//!
//! Tests for checkpoint/handoff flow and persistence.

#![allow(missing_docs)]

use std::sync::Arc;

use do_memory_core::memory::SelfLearningMemory;
use do_memory_core::memory::checkpoint::{
    HandoffBudget, checkpoint_episode, get_compact_handoff_pack, get_handoff_pack,
    resume_from_compact, resume_from_handoff,
};
use do_memory_core::storage::StorageBackend;
use do_memory_core::types::ExecutionResult;
use do_memory_core::{ExecutionStep, MemoryConfig, TaskContext, TaskType};
use do_memory_storage_redb::RedbStorage;
use do_memory_storage_turso::TursoStorage;
use libsql::Builder;
use tempfile::TempDir;

#[tokio::test]
async fn test_checkpoint_handoff_flow() {
    // Disable batching to ensure steps are persisted immediately for the test
    let config = MemoryConfig {
        batch_config: None,
        ..MemoryConfig::default()
    };
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

#[tokio::test]
async fn test_resume_handoff_metadata_persists_across_storage_reload() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("checkpoint_resume.db");
    let db = Builder::new_local(&db_path)
        .build()
        .await
        .expect("create local db");
    let turso = Arc::new(TursoStorage::from_database(db).expect("turso from db"));
    turso.initialize_schema().await.expect("init schema");

    let cache_dir = TempDir::new().expect("create cache dir");
    let redb_path = cache_dir.path().join("checkpoint_cache.redb");
    let redb = Arc::new(RedbStorage::new(&redb_path).await.expect("redb"));

    let durable: Arc<dyn StorageBackend> = turso.clone();
    let cache: Arc<dyn StorageBackend> = redb.clone();
    let config = MemoryConfig {
        batch_config: None,
        ..MemoryConfig::default()
    };

    let memory = SelfLearningMemory::with_storage(config.clone(), durable.clone(), cache.clone());

    let episode_id = memory
        .start_episode(
            "Durable handoff test task".to_string(),
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

    let checkpoint = checkpoint_episode(&memory, episode_id, "Durability handoff".to_string())
        .await
        .expect("create checkpoint");

    let handoff = get_handoff_pack(&memory, checkpoint.checkpoint_id)
        .await
        .expect("get handoff pack");

    let resumed_episode_id = resume_from_handoff(&memory, handoff)
        .await
        .expect("resume from handoff");

    drop(memory);

    let reloaded_memory = SelfLearningMemory::with_storage(config, durable, cache);
    let resumed_episode = reloaded_memory
        .get_episode(resumed_episode_id)
        .await
        .expect("get resumed episode after reload");

    assert_eq!(
        resumed_episode
            .metadata
            .get("resumed_from_checkpoint")
            .expect("resumed_from_checkpoint metadata"),
        &checkpoint.checkpoint_id.to_string()
    );
    assert_eq!(
        resumed_episode
            .metadata
            .get("resumed_from_episode")
            .expect("resumed_from_episode metadata"),
        &episode_id.to_string()
    );
    assert!(resumed_episode.metadata.contains_key("what_worked"));
    assert!(resumed_episode.metadata.contains_key("what_failed"));
    assert!(resumed_episode.metadata.contains_key("salient_facts"));
    assert!(
        resumed_episode
            .metadata
            .contains_key("suggested_next_steps")
    );
}

#[tokio::test]
async fn test_compact_handoff_multi_step_workflow_resume_quality() {
    let config = MemoryConfig {
        quality_threshold: 0.0,
        batch_config: None,
        ..MemoryConfig::default()
    };
    let memory = SelfLearningMemory::with_config(config);

    // 1. Initial Episode: Multi-step realistic engineering task
    let task_goal = "Refactor authentication module and write unit tests".to_string();
    let source_episode_id = memory
        .start_episode(
            task_goal.clone(),
            TaskContext::default(),
            TaskType::Refactoring,
        )
        .await;

    // Step 1: Read source file
    let mut step1 = ExecutionStep::new(1, "read_file".to_string(), "Read src/auth.rs".to_string());
    step1.result = Some(ExecutionResult::Success {
        output: "src/auth.rs content loaded".to_string(),
    });
    memory.log_step(source_episode_id, step1).await;

    // Step 2: Edit auth file
    let mut step2 = ExecutionStep::new(
        2,
        "write_file".to_string(),
        "Modify src/auth.rs".to_string(),
    );
    step2.result = Some(ExecutionResult::Success {
        output: "Updated src/auth.rs with JWT validation logic".to_string(),
    });
    memory.log_step(source_episode_id, step2).await;

    // Step 3: Test attempt 1 (fails)
    let mut step3 = ExecutionStep::new(
        3,
        "cargo_test".to_string(),
        "Run auth unit tests".to_string(),
    );
    step3.result = Some(ExecutionResult::Error {
        message: "compilation failed in src/auth_test.rs: missing import".to_string(),
    });
    memory.log_step(source_episode_id, step3).await;

    // Step 4: Fix test file
    let mut step4 = ExecutionStep::new(
        4,
        "write_file".to_string(),
        "Fix imports in src/auth_test.rs".to_string(),
    );
    step4.result = Some(ExecutionResult::Success {
        output: "Fixed imports in src/auth_test.rs".to_string(),
    });
    memory.log_step(source_episode_id, step4).await;

    // Step 5: Test attempt 2 (passes)
    let mut step5 = ExecutionStep::new(
        5,
        "cargo_test".to_string(),
        "Run auth unit tests".to_string(),
    );
    step5.result = Some(ExecutionResult::Success {
        output: "All 12 tests passed in src/auth_test.rs".to_string(),
    });
    memory.log_step(source_episode_id, step5).await;

    // 2. Take checkpoint at step 5 before switching agents/tasks
    let checkpoint = checkpoint_episode(
        &memory,
        source_episode_id,
        "Completed auth refactor, pausing before docs".to_string(),
    )
    .await
    .expect("checkpoint at step 5");

    // 3. Generate compact handoff pack
    let compact_pack =
        get_compact_handoff_pack(&memory, checkpoint.checkpoint_id, HandoffBudget::default())
            .await
            .expect("get compact handoff pack");

    // Assert compact handoff context quality & artifacts
    assert_eq!(compact_pack.episode_id, source_episode_id);
    assert_eq!(compact_pack.current_goal, task_goal);
    assert_eq!(compact_pack.steps_done, 5);
    assert_eq!(compact_pack.steps_total, 5);
    assert_eq!(compact_pack.status, "in_progress");
    assert_eq!(compact_pack.evidence_excerpts.len(), 5);
    assert!(!compact_pack.verified_findings.is_empty());
    assert!(
        compact_pack
            .artifact_refs
            .contains(&"src/auth.rs".to_string())
    );
    assert!(
        compact_pack
            .artifact_refs
            .contains(&"src/auth_test.rs".to_string())
    );

    // 4. Resume in new episode using compact pack
    let resumed_episode_id = resume_from_compact(&memory, compact_pack)
        .await
        .expect("resume from compact pack");

    assert_ne!(resumed_episode_id, source_episode_id);

    let resumed_episode = memory
        .get_episode(resumed_episode_id)
        .await
        .expect("get resumed episode");
    assert_eq!(resumed_episode.task_description, task_goal);
    assert_eq!(
        resumed_episode
            .metadata
            .get("resumed_from_checkpoint")
            .map(String::as_str),
        Some(checkpoint.checkpoint_id.to_string()).as_deref()
    );
    assert_eq!(
        resumed_episode
            .metadata
            .get("resumed_from_episode")
            .map(String::as_str),
        Some(source_episode_id.to_string()).as_deref()
    );

    // 5. Continued execution in resumed episode based on handoff guidance
    let mut step_b1 =
        ExecutionStep::new(1, "write_file".to_string(), "Add docs/auth.md".to_string());
    step_b1.result = Some(ExecutionResult::Success {
        output: "Created docs/auth.md".to_string(),
    });
    memory.log_step(resumed_episode_id, step_b1).await;

    memory
        .complete_episode(
            resumed_episode_id,
            do_memory_core::types::TaskOutcome::Success {
                verdict: "Auth refactoring and documentation completed".to_string(),
                artifacts: vec![
                    "src/auth.rs".to_string(),
                    "src/auth_test.rs".to_string(),
                    "docs/auth.md".to_string(),
                ],
            },
        )
        .await
        .expect("complete resumed episode");

    let final_resumed_episode = memory
        .get_episode(resumed_episode_id)
        .await
        .expect("get final resumed episode");
    assert!(final_resumed_episode.is_complete());
}

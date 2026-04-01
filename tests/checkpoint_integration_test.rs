use std::sync::Arc;

use do_memory_core::memory::SelfLearningMemory;
use do_memory_core::memory::checkpoint::{
    checkpoint_episode, get_handoff_pack, resume_from_handoff,
};
use do_memory_core::storage::StorageBackend;
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

//! End-to-end persistence and playbook synthesis tests for semantic `EpisodeSummary`.
//!
//! Verifies that:
//! 1. `complete_episode` persists generated semantic summaries to backends that
//!    advertise `supports_episode_summaries()` and skips backends that do not.
//! 2. `get_episode_summary` resolves from memory, cache, or durable storage and
//!    hydrates the cache/fallback on a durable hit, or returns `Ok(None)` on a miss.
//! 3. `retrieve_playbooks` on a restarted `SelfLearningMemory` uses the
//!    persisted `EpisodeSummary` (including extracted `key_concepts`) rather
//!    than fabricating an empty-concept summary when episode tags are empty.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use do_memory_core::episode::{EpisodeRetentionPolicy, RetentionCriterion};
use do_memory_core::memory::SelfLearningMemory;
use do_memory_core::semantic::EpisodeSummary;
use do_memory_core::storage::circuit_breaker::CircuitBreakerConfig;
use do_memory_core::{
    ExecutionResult, ExecutionStep, MemoryConfig, StorageBackend, TaskContext, TaskOutcome,
    TaskType,
};
use do_memory_storage_redb::RedbStorage;
use do_memory_storage_turso::{ResilientStorage, TursoStorage};
use libsql::Builder;
use std::sync::Arc;
use tempfile::TempDir;
use uuid::Uuid;

fn permissive_config() -> MemoryConfig {
    MemoryConfig {
        quality_threshold: 0.0,
        ..MemoryConfig::default()
    }
}

async fn raw_turso(dir: &TempDir, name: &str) -> TursoStorage {
    let db_path = dir.path().join(name);
    let db = Builder::new_local(&db_path).build().await.unwrap();
    let storage = TursoStorage::from_database(db).unwrap();
    storage.initialize_schema().await.unwrap();
    storage
}

async fn setup_turso(dir: &TempDir, name: &str) -> Arc<TursoStorage> {
    Arc::new(raw_turso(dir, name).await)
}

async fn setup_redb(dir: &TempDir, name: &str) -> Arc<RedbStorage> {
    let db_path = dir.path().join(name);
    Arc::new(RedbStorage::new(&db_path).await.unwrap())
}

#[tokio::test]
async fn complete_episode_persists_summary_and_resolves_across_tiers() {
    let dir = TempDir::new().unwrap();
    let durable = setup_turso(&dir, "durable.db").await;
    let non_advertising_cache = Arc::new(ResilientStorage::new(
        raw_turso(&dir, "unrouted.db").await,
        CircuitBreakerConfig::default(),
    ));

    let memory = SelfLearningMemory::with_storage(
        permissive_config(),
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        Arc::clone(&non_advertising_cache) as Arc<dyn StorageBackend>,
    );

    let context = TaskContext {
        domain: "authentication".to_string(),
        language: Some("rust".to_string()),
        framework: Some("axum".to_string()),
        ..TaskContext::default()
    };
    let episode_id = memory
        .start_episode(
            "Implement JWT token validation middleware".to_string(),
            context,
            TaskType::CodeGeneration,
        )
        .await;

    let mut step = ExecutionStep::new(
        1,
        "validator".to_string(),
        "Verify signature and expiration claims".to_string(),
    );
    step.result = Some(ExecutionResult::Success {
        output: "Validated RS256 signature".to_string(),
    });
    memory.log_step(episode_id, step).await;

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "JWT middleware verified".to_string(),
                artifacts: vec!["src/auth/jwt.rs".to_string()],
            },
        )
        .await
        .expect("episode completion must succeed");

    // Durable backend received the summary.
    let durable_summary = durable
        .get_episode_summary(episode_id)
        .await
        .unwrap()
        .expect("advertising durable backend must persist EpisodeSummary");
    assert_eq!(durable_summary.episode_id, episode_id);

    // Cold-process simulation: empty Redb cache + populated Turso durable store.
    let fresh_cache = setup_redb(&dir, "fresh_cache.redb").await;
    let restarted = SelfLearningMemory::with_storage(
        permissive_config(),
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        Arc::clone(&fresh_cache) as Arc<dyn StorageBackend>,
    );

    // Miss path: unknown episode returns Ok(None).
    assert!(
        restarted
            .get_episode_summary(Uuid::new_v4())
            .await
            .unwrap()
            .is_none(),
        "unknown episode ID must return Ok(None)"
    );

    // Durable hit path: resolves from Turso and mirrors to Redb cache.
    let loaded = restarted
        .get_episode_summary(episode_id)
        .await
        .unwrap()
        .expect("summary must resolve from durable storage after restart");
    assert_eq!(loaded.episode_id, episode_id);
    assert!(
        !loaded.key_concepts.is_empty(),
        "SemanticSummarizer must extract non-empty key_concepts"
    );
    assert!(
        fresh_cache
            .get_episode_summary(episode_id)
            .await
            .unwrap()
            .is_some(),
        "durable summary hit must mirror into advertising cache backend"
    );

    // Cache hit path: a brand-new SelfLearningMemory with empty durable store
    // and populated Redb cache resolves directly from cache_storage.
    let empty_durable = setup_turso(&dir, "empty_durable.db").await;
    let cache_only_memory = SelfLearningMemory::with_storage(
        permissive_config(),
        empty_durable as Arc<dyn StorageBackend>,
        Arc::clone(&fresh_cache) as Arc<dyn StorageBackend>,
    );
    let from_cache = cache_only_memory
        .get_episode_summary(episode_id)
        .await
        .unwrap()
        .expect("summary must resolve from cache storage when fallback is empty");
    assert_eq!(from_cache.episode_id, episode_id);
}

#[tokio::test]
async fn retrieve_playbooks_uses_persisted_episode_summary_across_restart() {
    let dir = TempDir::new().unwrap();
    let durable = setup_turso(&dir, "playbooks_durable.db").await;
    let cache = setup_redb(&dir, "playbooks_cache.redb").await;

    let context = TaskContext {
        domain: "database-migration".to_string(),
        tags: vec![],
        ..TaskContext::default()
    };

    let memory = SelfLearningMemory::with_storage(
        permissive_config(),
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );

    let episode_id = memory
        .start_episode(
            "Run zero-downtime schema migration".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;
    let mut step = ExecutionStep::new(
        1,
        "migrator".to_string(),
        "Apply expand-contract column migration".to_string(),
    );
    step.result = Some(ExecutionResult::Success {
        output: "Migration applied cleanly".to_string(),
    });
    memory.log_step(episode_id, step).await;
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Migration complete".to_string(),
                artifacts: vec!["migrations/001.sql".to_string()],
            },
        )
        .await
        .unwrap();

    // Overwrite the durable summary with a distinctive concept marker to prove
    // `retrieve_playbooks` reads the persisted summary on a cold restart.
    let mut custom_summary = EpisodeSummary::new(episode_id);
    custom_summary.summary_text = "Zero-downtime expand-contract migration summary".to_string();
    custom_summary.key_concepts = vec!["expand-contract-sentinel".to_string()];
    durable
        .store_episode_summary(&custom_summary)
        .await
        .unwrap();

    let cold_cache = setup_redb(&dir, "cold_cache.redb").await;
    let restarted = SelfLearningMemory::with_storage(
        permissive_config(),
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        cold_cache as Arc<dyn StorageBackend>,
    );

    let playbooks = restarted
        .retrieve_playbooks(
            "Run zero-downtime schema migration",
            "database-migration",
            TaskType::CodeGeneration,
            context,
            1,
            5,
        )
        .await;

    assert_eq!(playbooks.len(), 1);
    assert!(
        playbooks[0]
            .expected_outcome
            .contains("expand-contract-sentinel"),
        "expected playbook outcome to include concept from persisted EpisodeSummary, got: {}",
        playbooks[0].expected_outcome
    );
}

#[tokio::test]
async fn cleanup_episodes_prunes_failed_candidates_across_memory_and_storage() {
    let dir = TempDir::new().unwrap();
    let durable = setup_turso(&dir, "cleanup_durable.db").await;
    let cache = setup_redb(&dir, "cleanup_cache.redb").await;
    let memory = SelfLearningMemory::with_storage(
        permissive_config(),
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );

    let keep_id = memory
        .start_episode(
            "Successful release workflow".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        )
        .await;
    let mut step = ExecutionStep::new(1, "builder".to_string(), "Compile release".to_string());
    step.result = Some(ExecutionResult::Success {
        output: "ok".to_string(),
    });
    memory.log_step(keep_id, step).await;
    memory
        .complete_episode(
            keep_id,
            TaskOutcome::Success {
                verdict: "Shipped".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    let drop_id = memory
        .start_episode(
            "Aborted broken experiment".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;
    let mut bad_step = ExecutionStep::new(1, "runner".to_string(), "Run broken check".to_string());
    bad_step.result = Some(ExecutionResult::Error {
        message: "failed".to_string(),
    });
    memory.log_step(drop_id, bad_step).await;
    memory
        .complete_episode(
            drop_id,
            TaskOutcome::Failure {
                reason: "broken".to_string(),
                error_details: None,
            },
        )
        .await
        .unwrap();

    let policy = EpisodeRetentionPolicy::default().with_criterion(RetentionCriterion::FailedOnly);
    assert_eq!(memory.count_cleanup_candidates(&policy).await.unwrap(), 1);

    let dry_run = memory
        .cleanup_episodes(&policy.clone().with_dry_run(true))
        .await
        .unwrap();
    assert_eq!(dry_run.deleted_ids, vec![drop_id]);
    assert!(memory.get_episode(drop_id).await.is_ok());

    let executed = memory.cleanup_episodes(&policy).await.unwrap();
    assert_eq!(executed.deleted_ids, vec![drop_id]);
    assert_eq!(executed.kept_ids, vec![keep_id]);
    assert!(memory.get_episode(drop_id).await.is_err());
    assert!(memory.get_episode_summary(drop_id).await.unwrap().is_none());
    assert!(memory.get_episode(keep_id).await.is_ok());
}

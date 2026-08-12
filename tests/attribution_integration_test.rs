#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Attribution Integration Tests
//!
//! Tests for recommendation attribution flow and persistence.

#![allow(missing_docs)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::float_cmp)]

use chrono::Utc;
use do_memory_core::episode::PatternId;
use do_memory_core::memory::SelfLearningMemory;
use do_memory_core::memory::attribution::{RecommendationFeedback, RecommendationSession};
use do_memory_core::storage::StorageBackend;
use do_memory_core::types::{MemoryConfig, TaskContext, TaskOutcome, TaskType};
use do_memory_core::{Episode, Heuristic, Pattern, PersistenceReceipt};
use do_memory_storage_redb::RedbStorage;
use do_memory_storage_turso::TursoStorage;
use libsql::Builder;
use std::sync::Arc;
use tempfile::TempDir;
use uuid::Uuid;

/// Non-capable no-op backend: never advertises recommendation attribution
/// (ADR-081 §2), so it can never contribute a durable receipt. Used as the
/// inert companion in the Turso-only / redb-only cold-tracker tests.
#[derive(Default)]
struct InertBackend;

#[async_trait::async_trait]
impl StorageBackend for InertBackend {
    async fn store_episode(&self, _episode: &Episode) -> do_memory_core::Result<()> {
        Ok(())
    }
    async fn get_episode(&self, _id: Uuid) -> do_memory_core::Result<Option<Episode>> {
        Ok(None)
    }
    async fn delete_episode(&self, _id: Uuid) -> do_memory_core::Result<()> {
        Ok(())
    }
    async fn store_pattern(&self, _pattern: &Pattern) -> do_memory_core::Result<()> {
        Ok(())
    }
    async fn get_pattern(&self, _id: PatternId) -> do_memory_core::Result<Option<Pattern>> {
        Ok(None)
    }
    async fn store_heuristic(&self, _heuristic: &Heuristic) -> do_memory_core::Result<()> {
        Ok(())
    }
    async fn get_heuristic(&self, _id: Uuid) -> do_memory_core::Result<Option<Heuristic>> {
        Ok(None)
    }
    async fn query_episodes_since(
        &self,
        _since: chrono::DateTime<Utc>,
        _limit: Option<usize>,
    ) -> do_memory_core::Result<Vec<Episode>> {
        Ok(vec![])
    }
    async fn query_episodes_by_metadata(
        &self,
        _key: &str,
        _value: &str,
        _limit: Option<usize>,
    ) -> do_memory_core::Result<Vec<Episode>> {
        Ok(vec![])
    }
    async fn store_embedding(&self, _id: &str, _embedding: Vec<f32>) -> do_memory_core::Result<()> {
        Ok(())
    }
    async fn get_embedding(&self, _id: &str) -> do_memory_core::Result<Option<Vec<f32>>> {
        Ok(None)
    }
    async fn delete_embedding(&self, _id: &str) -> do_memory_core::Result<bool> {
        Ok(true)
    }
    async fn store_embeddings_batch(
        &self,
        _embeddings: Vec<(String, Vec<f32>)>,
    ) -> do_memory_core::Result<()> {
        Ok(())
    }
    async fn get_embeddings_batch(
        &self,
        _ids: &[String],
    ) -> do_memory_core::Result<Vec<Option<Vec<f32>>>> {
        Ok(vec![])
    }
}

#[tokio::test]
async fn test_full_attribution_flow() {
    let memory = SelfLearningMemory::new();

    // 1. Start an episode
    let episode_id = memory
        .start_episode(
            "Integration test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // 2. Simulate recommendations
    let pattern_id = "test-pattern-1".to_string();
    let session = RecommendationSession {
        session_id: Uuid::new_v4(),
        episode_id,
        timestamp: Utc::now(),
        recommended_pattern_ids: vec![pattern_id.clone()],
        recommended_playbook_ids: vec![],
    };
    let session_id = session.session_id;
    memory.record_recommendation_session(session).await;

    // 3. Record feedback after task completion
    let feedback = RecommendationFeedback {
        session_id,
        applied_pattern_ids: vec![pattern_id],
        consulted_episode_ids: vec![],
        outcome: TaskOutcome::Success {
            verdict: "Task succeeded".to_string(),
            artifacts: vec![],
        },
        agent_rating: Some(1.0),
    };
    memory
        .record_recommendation_feedback(feedback)
        .await
        .expect("Record feedback");

    // 4. Verify stats
    let stats = memory.get_recommendation_stats().await;
    assert_eq!(stats.total_sessions, 1);
    assert_eq!(stats.total_feedback, 1);
    assert_eq!(stats.patterns_applied, 1);
    assert_eq!(stats.adoption_rate, 1.0);
    assert_eq!(stats.success_after_adoption_rate, 1.0);
}

#[tokio::test]
async fn test_recommendation_persistence_with_storage() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("attribution.db");
    let db = Builder::new_local(&db_path)
        .build()
        .await
        .expect("create local db");
    let turso = Arc::new(TursoStorage::from_database(db).expect("turso from db"));
    turso.initialize_schema().await.expect("init schema");

    let cache_dir = TempDir::new().expect("create cache dir");
    let redb_path = cache_dir.path().join("cache.redb");
    let redb = Arc::new(RedbStorage::new(&redb_path).await.expect("redb"));

    let durable: Arc<dyn StorageBackend> = turso.clone();
    let cache: Arc<dyn StorageBackend> = redb.clone();
    let config = MemoryConfig::default();

    let memory = SelfLearningMemory::with_storage(config.clone(), durable.clone(), cache.clone());

    let episode_id = memory
        .start_episode(
            "Persistent attribution".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    let session = RecommendationSession {
        session_id: Uuid::new_v4(),
        episode_id,
        timestamp: Utc::now(),
        recommended_pattern_ids: vec!["persist-pattern".to_string()],
        recommended_playbook_ids: vec![],
    };
    let session_id = session.session_id;
    memory.record_recommendation_session(session).await;

    let feedback = RecommendationFeedback {
        session_id,
        applied_pattern_ids: vec!["persist-pattern".to_string()],
        consulted_episode_ids: vec![],
        outcome: TaskOutcome::Success {
            verdict: "persisted".to_string(),
            artifacts: vec![],
        },
        agent_rating: Some(0.75),
    };
    memory
        .record_recommendation_feedback(feedback)
        .await
        .expect("feedback");

    drop(memory);

    let memory = SelfLearningMemory::with_storage(config, durable, cache);

    let stored_session = memory
        .get_recommendation_session_for_episode(episode_id)
        .await;
    assert!(stored_session.is_some());
    assert_eq!(stored_session.unwrap().session_id, session_id);

    let stored_feedback = memory.get_recommendation_feedback(session_id).await;
    assert!(stored_feedback.is_some());

    let stats = memory.get_recommendation_stats().await;
    assert_eq!(stats.total_sessions, 1);
    assert_eq!(stats.total_feedback, 1);
    assert_eq!(stats.patterns_applied, 1);
}

/// ADR-081 AC-1 (Turso-only): a session persisted to the durable Turso backend
/// before a restart must accept feedback from a cold tracker, and the checked
/// feedback receipt must be `Persisted`.
#[tokio::test]
async fn feedback_accepted_after_cold_restart_turso_only() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let db_path = temp_dir.path().join("cold_turso.db");
    let db = Builder::new_local(&db_path)
        .build()
        .await
        .expect("create local db");
    let turso = Arc::new(TursoStorage::from_database(db).expect("turso from db"));
    turso.initialize_schema().await.expect("init schema");
    let turso_dyn: Arc<dyn StorageBackend> = turso.clone();

    let episode_id = Uuid::new_v4();
    let session = RecommendationSession {
        session_id: Uuid::new_v4(),
        episode_id,
        timestamp: Utc::now(),
        recommended_pattern_ids: vec!["cold-turso-pattern".to_string()],
        recommended_playbook_ids: vec![],
    };
    let session_id = session.session_id;

    // First process: persist the session through Turso only (inert cache).
    {
        let memory = SelfLearningMemory::with_storage(
            MemoryConfig::default(),
            turso_dyn.clone(),
            Arc::new(InertBackend) as Arc<dyn StorageBackend>,
        );
        memory.record_recommendation_session(session.clone()).await;
        assert!(
            turso
                .get_recommendation_session(session_id)
                .await
                .expect("read session from Turso")
                .is_some(),
            "the session must be durably stored in Turso"
        );
    }

    // Second process: cold tracker, same Turso backend, fresh inert companion.
    let restarted = SelfLearningMemory::with_storage(
        MemoryConfig::default(),
        turso_dyn.clone(),
        Arc::new(InertBackend) as Arc<dyn StorageBackend>,
    );

    let feedback = RecommendationFeedback {
        session_id,
        applied_pattern_ids: vec!["cold-turso-pattern".to_string()],
        consulted_episode_ids: vec![],
        outcome: TaskOutcome::Success {
            verdict: "cold restart".to_string(),
            artifacts: vec![],
        },
        agent_rating: Some(0.9),
    };
    restarted
        .record_recommendation_feedback(feedback.clone())
        .await
        .expect("feedback for a durable Turso session must be accepted after restart");

    let receipt = restarted
        .record_recommendation_feedback_checked(feedback)
        .await
        .expect("checked feedback for a durable Turso session must be accepted");
    assert!(
        matches!(receipt, PersistenceReceipt::Persisted { .. }),
        "Turso-only durable feedback must yield Persisted, got: {receipt:?}"
    );

    // Durable retrieval: the feedback must be readable back from Turso.
    let stored = turso
        .get_recommendation_feedback(session_id)
        .await
        .expect("read feedback from Turso");
    assert!(
        stored.is_some(),
        "feedback must be durably retrievable from Turso after the cold restart"
    );
}

/// ADR-081 AC-1 (redb-only): a session persisted to the redb cache backend
/// before a restart must accept feedback from a cold tracker, and the checked
/// feedback receipt must be `Persisted`.
#[tokio::test]
async fn feedback_accepted_after_cold_restart_redb_only() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let redb_path = temp_dir.path().join("cold_cache.redb");
    let redb = Arc::new(RedbStorage::new(&redb_path).await.expect("redb"));
    let redb_dyn: Arc<dyn StorageBackend> = redb.clone();

    let episode_id = Uuid::new_v4();
    let session = RecommendationSession {
        session_id: Uuid::new_v4(),
        episode_id,
        timestamp: Utc::now(),
        recommended_pattern_ids: vec!["cold-redb-pattern".to_string()],
        recommended_playbook_ids: vec![],
    };
    let session_id = session.session_id;

    // First process: persist the session through redb only (inert durable side).
    {
        let memory = SelfLearningMemory::with_storage(
            MemoryConfig::default(),
            Arc::new(InertBackend) as Arc<dyn StorageBackend>,
            redb_dyn.clone(),
        );
        memory.record_recommendation_session(session.clone()).await;
        assert!(
            redb.get_recommendation_session(session_id)
                .await
                .expect("read session from redb")
                .is_some(),
            "the session must be durably stored in redb"
        );
    }

    // Second process: cold tracker, same redb backend, fresh inert companion.
    let restarted = SelfLearningMemory::with_storage(
        MemoryConfig::default(),
        Arc::new(InertBackend) as Arc<dyn StorageBackend>,
        redb_dyn.clone(),
    );

    let feedback = RecommendationFeedback {
        session_id,
        applied_pattern_ids: vec!["cold-redb-pattern".to_string()],
        consulted_episode_ids: vec![],
        outcome: TaskOutcome::Success {
            verdict: "cold restart".to_string(),
            artifacts: vec![],
        },
        agent_rating: Some(0.8),
    };
    restarted
        .record_recommendation_feedback(feedback.clone())
        .await
        .expect("feedback for a durable redb session must be accepted after restart");

    let receipt = restarted
        .record_recommendation_feedback_checked(feedback)
        .await
        .expect("checked feedback for a durable redb session must be accepted");
    assert!(
        matches!(receipt, PersistenceReceipt::Persisted { .. }),
        "redb-only durable feedback must yield Persisted, got: {receipt:?}"
    );

    // Durable retrieval: the feedback must be readable back from redb.
    let stored = redb
        .get_recommendation_feedback(session_id)
        .await
        .expect("read feedback from redb");
    assert!(
        stored.is_some(),
        "feedback must be durably retrievable from redb after the cold restart"
    );
}

//! Attribution Integration Tests
//!
//! Tests for recommendation attribution flow and persistence.

#![allow(missing_docs)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::float_cmp)]

use chrono::Utc;
use do_memory_core::memory::SelfLearningMemory;
use do_memory_core::memory::attribution::{RecommendationFeedback, RecommendationSession};
use do_memory_core::storage::StorageBackend;
use do_memory_core::types::{MemoryConfig, TaskContext, TaskOutcome, TaskType};
use do_memory_storage_redb::RedbStorage;
use do_memory_storage_turso::TursoStorage;
use libsql::Builder;
use std::sync::Arc;
use tempfile::TempDir;
use uuid::Uuid;

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

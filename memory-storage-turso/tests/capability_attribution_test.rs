//! Attribution capability advertisement tests (ADR-081 §2; historical #940
//! Codecov precedent for capability overrides).
//!
//! `TursoStorage`, `ResilientStorage`, and `CachedTursoStorage` must all
//! advertise recommendation-attribution capability so the checked persistence
//! path (`persist_session_checked` / `persist_feedback_checked`) counts them as
//! durable. The resilient and cached wrappers must delegate to their inner
//! Turso backend rather than hardcode an independent answer.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use do_memory_core::StorageBackend;
use do_memory_core::TaskOutcome;
use do_memory_core::memory::attribution::{RecommendationFeedback, RecommendationSession};
use do_memory_core::storage::circuit_breaker::CircuitBreakerConfig;
use do_memory_storage_turso::{CacheConfig, CachedTursoStorage, ResilientStorage, TursoStorage};
use libsql::Builder;
use tempfile::TempDir;
use uuid::Uuid;

/// Build a local-file Turso backend (no network, no credentials).
async fn local_turso() -> (TursoStorage, TempDir) {
    let dir = TempDir::new().expect("create temp dir");
    let db_path = dir.path().join("capability.db");
    let db = Builder::new_local(&db_path)
        .build()
        .await
        .expect("create local db");
    let storage = TursoStorage::from_database(db).expect("turso from db");
    storage.initialize_schema().await.expect("init schema");
    (storage, dir)
}

#[tokio::test]
async fn turso_storage_advertises_recommendation_attribution() {
    let (storage, _dir) = local_turso().await;
    assert!(
        storage.supports_recommendation_attribution(),
        "TursoStorage must advertise recommendation-attribution capability"
    );
}

#[tokio::test]
async fn turso_storage_advertises_ranking_adaptation() {
    let (storage, _dir) = local_turso().await;
    assert!(
        storage.supports_ranking_adaptation(),
        "TursoStorage must advertise ranking-adaptation capability"
    );
}

#[tokio::test]
async fn resilient_storage_delegates_ranking_capability_to_inner_turso() {
    let (storage, _dir) = local_turso().await;
    let resilient = ResilientStorage::new(storage, CircuitBreakerConfig::default());
    assert!(
        resilient.supports_ranking_adaptation(),
        "ResilientStorage must delegate ranking-capability to its inner Turso backend"
    );
}

#[tokio::test]
async fn cached_turso_storage_delegates_ranking_capability_to_inner_backend() {
    let (storage, _dir) = local_turso().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());
    assert!(
        cached.supports_ranking_adaptation(),
        "CachedTursoStorage must delegate ranking-capability to its inner Turso backend"
    );
}

/// ADR-082: the Turso `list_recommendation_*` surface must round-trip every
/// stored entry (the durable read surface for index rebuilds).
#[tokio::test]
async fn turso_lists_all_recommendation_history() {
    let (storage, _dir) = local_turso().await;

    let sessions: Vec<RecommendationSession> = (0..2)
        .map(|i| RecommendationSession {
            session_id: Uuid::new_v4(),
            episode_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            recommended_pattern_ids: vec![format!("pattern-{i}")],
            recommended_playbook_ids: vec![],
        })
        .collect();
    let feedback: Vec<RecommendationFeedback> = sessions
        .iter()
        .map(|s| RecommendationFeedback {
            session_id: s.session_id,
            applied_pattern_ids: s.recommended_pattern_ids.clone(),
            consulted_episode_ids: vec![],
            outcome: TaskOutcome::Success {
                verdict: "ok".to_string(),
                artifacts: vec![],
            },
            agent_rating: Some(0.9),
        })
        .collect();

    for s in &sessions {
        storage.store_recommendation_session(s).await.unwrap();
    }
    for f in &feedback {
        storage.store_recommendation_feedback(f).await.unwrap();
    }

    let listed_sessions = storage.list_recommendation_sessions().await.unwrap();
    let listed_feedback = storage.list_recommendation_feedback().await.unwrap();

    let mut got_sessions: Vec<_> = listed_sessions.into_iter().map(|s| s.session_id).collect();
    got_sessions.sort();
    let mut want_sessions: Vec<_> = sessions.iter().map(|s| s.session_id).collect();
    want_sessions.sort();
    assert_eq!(got_sessions, want_sessions, "all sessions must be listed");

    let mut got_feedback: Vec<_> = listed_feedback.into_iter().map(|f| f.session_id).collect();
    got_feedback.sort();
    let mut want_feedback: Vec<_> = feedback.iter().map(|f| f.session_id).collect();
    want_feedback.sort();
    assert_eq!(got_feedback, want_feedback, "all feedback must be listed");
}

#[tokio::test]
async fn resilient_storage_delegates_capability_to_inner_turso() {
    let (storage, _dir) = local_turso().await;
    let resilient = ResilientStorage::new(storage, CircuitBreakerConfig::default());
    assert!(
        resilient.supports_recommendation_attribution(),
        "ResilientStorage must delegate capability to its inner Turso backend"
    );
}

#[tokio::test]
async fn cached_turso_storage_delegates_capability_to_inner_backend() {
    let (storage, _dir) = local_turso().await;
    let cached = CachedTursoStorage::new(storage, CacheConfig::default());
    assert!(
        cached.supports_recommendation_attribution(),
        "CachedTursoStorage must delegate capability to its inner Turso backend"
    );
}

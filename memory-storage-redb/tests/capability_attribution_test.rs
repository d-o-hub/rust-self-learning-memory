//! Attribution capability advertisement test (ADR-081 §2; historical #940
//! Codecov precedent for capability overrides).
//!
//! The compiled `RedbStorage` implementation (`src/backend_impl.rs`) must
//! advertise recommendation-attribution capability so the checked persistence
//! path counts it as durable. The uncompiled duplicate in `src/redb_cache.rs`
//! is intentionally untouched: `src/lib.rs` includes `backend_impl` only.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use do_memory_core::StorageBackend;
use do_memory_core::TaskOutcome;
use do_memory_core::memory::attribution::{RecommendationFeedback, RecommendationSession};
use do_memory_storage_redb::RedbStorage;
use tempfile::TempDir;
use uuid::Uuid;

#[tokio::test]
async fn redb_storage_advertises_recommendation_attribution() {
    let dir = TempDir::new().expect("create temp dir");
    let db_path = dir.path().join("capability.redb");
    let storage = RedbStorage::new(&db_path).await.expect("create redb");
    assert!(
        storage.supports_recommendation_attribution(),
        "the compiled RedbStorage must advertise recommendation-attribution capability"
    );
}

/// ADR-082: the redb backend must advertise ranking-adaptation capability so the
/// learned index rebuild can read its durable history.
#[tokio::test]
async fn redb_storage_advertises_ranking_adaptation() {
    let dir = TempDir::new().expect("create temp dir");
    let db_path = dir.path().join("ranking-capability.redb");
    let storage = RedbStorage::new(&db_path).await.expect("create redb");
    assert!(
        storage.supports_ranking_adaptation(),
        "the compiled RedbStorage must advertise ranking-adaptation capability"
    );
}

/// ADR-082: `list_recommendation_sessions` / `list_recommendation_feedback` must
/// round-trip every stored entry (the durable read surface for index rebuilds).
#[tokio::test]
async fn redb_lists_all_recommendation_history() {
    let dir = TempDir::new().expect("create temp dir");
    let db_path = dir.path().join("ranking-list.redb");
    let storage = RedbStorage::new(&db_path).await.expect("create redb");

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

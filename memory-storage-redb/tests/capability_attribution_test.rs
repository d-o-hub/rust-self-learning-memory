//! Attribution capability advertisement test (ADR-081 §2; historical #940
//! Codecov precedent for capability overrides).
//!
//! The compiled `RedbStorage` implementation (`src/backend_impl.rs`) must
//! advertise recommendation-attribution capability so the checked persistence
//! path counts it as durable. The former uncompiled duplicate in
//! `src/redb_cache.rs` was removed in #1087 slice 1; `src/lib.rs` includes
//! `backend_impl` only.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use do_memory_core::TaskContext;
use do_memory_core::TaskOutcome;
use do_memory_core::episode::{EpisodeRelationship, RelationshipMetadata, RelationshipType};
use do_memory_core::memory::attribution::{RecommendationFeedback, RecommendationSession};
use do_memory_core::procedural::ProceduralMemory;
use do_memory_core::{StorageBackend, StorageBackendCapabilities};
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

/// #1087 slices 2-3: the compiled redb backend persists relationships
/// (episode↔episode and episode↔pattern) and procedural memory in its own
/// tables, so it must advertise both capabilities — and must not claim the
/// cleanup capability it does not implement.
#[tokio::test]
async fn redb_storage_advertises_optional_persistence_capabilities() {
    let dir = TempDir::new().expect("create temp dir");
    let db_path = dir.path().join("optional-capability.redb");
    let storage = RedbStorage::new(&db_path).await.expect("create redb");

    assert!(
        storage.supports_relationship_persistence(),
        "redb stores relationships in RELATIONSHIPS_TABLE"
    );
    assert!(
        storage.supports_procedural_memory(),
        "redb stores procedural memory in PROCEDURAL_TABLE"
    );
    assert!(
        !storage.supports_episode_cleanup(),
        "redb has no retention/GC implementation, so cleanup stays unavailable"
    );
}

/// #1087: the advertised capabilities are backed by durable round-trips through
/// the `StorageBackend` trait object, not by fabricated defaults.
#[tokio::test]
async fn redb_optional_operations_round_trip_through_trait_object() {
    let dir = TempDir::new().expect("create temp dir");
    let db_path = dir.path().join("optional-roundtrip.redb");
    let storage = RedbStorage::new(&db_path).await.expect("create redb");
    let backend: &dyn StorageBackend = &storage;

    let from = Uuid::new_v4();
    let to = Uuid::new_v4();
    let relationship = EpisodeRelationship::new(
        from,
        to,
        RelationshipType::RelatedTo,
        RelationshipMetadata::default(),
    );
    backend.store_relationship(&relationship).await.unwrap();
    assert!(
        backend
            .relationship_exists(from, to, RelationshipType::RelatedTo)
            .await
            .unwrap(),
        "a stored relationship must be visible through the trait object"
    );
    assert_eq!(
        backend
            .get_relationship_by_id(relationship.id)
            .await
            .unwrap()
            .map(|r| r.id),
        Some(relationship.id)
    );

    let procedural = ProceduralMemory::new(
        "skill".to_string(),
        "round trip".to_string(),
        TaskContext::default(),
        Vec::new(),
    );
    backend.store_procedural(&procedural).await.unwrap();
    assert_eq!(
        backend
            .get_procedural(procedural.id)
            .await
            .unwrap()
            .map(|p| p.id),
        Some(procedural.id)
    );
    backend.delete_procedural(procedural.id).await.unwrap();
    assert!(
        backend
            .get_procedural(procedural.id)
            .await
            .unwrap()
            .is_none(),
        "deleted procedural memory must be gone"
    );
}

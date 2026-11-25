//! Attribution capability advertisement tests (ADR-081 §2; historical #940
//! Codecov precedent for capability overrides).
//!
//! `TursoStorage`, `ResilientStorage`, and `CachedTursoStorage` must all
//! advertise recommendation-attribution capability so the checked persistence
//! path (`persist_session_checked` / `persist_feedback_checked`) counts them as
//! durable. The resilient and cached wrappers must delegate to their inner
//! Turso backend rather than hardcode an independent answer.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use do_memory_core::TaskContext;
use do_memory_core::TaskOutcome;
use do_memory_core::episode::{EpisodeRelationship, RelationshipMetadata, RelationshipType};
use do_memory_core::memory::attribution::{RecommendationFeedback, RecommendationSession};
use do_memory_core::procedural::ProceduralMemory;
use do_memory_core::storage::circuit_breaker::CircuitBreakerConfig;
use do_memory_core::{Error, StorageBackend, StorageBackendCapabilities};
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

/// #1087 slices 2-3: relationship and procedural capabilities must be truthful
/// per backend. `TursoStorage` persists both with SQL, so it advertises them;
/// the resilient and cached wrappers do not route those methods, so they must
/// not claim them and their inherited defaults must return the typed
/// capability error instead of a silent success.
#[tokio::test]
async fn relationship_and_procedural_capability_is_truthful_per_backend() {
    let (storage, _dir) = local_turso().await;
    assert!(
        storage.supports_relationship_persistence(),
        "TursoStorage persists relationships in SQL"
    );
    assert!(
        storage.supports_procedural_memory(),
        "TursoStorage persists procedural memory in SQL"
    );
    assert!(
        storage.supports_episode_summaries(),
        "TursoStorage persists episode summaries in SQL"
    );
    assert!(
        storage.supports_episode_cleanup(),
        "TursoStorage implements retention policy episode cleanup"
    );

    let (resilient_inner, _dir2) = local_turso().await;
    let resilient = ResilientStorage::new(resilient_inner, CircuitBreakerConfig::default());
    let (cached_inner, _dir3) = local_turso().await;
    let cached = CachedTursoStorage::new(cached_inner, CacheConfig::default());

    for backend in [
        &resilient as &dyn StorageBackend,
        &cached as &dyn StorageBackend,
    ] {
        assert!(
            !backend.supports_relationship_persistence(),
            "an unrouted wrapper must not advertise relationship persistence"
        );
        assert!(
            !backend.supports_procedural_memory(),
            "an unrouted wrapper must not advertise procedural memory"
        );
        assert!(
            backend.supports_episode_summaries(),
            "routed wrapper must delegate episode summary capability"
        );
        assert!(
            backend.supports_episode_cleanup(),
            "routed wrapper must delegate episode cleanup capability"
        );
    }

    let relationship = EpisodeRelationship::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        RelationshipType::RelatedTo,
        RelationshipMetadata::default(),
    );
    let err = resilient
        .store_relationship(&relationship)
        .await
        .expect_err("an unrouted wrapper must not fake a relationship write");
    assert!(matches!(
        err,
        Error::CapabilityUnavailable {
            operation: "store_relationship"
        }
    ));

    let procedural = ProceduralMemory::new(
        "skill".to_string(),
        "unrouted".to_string(),
        TaskContext::default(),
        Vec::new(),
    );
    let err = cached
        .store_procedural(&procedural)
        .await
        .expect_err("an unrouted wrapper must not fake a procedural write");
    assert!(matches!(
        err,
        Error::CapabilityUnavailable {
            operation: "store_procedural"
        }
    ));
}

#[tokio::test]
async fn turso_summary_and_cleanup_operations_round_trip() {
    use do_memory_core::episode::{EpisodeRetentionPolicy, RetentionCriterion};
    use do_memory_core::semantic::EpisodeSummary;
    use do_memory_core::{Episode, TaskType};

    let (storage, _dir) = local_turso().await;
    let backend: &dyn StorageBackend = &storage;

    let mut kept = Episode::new(
        "Retained successful episode".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    );
    kept.complete(TaskOutcome::Success {
        verdict: "ok".to_string(),
        artifacts: vec![],
    });
    backend.store_episode(&kept).await.unwrap();

    let mut candidate = Episode::new(
        "Expired failed candidate".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );
    candidate.complete(TaskOutcome::Failure {
        reason: "error".to_string(),
        error_details: None,
    });
    backend.store_episode(&candidate).await.unwrap();

    let mut summary = EpisodeSummary::new(kept.episode_id);
    summary.summary_text = "Turso durable summary text".to_string();
    summary.key_concepts = vec!["turso-sql".to_string()];
    backend.store_episode_summary(&summary).await.unwrap();

    let fetched = backend
        .get_episode_summary(kept.episode_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fetched.key_concepts, vec!["turso-sql".to_string()]);

    let policy = EpisodeRetentionPolicy::default().with_criterion(RetentionCriterion::FailedOnly);
    assert_eq!(backend.count_cleanup_candidates(&policy).await.unwrap(), 1);

    let dry = backend
        .cleanup_episodes(&policy.clone().with_dry_run(true))
        .await
        .unwrap();
    assert_eq!(dry.deleted, 1);
    assert_eq!(dry.kept, 1);

    let live = backend.cleanup_episodes(&policy).await.unwrap();
    assert_eq!(live.deleted_ids, vec![candidate.episode_id]);
    assert_eq!(live.kept_ids, vec![kept.episode_id]);
    assert!(
        backend
            .get_episode(candidate.episode_id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        backend
            .get_episode(kept.episode_id)
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn turso_wrappers_route_summary_and_cleanup_and_evict_cache() {
    use do_memory_core::episode::{EpisodeRetentionPolicy, RetentionCriterion};
    use do_memory_core::semantic::EpisodeSummary;
    use do_memory_core::{Episode, TaskType};

    let (cached_inner, _dir1) = local_turso().await;
    let cached = CachedTursoStorage::new(cached_inner, CacheConfig::default());
    let cached_backend: &dyn StorageBackend = &cached;

    let mut ep = Episode::new(
        "Failed cached candidate".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );
    ep.complete(TaskOutcome::Failure {
        reason: "boom".to_string(),
        error_details: None,
    });
    cached_backend.store_episode(&ep).await.unwrap();
    // Warm the adaptive episode cache
    assert!(
        cached_backend
            .get_episode(ep.episode_id)
            .await
            .unwrap()
            .is_some()
    );

    let mut summary = EpisodeSummary::new(ep.episode_id);
    summary.summary_text = "Cached wrapper summary".to_string();
    summary.key_concepts = vec!["wrapper-cache".to_string()];
    cached_backend
        .store_episode_summary(&summary)
        .await
        .unwrap();
    assert_eq!(
        cached_backend
            .get_episode_summary(ep.episode_id)
            .await
            .unwrap()
            .unwrap()
            .summary_text,
        "Cached wrapper summary"
    );

    let policy = EpisodeRetentionPolicy::default().with_criterion(RetentionCriterion::FailedOnly);
    assert_eq!(
        cached_backend
            .count_cleanup_candidates(&policy)
            .await
            .unwrap(),
        1
    );
    let cleaned = cached_backend.cleanup_episodes(&policy).await.unwrap();
    assert_eq!(cleaned.deleted_ids, vec![ep.episode_id]);
    assert!(
        cached_backend
            .get_episode(ep.episode_id)
            .await
            .unwrap()
            .is_none(),
        "CachedTursoStorage must evict cleaned episodes from its in-memory adaptive cache"
    );

    let (resilient_inner, _dir2) = local_turso().await;
    let resilient = ResilientStorage::new(resilient_inner, CircuitBreakerConfig::default());
    let resilient_backend: &dyn StorageBackend = &resilient;

    resilient_backend.store_episode(&ep).await.unwrap();
    resilient_backend
        .store_episode_summary(&summary)
        .await
        .unwrap();
    assert!(
        resilient_backend
            .get_episode_summary(ep.episode_id)
            .await
            .unwrap()
            .is_some()
    );
    assert_eq!(
        resilient_backend
            .count_cleanup_candidates(&policy)
            .await
            .unwrap(),
        1
    );
    let res_cleaned = resilient_backend.cleanup_episodes(&policy).await.unwrap();
    assert_eq!(res_cleaned.deleted_ids, vec![ep.episode_id]);
}

//! ADR-080/081 receipt matrix and attributed-recommendation behavior.
//!
//! `persist_session_checked` must emit the correct `PersistenceReceipt` state
//! for every combination of configured backends (`MemoryOnly` / `Persisted` /
//! `PartiallyPersisted` / `PersistenceFailed`), and the attributed entry points
//! (`recommend_patterns_attributed`, `retrieve_playbooks_attributed`) must
//! reject nil episode IDs and record sessions with the exact recommended IDs.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(clippy::uninlined_format_args)]

use async_trait::async_trait;
use do_memory_core::AttributedPlaybookRequest;
use do_memory_core::episode::PatternId;
use do_memory_core::memory::attribution::{RecommendationFeedback, RecommendationSession};
use do_memory_core::storage::StorageBackend;
use do_memory_core::{
    Episode, Heuristic, MemoryConfig, Pattern, PersistenceReceipt, Result, SelfLearningMemory,
    TaskContext, TaskOutcome, TaskType,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

mod common;

use common::{
    ContextBuilder, PatternType, create_completed_episode_with_pattern, setup_test_memory,
};

fn test_context() -> TaskContext {
    ContextBuilder::new("error-handling")
        .language("rust")
        .build()
}

// ============================================================================
// Mock backends
// ============================================================================

/// In-memory backend for recommendation sessions and feedback.
///
/// `fail_writes` makes `store_recommendation_session` fail, so the same struct
/// can simulate both a healthy and a broken backend for the receipt matrix.
#[derive(Default)]
struct TestBackend {
    sessions: Mutex<HashMap<Uuid, RecommendationSession>>,
    feedback: Mutex<HashMap<Uuid, RecommendationFeedback>>,
    fail_writes: bool,
}

#[async_trait]
impl StorageBackend for TestBackend {
    fn supports_recommendation_attribution(&self) -> bool {
        true
    }

    async fn store_recommendation_session(&self, session: &RecommendationSession) -> Result<()> {
        if self.fail_writes {
            return Err(do_memory_core::error::Error::Storage(
                "injected write failure".to_string(),
            ));
        }
        self.sessions
            .lock()
            .unwrap()
            .insert(session.session_id, session.clone());
        Ok(())
    }

    async fn get_recommendation_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<RecommendationSession>> {
        Ok(self.sessions.lock().unwrap().get(&session_id).cloned())
    }

    async fn store_recommendation_feedback(&self, feedback: &RecommendationFeedback) -> Result<()> {
        if self.fail_writes {
            return Err(do_memory_core::error::Error::Storage(
                "injected feedback write failure".to_string(),
            ));
        }
        self.feedback
            .lock()
            .unwrap()
            .insert(feedback.session_id, feedback.clone());
        Ok(())
    }

    async fn get_recommendation_feedback(
        &self,
        session_id: Uuid,
    ) -> Result<Option<RecommendationFeedback>> {
        Ok(self.feedback.lock().unwrap().get(&session_id).cloned())
    }

    // --- required trait surface (no-op for this test) ---
    async fn store_episode(&self, _episode: &Episode) -> Result<()> {
        Ok(())
    }
    async fn get_episode(&self, _id: Uuid) -> Result<Option<Episode>> {
        Ok(None)
    }
    async fn delete_episode(&self, _id: Uuid) -> Result<()> {
        Ok(())
    }
    async fn store_pattern(&self, _pattern: &Pattern) -> Result<()> {
        Ok(())
    }
    async fn get_pattern(&self, _id: PatternId) -> Result<Option<Pattern>> {
        Ok(None)
    }
    async fn store_heuristic(&self, _heuristic: &Heuristic) -> Result<()> {
        Ok(())
    }
    async fn get_heuristic(&self, _id: Uuid) -> Result<Option<Heuristic>> {
        Ok(None)
    }
    async fn query_episodes_since(
        &self,
        _since: chrono::DateTime<chrono::Utc>,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        Ok(vec![])
    }
    async fn query_episodes_by_metadata(
        &self,
        _key: &str,
        _value: &str,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        Ok(vec![])
    }
    async fn store_embedding(&self, _id: &str, _embedding: Vec<f32>) -> Result<()> {
        Ok(())
    }
    async fn get_embedding(&self, _id: &str) -> Result<Option<Vec<f32>>> {
        Ok(None)
    }
    async fn delete_embedding(&self, _id: &str) -> Result<bool> {
        Ok(true)
    }
    async fn store_embeddings_batch(&self, _embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        Ok(())
    }
    async fn get_embeddings_batch(&self, _ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        Ok(vec![])
    }
}

/// Backend that relies entirely on the default (no-op) trait surface.
#[derive(Default)]
struct InertBackend;

#[async_trait]
impl StorageBackend for InertBackend {
    async fn store_episode(&self, _episode: &Episode) -> Result<()> {
        Ok(())
    }
    async fn get_episode(&self, _id: Uuid) -> Result<Option<Episode>> {
        Ok(None)
    }
    async fn delete_episode(&self, _id: Uuid) -> Result<()> {
        Ok(())
    }
    async fn store_pattern(&self, _pattern: &Pattern) -> Result<()> {
        Ok(())
    }
    async fn get_pattern(&self, _id: PatternId) -> Result<Option<Pattern>> {
        Ok(None)
    }
    async fn store_heuristic(&self, _heuristic: &Heuristic) -> Result<()> {
        Ok(())
    }
    async fn get_heuristic(&self, _id: Uuid) -> Result<Option<Heuristic>> {
        Ok(None)
    }
    async fn query_episodes_since(
        &self,
        _since: chrono::DateTime<chrono::Utc>,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        Ok(vec![])
    }
    async fn query_episodes_by_metadata(
        &self,
        _key: &str,
        _value: &str,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        Ok(vec![])
    }
    async fn store_embedding(&self, _id: &str, _embedding: Vec<f32>) -> Result<()> {
        Ok(())
    }
    async fn get_embedding(&self, _id: &str) -> Result<Option<Vec<f32>>> {
        Ok(None)
    }
    async fn delete_embedding(&self, _id: &str) -> Result<bool> {
        Ok(true)
    }
    async fn store_embeddings_batch(&self, _embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        Ok(())
    }
    async fn get_embeddings_batch(&self, _ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        Ok(vec![])
    }
}

fn with_backends(
    durable: Arc<dyn StorageBackend>,
    cache: Arc<dyn StorageBackend>,
) -> SelfLearningMemory {
    // quality_threshold 0.0 lets the shared helpers complete low-quality
    // test episodes (matches setup_test_memory).
    let config = MemoryConfig {
        quality_threshold: 0.0,
        ..Default::default()
    };
    SelfLearningMemory::with_storage(config, durable, cache)
}

fn ok_backend() -> Arc<TestBackend> {
    Arc::new(TestBackend {
        fail_writes: false,
        ..Default::default()
    })
}

fn failing_backend() -> Arc<TestBackend> {
    Arc::new(TestBackend {
        fail_writes: true,
        ..Default::default()
    })
}

fn test_session(episode_id: Uuid) -> RecommendationSession {
    RecommendationSession {
        session_id: Uuid::new_v4(),
        episode_id,
        timestamp: chrono::Utc::now(),
        recommended_pattern_ids: vec!["pattern-a".to_string()],
        recommended_playbook_ids: vec![],
    }
}

fn test_feedback(session_id: Uuid) -> RecommendationFeedback {
    RecommendationFeedback {
        session_id,
        applied_pattern_ids: vec!["pattern-a".to_string()],
        consulted_episode_ids: vec![],
        outcome: TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        },
        agent_rating: Some(0.9),
    }
}

// ============================================================================
// recommend_patterns_attributed
// ============================================================================

#[tokio::test]
async fn attributed_pattern_recommendation_rejects_nil_episode() {
    let memory = setup_test_memory();
    let context = test_context();

    let result = memory
        .recommend_patterns_attributed(Uuid::nil(), "retry with backoff", context, 3)
        .await;

    assert!(
        result.is_err(),
        "a nil episode ID must be rejected by the attributed pattern path"
    );
}

#[tokio::test]
async fn attributed_pattern_recommendation_rejects_nonexistent_episode() {
    // ADR-080 §1: a well-formed but never-created episode UUID must be rejected
    // (InvalidInput) and must never produce a recommendation session.
    let memory = setup_test_memory();
    let context = test_context();
    let missing_episode = Uuid::new_v4();

    let result = memory
        .recommend_patterns_attributed(missing_episode, "retry with backoff", context, 3)
        .await;

    assert!(
        result.is_err(),
        "a nonexistent episode must be rejected by the attributed pattern path"
    );
    assert!(
        memory
            .get_recommendation_session_for_episode(missing_episode)
            .await
            .is_none(),
        "a nonexistent episode must never create a recommendation session"
    );
}

#[tokio::test]
async fn attributed_pattern_recommendation_records_session_memory_only() {
    let memory = setup_test_memory();
    let episode_id =
        create_completed_episode_with_pattern(&memory, PatternType::ErrorRecovery).await;
    let context = test_context();

    let attr = memory
        .recommend_patterns_attributed(episode_id, "implement retry with backoff", context, 3)
        .await
        .expect("attributed recommendation should succeed");

    // No backends configured -> MemoryOnly receipt.
    assert!(
        matches!(attr.receipt, PersistenceReceipt::MemoryOnly { .. }),
        "no backends must yield MemoryOnly, got: {:?}",
        attr.receipt
    );
    assert_eq!(attr.session.episode_id, episode_id);
    // The session must record exactly the IDs of the returned recommendations.
    let recommended: Vec<String> = attr
        .recommendations
        .iter()
        .map(|r| r.pattern.id().to_string())
        .collect();
    assert_eq!(attr.session.recommended_pattern_ids, recommended);

    // The session must be discoverable through the episode lookup.
    let found = memory
        .get_recommendation_session_for_episode(episode_id)
        .await
        .expect("recorded session must resolve by episode");
    assert_eq!(found.session_id, attr.session.session_id);
}

#[tokio::test]
async fn attributed_pattern_recommendation_receipt_persisted_with_all_backends() {
    let durable = ok_backend();
    let cache = ok_backend();
    let memory = with_backends(
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );
    let episode_id =
        create_completed_episode_with_pattern(&memory, PatternType::ErrorRecovery).await;
    let context = test_context();

    let attr = memory
        .recommend_patterns_attributed(episode_id, "implement retry with backoff", context, 3)
        .await
        .unwrap();

    assert!(
        matches!(attr.receipt, PersistenceReceipt::Persisted { .. }),
        "two healthy backends must yield Persisted, got: {:?}",
        attr.receipt
    );
    // The session must actually be in the durable backend.
    assert!(
        durable
            .sessions
            .lock()
            .unwrap()
            .contains_key(&attr.session.session_id)
    );
}

#[tokio::test]
async fn attributed_pattern_recommendation_receipt_partially_persisted_reports_in_order() {
    // Deterministic ordering: backends are tried turso-then-redb, so the
    // receipt must report exactly the failing backend(s) in try order.
    for (durable, cache, expected) in [
        (ok_backend(), failing_backend(), vec!["redb"]),
        (failing_backend(), ok_backend(), vec!["turso"]),
    ] {
        let memory = with_backends(
            Arc::clone(&durable) as Arc<dyn StorageBackend>,
            Arc::clone(&cache) as Arc<dyn StorageBackend>,
        );
        let episode_id =
            create_completed_episode_with_pattern(&memory, PatternType::ErrorRecovery).await;
        let context = test_context();

        let attr = memory
            .recommend_patterns_attributed(episode_id, "implement retry with backoff", context, 3)
            .await
            .unwrap();

        match attr.receipt {
            PersistenceReceipt::PartiallyPersisted {
                failed_backends, ..
            } => {
                assert_eq!(
                    failed_backends, expected,
                    "the failing backend must be reported in try order"
                );
            }
            other => panic!("one failing backend must yield PartiallyPersisted, got: {other:?}"),
        }
    }
}

#[tokio::test]
async fn attributed_pattern_recommendation_receipt_persistence_failed() {
    let durable = failing_backend();
    let cache = failing_backend();
    let memory = with_backends(
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );
    let episode_id =
        create_completed_episode_with_pattern(&memory, PatternType::ErrorRecovery).await;
    let context = test_context();

    let attr = memory
        .recommend_patterns_attributed(episode_id, "implement retry with backoff", context, 3)
        .await
        .unwrap();

    match attr.receipt {
        PersistenceReceipt::PersistenceFailed {
            failed_backends, ..
        } => {
            // Deterministic ordering: turso is tried first, then redb.
            assert_eq!(
                failed_backends,
                vec!["turso", "redb"],
                "both backends must be reported in try order"
            );
        }
        other => panic!("two failing backends must yield PersistenceFailed, got: {other:?}"),
    }
}

#[tokio::test]
async fn attributed_pattern_recommendation_non_advertising_backend_yields_memory_only() {
    // ADR-081 §2 AC-3: a configured backend that does not advertise
    // recommendation-attribution capability must never count toward a durable
    // receipt, even when its (no-op) write surface would succeed. Both sides
    // being InertBackend must therefore yield MemoryOnly, never Persisted,
    // PartiallyPersisted, or PersistenceFailed.
    let memory = with_backends(
        Arc::new(InertBackend) as Arc<dyn StorageBackend>,
        Arc::new(InertBackend) as Arc<dyn StorageBackend>,
    );
    let episode_id =
        create_completed_episode_with_pattern(&memory, PatternType::ErrorRecovery).await;
    let context = test_context();

    let attr = memory
        .recommend_patterns_attributed(episode_id, "implement retry with backoff", context, 3)
        .await
        .expect("attributed recommendation should succeed");

    assert!(
        matches!(attr.receipt, PersistenceReceipt::MemoryOnly { .. }),
        "non-advertising backends must yield MemoryOnly, got: {:?}",
        attr.receipt
    );
    assert_eq!(attr.session.episode_id, episode_id);
}

#[tokio::test]
async fn re_persisting_already_durable_session_is_noop() {
    // Recording the same session twice (e.g. a re-hydration or a caller retry)
    // must not duplicate storage entries nor grow the episode index, and the
    // second pass must still land in the durable backend.
    let durable = ok_backend();
    let memory = with_backends(
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        Arc::new(InertBackend) as Arc<dyn StorageBackend>,
    );
    let episode_id = Uuid::new_v4();
    let session = RecommendationSession {
        session_id: Uuid::new_v4(),
        episode_id,
        timestamp: chrono::Utc::now(),
        recommended_pattern_ids: vec!["pattern-a".to_string()],
        recommended_playbook_ids: vec![],
    };

    memory.record_recommendation_session(session.clone()).await;
    memory.record_recommendation_session(session.clone()).await;

    {
        let stored = durable.sessions.lock().unwrap();
        assert_eq!(
            stored.len(),
            1,
            "re-persisting the same session must not duplicate storage entries"
        );
        assert!(stored.contains_key(&session.session_id));
    }

    let found = memory
        .get_recommendation_session_for_episode(episode_id)
        .await
        .expect("episode lookup must resolve after duplicate recording");
    assert_eq!(
        found.session_id, session.session_id,
        "the episode index must not grow for duplicate recordings"
    );
}

#[tokio::test]
async fn attributed_pattern_session_survives_restart_when_durable() {
    // ADR-081 §1: a Persisted session must be resolvable by a cold tracker.
    let durable = ok_backend();
    let episode_id = Uuid::new_v4();
    let session_id;

    {
        let memory = with_backends(
            Arc::clone(&durable) as Arc<dyn StorageBackend>,
            Arc::new(InertBackend) as Arc<dyn StorageBackend>,
        );
        let session = RecommendationSession {
            session_id: Uuid::new_v4(),
            episode_id,
            timestamp: chrono::Utc::now(),
            recommended_pattern_ids: vec!["pattern-a".to_string()],
            recommended_playbook_ids: vec![],
        };
        session_id = session.session_id;
        memory.record_recommendation_session(session.clone()).await;
        // record_recommendation_session persists through the storage chain.
        assert!(
            durable.sessions.lock().unwrap().contains_key(&session_id),
            "the durable backend must hold the session after recording"
        );
    }

    // Cold tracker, same durable backend: feedback must still resolve.
    let restarted = with_backends(
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        Arc::new(InertBackend) as Arc<dyn StorageBackend>,
    );
    let feedback = RecommendationFeedback {
        session_id,
        applied_pattern_ids: vec!["pattern-a".to_string()],
        consulted_episode_ids: vec![],
        outcome: TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        },
        agent_rating: Some(0.9),
    };
    let result = restarted.record_recommendation_feedback(feedback).await;
    assert!(
        result.is_ok(),
        "feedback for a durable session must be accepted after restart, got: {:?}",
        result.err()
    );
}

// ============================================================================
// retrieve_playbooks_attributed
// ============================================================================

#[tokio::test]
async fn attributed_playbook_recommendation_rejects_nil_episode() {
    let memory = setup_test_memory();
    let context = test_context();

    let result = memory
        .retrieve_playbooks_attributed(AttributedPlaybookRequest {
            episode_id: Uuid::nil(),
            task_description: "read and validate config".to_string(),
            domain: "api-testing".to_string(),
            task_type: TaskType::CodeGeneration,
            context,
            max_playbooks: 1,
            max_steps_per_playbook: 10,
        })
        .await;

    assert!(
        result.is_err(),
        "a nil episode ID must be rejected by the attributed playbook path"
    );
}

#[tokio::test]
async fn attributed_playbook_recommendation_rejects_nonexistent_episode() {
    // ADR-080 §1: a well-formed but never-created episode UUID must be rejected
    // (InvalidInput) and must never produce a recommendation session.
    let memory = setup_test_memory();
    let context = test_context();
    let missing_episode = Uuid::new_v4();

    let result = memory
        .retrieve_playbooks_attributed(AttributedPlaybookRequest {
            episode_id: missing_episode,
            task_description: "read and validate config".to_string(),
            domain: "api-testing".to_string(),
            task_type: TaskType::CodeGeneration,
            context,
            max_playbooks: 1,
            max_steps_per_playbook: 10,
        })
        .await;

    assert!(
        result.is_err(),
        "a nonexistent episode must be rejected by the attributed playbook path"
    );
    assert!(
        memory
            .get_recommendation_session_for_episode(missing_episode)
            .await
            .is_none(),
        "a nonexistent episode must never create a recommendation session"
    );
}

#[tokio::test]
async fn attributed_playbook_recommendation_records_session() {
    let memory = setup_test_memory();
    let episode_id =
        create_completed_episode_with_pattern(&memory, PatternType::ToolSequence).await;
    let context = test_context();

    let attr = memory
        .retrieve_playbooks_attributed(AttributedPlaybookRequest {
            episode_id,
            task_description: "read and validate config".to_string(),
            domain: "api-testing".to_string(),
            task_type: TaskType::CodeGeneration,
            context,
            max_playbooks: 1,
            max_steps_per_playbook: 10,
        })
        .await
        .expect("attributed playbook retrieval should succeed");

    assert_eq!(attr.session.episode_id, episode_id);
    assert!(
        matches!(attr.receipt, PersistenceReceipt::MemoryOnly { .. }),
        "no backends must yield MemoryOnly, got: {:?}",
        attr.receipt
    );
    // Playbook generation may legitimately produce zero playbooks for a thin
    // store; the session and receipt contract must hold either way.
    assert_eq!(
        attr.session.recommended_playbook_ids.len(),
        attr.playbooks.len()
    );
    let found = memory
        .get_recommendation_session_for_episode(episode_id)
        .await
        .expect("recorded playbook session must resolve by episode");
    assert_eq!(found.session_id, attr.session.session_id);
}

#[tokio::test]
async fn attributed_playbook_recommendation_thin_store_still_records_session() {
    // ADR-080 §3: a successful generation never skips the session, even for a
    // thin store with no patterns/reflections. The wrapper records exactly the
    // returned playbook IDs (a future zero-playbook generation would therefore
    // record an empty session; today the template generator emits one playbook).
    let memory = setup_test_memory();
    let episode_id = memory
        .start_episode(
            "thin store playbook attribution".to_string(),
            test_context(),
            TaskType::CodeGeneration,
        )
        .await;

    let attr = memory
        .retrieve_playbooks_attributed(AttributedPlaybookRequest {
            episode_id,
            task_description: "read and validate config".to_string(),
            domain: "api-testing".to_string(),
            task_type: TaskType::CodeGeneration,
            context: test_context(),
            max_playbooks: 1,
            max_steps_per_playbook: 10,
        })
        .await
        .expect("attributed playbook retrieval on a thin store should succeed");

    assert_eq!(attr.session.episode_id, episode_id);
    // The session records exactly the returned playbook IDs; the wrapper must
    // never record a session for playbooks that were not returned.
    let returned_ids: Vec<uuid::Uuid> = attr.playbooks.iter().map(|p| p.playbook_id).collect();
    assert_eq!(attr.session.recommended_playbook_ids, returned_ids);

    let found = memory
        .get_recommendation_session_for_episode(episode_id)
        .await
        .expect("the thin-store attributed session must resolve by episode");
    assert_eq!(found.session_id, attr.session.session_id);
}

// ============================================================================
// Checked manual receipt matrix (ADR-081 AC-3/AC-4)
// ============================================================================

/// `record_recommendation_session_checked` must return the exact receipt state
/// for every combination of configured backends, using the stable backend IDs
/// `turso`/`redb` in try order.
#[tokio::test]
async fn checked_manual_session_receipt_matches_backend_combinations() {
    // No backends -> MemoryOnly.
    let memory = setup_test_memory();
    let receipt = memory
        .record_recommendation_session_checked(test_session(Uuid::new_v4()))
        .await;
    assert!(
        matches!(receipt, PersistenceReceipt::MemoryOnly { .. }),
        "no backends must yield MemoryOnly, got: {receipt:?}"
    );

    // Two healthy backends -> Persisted.
    let durable = ok_backend();
    let cache = ok_backend();
    let memory = with_backends(
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );
    let session = test_session(Uuid::new_v4());
    let session_id = session.session_id;
    let receipt = memory
        .record_recommendation_session_checked(session.clone())
        .await;
    assert!(
        matches!(receipt, PersistenceReceipt::Persisted { .. }),
        "two healthy backends must yield Persisted, got: {receipt:?}"
    );
    assert!(durable.sessions.lock().unwrap().contains_key(&session_id));

    // One failing backend -> PartiallyPersisted naming the failing backend in
    // try order (turso before redb).
    for (durable, cache, expected) in [
        (ok_backend(), failing_backend(), vec!["redb"]),
        (failing_backend(), ok_backend(), vec!["turso"]),
    ] {
        let memory = with_backends(
            Arc::clone(&durable) as Arc<dyn StorageBackend>,
            Arc::clone(&cache) as Arc<dyn StorageBackend>,
        );
        let receipt = memory
            .record_recommendation_session_checked(test_session(Uuid::new_v4()))
            .await;
        match receipt {
            PersistenceReceipt::PartiallyPersisted {
                failed_backends, ..
            } => assert_eq!(
                failed_backends, expected,
                "one failing backend must be reported in try order"
            ),
            other => panic!("one failing backend must yield PartiallyPersisted, got: {other:?}"),
        }
    }

    // Both failing -> PersistenceFailed listing both backends in try order.
    let memory = with_backends(
        failing_backend() as Arc<dyn StorageBackend>,
        failing_backend() as Arc<dyn StorageBackend>,
    );
    let receipt = memory
        .record_recommendation_session_checked(test_session(Uuid::new_v4()))
        .await;
    match receipt {
        PersistenceReceipt::PersistenceFailed {
            failed_backends, ..
        } => assert_eq!(
            failed_backends,
            vec!["turso", "redb"],
            "both failing backends must be reported in try order"
        ),
        other => panic!("two failing backends must yield PersistenceFailed, got: {other:?}"),
    }

    // Non-advertising backends -> MemoryOnly, never Persisted (ADR-081 §2).
    let memory = with_backends(
        Arc::new(InertBackend) as Arc<dyn StorageBackend>,
        Arc::new(InertBackend) as Arc<dyn StorageBackend>,
    );
    let receipt = memory
        .record_recommendation_session_checked(test_session(Uuid::new_v4()))
        .await;
    assert!(
        matches!(receipt, PersistenceReceipt::MemoryOnly { .. }),
        "non-advertising backends must yield MemoryOnly, got: {receipt:?}"
    );
}

/// `record_recommendation_feedback_checked` returns the exact receipt state for
/// every backend combination after a valid session exists, and still rejects an
/// unknown session with an error.
#[tokio::test]
async fn checked_manual_feedback_receipt_matches_backend_combinations() {
    // Unknown session -> Err, regardless of backend health.
    let memory = setup_test_memory();
    let result = memory
        .record_recommendation_feedback_checked(test_feedback(Uuid::new_v4()))
        .await;
    assert!(
        result.is_err(),
        "feedback for an unknown session must be rejected by the checked path"
    );

    // No backends -> MemoryOnly.
    let memory = setup_test_memory();
    let session = test_session(Uuid::new_v4());
    memory
        .record_recommendation_session_checked(session.clone())
        .await;
    let receipt = memory
        .record_recommendation_feedback_checked(test_feedback(session.session_id))
        .await
        .expect("feedback for a tracked session must be accepted");
    assert!(
        matches!(receipt, PersistenceReceipt::MemoryOnly { .. }),
        "no backends must yield MemoryOnly for feedback, got: {receipt:?}"
    );

    // Two healthy backends -> Persisted.
    let durable = ok_backend();
    let cache = ok_backend();
    let memory = with_backends(
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );
    let session = test_session(Uuid::new_v4());
    memory
        .record_recommendation_session_checked(session.clone())
        .await;
    let receipt = memory
        .record_recommendation_feedback_checked(test_feedback(session.session_id))
        .await
        .expect("feedback for a tracked session must be accepted");
    assert!(
        matches!(receipt, PersistenceReceipt::Persisted { .. }),
        "two healthy backends must yield Persisted for feedback, got: {receipt:?}"
    );

    // One failing backend -> PartiallyPersisted naming it in try order.
    for (durable, cache, expected) in [
        (ok_backend(), failing_backend(), vec!["redb"]),
        (failing_backend(), ok_backend(), vec!["turso"]),
    ] {
        let memory = with_backends(
            Arc::clone(&durable) as Arc<dyn StorageBackend>,
            Arc::clone(&cache) as Arc<dyn StorageBackend>,
        );
        let session = test_session(Uuid::new_v4());
        memory
            .record_recommendation_session_checked(session.clone())
            .await;
        let receipt = memory
            .record_recommendation_feedback_checked(test_feedback(session.session_id))
            .await
            .expect("feedback for a tracked session must be accepted");
        match receipt {
            PersistenceReceipt::PartiallyPersisted {
                failed_backends, ..
            } => assert_eq!(
                failed_backends, expected,
                "the failing feedback backend must be reported in try order"
            ),
            other => panic!("one failing backend must yield PartiallyPersisted, got: {other:?}"),
        }
    }

    // Both failing -> PersistenceFailed listing both backends in try order.
    let memory = with_backends(
        failing_backend() as Arc<dyn StorageBackend>,
        failing_backend() as Arc<dyn StorageBackend>,
    );
    let session = test_session(Uuid::new_v4());
    memory
        .record_recommendation_session_checked(session.clone())
        .await;
    let receipt = memory
        .record_recommendation_feedback_checked(test_feedback(session.session_id))
        .await
        .expect("feedback for a tracked session must be accepted");
    match receipt {
        PersistenceReceipt::PersistenceFailed {
            failed_backends, ..
        } => assert_eq!(
            failed_backends,
            vec!["turso", "redb"],
            "both failing feedback backends must be reported in try order"
        ),
        other => panic!("two failing backends must yield PersistenceFailed, got: {other:?}"),
    }

    // Non-advertising backends -> MemoryOnly.
    let memory = with_backends(
        Arc::new(InertBackend) as Arc<dyn StorageBackend>,
        Arc::new(InertBackend) as Arc<dyn StorageBackend>,
    );
    let session = test_session(Uuid::new_v4());
    memory
        .record_recommendation_session_checked(session.clone())
        .await;
    let receipt = memory
        .record_recommendation_feedback_checked(test_feedback(session.session_id))
        .await
        .expect("feedback for a tracked session must be accepted");
    assert!(
        matches!(receipt, PersistenceReceipt::MemoryOnly { .. }),
        "non-advertising backends must yield MemoryOnly for feedback, got: {receipt:?}"
    );
}

/// A successful empty recommendation (a fresh store with no patterns) must
/// still record an attributed session with empty recommended IDs (ADR-080 §3).
#[tokio::test]
async fn attributed_pattern_recommendation_empty_recommendations_still_record_session() {
    let memory = setup_test_memory();
    let episode_id = memory
        .start_episode(
            "empty-store attribution".to_string(),
            test_context(),
            TaskType::CodeGeneration,
        )
        .await;

    let attr = memory
        .recommend_patterns_attributed(episode_id, "no patterns stored", test_context(), 3)
        .await
        .expect("an empty recommendation must still succeed");

    assert!(
        attr.recommendations.is_empty(),
        "a store with no patterns must recommend nothing"
    );
    assert_eq!(attr.session.recommended_pattern_ids, Vec::<String>::new());
    assert_eq!(attr.session.episode_id, episode_id);

    let found = memory
        .get_recommendation_session_for_episode(episode_id)
        .await
        .expect("the empty attributed session must still be recorded");
    assert_eq!(found.session_id, attr.session.session_id);
}

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
        .retrieve_playbooks_attributed(
            Uuid::nil(),
            "read and validate config",
            "api-testing",
            TaskType::CodeGeneration,
            context,
            1,
            10,
        )
        .await;

    assert!(
        result.is_err(),
        "a nil episode ID must be rejected by the attributed playbook path"
    );
}

#[tokio::test]
async fn attributed_playbook_recommendation_records_session() {
    let memory = setup_test_memory();
    let episode_id =
        create_completed_episode_with_pattern(&memory, PatternType::ToolSequence).await;
    let context = test_context();

    let attr = memory
        .retrieve_playbooks_attributed(
            episode_id,
            "read and validate config",
            "api-testing",
            TaskType::CodeGeneration,
            context,
            1,
            10,
        )
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

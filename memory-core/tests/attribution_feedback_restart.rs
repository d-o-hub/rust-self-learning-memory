//! ADR-081 §1: feedback must resolve its session through memory → storage.
//!
//! The regression these tests guard: ADR-080 §4 made an unknown session a hard
//! error, but session resolution only consulted the in-process tracker. A session
//! that was durably persisted before a restart was therefore unresolvable by a
//! cold tracker, and feedback that used to succeed began failing.

#![allow(clippy::unwrap_used, clippy::panic)] // test-only mock backends

use async_trait::async_trait;
use chrono::Utc;
use do_memory_core::episode::PatternId;
use do_memory_core::memory::attribution::{RecommendationFeedback, RecommendationSession};
use do_memory_core::storage::StorageBackend;
use do_memory_core::{
    Episode, Heuristic, MemoryConfig, Pattern, Result, SelfLearningMemory, TaskOutcome,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Backend that durably stores recommendation sessions and feedback in memory.
///
/// Sharing one instance across two `SelfLearningMemory` values simulates a restart:
/// the storage survives, the in-process tracker does not.
#[derive(Default)]
struct SessionStoringBackend {
    sessions: Mutex<HashMap<Uuid, RecommendationSession>>,
    feedback: Mutex<HashMap<Uuid, RecommendationFeedback>>,
}

/// Backend that stores nothing, to isolate the resolution chain to one side.
#[derive(Default)]
struct InertBackend;

#[async_trait]
impl StorageBackend for SessionStoringBackend {
    async fn store_recommendation_session(&self, session: &RecommendationSession) -> Result<()> {
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
        _since: chrono::DateTime<Utc>,
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
        _since: chrono::DateTime<Utc>,
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

fn test_session(episode_id: Uuid) -> RecommendationSession {
    RecommendationSession {
        session_id: Uuid::new_v4(),
        episode_id,
        timestamp: Utc::now(),
        recommended_pattern_ids: vec!["pattern-a".to_string(), "pattern-b".to_string()],
        recommended_playbook_ids: vec![],
    }
}

fn test_feedback(session_id: Uuid, applied: Vec<&str>) -> RecommendationFeedback {
    RecommendationFeedback {
        session_id,
        applied_pattern_ids: applied.into_iter().map(String::from).collect(),
        consulted_episode_ids: vec![],
        outcome: TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        },
        agent_rating: Some(0.9),
    }
}

/// The regression itself: a session persisted before restart must accept feedback
/// after restart, when the in-memory tracker is cold.
#[tokio::test]
async fn feedback_accepted_after_restart_via_durable_backend() {
    let durable = Arc::new(SessionStoringBackend::default());
    let session = test_session(Uuid::new_v4());
    let session_id = session.session_id;

    // First process: record the session, which persists it to the shared backend.
    {
        let memory = SelfLearningMemory::with_storage(
            MemoryConfig::default(),
            Arc::clone(&durable) as Arc<dyn StorageBackend>,
            Arc::new(InertBackend) as Arc<dyn StorageBackend>,
        );
        memory.record_recommendation_session(session.clone()).await;
    }

    // Second process: cold tracker, same durable storage.
    let restarted = SelfLearningMemory::with_storage(
        MemoryConfig::default(),
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        Arc::new(InertBackend) as Arc<dyn StorageBackend>,
    );

    let result = restarted
        .record_recommendation_feedback(test_feedback(session_id, vec!["pattern-a"]))
        .await;

    assert!(
        result.is_ok(),
        "feedback for a durably persisted session must be accepted after restart, got: {:?}",
        result.err()
    );
}

/// The same guarantee when only the cache backend holds the session.
#[tokio::test]
async fn feedback_accepted_after_restart_via_cache_backend() {
    let cache = Arc::new(SessionStoringBackend::default());
    let session = test_session(Uuid::new_v4());
    let session_id = session.session_id;

    {
        let memory = SelfLearningMemory::with_storage(
            MemoryConfig::default(),
            Arc::new(InertBackend) as Arc<dyn StorageBackend>,
            Arc::clone(&cache) as Arc<dyn StorageBackend>,
        );
        memory.record_recommendation_session(session.clone()).await;
    }

    let restarted = SelfLearningMemory::with_storage(
        MemoryConfig::default(),
        Arc::new(InertBackend) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );

    let result = restarted
        .record_recommendation_feedback(test_feedback(session_id, vec!["pattern-b"]))
        .await;

    assert!(
        result.is_ok(),
        "feedback resolved from the cache backend must be accepted, got: {:?}",
        result.err()
    );
}

/// ADR-081 AC-2: rejection is retained for a session that exists nowhere.
#[tokio::test]
async fn feedback_rejected_when_session_exists_in_no_backend() {
    let durable = Arc::new(SessionStoringBackend::default());
    let memory = SelfLearningMemory::with_storage(
        MemoryConfig::default(),
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        Arc::new(InertBackend) as Arc<dyn StorageBackend>,
    );

    let result = memory
        .record_recommendation_feedback(test_feedback(Uuid::new_v4(), vec![]))
        .await;

    assert!(
        result.is_err(),
        "feedback for a session in no tracker and no backend must be rejected"
    );
}

/// Integrity survives storage resolution: applied IDs are still checked against
/// the recommended set of the session loaded from storage.
#[tokio::test]
async fn non_recommended_applied_id_rejected_after_storage_resolution() {
    let durable = Arc::new(SessionStoringBackend::default());
    let session = test_session(Uuid::new_v4());
    let session_id = session.session_id;

    {
        let memory = SelfLearningMemory::with_storage(
            MemoryConfig::default(),
            Arc::clone(&durable) as Arc<dyn StorageBackend>,
            Arc::new(InertBackend) as Arc<dyn StorageBackend>,
        );
        memory.record_recommendation_session(session.clone()).await;
    }

    let restarted = SelfLearningMemory::with_storage(
        MemoryConfig::default(),
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        Arc::new(InertBackend) as Arc<dyn StorageBackend>,
    );

    let result = restarted
        .record_recommendation_feedback(test_feedback(session_id, vec!["never-recommended"]))
        .await;

    assert!(
        result.is_err(),
        "an applied ID outside the recommended set must be rejected even when the session came from storage"
    );
}

/// Repeated feedback submissions each re-resolve the same session from storage.
/// The latest-session lookup must stay correct and stable across them.
#[tokio::test]
async fn repeated_hydration_keeps_episode_lookup_stable() {
    let durable = Arc::new(SessionStoringBackend::default());
    let episode_id = Uuid::new_v4();
    let session = test_session(episode_id);
    let session_id = session.session_id;

    {
        let memory = SelfLearningMemory::with_storage(
            MemoryConfig::default(),
            Arc::clone(&durable) as Arc<dyn StorageBackend>,
            Arc::new(InertBackend) as Arc<dyn StorageBackend>,
        );
        memory.record_recommendation_session(session.clone()).await;
    }

    let restarted = SelfLearningMemory::with_storage(
        MemoryConfig::default(),
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        Arc::new(InertBackend) as Arc<dyn StorageBackend>,
    );

    // Three feedback submissions, each hydrating the same session.
    for pattern in ["pattern-a", "pattern-b", "pattern-a"] {
        restarted
            .record_recommendation_feedback(test_feedback(session_id, vec![pattern]))
            .await
            .unwrap();
    }

    let latest = restarted
        .get_recommendation_session_for_episode(episode_id)
        .await
        .expect("episode lookup must resolve after repeated hydration");

    assert_eq!(
        latest.session_id, session_id,
        "repeated hydration must not disturb the latest-session lookup"
    );
}

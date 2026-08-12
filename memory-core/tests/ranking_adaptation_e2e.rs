//! End-to-end feedback-to-ranking adaptation (ADR-082).
//!
//! Exercises the public `SelfLearningMemory` surface end to end:
//! - a success feedback lifts the applied pattern's recommendation rank,
//! - replacement feedback swings the rank back,
//! - a cold restart rebuilds the learned index from durable storage,
//! - a non-capable backend contributes nothing (stale durable rows ignored).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)] // test fixtures

use async_trait::async_trait;
use chrono::Utc;
use do_memory_core::episode::PatternId;
use do_memory_core::memory::attribution::{RecommendationFeedback, RecommendationSession};
use do_memory_core::storage::StorageBackend;
use do_memory_core::{
    ComplexityLevel, Episode, Heuristic, MemoryConfig, Pattern, Result, SelfLearningMemory,
    TaskContext, TaskOutcome, TaskType,
};
use do_memory_storage_redb::RedbStorage;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

fn create_test_pattern(id: Uuid, success_rate: f32) -> Pattern {
    Pattern::ToolSequence {
        id,
        tools: vec!["tool1".to_string(), "tool2".to_string()],
        context: TaskContext {
            domain: "web-api".to_string(),
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            tags: vec!["rust".to_string()],
        },
        success_rate,
        avg_latency: chrono::Duration::milliseconds(100),
        occurrence_count: 5,
        effectiveness: do_memory_core::patterns::PatternEffectiveness::new(),
    }
}

fn recommend_context() -> TaskContext {
    TaskContext {
        domain: "web-api".to_string(),
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Moderate,
        tags: vec!["rust".to_string()],
    }
}

async fn seed_patterns(memory: &SelfLearningMemory, patterns: &[Pattern]) {
    let mut map = memory.patterns_fallback().write().await;
    for p in patterns {
        map.insert(p.id(), p.clone());
    }
}

fn session_for(episode_id: Uuid, pattern: &Pattern) -> RecommendationSession {
    RecommendationSession {
        session_id: Uuid::new_v4(),
        episode_id,
        timestamp: Utc::now(),
        recommended_pattern_ids: vec![pattern.id().to_string()],
        recommended_playbook_ids: vec![],
    }
}

fn feedback_for(
    session_id: Uuid,
    pattern: &Pattern,
    outcome: TaskOutcome,
) -> RecommendationFeedback {
    RecommendationFeedback {
        session_id,
        applied_pattern_ids: vec![pattern.id().to_string()],
        consulted_episode_ids: vec![],
        outcome,
        agent_rating: Some(0.9),
    }
}

/// Non-capable backend that still stores attribution rows (legacy shape). It
/// advertises neither recommendation-attribution nor ranking-adaptation, so the
/// derived index must never consult its durable rows.
#[derive(Default)]
struct LegacyOnlyBackend {
    sessions: Mutex<HashMap<Uuid, RecommendationSession>>,
    feedback: Mutex<HashMap<Uuid, RecommendationFeedback>>,
}

#[async_trait]
impl StorageBackend for LegacyOnlyBackend {
    async fn store_recommendation_session(&self, session: &RecommendationSession) -> Result<()> {
        self.sessions
            .lock()
            .insert(session.session_id, session.clone());
        Ok(())
    }
    async fn get_recommendation_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<RecommendationSession>> {
        Ok(self.sessions.lock().get(&session_id).cloned())
    }
    async fn store_recommendation_feedback(&self, feedback: &RecommendationFeedback) -> Result<()> {
        self.feedback
            .lock()
            .insert(feedback.session_id, feedback.clone());
        Ok(())
    }
    async fn get_recommendation_feedback(
        &self,
        session_id: Uuid,
    ) -> Result<Option<RecommendationFeedback>> {
        Ok(self.feedback.lock().get(&session_id).cloned())
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

/// Baseline guard: with no feedback the learned re-rank is a no-op, so the plain
/// relevance order (`p_high` first) is preserved exactly.
#[tokio::test]
async fn default_order_unchanged_without_feedback() {
    let memory = SelfLearningMemory::new();
    let p_high = create_test_pattern(Uuid::new_v4(), 0.9);
    let p_low = create_test_pattern(Uuid::new_v4(), 0.1);
    seed_patterns(&memory, &[p_high.clone(), p_low.clone()]).await;

    let results = memory
        .recommend_patterns_for_task("Build an async REST API", recommend_context(), 2)
        .await
        .unwrap();

    assert_eq!(
        results.len(),
        2,
        "both domain-matching patterns must be recommended"
    );
    assert_eq!(
        results[0].pattern.id(),
        p_high.id(),
        "baseline: p_high (higher success rate) must rank first"
    );
    assert_eq!(
        results[1].pattern.id(),
        p_low.id(),
        "baseline: p_low must rank second"
    );
}

/// Success feedback for a session recommending only `p_low` must lift `p_low` to the
/// top of the next recommendation.
#[tokio::test]
async fn success_feedback_lifts_rank() {
    let memory = SelfLearningMemory::new();
    let p_high = create_test_pattern(Uuid::new_v4(), 0.9);
    let p_low = create_test_pattern(Uuid::new_v4(), 0.1);
    seed_patterns(&memory, &[p_high.clone(), p_low.clone()]).await;

    let episode_id = memory
        .start_episode(
            "Build an async REST API".to_string(),
            recommend_context(),
            TaskType::CodeGeneration,
        )
        .await;

    let session = session_for(episode_id, &p_low);
    let session_id = session.session_id;
    memory
        .record_recommendation_session_checked(session.clone())
        .await;

    memory
        .record_recommendation_feedback(feedback_for(
            session_id,
            &p_low,
            TaskOutcome::Success {
                verdict: "REST API built".to_string(),
                artifacts: vec![],
            },
        ))
        .await
        .unwrap();

    let results = memory
        .recommend_patterns_for_task("Build an async REST API", recommend_context(), 2)
        .await
        .unwrap();

    assert_eq!(
        results[0].pattern.id(),
        p_low.id(),
        "success feedback must lift p_low above p_high"
    );
    assert_eq!(
        results[1].pattern.id(),
        p_high.id(),
        "boosted p_low now leads, p_high follows"
    );
}

/// Replacing the same session's feedback with a Failure removes the boost: the
/// order returns toward the baseline.
#[tokio::test]
async fn replacement_feedback_swings_rank() {
    let memory = SelfLearningMemory::new();
    let p_high = create_test_pattern(Uuid::new_v4(), 0.9);
    let p_low = create_test_pattern(Uuid::new_v4(), 0.1);
    seed_patterns(&memory, &[p_high.clone(), p_low.clone()]).await;

    let episode_id = memory
        .start_episode(
            "Build an async REST API".to_string(),
            recommend_context(),
            TaskType::CodeGeneration,
        )
        .await;
    let session = session_for(episode_id, &p_low);
    let session_id = session.session_id;
    memory
        .record_recommendation_session_checked(session.clone())
        .await;

    // First a success, then a replacement failure for the same session.
    memory
        .record_recommendation_feedback(feedback_for(
            session_id,
            &p_low,
            TaskOutcome::Success {
                verdict: "OK".to_string(),
                artifacts: vec![],
            },
        ))
        .await
        .unwrap();

    let after_success = memory
        .recommend_patterns_for_task("Build an async REST API", recommend_context(), 2)
        .await
        .unwrap();
    assert_eq!(
        after_success[0].pattern.id(),
        p_low.id(),
        "precondition: success feedback lifts p_low"
    );

    memory
        .record_recommendation_feedback(feedback_for(
            session_id,
            &p_low,
            TaskOutcome::Failure {
                reason: "approach regressed".to_string(),
                error_details: None,
            },
        ))
        .await
        .unwrap();

    let after_failure = memory
        .recommend_patterns_for_task("Build an async REST API", recommend_context(), 2)
        .await
        .unwrap();
    assert_eq!(
        after_failure[0].pattern.id(),
        p_high.id(),
        "replacement Failure must drop p_low back below p_high"
    );
}

/// A cold restart must rebuild the learned index from the durable backend's
/// `list_recommendation_*` surface: success feedback persisted before the
/// restart still lifts `p_low` after reopening the same redb.
#[tokio::test]
async fn cold_restart_rebuilds_from_storage() {
    let dir = tempfile::TempDir::new().unwrap();
    let db_path = dir.path().join("ranking.redb");
    let p_high = create_test_pattern(Uuid::new_v4(), 0.9);
    let p_low = create_test_pattern(Uuid::new_v4(), 0.1);

    // First process: durable redb, session + success feedback persisted.
    {
        let redb = Arc::new(RedbStorage::new(&db_path).await.unwrap());
        let memory = SelfLearningMemory::with_storage(
            MemoryConfig::default(),
            Arc::clone(&redb) as Arc<dyn StorageBackend>,
            Arc::clone(&redb) as Arc<dyn StorageBackend>,
        );
        let episode_id = memory
            .start_episode(
                "Build an async REST API".to_string(),
                recommend_context(),
                TaskType::CodeGeneration,
            )
            .await;
        let session = session_for(episode_id, &p_low);
        let session_id = session.session_id;
        memory
            .record_recommendation_session_checked(session.clone())
            .await;
        memory
            .record_recommendation_feedback(feedback_for(
                session_id,
                &p_low,
                TaskOutcome::Success {
                    verdict: "REST API built".to_string(),
                    artifacts: vec![],
                },
            ))
            .await
            .unwrap();
    }

    // Second process: fresh memory, same redb file, cold tracker.
    let redb2 = Arc::new(RedbStorage::new(&db_path).await.unwrap());
    let restarted = SelfLearningMemory::with_storage(
        MemoryConfig::default(),
        Arc::clone(&redb2) as Arc<dyn StorageBackend>,
        Arc::clone(&redb2) as Arc<dyn StorageBackend>,
    );
    seed_patterns(&restarted, &[p_high.clone(), p_low.clone()]).await;

    let results = restarted
        .recommend_patterns_for_task("Build an async REST API", recommend_context(), 2)
        .await
        .unwrap();

    assert_eq!(
        results[0].pattern.id(),
        p_low.id(),
        "learned index must be rebuilt from durable storage after a cold restart"
    );
}

/// Non-capable backends contribute nothing: stale attribution rows in a legacy
/// backend are ignored, so a cold restart sees an empty learned index.
#[tokio::test]
async fn non_capable_backend_ignored_after_cold_restart() {
    let legacy = Arc::new(LegacyOnlyBackend::default());
    let p_high = create_test_pattern(Uuid::new_v4(), 0.9);
    let p_low = create_test_pattern(Uuid::new_v4(), 0.1);

    // First process records a session + success feedback, which the legacy
    // backend stores directly (it does not advertise ranking adaptation).
    let session = session_for(Uuid::new_v4(), &p_low);
    let session_id = session.session_id;
    StorageBackend::store_recommendation_session(&*legacy, &session)
        .await
        .unwrap();
    StorageBackend::store_recommendation_feedback(
        &*legacy,
        &feedback_for(
            session_id,
            &p_low,
            TaskOutcome::Success {
                verdict: "OK".to_string(),
                artifacts: vec![],
            },
        ),
    )
    .await
    .unwrap();

    // Cold restart on the same (non-capable) backend: durable rows exist but the
    // capability gate must keep them out of the learned index.
    let restarted = SelfLearningMemory::with_storage(
        MemoryConfig::default(),
        Arc::clone(&legacy) as Arc<dyn StorageBackend>,
        Arc::clone(&legacy) as Arc<dyn StorageBackend>,
    );
    seed_patterns(&restarted, &[p_high.clone(), p_low.clone()]).await;

    assert!(
        !legacy.supports_ranking_adaptation(),
        "legacy backend must not advertise ranking adaptation"
    );

    let results = restarted
        .recommend_patterns_for_task("Build an async REST API", recommend_context(), 2)
        .await
        .unwrap();

    assert_eq!(
        results[0].pattern.id(),
        p_high.id(),
        "non-capable backend data must not lift p_low after a cold restart"
    );
}

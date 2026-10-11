//! Coverage for `StorageBackend` default method bodies (Codecov patch).

use super::{StorageBackend, StorageBackendCapabilities};
use crate::episode::{
    Direction, EpisodePatternRelationship, EpisodeRelationship, EpisodeRetentionPolicy,
    RelationshipMetadata, RelationshipType,
};
use crate::memory::attribution::{RecommendationFeedback, RecommendationSession};
use crate::procedural::ProceduralMemory;
use crate::{
    Episode, Error, Heuristic, Pattern, PatternId, Result, TaskContext, TaskOutcome, TaskType,
};
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

/// Minimal backend: only required methods; defaults exercise trait default bodies.
struct StubBackend;

#[async_trait]
impl StorageBackend for StubBackend {
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
        Ok(false)
    }
    async fn store_embeddings_batch(&self, _embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        Ok(())
    }
    async fn get_embeddings_batch(&self, _ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        Ok(vec![])
    }
}

/// The stub implements no optional operation, so every predicate stays `false`.
impl StorageBackendCapabilities for StubBackend {}

fn sample_relationship() -> EpisodeRelationship {
    EpisodeRelationship::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        RelationshipType::RelatedTo,
        RelationshipMetadata::default(),
    )
}

fn sample_session() -> RecommendationSession {
    RecommendationSession {
        session_id: Uuid::new_v4(),
        episode_id: Uuid::new_v4(),
        timestamp: Utc::now(),
        recommended_pattern_ids: vec!["p1".to_string()],
        recommended_playbook_ids: vec![],
    }
}

fn sample_feedback(session_id: Uuid) -> RecommendationFeedback {
    RecommendationFeedback {
        session_id,
        applied_pattern_ids: vec![],
        consulted_episode_ids: vec![],
        outcome: TaskOutcome::Success {
            verdict: "ok".to_string(),
            artifacts: vec![],
        },
        agent_rating: None,
    }
}

fn sample_pattern_rel() -> EpisodePatternRelationship {
    EpisodePatternRelationship::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        RelationshipType::RelatedTo,
        RelationshipMetadata::default(),
    )
}

fn sample_procedural() -> ProceduralMemory {
    use crate::patterns::PatternEffectiveness;
    ProceduralMemory {
        id: Uuid::new_v4(),
        name: "stub".to_string(),
        description: "test".to_string(),
        context: TaskContext::default(),
        steps: vec![],
        effectiveness: PatternEffectiveness::default(),
        source_episodes: vec![],
        source_patterns: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[tokio::test]
async fn storage_backend_default_episode_batch_loops_singles() {
    let backend = StubBackend;

    // Empty batch is a no-op success.
    backend.store_episodes_batch(&[]).await.unwrap();

    // Default body delegates to store_episode per item.
    let episodes = vec![
        Episode::new("one".into(), TaskContext::default(), TaskType::Testing),
        Episode::new("two".into(), TaskContext::default(), TaskType::Testing),
    ];
    backend.store_episodes_batch(&episodes).await.unwrap();
}

/// Assert that an optional default returns the typed capability error naming
/// `operation` (#1087: no default may report durable success).
fn assert_capability_unavailable<T: std::fmt::Debug>(operation: &'static str, result: Result<T>) {
    match result {
        Err(Error::CapabilityUnavailable { operation: got }) => {
            assert_eq!(got, operation, "wrong operation reported");
        }
        Err(other) => panic!("expected CapabilityUnavailable for {operation}, got {other}"),
        Ok(value) => panic!("expected CapabilityUnavailable for {operation}, got {value:?}"),
    }
}

#[tokio::test]
async fn storage_backend_defaults_gate_optional_operations() {
    let backend = StubBackend;
    let id = Uuid::new_v4();
    let rel = sample_relationship();
    let session = sample_session();
    let feedback = sample_feedback(session.session_id);
    let pattern_rel = sample_pattern_rel();
    let policy = EpisodeRetentionPolicy::default();
    let procedural = sample_procedural();

    // A backend that implements none of the optional operations advertises none.
    assert!(!backend.supports_recommendation_attribution());
    assert!(!backend.supports_ranking_adaptation());
    assert!(!backend.supports_episode_cleanup());
    assert!(!backend.supports_relationship_persistence());
    assert!(!backend.supports_procedural_memory());

    // Intentional, documented fallbacks: pattern listing and the ADR-081
    // recommendation defaults (capability-gated by their callers).
    assert_eq!(backend.get_all_patterns().await.unwrap().len(), 0);
    backend
        .store_recommendation_session(&session)
        .await
        .unwrap();
    assert!(
        backend
            .get_recommendation_session(session.session_id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        backend
            .get_recommendation_session_for_episode(session.episode_id)
            .await
            .unwrap()
            .is_none()
    );
    backend
        .store_recommendation_feedback(&feedback)
        .await
        .unwrap();
    assert!(
        backend
            .get_recommendation_feedback(session.session_id)
            .await
            .unwrap()
            .is_none()
    );
    let _stats = backend.get_recommendation_stats().await.unwrap();

    // #1087 slices 1-3: relationship, cleanup, and procedural defaults return
    // `Error::CapabilityUnavailable` instead of `Ok(())`, `Ok(None)`, or
    // empty vectors that could pass for durable success.
    assert_capability_unavailable("store_relationship", backend.store_relationship(&rel).await);
    assert_capability_unavailable(
        "remove_relationship",
        backend.remove_relationship(rel.id).await,
    );
    assert_capability_unavailable(
        "get_relationships",
        backend.get_relationships(id, Direction::Both).await,
    );
    assert_capability_unavailable(
        "get_all_relationships",
        backend.get_all_relationships().await,
    );
    assert_capability_unavailable(
        "get_relationship_by_id",
        backend.get_relationship_by_id(rel.id).await,
    );
    assert_capability_unavailable(
        "relationship_exists",
        backend
            .relationship_exists(id, id, RelationshipType::RelatedTo)
            .await,
    );
    assert_capability_unavailable(
        "store_episode_pattern_relationship",
        backend
            .store_episode_pattern_relationship(&pattern_rel)
            .await,
    );
    assert_capability_unavailable(
        "get_episode_pattern_relationships",
        backend.get_episode_pattern_relationships(id).await,
    );
    assert_capability_unavailable(
        "get_weighted_neighbors",
        backend.get_weighted_neighbors(id).await,
    );
    assert_capability_unavailable("cleanup_episodes", backend.cleanup_episodes(&policy).await);
    assert_capability_unavailable(
        "count_cleanup_candidates",
        backend.count_cleanup_candidates(&policy).await,
    );
    assert_capability_unavailable(
        "store_procedural",
        backend.store_procedural(&procedural).await,
    );
    assert_capability_unavailable(
        "get_procedural",
        backend.get_procedural(procedural.id).await,
    );
    assert_capability_unavailable(
        "delete_procedural",
        backend.delete_procedural(procedural.id).await,
    );
    assert_capability_unavailable("query_procedural", backend.query_procedural(Some(10)).await);

    // Keep Episode/TaskType referenced so stub stays honest for required path
    let _ep = Episode::new("stub".into(), TaskContext::default(), TaskType::Testing);
}

/// The default liveness probe runs through the backend's own reads, so a backend that answers
/// `get_episode` is healthy and one that does not is not (`#1085`).
#[tokio::test]
async fn storage_backend_default_health_check_reads_through_the_backend() {
    let backend = StubBackend;
    backend
        .health_check()
        .await
        .expect("a backend whose required read succeeds must pass the default probe");
}

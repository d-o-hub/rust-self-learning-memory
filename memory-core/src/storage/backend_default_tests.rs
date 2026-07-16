//! Coverage for `StorageBackend` default method bodies (Codecov patch).

use super::StorageBackend;
use crate::episode::{
    Direction, EpisodePatternRelationship, EpisodeRelationship, EpisodeRetentionPolicy,
    RelationshipMetadata, RelationshipType,
};
use crate::memory::attribution::{RecommendationFeedback, RecommendationSession};
use crate::procedural::ProceduralMemory;
use crate::{Episode, Heuristic, Pattern, PatternId, Result, TaskContext, TaskOutcome, TaskType};
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
async fn storage_backend_default_methods_return_empty_success() {
    let backend = StubBackend;
    let id = Uuid::new_v4();
    let rel = sample_relationship();
    let session = sample_session();
    let feedback = sample_feedback(session.session_id);
    let pattern_rel = sample_pattern_rel();
    let policy = EpisodeRetentionPolicy::default();
    let procedural = sample_procedural();

    // Defaults with real bodies (Codecov targets)
    assert!(backend.get_all_patterns().await.unwrap().is_empty());
    backend.store_relationship(&rel).await.unwrap();
    backend.remove_relationship(rel.id).await.unwrap();
    assert!(
        backend
            .get_relationships(id, Direction::Both)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(backend.get_all_relationships().await.unwrap().is_empty());
    assert!(
        backend
            .get_relationship_by_id(rel.id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        !backend
            .relationship_exists(id, id, RelationshipType::RelatedTo)
            .await
            .unwrap()
    );
    backend
        .store_episode_pattern_relationship(&pattern_rel)
        .await
        .unwrap();
    assert!(
        backend
            .get_episode_pattern_relationships(id)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(backend.get_weighted_neighbors(id).await.unwrap().is_empty());

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

    let cleanup = backend.cleanup_episodes(&policy).await.unwrap();
    assert_eq!(cleanup.deleted, 0);
    assert_eq!(backend.count_cleanup_candidates(&policy).await.unwrap(), 0);

    backend.store_procedural(&procedural).await.unwrap();
    assert!(
        backend
            .get_procedural(procedural.id)
            .await
            .unwrap()
            .is_none()
    );
    backend.delete_procedural(procedural.id).await.unwrap();
    assert!(backend.query_procedural(Some(10)).await.unwrap().is_empty());

    // Keep Episode/TaskType referenced so stub stays honest for required path
    let _ep = Episode::new("stub".into(), TaskContext::default(), TaskType::Testing);
}

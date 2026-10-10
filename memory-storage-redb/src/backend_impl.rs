use async_trait::async_trait;
use do_memory_core::memory::attribution::{
    RecommendationFeedback, RecommendationSession, RecommendationStats,
};
use do_memory_core::procedural::ProceduralMemory;
use do_memory_core::{
    Episode, Error, Heuristic, Pattern, Result, StorageBackend, StorageBackendCapabilities,
    episode::PatternId,
};
use uuid::Uuid;

use crate::RedbStorage;

#[async_trait]
impl StorageBackend for RedbStorage {
    async fn store_episode(&self, episode: &Episode) -> Result<()> {
        self.store_episode(episode).await
    }

    async fn get_episode(&self, id: Uuid) -> Result<Option<Episode>> {
        self.get_episode(id).await
    }

    /// A read transaction against the open database file: an unusable cache fails the probe
    /// instead of reporting itself healthy because a path existed (#1085). The message is
    /// fixed, so no filesystem detail reaches health output.
    async fn health_check(&self) -> Result<()> {
        if RedbStorage::health_check(self).await? {
            Ok(())
        } else {
            Err(Error::Storage(
                "redb health probe could not open a read".to_string(),
            ))
        }
    }

    async fn delete_episode(&self, id: Uuid) -> Result<()> {
        self.delete_episode(id).await
    }

    async fn store_pattern(&self, pattern: &Pattern) -> Result<()> {
        self.store_pattern(pattern).await
    }

    async fn get_pattern(&self, id: PatternId) -> Result<Option<Pattern>> {
        self.get_pattern(id).await
    }

    async fn get_all_patterns(&self) -> Result<Vec<Pattern>> {
        self.get_all_patterns(&crate::episodes::RedbQuery::default())
            .await
    }

    async fn store_heuristic(&self, heuristic: &Heuristic) -> Result<()> {
        self.store_heuristic(heuristic).await
    }

    async fn get_heuristic(&self, id: Uuid) -> Result<Option<Heuristic>> {
        self.get_heuristic(id).await
    }

    async fn query_episodes_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
        limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        self.query_episodes_since(since, limit).await
    }

    async fn query_episodes_by_metadata(
        &self,
        key: &str,
        value: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        self.query_episodes_by_metadata(key, value, limit).await
    }

    async fn store_embedding(&self, id: &str, embedding: Vec<f32>) -> Result<()> {
        self.store_embedding_impl(id, embedding).await
    }

    async fn get_embedding(&self, id: &str) -> Result<Option<Vec<f32>>> {
        self.get_embedding_impl(id).await
    }

    async fn delete_embedding(&self, id: &str) -> Result<bool> {
        self.delete_embedding_impl(id).await
    }

    async fn store_embeddings_batch(&self, embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        self.store_embeddings_batch_impl(embeddings).await
    }

    async fn get_embeddings_batch(&self, ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        self.get_embeddings_batch_impl(ids).await
    }

    async fn store_relationship(
        &self,
        relationship: &do_memory_core::episode::EpisodeRelationship,
    ) -> Result<()> {
        self.store_relationship(relationship).await
    }

    async fn remove_relationship(&self, relationship_id: Uuid) -> Result<()> {
        self.remove_relationship(relationship_id).await
    }

    async fn get_relationships(
        &self,
        episode_id: Uuid,
        direction: do_memory_core::episode::Direction,
    ) -> Result<Vec<do_memory_core::episode::EpisodeRelationship>> {
        self.get_relationships(episode_id, direction).await
    }

    async fn relationship_exists(
        &self,
        from_episode_id: Uuid,
        to_episode_id: Uuid,
        relationship_type: do_memory_core::episode::RelationshipType,
    ) -> Result<bool> {
        self.relationship_exists(from_episode_id, to_episode_id, relationship_type)
            .await
    }

    async fn store_episode_pattern_relationship(
        &self,
        relationship: &do_memory_core::episode::EpisodePatternRelationship,
    ) -> Result<()> {
        self.store_episode_pattern_relationship(relationship).await
    }

    async fn get_episode_pattern_relationships(
        &self,
        episode_id: Uuid,
    ) -> Result<Vec<do_memory_core::episode::EpisodePatternRelationship>> {
        self.get_episode_pattern_relationships(episode_id).await
    }

    async fn get_weighted_neighbors(&self, episode_id: Uuid) -> Result<Vec<(Uuid, f32, bool)>> {
        self.get_weighted_neighbors(episode_id).await
    }

    async fn get_all_relationships(
        &self,
    ) -> Result<Vec<do_memory_core::episode::EpisodeRelationship>> {
        self.get_all_relationships().await
    }

    async fn get_relationship_by_id(
        &self,
        relationship_id: Uuid,
    ) -> Result<Option<do_memory_core::episode::EpisodeRelationship>> {
        self.get_relationship_by_id(relationship_id).await
    }

    async fn store_recommendation_session(&self, session: &RecommendationSession) -> Result<()> {
        RedbStorage::store_recommendation_session(self, session).await
    }

    async fn get_recommendation_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<RecommendationSession>> {
        RedbStorage::get_recommendation_session(self, session_id).await
    }

    async fn get_recommendation_session_for_episode(
        &self,
        episode_id: Uuid,
    ) -> Result<Option<RecommendationSession>> {
        RedbStorage::get_recommendation_session_for_episode(self, episode_id).await
    }

    async fn store_recommendation_feedback(&self, feedback: &RecommendationFeedback) -> Result<()> {
        RedbStorage::store_recommendation_feedback(self, feedback).await
    }

    async fn get_recommendation_feedback(
        &self,
        session_id: Uuid,
    ) -> Result<Option<RecommendationFeedback>> {
        RedbStorage::get_recommendation_feedback(self, session_id).await
    }

    async fn get_recommendation_stats(&self) -> Result<RecommendationStats> {
        RedbStorage::get_recommendation_stats(self).await
    }

    async fn list_recommendation_sessions(&self) -> Result<Vec<RecommendationSession>> {
        RedbStorage::list_recommendation_sessions(self).await
    }

    async fn list_recommendation_feedback(&self) -> Result<Vec<RecommendationFeedback>> {
        RedbStorage::list_recommendation_feedback(self).await
    }

    async fn store_procedural(&self, procedural: &ProceduralMemory) -> Result<()> {
        self.store_procedural(procedural).await
    }

    async fn get_procedural(&self, id: Uuid) -> Result<Option<ProceduralMemory>> {
        self.get_procedural(id).await
    }

    async fn delete_procedural(&self, id: Uuid) -> Result<()> {
        self.delete_procedural(id).await
    }

    async fn query_procedural(&self, limit: Option<usize>) -> Result<Vec<ProceduralMemory>> {
        self.query_procedural(limit).await
    }
}

/// Capability matrix for the redb cache backend (ADR-081): the trait impl is
/// split so one block states what this backend can actually honor.
///
/// redb persists episode↔episode relationships, episode↔pattern relationships,
/// and procedural memory in its own tables (`src/relationships.rs`,
/// `src/procedural.rs`), and stores recommendation-attribution history
/// (`src/recommendations.rs`). Episode cleanup stays unsupported — there is no
/// retention/GC implementation for redb.
impl StorageBackendCapabilities for RedbStorage {
    fn supports_recommendation_attribution(&self) -> bool {
        true
    }

    fn supports_ranking_adaptation(&self) -> bool {
        true
    }

    fn supports_relationship_persistence(&self) -> bool {
        true
    }

    fn supports_procedural_memory(&self) -> bool {
        true
    }
}

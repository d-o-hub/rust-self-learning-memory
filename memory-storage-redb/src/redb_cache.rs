//! StorageBackend trait implementation for RedbStorage
//!
//! Implements the unified StorageBackend trait for the redb cache layer.

use async_trait::async_trait;
use memory_core::{
    Episode, Heuristic, Pattern, Result, StorageBackend,
    episode::{Direction, EpisodeRelationship, PatternId, RelationshipType},
};
use uuid::Uuid;

use crate::RedbStorage;

/// Implement the unified StorageBackend trait for RedbStorage
#[async_trait]
impl StorageBackend for RedbStorage {
    async fn store_episode(&self, episode: &Episode) -> Result<()> {
        self.store_episode(episode).await
    }

    async fn get_episode(&self, id: Uuid) -> Result<Option<Episode>> {
        self.get_episode(id).await
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

    // ========== Relationship Storage Methods ==========

    async fn store_relationship(&self, relationship: &EpisodeRelationship) -> Result<()> {
        self.store_relationship(relationship).await
    }

    async fn remove_relationship(&self, relationship_id: Uuid) -> Result<()> {
        self.remove_relationship(relationship_id).await
    }

    async fn get_relationships(
        &self,
        episode_id: Uuid,
        direction: Direction,
    ) -> Result<Vec<EpisodeRelationship>> {
        self.get_relationships(episode_id, direction).await
    }

    async fn relationship_exists(
        &self,
        from_episode_id: Uuid,
        to_episode_id: Uuid,
        relationship_type: RelationshipType,
    ) -> Result<bool> {
        self.relationship_exists(from_episode_id, to_episode_id, relationship_type)
            .await
    }
}

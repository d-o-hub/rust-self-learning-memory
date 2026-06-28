//! StorageBackend trait implementation for CachedTursoStorage
//!
//! Delegates all operations to the underlying cached/storage methods.

use super::wrapper::CachedTursoStorage;
use async_trait::async_trait;
use do_memory_core::memory::attribution::{
    RecommendationFeedback, RecommendationSession, RecommendationStats,
};
use do_memory_core::{
    Episode, Error, Heuristic, Pattern, Result, StorageBackend, episode::PatternId,
};
use uuid::Uuid;

#[async_trait]
impl StorageBackend for CachedTursoStorage {
    async fn store_episode(&self, episode: &Episode) -> Result<()> {
        self.store_episode_cached(episode)
            .await
            .map_err(|e| Error::Storage(format!("Cache store error: {}", e)))
    }

    async fn get_episode(&self, id: Uuid) -> Result<Option<Episode>> {
        self.get_episode_cached(id)
            .await
            .map_err(|e| Error::Storage(format!("Cache get error: {}", e)))
    }

    async fn delete_episode(&self, id: Uuid) -> Result<()> {
        self.delete_episode_cached(id)
            .await
            .map_err(|e| Error::Storage(format!("Cache delete error: {}", e)))
    }

    async fn store_pattern(&self, pattern: &Pattern) -> Result<()> {
        self.store_pattern_cached(pattern)
            .await
            .map_err(|e| Error::Storage(format!("Cache store error: {}", e)))
    }

    async fn get_pattern(&self, id: PatternId) -> Result<Option<Pattern>> {
        self.get_pattern_cached(id)
            .await
            .map_err(|e| Error::Storage(format!("Cache get error: {}", e)))
    }

    async fn store_heuristic(&self, heuristic: &Heuristic) -> Result<()> {
        self.store_heuristic_cached(heuristic)
            .await
            .map_err(|e| Error::Storage(format!("Cache store error: {}", e)))
    }

    async fn get_heuristic(&self, id: Uuid) -> Result<Option<Heuristic>> {
        self.get_heuristic_cached(id)
            .await
            .map_err(|e| Error::Storage(format!("Cache get error: {}", e)))
    }

    async fn query_episodes_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
        limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        self.storage
            .query_episodes_since(since, limit)
            .await
            .map_err(|e| Error::Storage(format!("Query error: {}", e)))
    }

    async fn query_episodes_by_metadata(
        &self,
        key: &str,
        value: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        self.storage
            .query_episodes_by_metadata(key, value, limit)
            .await
            .map_err(|e| Error::Storage(format!("Query error: {}", e)))
    }

    async fn store_embedding(&self, id: &str, embedding: Vec<f32>) -> Result<()> {
        self.storage
            .store_embedding(id, embedding)
            .await
            .map_err(|e| Error::Storage(format!("Store embedding error: {}", e)))
    }

    async fn get_embedding(&self, id: &str) -> Result<Option<Vec<f32>>> {
        self.storage
            .get_embedding(id)
            .await
            .map_err(|e| Error::Storage(format!("Get embedding error: {}", e)))
    }

    async fn delete_embedding(&self, id: &str) -> Result<bool> {
        self.storage
            .delete_embedding(id)
            .await
            .map_err(|e| Error::Storage(format!("Delete embedding error: {}", e)))
    }

    async fn store_embeddings_batch(&self, embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        self.storage
            .store_embeddings_batch(embeddings)
            .await
            .map_err(|e| Error::Storage(format!("Batch store embeddings error: {}", e)))
    }

    async fn get_embeddings_batch(&self, ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        self.storage
            .get_embeddings_batch(ids)
            .await
            .map_err(|e| Error::Storage(format!("Batch get embeddings error: {}", e)))
    }

    async fn store_recommendation_session(&self, session: &RecommendationSession) -> Result<()> {
        self.storage
            .store_recommendation_session(session)
            .await
            .map_err(|e| Error::Storage(format!("Store recommendation session error: {}", e)))
    }

    async fn get_recommendation_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<RecommendationSession>> {
        self.storage
            .get_recommendation_session(session_id)
            .await
            .map_err(|e| Error::Storage(format!("Get recommendation session error: {}", e)))
    }

    async fn get_recommendation_session_for_episode(
        &self,
        episode_id: Uuid,
    ) -> Result<Option<RecommendationSession>> {
        self.storage
            .get_recommendation_session_for_episode(episode_id)
            .await
            .map_err(|e| {
                Error::Storage(format!("Get recommendation session (episode) error: {}", e))
            })
    }

    async fn store_recommendation_feedback(&self, feedback: &RecommendationFeedback) -> Result<()> {
        self.storage
            .store_recommendation_feedback(feedback)
            .await
            .map_err(|e| Error::Storage(format!("Store recommendation feedback error: {}", e)))
    }

    async fn get_recommendation_feedback(
        &self,
        session_id: Uuid,
    ) -> Result<Option<RecommendationFeedback>> {
        self.storage
            .get_recommendation_feedback(session_id)
            .await
            .map_err(|e| Error::Storage(format!("Get recommendation feedback error: {}", e)))
    }

    async fn get_recommendation_stats(&self) -> Result<RecommendationStats> {
        self.storage
            .get_recommendation_stats()
            .await
            .map_err(|e| Error::Storage(format!("Get recommendation stats error: {}", e)))
    }
}

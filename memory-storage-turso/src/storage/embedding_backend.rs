//! EmbeddingStorageBackend trait implementation for TursoStorage

use crate::TursoStorage;
use async_trait::async_trait;
use do_memory_core::embeddings::{EmbeddingStorageBackend, SimilaritySearchResult};
use do_memory_core::{Episode, Pattern, Result, episode::PatternId};
use tracing::{debug, info};
use uuid::Uuid;

#[async_trait]
impl EmbeddingStorageBackend for TursoStorage {
    async fn store_episode_embedding(&self, episode_id: Uuid, embedding: Vec<f32>) -> Result<()> {
        debug!("Storing episode embedding: {}", episode_id);
        self._store_embedding_internal(&episode_id.to_string(), "episode", &embedding)
            .await
    }

    async fn store_pattern_embedding(
        &self,
        pattern_id: PatternId,
        embedding: Vec<f32>,
    ) -> Result<()> {
        debug!("Storing pattern embedding: {}", pattern_id);
        self._store_embedding_internal(&pattern_id.to_string(), "pattern", &embedding)
            .await
    }

    async fn get_episode_embedding(&self, episode_id: Uuid) -> Result<Option<Vec<f32>>> {
        debug!("Retrieving episode embedding: {}", episode_id);
        self._get_embedding_internal(&episode_id.to_string(), "episode")
            .await
    }

    async fn get_pattern_embedding(&self, pattern_id: PatternId) -> Result<Option<Vec<f32>>> {
        debug!("Retrieving pattern embedding: {}", pattern_id);
        self._get_embedding_internal(&pattern_id.to_string(), "pattern")
            .await
    }

    async fn find_similar_episodes(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Episode>>> {
        debug!(
            "Finding similar episodes (limit: {}, threshold: {})",
            limit, threshold
        );

        let (conn, _conn_id) = self.get_connection_with_id().await?;

        // Try to use native vector search if migration is applied
        if let Ok(results) = self
            .find_similar_episodes_native(&conn, &query_embedding, limit, threshold)
            .await
        {
            info!(
                "Found {} similar episodes using native vector search",
                results.len()
            );
            return Ok(results);
        }

        // Fallback to brute-force search if migration not applied
        debug!("Falling back to brute-force search (migration not applied)");
        self.find_similar_episodes_brute_force(&query_embedding, limit, threshold)
            .await
    }

    async fn find_similar_patterns(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Pattern>>> {
        debug!(
            "Finding similar patterns (limit: {}, threshold: {})",
            limit, threshold
        );

        let (conn, _conn_id) = self.get_connection_with_id().await?;

        // Try to use native vector search if migration is applied
        if let Ok(results) = self
            .find_similar_patterns_native(&conn, &query_embedding, limit, threshold)
            .await
        {
            info!(
                "Found {} similar patterns using native vector search",
                results.len()
            );
            return Ok(results);
        }

        // Fallback to brute-force search if migration not applied
        debug!("Falling back to brute-force search (migration not applied)");
        self.find_similar_patterns_brute_force(&query_embedding, limit, threshold)
            .await
    }
}

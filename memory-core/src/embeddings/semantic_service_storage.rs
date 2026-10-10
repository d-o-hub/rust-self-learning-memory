//! Storage-backed retrieval helpers for `SemanticService`.

use anyhow::Result;

use crate::embeddings::semantic_service::SemanticService;
use crate::embeddings::similarity::SimilaritySearchResult;
use crate::episode::Episode;
use crate::patterns::Pattern;

impl SemanticService {
    /// Find episodes similar to a pre-computed embedding vector
    ///
    /// This method allows searching with a pre-computed embedding, useful when
    /// the embedding has been generated externally or cached.
    ///
    /// # Arguments
    /// * `embedding` - Pre-computed embedding vector to search with
    /// * `limit` - Maximum number of results to return
    /// * `threshold` - Minimum similarity score (0.0-1.0)
    ///
    /// # Returns
    /// Vector of similar episodes with their similarity scores
    pub async fn find_episodes_by_embedding(
        &self,
        embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Episode>>> {
        self.storage
            .find_similar_episodes(embedding, limit, threshold)
            .await
            .map_err(|e| anyhow::Error::msg(e.to_string()))
    }

    /// Find patterns similar to a pre-computed embedding vector
    ///
    /// This method allows searching with a pre-computed embedding, useful when
    /// the embedding has been generated externally or cached.
    ///
    /// # Arguments
    /// * `embedding` - Pre-computed embedding vector to search with
    /// * `limit` - Maximum number of results to return
    /// * `threshold` - Minimum similarity score (0.0-1.0)
    ///
    /// # Returns
    /// Vector of similar patterns with their similarity scores
    pub async fn find_patterns_by_embedding(
        &self,
        embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Pattern>>> {
        self.storage
            .find_similar_patterns(embedding, limit, threshold)
            .await
            .map_err(|e| anyhow::Error::msg(e.to_string()))
    }

    /// Get embeddings for multiple episodes in batch
    ///
    /// This method retrieves embeddings for multiple episode IDs efficiently.
    /// For backends that don't support batch operations, it falls back to individual lookups.
    pub async fn get_embeddings_batch(
        &self,
        episode_ids: &[uuid::Uuid],
    ) -> Result<Vec<Option<Vec<f32>>>> {
        // Use individual lookups for now (batch optimization can be added later)
        let mut results = Vec::with_capacity(episode_ids.len());
        for episode_id in episode_ids {
            let embedding = self.storage.get_episode_embedding(*episode_id).await?;
            results.push(embedding);
        }
        Ok(results)
    }
}

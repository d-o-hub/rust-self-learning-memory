//! Storage backend for embeddings

use super::similarity::SimilaritySearchResult;
use crate::episode::Episode;
use crate::episode::PatternId;
use crate::pattern::Pattern;
use crate::Result;
use async_trait::async_trait;
use uuid::Uuid;

/// Trait for embedding storage backends
#[async_trait]
pub trait EmbeddingStorageBackend: Send + Sync {
    /// Store an episode embedding
    async fn store_episode_embedding(&self, episode_id: Uuid, embedding: Vec<f32>) -> Result<()>;

    /// Store a pattern embedding
    async fn store_pattern_embedding(
        &self,
        pattern_id: PatternId,
        embedding: Vec<f32>,
    ) -> Result<()>;

    /// Get an episode embedding
    async fn get_episode_embedding(&self, episode_id: Uuid) -> Result<Option<Vec<f32>>>;

    /// Get a pattern embedding
    async fn get_pattern_embedding(&self, pattern_id: PatternId) -> Result<Option<Vec<f32>>>;

    /// Find similar episodes using vector similarity
    async fn find_similar_episodes(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Episode>>>;

    /// Find similar patterns using vector similarity
    async fn find_similar_patterns(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Pattern>>>;
}

/// In-memory embedding storage for testing and fallback
pub struct InMemoryEmbeddingStorage {
    episode_embeddings:
        std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<Uuid, Vec<f32>>>>,
    pattern_embeddings:
        std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<PatternId, Vec<f32>>>>,
    episodes: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<Uuid, Episode>>>,
    patterns: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<PatternId, Pattern>>>,
}

impl InMemoryEmbeddingStorage {
    pub fn new() -> Self {
        Self {
            episode_embeddings: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
            pattern_embeddings: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
            episodes: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
            patterns: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
        }
    }

    /// Add an episode for testing
    pub async fn add_episode(&self, episode: Episode) {
        let mut episodes = self.episodes.write().await;
        episodes.insert(episode.episode_id, episode);
    }

    /// Add a pattern for testing
    pub async fn add_pattern(&self, pattern: Pattern) {
        let mut patterns = self.patterns.write().await;
        let pattern_id = match &pattern {
            Pattern::ToolSequence { id, .. } => *id,
            Pattern::DecisionPoint { .. } => uuid::Uuid::new_v4(), // Generate new ID
            Pattern::ErrorRecovery { .. } => uuid::Uuid::new_v4(), // Generate new ID
            Pattern::ContextPattern { .. } => uuid::Uuid::new_v4(), // Generate new ID
        };
        patterns.insert(pattern_id, pattern);
    }
}

impl Default for InMemoryEmbeddingStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EmbeddingStorageBackend for InMemoryEmbeddingStorage {
    async fn store_episode_embedding(&self, episode_id: Uuid, embedding: Vec<f32>) -> Result<()> {
        let mut embeddings = self.episode_embeddings.write().await;
        embeddings.insert(episode_id, embedding);
        Ok(())
    }

    async fn store_pattern_embedding(
        &self,
        pattern_id: PatternId,
        embedding: Vec<f32>,
    ) -> Result<()> {
        let mut embeddings = self.pattern_embeddings.write().await;
        embeddings.insert(pattern_id, embedding);
        Ok(())
    }

    async fn get_episode_embedding(&self, episode_id: Uuid) -> Result<Option<Vec<f32>>> {
        let embeddings = self.episode_embeddings.read().await;
        Ok(embeddings.get(&episode_id).cloned())
    }

    async fn get_pattern_embedding(&self, pattern_id: PatternId) -> Result<Option<Vec<f32>>> {
        let embeddings = self.pattern_embeddings.read().await;
        Ok(embeddings.get(&pattern_id).cloned())
    }

    async fn find_similar_episodes(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Episode>>> {
        let embeddings = self.episode_embeddings.read().await;
        let episodes = self.episodes.read().await;

        let mut results = Vec::new();

        for (episode_id, embedding) in embeddings.iter() {
            if let Some(episode) = episodes.get(episode_id) {
                let similarity = super::similarity::cosine_similarity(&query_embedding, embedding);

                if similarity >= threshold {
                    results.push(SimilaritySearchResult {
                        item: episode.clone(),
                        similarity,
                        metadata: super::similarity::SimilarityMetadata {
                            embedding_model: "unknown".to_string(),
                            embedding_timestamp: None,
                            context: serde_json::json!({}),
                        },
                    });
                }
            }
        }

        // Sort by similarity (highest first)
        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        results.truncate(limit);

        Ok(results)
    }

    async fn find_similar_patterns(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Pattern>>> {
        let embeddings = self.pattern_embeddings.read().await;
        let patterns = self.patterns.read().await;

        let mut results = Vec::new();

        for (pattern_id, embedding) in embeddings.iter() {
            if let Some(pattern) = patterns.get(pattern_id) {
                let similarity = super::similarity::cosine_similarity(&query_embedding, embedding);

                if similarity >= threshold {
                    results.push(SimilaritySearchResult {
                        item: pattern.clone(),
                        similarity,
                        metadata: super::similarity::SimilarityMetadata {
                            embedding_model: "unknown".to_string(),
                            embedding_timestamp: None,
                            context: serde_json::json!({}),
                        },
                    });
                }
            }
        }

        // Sort by similarity (highest first)
        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        results.truncate(limit);

        Ok(results)
    }
}

/// Wrapper around existing storage backends to add embedding support
pub struct EmbeddingStorage<T: crate::storage::StorageBackend + EmbeddingStorageBackend> {
    storage: std::sync::Arc<T>,
    fallback: InMemoryEmbeddingStorage,
}

impl<T: crate::storage::StorageBackend + EmbeddingStorageBackend> EmbeddingStorage<T> {
    pub fn new(storage: std::sync::Arc<T>) -> Self {
        Self {
            storage,
            fallback: InMemoryEmbeddingStorage::new(),
        }
    }
}

#[async_trait]
impl<T: crate::storage::StorageBackend + EmbeddingStorageBackend> EmbeddingStorageBackend
    for EmbeddingStorage<T>
{
    async fn store_episode_embedding(&self, episode_id: Uuid, embedding: Vec<f32>) -> Result<()> {
        // Try to store in main storage, fall back to in-memory
        if let Err(e) = self
            .storage
            .store_episode_embedding(episode_id, embedding.clone())
            .await
        {
            tracing::warn!("Failed to store episode embedding in main storage: {}", e);
            self.fallback
                .store_episode_embedding(episode_id, embedding)
                .await?;
        }
        Ok(())
    }

    async fn store_pattern_embedding(
        &self,
        pattern_id: PatternId,
        embedding: Vec<f32>,
    ) -> Result<()> {
        // Try to store in main storage, fall back to in-memory
        if let Err(e) = self
            .storage
            .store_pattern_embedding(pattern_id, embedding.clone())
            .await
        {
            tracing::warn!("Failed to store pattern embedding in main storage: {}", e);
            self.fallback
                .store_pattern_embedding(pattern_id, embedding)
                .await?;
        }
        Ok(())
    }

    async fn get_episode_embedding(&self, episode_id: Uuid) -> Result<Option<Vec<f32>>> {
        // Try main storage first, then fallback
        if let Ok(Some(embedding)) = self.storage.get_episode_embedding(episode_id).await {
            return Ok(Some(embedding));
        }

        self.fallback.get_episode_embedding(episode_id).await
    }

    async fn get_pattern_embedding(&self, pattern_id: PatternId) -> Result<Option<Vec<f32>>> {
        // Try main storage first, then fallback
        if let Ok(Some(embedding)) = self.storage.get_pattern_embedding(pattern_id).await {
            return Ok(Some(embedding));
        }

        self.fallback.get_pattern_embedding(pattern_id).await
    }

    async fn find_similar_episodes(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Episode>>> {
        // Try main storage first
        if let Ok(results) = self
            .storage
            .find_similar_episodes(query_embedding.clone(), limit, threshold)
            .await
        {
            return Ok(results);
        }

        // Fall back to in-memory search
        self.fallback
            .find_similar_episodes(query_embedding, limit, threshold)
            .await
    }

    async fn find_similar_patterns(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Pattern>>> {
        // Try main storage first
        if let Ok(results) = self
            .storage
            .find_similar_patterns(query_embedding.clone(), limit, threshold)
            .await
        {
            return Ok(results);
        }

        // Fall back to in-memory search
        self.fallback
            .find_similar_patterns(query_embedding, limit, threshold)
            .await
    }
}

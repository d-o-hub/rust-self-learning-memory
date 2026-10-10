//! Storage backend for embeddings

use super::similarity::SimilaritySearchResult;
use crate::Result;
use crate::episode::Episode;
use crate::episode::PatternId;
use crate::patterns::Pattern;
use async_trait::async_trait;
use uuid::Uuid;

/// Logical namespace for episode vectors inside an [`EmbeddingStorageScope`].
pub const EPISODE_NAMESPACE: &str = "episode";

/// Logical namespace for pattern vectors inside an [`EmbeddingStorageScope`].
pub const PATTERN_NAMESPACE: &str = "pattern";

/// Schema version embedded in identity-scoped logical keys.
///
/// Bump this when the key layout or vector encoding changes so vectors written
/// by an older layout can never be returned as current.
pub const EMBEDDING_STORAGE_SCHEMA_VERSION: u32 = 1;

/// Identity scope for embedding vectors.
///
/// Captures the active provider identity (`kind:model:dims`) together with a
/// deterministic configuration revision. Both are embedded in every logical
/// key, so vectors written under one scope are never returned under another —
/// reconfiguring the provider, model, dimension or provider configuration
/// cannot silently serve vectors produced by a different activation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddingStorageScope {
    provider_identity: String,
    config_revision: u64,
}

impl EmbeddingStorageScope {
    /// Create a scope for a provider identity and configuration revision.
    #[must_use]
    pub fn new(provider_identity: impl Into<String>, config_revision: u64) -> Self {
        Self {
            provider_identity: provider_identity.into(),
            config_revision,
        }
    }

    /// The provider identity (`kind:model:dims`).
    #[must_use]
    pub fn provider_identity(&self) -> &str {
        &self.provider_identity
    }

    /// Deterministic revision of the provider configuration.
    #[must_use]
    pub fn config_revision(&self) -> u64 {
        self.config_revision
    }

    /// Stable logical key prefix shared by every vector in this scope.
    #[must_use]
    pub fn key_prefix(&self) -> String {
        format!(
            "emb_v{}:{}#r{}",
            EMBEDDING_STORAGE_SCHEMA_VERSION, self.provider_identity, self.config_revision
        )
    }

    /// Prefix of every entry in the given logical namespace.
    #[must_use]
    pub fn entry_prefix(&self, namespace: &str) -> String {
        format!("{}:{}:", self.key_prefix(), namespace)
    }

    /// Full logical key for one namespace member.
    #[must_use]
    pub fn logical_key(&self, namespace: &str, id: &str) -> String {
        format!("{}{}", self.entry_prefix(namespace), id)
    }
}

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

    /// Identity scope of the vectors held by this backend, when known.
    ///
    /// Status/query paths use this to report which provider identity and
    /// configuration revision the stored vectors belong to. Backends that do
    /// not scope their keys return `None` by default.
    fn storage_scope(&self) -> Option<EmbeddingStorageScope> {
        None
    }

    /// Whether vectors survive a process restart.
    ///
    /// Defaults to `false` so an unknown backend never implies durability.
    fn is_durable(&self) -> bool {
        false
    }
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
    #[must_use]
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
            Pattern::DecisionPoint { .. }
            | Pattern::ErrorRecovery { .. }
            | Pattern::ContextPattern { .. } => {
                uuid::Uuid::new_v4() // Generate new ID for non-ToolSequence patterns
            }
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

/// Mock embedding storage for testing
#[cfg(test)]
pub struct MockEmbeddingStorage;

#[cfg(test)]
#[async_trait]
impl EmbeddingStorageBackend for MockEmbeddingStorage {
    async fn store_episode_embedding(&self, _episode_id: Uuid, _embedding: Vec<f32>) -> Result<()> {
        Ok(())
    }

    async fn store_pattern_embedding(
        &self,
        _pattern_id: PatternId,
        _embedding: Vec<f32>,
    ) -> Result<()> {
        Ok(())
    }

    async fn get_episode_embedding(&self, _episode_id: Uuid) -> Result<Option<Vec<f32>>> {
        Ok(None)
    }

    async fn get_pattern_embedding(&self, _pattern_id: PatternId) -> Result<Option<Vec<f32>>> {
        Ok(None)
    }

    async fn find_similar_episodes(
        &self,
        _query_embedding: Vec<f32>,
        _limit: usize,
        _threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Episode>>> {
        Ok(Vec::new())
    }

    async fn find_similar_patterns(
        &self,
        _query_embedding: Vec<f32>,
        _limit: usize,
        _threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Pattern>>> {
        Ok(Vec::new())
    }
}

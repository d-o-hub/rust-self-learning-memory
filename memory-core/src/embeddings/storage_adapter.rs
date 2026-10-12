//! Identity-scoped embedding storage adapter over configured [`StorageBackend`] handles.
//!
//! `SemanticService` stores and searches vectors through a
//! [`EmbeddingStorageBackend`]. The concrete storage backends (Turso, redb)
//! implement that trait themselves, but MCP only has `Arc<dyn StorageBackend>`
//! handles — it cannot see the embedding-specific impl behind the trait object.
//!
//! [`EmbeddingStorageAdapter`] closes that gap: it implements
//! [`EmbeddingStorageBackend`] using only the generic embedding methods of
//! [`StorageBackend`] (`store_embedding`, `get_embedding`,
//! `get_embeddings_batch`, `list_embedding_ids`), composing a durable primary
//! with an optional cache without requiring one concrete type to implement two
//! unrelated traits.
//!
//! # Identity scoping
//!
//! Every logical key carries an [`EmbeddingStorageScope`] — the active provider
//! identity (`kind:model:dims`) plus a deterministic configuration revision.
//! Vectors written under one scope are never returned under another, so a
//! reconfiguration cannot silently serve vectors produced by a different
//! provider, model, dimension or configuration revision.
//!
//! # Read/write semantics
//!
//! * **Write-through** — the primary backend is authoritative. A primary write
//!   failure is returned to the caller; a cache write failure is logged and
//!   ignored (the cache is a derivative that can be repopulated).
//! * **Read-through** — point reads consult the cache first, then the primary;
//!   a primary hit backfills the cache on a best-effort basis. A cache failure
//!   falls through to the primary; a primary failure is returned.
//! * **Similarity search** — always scans the primary (the authoritative store),
//!   never the cache, so results are deterministic and never double-count a
//!   cache-derived copy.

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::episode::PatternId;
use crate::patterns::Pattern;
use crate::{Episode, Result, StorageBackend};

use super::similarity::{SimilarityMetadata, SimilaritySearchResult, cosine_similarity};
use super::storage::{
    EPISODE_NAMESPACE, EmbeddingStorageBackend, EmbeddingStorageScope, InMemoryEmbeddingStorage,
    PATTERN_NAMESPACE,
};

/// How a freshly selected embedding store relates to durable storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmbeddingStorageMode {
    /// Vectors are persisted through configured storage backends.
    Durable {
        /// Identity scope used for the logical keys.
        scope: EmbeddingStorageScope,
        /// Whether a separate cache backend was composed in.
        has_cache: bool,
    },
    /// Vectors live only in this process and are lost on restart.
    Ephemeral {
        /// Identity scope used for the (process-local) logical keys.
        scope: EmbeddingStorageScope,
    },
}

impl EmbeddingStorageMode {
    /// Whether vectors survive a process restart.
    #[must_use]
    pub fn is_durable(&self) -> bool {
        matches!(self, Self::Durable { .. })
    }

    /// The identity scope backing this storage.
    #[must_use]
    pub fn scope(&self) -> &EmbeddingStorageScope {
        match self {
            Self::Durable { scope, .. } | Self::Ephemeral { scope } => scope,
        }
    }

    /// Short label for status output (`"durable"` / `"ephemeral"`).
    #[must_use]
    pub fn label(&self) -> &'static str {
        if self.is_durable() {
            "durable"
        } else {
            "ephemeral"
        }
    }
}

/// An [`EmbeddingStorageBackend`] plus its durability mode.
///
/// Returned by [`SelfLearningMemory::embedding_storage`](crate::SelfLearningMemory::embedding_storage)
/// so callers can both install the store and report truthfully whether vectors
/// persist across restarts.
pub struct SelectedEmbeddingStorage {
    /// The backing store to hand to `SemanticService`.
    pub storage: Box<dyn EmbeddingStorageBackend>,
    /// Durability/identity metadata for status and output.
    pub mode: EmbeddingStorageMode,
}

impl SelectedEmbeddingStorage {
    /// Compose a durable adapter over the configured primary and optional cache.
    #[must_use]
    pub fn durable(
        primary: Arc<dyn StorageBackend>,
        cache: Option<Arc<dyn StorageBackend>>,
        scope: EmbeddingStorageScope,
    ) -> Self {
        let adapter =
            EmbeddingStorageAdapter::new(Arc::clone(&primary), cache.clone(), scope.clone());
        Self {
            storage: Box::new(adapter),
            mode: EmbeddingStorageMode::Durable {
                scope,
                has_cache: cache.is_some(),
            },
        }
    }

    /// Build an explicitly ephemeral in-process store for the given scope.
    #[must_use]
    pub fn ephemeral(scope: EmbeddingStorageScope) -> Self {
        Self {
            storage: Box::new(EphemeralEmbeddingStorage::new(scope.clone())),
            mode: EmbeddingStorageMode::Ephemeral { scope },
        }
    }
}

/// Durable [`EmbeddingStorageBackend`] composed from `Arc<dyn StorageBackend>`.
pub struct EmbeddingStorageAdapter {
    primary: Arc<dyn StorageBackend>,
    cache: Option<Arc<dyn StorageBackend>>,
    scope: EmbeddingStorageScope,
}

impl EmbeddingStorageAdapter {
    /// Create an adapter over the primary (authoritative) and optional cache.
    #[must_use]
    pub fn new(
        primary: Arc<dyn StorageBackend>,
        cache: Option<Arc<dyn StorageBackend>>,
        scope: EmbeddingStorageScope,
    ) -> Self {
        Self {
            primary,
            cache,
            scope,
        }
    }

    /// Identity scope backing this adapter's logical keys.
    #[must_use]
    pub fn scope(&self) -> &EmbeddingStorageScope {
        &self.scope
    }

    /// Score every persisted `kind` vector for the current scope.
    ///
    /// Returns `(logical id, similarity)` pairs sorted by descending similarity
    /// with a deterministic id tie-break. Primary read failures propagate.
    async fn scored_candidates(
        &self,
        kind: &str,
        query_embedding: &[f32],
        threshold: f32,
    ) -> Result<Vec<(String, f32)>> {
        let prefix = self.scope.entry_prefix(kind);
        let keys: Vec<String> = self
            .primary
            .list_embedding_ids()
            .await?
            .into_iter()
            .filter(|key| key.starts_with(&prefix))
            .collect();

        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let embeddings = self.primary.get_embeddings_batch(&keys).await?;

        let mut scored: Vec<(String, f32)> = Vec::new();
        for (key, maybe_embedding) in keys.iter().zip(embeddings) {
            let Some(embedding) = maybe_embedding else {
                continue;
            };
            let similarity = cosine_similarity(query_embedding, &embedding);
            if similarity >= threshold {
                scored.push((key[prefix.len()..].to_string(), similarity));
            }
        }

        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        Ok(scored)
    }

    fn metadata(&self, context: serde_json::Value) -> SimilarityMetadata {
        SimilarityMetadata {
            embedding_model: self.scope.provider_identity().to_string(),
            embedding_timestamp: None,
            context,
        }
    }

    async fn store(&self, logical_key: &str, embedding: Vec<f32>) -> Result<()> {
        self.primary
            .store_embedding(logical_key, embedding.clone())
            .await?;

        if let Some(cache) = &self.cache {
            if let Err(error) = cache.store_embedding(logical_key, embedding).await {
                tracing::warn!(
                    %error,
                    key = logical_key,
                    "cache write failed; primary embedding remains authoritative"
                );
            }
        }
        Ok(())
    }

    async fn load(&self, logical_key: &str) -> Result<Option<Vec<f32>>> {
        if let Some(cache) = &self.cache {
            match cache.get_embedding(logical_key).await {
                Ok(Some(embedding)) => return Ok(Some(embedding)),
                Ok(None) => {}
                Err(error) => {
                    tracing::warn!(
                        %error,
                        key = logical_key,
                        "cache read failed; falling back to primary"
                    );
                }
            }
        }

        let hit = self.primary.get_embedding(logical_key).await?;
        if let (Some(embedding), Some(cache)) = (hit.as_ref(), self.cache.as_ref()) {
            if let Err(error) = cache.store_embedding(logical_key, embedding.clone()).await {
                tracing::warn!(
                    %error,
                    key = logical_key,
                    "cache backfill failed after primary read"
                );
            }
        }
        Ok(hit)
    }
}

#[async_trait]
impl EmbeddingStorageBackend for EmbeddingStorageAdapter {
    async fn store_episode_embedding(&self, episode_id: Uuid, embedding: Vec<f32>) -> Result<()> {
        let key = self
            .scope
            .logical_key(EPISODE_NAMESPACE, &episode_id.to_string());
        self.store(&key, embedding).await
    }

    async fn store_pattern_embedding(
        &self,
        pattern_id: PatternId,
        embedding: Vec<f32>,
    ) -> Result<()> {
        let key = self
            .scope
            .logical_key(PATTERN_NAMESPACE, &pattern_id.to_string());
        self.store(&key, embedding).await
    }

    async fn get_episode_embedding(&self, episode_id: Uuid) -> Result<Option<Vec<f32>>> {
        let key = self
            .scope
            .logical_key(EPISODE_NAMESPACE, &episode_id.to_string());
        self.load(&key).await
    }

    async fn get_pattern_embedding(&self, pattern_id: PatternId) -> Result<Option<Vec<f32>>> {
        let key = self
            .scope
            .logical_key(PATTERN_NAMESPACE, &pattern_id.to_string());
        self.load(&key).await
    }

    async fn find_similar_episodes(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Episode>>> {
        let scored = self
            .scored_candidates(EPISODE_NAMESPACE, &query_embedding, threshold)
            .await?;

        let mut results = Vec::new();
        for (id, similarity) in scored {
            if results.len() >= limit {
                break;
            }
            let Ok(episode_id) = Uuid::parse_str(&id) else {
                continue;
            };
            let Some(episode) = self.primary.get_episode(episode_id).await? else {
                continue;
            };
            results.push(SimilaritySearchResult {
                item: episode,
                similarity,
                metadata: self.metadata(serde_json::json!({
                    "storage_scope": self.scope.key_prefix(),
                    "config_revision": self.scope.config_revision(),
                })),
            });
        }
        Ok(results)
    }

    async fn find_similar_patterns(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Pattern>>> {
        let scored = self
            .scored_candidates(PATTERN_NAMESPACE, &query_embedding, threshold)
            .await?;

        let mut results = Vec::new();
        for (id, similarity) in scored {
            if results.len() >= limit {
                break;
            }
            let Ok(pattern_id) = Uuid::parse_str(&id) else {
                continue;
            };
            let Some(pattern) = self.primary.get_pattern(pattern_id).await? else {
                continue;
            };
            results.push(SimilaritySearchResult {
                item: pattern,
                similarity,
                metadata: self.metadata(serde_json::json!({
                    "storage_scope": self.scope.key_prefix(),
                    "config_revision": self.scope.config_revision(),
                })),
            });
        }
        Ok(results)
    }

    fn storage_scope(&self) -> Option<EmbeddingStorageScope> {
        Some(self.scope.clone())
    }

    fn is_durable(&self) -> bool {
        true
    }
}

/// Explicitly process-local embedding store.
///
/// Used when no storage backend is configured. Carries a scope so status output
/// and messages can state exactly which provider/config revision the vectors
/// belong to, while `is_durable()` stays `false` so no persistence is implied.
pub struct EphemeralEmbeddingStorage {
    inner: InMemoryEmbeddingStorage,
    scope: EmbeddingStorageScope,
}

impl EphemeralEmbeddingStorage {
    /// Create an ephemeral store for the given identity scope.
    #[must_use]
    pub fn new(scope: EmbeddingStorageScope) -> Self {
        Self {
            inner: InMemoryEmbeddingStorage::new(),
            scope,
        }
    }

    /// Identity scope backing this store.
    #[must_use]
    pub fn scope(&self) -> &EmbeddingStorageScope {
        &self.scope
    }
}

#[async_trait]
impl EmbeddingStorageBackend for EphemeralEmbeddingStorage {
    async fn store_episode_embedding(&self, episode_id: Uuid, embedding: Vec<f32>) -> Result<()> {
        self.inner
            .store_episode_embedding(episode_id, embedding)
            .await
    }

    async fn store_pattern_embedding(
        &self,
        pattern_id: PatternId,
        embedding: Vec<f32>,
    ) -> Result<()> {
        self.inner
            .store_pattern_embedding(pattern_id, embedding)
            .await
    }

    async fn get_episode_embedding(&self, episode_id: Uuid) -> Result<Option<Vec<f32>>> {
        self.inner.get_episode_embedding(episode_id).await
    }

    async fn get_pattern_embedding(&self, pattern_id: PatternId) -> Result<Option<Vec<f32>>> {
        self.inner.get_pattern_embedding(pattern_id).await
    }

    async fn find_similar_episodes(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Episode>>> {
        self.inner
            .find_similar_episodes(query_embedding, limit, threshold)
            .await
    }

    async fn find_similar_patterns(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SimilaritySearchResult<Pattern>>> {
        self.inner
            .find_similar_patterns(query_embedding, limit, threshold)
            .await
    }

    fn storage_scope(&self) -> Option<EmbeddingStorageScope> {
        Some(self.scope.clone())
    }

    fn is_durable(&self) -> bool {
        false
    }
}

#[cfg(test)]
#[path = "storage_adapter_tests.rs"]
mod tests;

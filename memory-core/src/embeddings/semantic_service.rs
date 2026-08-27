//! Semantic service for embedding operations.

use crate::episode::Episode;
use crate::patterns::Pattern;
use crate::types::TaskContext;
use anyhow::Result;

use super::config::{EmbeddingConfig, ProviderConfig};
use super::local::LocalEmbeddingProvider;
#[cfg(feature = "mistral")]
use super::mistral::MistralEmbeddingProvider;
#[cfg(feature = "openai")]
use super::openai::OpenAIEmbeddingProvider;
use super::provider::EmbeddingHealth;
use super::similarity::SimilaritySearchResult;
use super::storage::EmbeddingStorageBackend;

/// Default embedding dimension for sentence transformers
pub const DEFAULT_EMBEDDING_DIM: usize = 384;

/// Main semantic embedding service for the memory system
///
/// Coordinates embedding generation, storage, and semantic search across
/// episodes and patterns. Integrates with the existing storage backends.
pub struct SemanticService {
    /// Embedding provider for text-to-vector conversion
    pub provider: Box<dyn super::provider::EmbeddingProvider>,
    /// Storage backend for embeddings
    storage: Box<dyn EmbeddingStorageBackend>,
    /// Configuration
    config: EmbeddingConfig,
}

impl SemanticService {
    /// Create a new semantic service with the specified provider and storage
    #[must_use]
    pub fn new(
        provider: Box<dyn super::provider::EmbeddingProvider>,
        storage: Box<dyn EmbeddingStorageBackend>,
        config: EmbeddingConfig,
    ) -> Self {
        Self {
            provider,
            storage,
            config,
        }
    }

    /// Get the embedding configuration
    #[must_use]
    pub fn config(&self) -> &EmbeddingConfig {
        &self.config
    }

    /// Get the embedding provider type
    pub async fn with_local_provider(
        storage: Box<dyn EmbeddingStorageBackend>,
        config: EmbeddingConfig,
    ) -> Result<Self> {
        let local_config = match &config.provider {
            ProviderConfig::Local(cfg) => cfg.clone(),
            _ => super::config::LocalConfig::default(),
        };
        let provider = Box::new(LocalEmbeddingProvider::new(local_config).await?);
        Ok(Self::new(provider, storage, config))
    }

    /// Create a semantic service with default local provider
    pub async fn default(storage: Box<dyn EmbeddingStorageBackend>) -> Result<Self> {
        let config = EmbeddingConfig::default();
        Self::with_local_provider(storage, config).await
    }

    /// Create a semantic service with automatic provider fallback
    ///
    /// Tries providers in order: Local → `OpenAI` → Mock (with warnings)
    /// This ensures maximum reliability by falling back to simpler options if preferred ones fail.
    pub async fn with_fallback(
        storage: Box<dyn EmbeddingStorageBackend>,
        config: EmbeddingConfig,
    ) -> Result<Self> {
        // Get the preferred provider and its dimension for fallback scenarios
        let preferred_provider = config.provider.clone();
        let default_dimension = preferred_provider.effective_dimension();

        // Try local provider first
        match LocalEmbeddingProvider::new(super::config::LocalConfig::new(
            "sentence-transformers/all-MiniLM-L6-v2",
            default_dimension,
        ))
        .await
        {
            Ok(provider) => {
                tracing::info!("Using local embedding provider");
                return Ok(Self::new(Box::new(provider), storage, config));
            }
            Err(e) => {
                tracing::warn!("Failed to initialize local provider: {}, trying OpenAI", e);
            }
        }

        // Try OpenAI provider as fallback
        #[cfg(feature = "openai")]
        {
            if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
                // Try to use OpenAI config if preferred, otherwise use default
                let openai_config = match &preferred_provider {
                    ProviderConfig::OpenAI(cfg) => cfg.clone(),
                    _ => super::config::OpenAIConfig::default(),
                };

                match OpenAIEmbeddingProvider::new(api_key, openai_config) {
                    Ok(provider) => {
                        tracing::info!("Using OpenAI embedding provider as fallback");
                        return Ok(Self::new(Box::new(provider), storage, config));
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to initialize OpenAI provider: {}. Falling back to mock.",
                            e
                        );
                    }
                }
            } else {
                tracing::warn!("OPENAI_API_KEY not set, cannot use OpenAI provider");
            }
        }

        // Final fallback to mock provider (with warning)
        tracing::error!(
            "All embedding providers failed, using mock provider (embeddings will be random)"
        );
        tracing::error!("To fix this, either:");
        tracing::error!("  1. Install local embedding model dependencies");
        #[cfg(feature = "openai")]
        tracing::error!("  2. Set OPENAI_API_KEY environment variable");

        // Use MockLocalModel as the final fallback
        let provider =
            super::mock_model::MockLocalModel::new("mock-model".to_string(), default_dimension);
        Ok(Self::new(Box::new(provider), storage, config))
    }

    /// Create a semantic service with `OpenAI` embedding provider
    #[cfg(feature = "openai")]
    pub fn with_openai_provider(
        api_key: String,
        storage: Box<dyn EmbeddingStorageBackend>,
        config: EmbeddingConfig,
    ) -> Result<Self> {
        // Extract OpenAI config from ProviderConfig
        let openai_config = match &config.provider {
            ProviderConfig::OpenAI(cfg) => cfg.clone(),
            _ => super::config::OpenAIConfig::default(),
        };
        let provider = Box::new(OpenAIEmbeddingProvider::new(api_key, openai_config)?);
        Ok(Self::new(provider, storage, config))
    }

    /// Build an exact-provider service — no cross-provider fallback.
    ///
    /// Unlike [`with_fallback`](Self::with_fallback), this factory honours the
    /// requested [`ProviderConfig`] precisely.  It will return an error rather
    /// than silently substituting a different provider.
    ///
    /// # Arguments
    /// * `provider_config` - The provider and model to use.  Must be `Local`,
    ///   `OpenAI` (feature = `"openai"`), or `Mistral` (feature = `"mistral"`).
    /// * `api_key` - Pre-resolved API key for cloud providers; `None` for local.
    /// * `storage` - Embedding storage backend.
    /// * `embedding_config` - General embedding parameters (threshold, batch
    ///   size, etc.).  Its `provider` field is replaced by `provider_config`.
    ///
    /// # Errors
    /// Returns an error when:
    /// - A required API key is absent.
    /// - The provider feature flag is not compiled in.
    /// - Provider construction fails.
    /// - The health probe (`provider.embed_text("probe")`) fails.
    /// - The provider's reported dimension does not match the config dimension.
    /// - An unsupported provider variant (`AzureOpenAI`, `Custom`) is requested.
    pub async fn build_exact(
        provider_config: &ProviderConfig,
        api_key: Option<String>,
        storage: Box<dyn EmbeddingStorageBackend>,
        embedding_config: EmbeddingConfig,
    ) -> Result<Self> {
        let expected_dim = provider_config.effective_dimension();

        // Build the provider box — exact match, no fallback.
        let provider: Box<dyn super::provider::EmbeddingProvider> = match provider_config {
            ProviderConfig::Local(cfg) => {
                let p = LocalEmbeddingProvider::new(cfg.clone()).await?;
                let health = p.health_state().await;
                if health != EmbeddingHealth::Real {
                    anyhow::bail!(
                        "Local embedding provider is not production-ready (health={health:?}). \
                         Ensure the model is available or set LocalConfig::allow_mock_fallback \
                         only for tests."
                    );
                }
                Box::new(p)
            }

            ProviderConfig::OpenAI(cfg) => {
                #[cfg(feature = "openai")]
                {
                    let key = api_key.ok_or_else(|| {
                        anyhow::anyhow!(
                            "OPENAI_API_KEY not set — cannot build OpenAI embedding provider"
                        )
                    })?;
                    Box::new(OpenAIEmbeddingProvider::new(key, cfg.clone())?)
                }
                #[cfg(not(feature = "openai"))]
                {
                    // Suppress unused-variable warning when feature is off.
                    let _ = (cfg, api_key);
                    anyhow::bail!(
                        "Feature 'openai' not enabled — recompile with --features openai to use \
                         the OpenAI embedding provider"
                    );
                }
            }

            ProviderConfig::Mistral(cfg) => {
                #[cfg(feature = "mistral")]
                {
                    let key = api_key.ok_or_else(|| {
                        anyhow::anyhow!(
                            "MISTRAL_API_KEY not set — cannot build Mistral embedding provider"
                        )
                    })?;
                    Box::new(MistralEmbeddingProvider::new(key, cfg.clone())?)
                }
                #[cfg(not(feature = "mistral"))]
                {
                    let _ = (cfg, api_key);
                    anyhow::bail!(
                        "Feature 'mistral' not enabled — recompile with --features mistral to \
                         use the Mistral embedding provider"
                    );
                }
            }

            ProviderConfig::AzureOpenAI(_) | ProviderConfig::Custom(_) => {
                anyhow::bail!(
                    "Provider not supported for MCP activation: use local, openai, or mistral"
                );
            }
        };

        // Health probe — verify the provider can actually embed text.
        provider
            .embed_text("probe")
            .await
            .map_err(|e| anyhow::anyhow!("Embedding provider health probe failed: {e}"))?;

        // Dimension sanity check.
        let actual_dim = provider.embedding_dimension();
        if actual_dim != expected_dim {
            anyhow::bail!(
                "Embedding dimension mismatch: provider reports {actual_dim} but config \
                 expects {expected_dim}"
            );
        }

        // Stamp the resolved provider config into the embedding config so the
        // service and its callers see consistent metadata.
        let mut config = embedding_config;
        config.provider = provider_config.clone();

        Ok(Self::new(provider, storage, config))
    }

    /// Generate and store embedding for an episode
    ///
    /// Creates a semantic representation of the episode by combining:
    /// - Task description
    /// - Context information (domain, language, framework)
    /// - Key execution steps
    /// - Outcome summary
    pub async fn embed_episode(&self, episode: &Episode) -> Result<Vec<f32>> {
        let text = super::semantic_text::episode_to_text(episode);
        let embedding = self.provider.embed_text(&text).await?;

        // Store the embedding
        self.storage
            .store_episode_embedding(episode.episode_id, embedding.clone())
            .await?;

        Ok(embedding)
    }

    /// Generate and store embedding for a pattern
    ///
    /// Creates a semantic representation based on:
    /// - Pattern description
    /// - Context where the pattern was extracted
    /// - Pattern metadata and effectiveness metrics
    pub async fn embed_pattern(&self, pattern: &Pattern) -> Result<Vec<f32>> {
        let text = super::semantic_text::pattern_to_text(pattern);
        let embedding = self.provider.embed_text(&text).await?;

        // Store the embedding
        self.storage
            .store_pattern_embedding(pattern.id(), embedding.clone())
            .await?;

        Ok(embedding)
    }

    /// Find semantically similar episodes for a query
    ///
    /// Uses vector similarity to find episodes that are semantically related
    /// to the query, going beyond keyword matching to understand meaning.
    pub async fn find_similar_episodes(
        &self,
        query: &str,
        context: &TaskContext,
        limit: usize,
    ) -> Result<Vec<SimilaritySearchResult<Episode>>> {
        // Create query text combining description and context
        let query_text = super::semantic_text::create_query_text(query, context);

        // Generate embedding for query
        let query_embedding = self.provider.embed_text(&query_text).await?;

        // Search for similar episodes
        self.storage
            .find_similar_episodes(query_embedding, limit, self.config.similarity_threshold)
            .await
            .map_err(|e| anyhow::Error::msg(e.to_string()))
    }

    /// Find semantically similar patterns for a context
    ///
    /// Identifies patterns that are semantically relevant to the given context,
    /// enabling better pattern reuse and recommendation.
    pub async fn find_similar_patterns(
        &self,
        context: &TaskContext,
        limit: usize,
    ) -> Result<Vec<SimilaritySearchResult<Pattern>>> {
        // Create context-based query
        let query_text = super::semantic_text::context_to_text(context);

        // Generate embedding for query
        let query_embedding = self.provider.embed_text(&query_text).await?;

        // Search for similar patterns
        self.storage
            .find_similar_patterns(query_embedding, limit, self.config.similarity_threshold)
            .await
            .map_err(|e| anyhow::Error::msg(e.to_string()))
    }

    /// Calculate similarity between two texts
    pub async fn text_similarity(&self, text1: &str, text2: &str) -> Result<f32> {
        self.provider.similarity(text1, text2).await
    }

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

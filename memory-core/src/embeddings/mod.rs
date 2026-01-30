//! # Semantic Embeddings
//!
//! Vector embeddings for semantic similarity search and enhanced context retrieval.
//!
//! This module provides:
//! - Text embedding generation (local and remote)
//! - Vector similarity calculations
//! - Semantic-enhanced episode and pattern retrieval
//! - Configurable embedding providers
//!
//! ## Architecture
//!
//! The embedding system supports multiple providers:
//! - **Local**: sentence-transformers via candle-transformers (offline)
//! - **`OpenAI`**: text-embedding-ada-002 and text-embedding-3.x (cloud)
//! - **Mistral**: mistral-embed and codestral-embed (cloud)
//! - **Azure `OpenAI`**: Azure-hosted OpenAI embeddings
//! - **Custom**: User-provided embedding functions
//!
//! ## Usage
//!
//! ```rust,no_run
//! use memory_core::embeddings::{EmbeddingProvider, LocalEmbeddingProvider, ProviderConfig, LocalConfig};
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Local embedding provider (offline)
//! let config = LocalConfig::default();
//! let provider = LocalEmbeddingProvider::new(config).await?;
//!
//! // Generate embedding for text
//! let embedding = provider.embed_text("implement REST API").await?;
//!
//! // Calculate similarity between two texts
//! let similarity = provider.similarity("REST API", "web service API").await?;
//! # Ok(())
//! # }
//! ```

mod circuit_breaker;
pub mod config;
mod local;
mod metrics;
#[cfg(feature = "mistral")]
mod mistral;
mod mock_model;
mod openai;
mod provider;
mod real_model;
mod similarity;
mod storage;
mod utils;

pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerState};
// New configuration types
pub use config::{
    AzureOpenAIConfig, CustomConfig, EmbeddingConfig, LocalConfig, MistralConfig, OpenAIConfig,
    OptimizationConfig, ProviderConfig,
};
pub use local::{
    get_recommended_model, list_available_models, LocalEmbeddingProvider, LocalModelUseCase,
};
pub use metrics::{LatencyTimer, MetricsSnapshot, ProviderMetrics};
#[cfg(feature = "mistral")]
pub use mistral::MistralEmbeddingProvider;
pub use mock_model::MockLocalModel;
#[cfg(feature = "openai")]
pub use openai::OpenAIEmbeddingProvider;
pub use provider::{EmbeddingProvider, EmbeddingResult};
pub use similarity::{
    cosine_similarity, EmbeddingWithMetadata, SimilarityMetadata, SimilaritySearchResult,
};
pub use storage::{EmbeddingStorage, EmbeddingStorageBackend, InMemoryEmbeddingStorage};

#[cfg(all(test, feature = "mistral"))]
mod mistral_tests;
#[cfg(test)]
pub mod tests;

use crate::episode::Episode;
use crate::pattern::Pattern;
use crate::types::TaskContext;
use anyhow::Result;
// Removed unused import

/// Default embedding dimension for sentence transformers
pub const DEFAULT_EMBEDDING_DIM: usize = 384;

/// Main semantic embedding service for the memory system
///
/// Coordinates embedding generation, storage, and semantic search across
/// episodes and patterns. Integrates with the existing storage backends.
pub struct SemanticService {
    /// Embedding provider for text-to-vector conversion
    pub provider: Box<dyn EmbeddingProvider>,
    /// Storage backend for embeddings
    storage: Box<dyn EmbeddingStorageBackend>,
    /// Configuration
    config: EmbeddingConfig,
}

impl SemanticService {
    /// Create a new semantic service with the specified provider and storage
    #[must_use]
    pub fn new(
        provider: Box<dyn EmbeddingProvider>,
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
            _ => LocalConfig::default(),
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
        match LocalEmbeddingProvider::new(LocalConfig::new(
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
                    _ => OpenAIConfig::default(),
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
        let provider = crate::embeddings::mock_model::MockLocalModel::new(
            "mock-model".to_string(),
            default_dimension,
        );
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
            _ => OpenAIConfig::default(),
        };
        let provider = Box::new(OpenAIEmbeddingProvider::new(api_key, openai_config)?);
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
        let text = self.episode_to_text(episode);
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
        let text = self.pattern_to_text(pattern);
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
        let query_text = self.create_query_text(query, context);

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
        let query_text = self.context_to_text(context);

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

    /// Convert episode to searchable text representation
    fn episode_to_text(&self, episode: &Episode) -> String {
        // Build text directly using format! to avoid intermediate Vec clones
        let mut text = episode.task_description.clone();

        // Context information
        use std::fmt::Write;
        let _ = write!(text, ". domain: {}", episode.context.domain);
        if let Some(lang) = &episode.context.language {
            let _ = write!(text, ". language: {lang}");
        }
        if let Some(framework) = &episode.context.framework {
            let _ = write!(text, ". framework: {framework}");
        }
        if !episode.context.tags.is_empty() {
            let _ = write!(text, ". tags: {}", episode.context.tags.join(", "));
        }

        // Execution summary
        if !episode.steps.is_empty() {
            // Collect unique tools while preserving order
            use std::collections::HashSet;
            let mut seen_tools = HashSet::new();
            let mut tools = Vec::new();
            for step in &episode.steps {
                if seen_tools.insert(step.tool.clone()) {
                    tools.push(step.tool.clone());
                }
            }
            let _ = write!(text, ". tools used: {}", tools.join(", "));

            let actions: Vec<String> = episode
                .steps
                .iter()
                .take(3) // Take first few actions
                .map(|step| step.action.clone())
                .collect();
            let _ = write!(text, ". actions: {}", actions.join(", "));
        }

        // Outcome summary
        if let Some(outcome) = &episode.outcome {
            match outcome {
                crate::types::TaskOutcome::Success { verdict, .. } => {
                    let _ = write!(text, ". outcome: success - {verdict}");
                }
                crate::types::TaskOutcome::PartialSuccess { verdict, .. } => {
                    let _ = write!(text, ". outcome: partial success - {verdict}");
                }
                crate::types::TaskOutcome::Failure { reason, .. } => {
                    let _ = write!(text, ". outcome: failure - {reason}");
                }
            }
        }

        text
    }

    /// Convert pattern to searchable text representation
    fn pattern_to_text(&self, pattern: &Pattern) -> String {
        let mut parts = Vec::new();

        // Pattern description based on type
        let description = match pattern {
            crate::pattern::Pattern::ToolSequence { tools, .. } => {
                format!("Tool sequence: {}", tools.join(" -> "))
            }
            crate::pattern::Pattern::DecisionPoint {
                condition, action, ..
            } => {
                format!("Decision: if {condition} then {action}")
            }
            crate::pattern::Pattern::ErrorRecovery {
                error_type,
                recovery_steps,
                ..
            } => {
                format!(
                    "Error recovery: {} -> {}",
                    error_type,
                    recovery_steps.join(" -> ")
                )
            }
            crate::pattern::Pattern::ContextPattern {
                context_features,
                recommended_approach,
                ..
            } => {
                format!(
                    "Context pattern: {} suggests {}",
                    context_features.join(", "),
                    recommended_approach
                )
            }
        };
        parts.push(description);

        // Context information
        if let Some(pattern_context) = pattern.context() {
            parts.push(format!("domain: {}", pattern_context.domain));
            if let Some(lang) = &pattern_context.language {
                parts.push(format!("language: {lang}"));
            }
            if !pattern_context.tags.is_empty() {
                parts.push(format!("tags: {}", pattern_context.tags.join(", ")));
            }
        }

        parts.join(". ")
    }

    /// Create query text from description and context
    fn create_query_text(&self, query: &str, context: &TaskContext) -> String {
        let mut parts = vec![query.to_string()];

        parts.push(format!("domain: {}", context.domain));
        if let Some(lang) = &context.language {
            parts.push(format!("language: {lang}"));
        }
        if let Some(framework) = &context.framework {
            parts.push(format!("framework: {framework}"));
        }
        if !context.tags.is_empty() {
            parts.push(format!("tags: {}", context.tags.join(", ")));
        }

        parts.join(". ")
    }

    /// Convert context to searchable text
    fn context_to_text(&self, context: &TaskContext) -> String {
        let mut parts = Vec::new();

        parts.push(format!("domain: {}", context.domain));
        if let Some(lang) = &context.language {
            parts.push(format!("language: {lang}"));
        }
        if let Some(framework) = &context.framework {
            parts.push(format!("framework: {framework}"));
        }
        if !context.tags.is_empty() {
            parts.push(format!("tags: {}", context.tags.join(", ")));
        }
        parts.push(format!("complexity: {:?}", context.complexity));

        parts.join(". ")
    }
}

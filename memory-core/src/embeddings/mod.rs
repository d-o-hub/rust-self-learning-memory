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
//! - **CSM HDC**: Hyperdimensional computing for CPU-only lexical matching (optional)
//! - **`OpenAI`**: text-embedding-ada-002 and text-embedding-3.x (cloud)
//! - **Mistral**: mistral-embed and codestral-embed (cloud)
//! - **Azure `OpenAI`**: Azure-hosted OpenAI embeddings
//! - **Custom**: User-provided embedding functions
//!
//! ## Usage
//!
//! ```rust,no_run
//! use do_memory_core::embeddings::{EmbeddingProvider, LocalEmbeddingProvider, ProviderConfig, LocalConfig};
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
#[cfg(feature = "local-embeddings")]
mod real_model;
mod semantic_service;
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
    LocalEmbeddingProvider, LocalModelUseCase, get_recommended_model, list_available_models,
};
pub use metrics::{LatencyTimer, MetricsSnapshot, ProviderMetrics};
#[cfg(feature = "mistral")]
pub use mistral::MistralEmbeddingProvider;
pub use mock_model::MockLocalModel;
#[cfg(feature = "openai")]
pub use openai::OpenAIEmbeddingProvider;
pub use provider::utils::normalize_vector;
pub use provider::{EmbeddingProvider, EmbeddingResult};
pub use semantic_service::{DEFAULT_EMBEDDING_DIM, SemanticService};
pub use similarity::{SimilarityMetadata, SimilaritySearchResult, cosine_similarity};
pub use storage::{EmbeddingStorage, EmbeddingStorageBackend, InMemoryEmbeddingStorage};

#[cfg(all(test, feature = "mistral"))]
mod mistral_tests;
#[cfg(test)]
pub mod tests;

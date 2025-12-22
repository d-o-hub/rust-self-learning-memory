//! Configuration for semantic embedding providers

use serde::{Deserialize, Serialize};

/// Configuration for the embedding system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Embedding provider type
    pub provider: EmbeddingProvider,
    /// Model configuration
    pub model: ModelConfig,
    /// Similarity threshold for search (0.0 to 1.0)
    pub similarity_threshold: f32,
    /// Maximum batch size for embedding generation
    pub batch_size: usize,
    /// Cache embeddings to avoid regeneration
    pub cache_embeddings: bool,
    /// Timeout for embedding requests (seconds)
    pub timeout_seconds: u64,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: EmbeddingProvider::Local,
            model: ModelConfig::default(),
            similarity_threshold: 0.7,
            batch_size: 32,
            cache_embeddings: true,
            timeout_seconds: 30,
        }
    }
}

/// Supported embedding providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub enum EmbeddingProvider {
    /// Local embedding using sentence transformers
    Local,
    /// `OpenAI`'s text embedding models
    OpenAI,
    /// Custom provider implementation
    Custom(String),
}

/// Model configuration for embedding providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model name/identifier
    pub model_name: String,
    /// Expected embedding dimension
    pub embedding_dimension: usize,
    /// Model-specific parameters
    pub parameters: serde_json::Value,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            embedding_dimension: 384,
            parameters: serde_json::json!({}),
        }
    }
}

impl ModelConfig {
    /// Create configuration for a local sentence transformer model
    #[must_use]
    pub fn local_sentence_transformer(model_name: &str, dimension: usize) -> Self {
        Self {
            model_name: model_name.to_string(),
            embedding_dimension: dimension,
            parameters: serde_json::json!({
                "normalize": true,
                "pooling": "mean"
            }),
        }
    }

    /// Create configuration for `OpenAI` text-embedding-ada-002
    #[must_use]
    pub fn openai_ada_002() -> Self {
        Self {
            model_name: "text-embedding-ada-002".to_string(),
            embedding_dimension: 1536,
            parameters: serde_json::json!({}),
        }
    }

    /// Create configuration for `OpenAI` text-embedding-3-small
    #[must_use]
    pub fn openai_3_small() -> Self {
        Self {
            model_name: "text-embedding-3-small".to_string(),
            embedding_dimension: 1536,
            parameters: serde_json::json!({}),
        }
    }

    /// Create configuration for `OpenAI` text-embedding-3-large
    #[must_use]
    pub fn openai_3_large() -> Self {
        Self {
            model_name: "text-embedding-3-large".to_string(),
            embedding_dimension: 3072,
            parameters: serde_json::json!({}),
        }
    }
}

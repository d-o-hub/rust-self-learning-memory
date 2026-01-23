//! Configuration for the embedding system

use serde::{Deserialize, Serialize};

use super::{model_config::ModelConfig, provider_enum::EmbeddingProvider};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_embedding_config() {
        let config = EmbeddingConfig::default();

        // Verify default provider is Local
        assert_eq!(config.provider, EmbeddingProvider::Local);

        // Verify default model configuration
        assert_eq!(
            config.model.model_name,
            "sentence-transformers/all-MiniLM-L6-v2"
        );
        assert_eq!(config.model.embedding_dimension, 384);

        // Verify default threshold and batch size
        assert_eq!(config.similarity_threshold, 0.7);
        assert_eq!(config.batch_size, 32);

        // Verify cache enabled and timeout
        assert!(config.cache_embeddings);
        assert_eq!(config.timeout_seconds, 30);
    }
}

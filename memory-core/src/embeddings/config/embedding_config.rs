//! Configuration for the embedding system

use serde::{Deserialize, Serialize};

use super::provider_config::ProviderConfig;

/// Configuration for the embedding system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Provider-specific configuration
    pub provider: ProviderConfig,
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
            provider: ProviderConfig::default(),
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
        match &config.provider {
            ProviderConfig::Local(local_config) => {
                assert_eq!(
                    local_config.model_name,
                    "sentence-transformers/all-MiniLM-L6-v2"
                );
                assert_eq!(local_config.embedding_dimension, 384);
            }
            _ => panic!("Expected Local provider in default config"),
        }

        // Verify default threshold and batch size
        assert_eq!(config.similarity_threshold, 0.7);
        assert_eq!(config.batch_size, 32);

        // Verify cache enabled and timeout
        assert!(config.cache_embeddings);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_embedding_config_with_openai() {
        let config = EmbeddingConfig {
            provider: ProviderConfig::openai_3_small(),
            similarity_threshold: 0.8,
            batch_size: 64,
            cache_embeddings: false,
            timeout_seconds: 60,
        };

        assert_eq!(config.similarity_threshold, 0.8);
        assert_eq!(config.batch_size, 64);
        assert!(!config.cache_embeddings);
        assert_eq!(config.timeout_seconds, 60);
    }
}

//! Supported embedding providers

use serde::{Deserialize, Serialize};

/// Supported embedding providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub enum EmbeddingProvider {
    /// Local embedding using sentence transformers
    Local,
    /// `OpenAI`'s text embedding models
    OpenAI,
    /// Mistral AI's embedding models
    Mistral,
    /// Azure `OpenAI` Service
    AzureOpenAI,
    /// Custom provider implementation
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_provider_variants() {
        // Test Local provider
        let local = EmbeddingProvider::Local;
        assert_eq!(local, EmbeddingProvider::Local);

        // Test OpenAI provider
        let openai = EmbeddingProvider::OpenAI;
        assert_eq!(openai, EmbeddingProvider::OpenAI);

        // Test Mistral provider
        let mistral = EmbeddingProvider::Mistral;
        assert_eq!(mistral, EmbeddingProvider::Mistral);

        // Test Azure OpenAI provider
        let azure = EmbeddingProvider::AzureOpenAI;
        assert_eq!(azure, EmbeddingProvider::AzureOpenAI);

        // Test Custom provider
        let custom = EmbeddingProvider::Custom("custom-provider".to_string());
        assert_eq!(
            custom,
            EmbeddingProvider::Custom("custom-provider".to_string())
        );

        // Test equality/inequality
        assert_ne!(local, openai);
        assert_ne!(openai, mistral);
    }
}

//! Default config and provider variant tests

use crate::embeddings::{EmbeddingConfig, EmbeddingProvider};

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

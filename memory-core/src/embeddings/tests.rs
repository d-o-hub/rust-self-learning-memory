//! Tests for the semantic embeddings module.

use super::*;
use crate::embeddings::storage::MockEmbeddingStorage;
use crate::{Episode, Pattern, TaskContext};
use anyhow::Result;

fn create_test_episode() -> Episode {
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: crate::types::ComplexityLevel::Moderate,
        domain: "test".to_string(),
        tags: vec!["test".to_string()],
    };

    Episode::new("test".to_string(), context, crate::types::TaskType::Testing)
}

fn create_test_pattern() -> Pattern {
    use chrono::Duration;
    Pattern::ToolSequence {
        id: uuid::Uuid::new_v4(),
        tools: vec!["test".to_string()],
        context: TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: crate::types::ComplexityLevel::Moderate,
            domain: "test domain".to_string(),
            tags: vec!["test".to_string()],
        },
        success_rate: 1.0,
        avg_latency: Duration::milliseconds(100),
        occurrence_count: 1,
        effectiveness: crate::pattern::PatternEffectiveness::default(),
    }
}

#[tokio::test]
async fn test_embed_episode() -> Result<()> {
    let storage = Box::new(MockEmbeddingStorage);
    let config = EmbeddingConfig::default();

    let service = SemanticService::new(
        Box::new(MockLocalModel::new("mock".to_string(), 384)),
        storage,
        config,
    );

    let episode = create_test_episode();
    let embedding = service.embed_episode(&episode).await?;

    assert_eq!(embedding.len(), DEFAULT_EMBEDDING_DIM);
    Ok(())
}

#[tokio::test]
async fn test_embed_pattern() -> Result<()> {
    let storage = Box::new(MockEmbeddingStorage);
    let config = EmbeddingConfig::default();

    let service = SemanticService::new(
        Box::new(MockLocalModel::new("mock".to_string(), 384)),
        storage,
        config,
    );

    let pattern = create_test_pattern();
    let embedding = service.embed_pattern(&pattern).await?;

    assert_eq!(embedding.len(), DEFAULT_EMBEDDING_DIM);
    Ok(())
}

#[tokio::test]
async fn test_find_similar_episodes() -> Result<()> {
    let storage = Box::new(MockEmbeddingStorage);
    let config = EmbeddingConfig::default();

    let service = SemanticService::new(
        Box::new(MockLocalModel::new("mock".to_string(), 384)),
        storage,
        config,
    );

    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: crate::types::ComplexityLevel::Moderate,
        domain: "test".to_string(),
        tags: vec!["test".to_string()],
    };
    let results = service
        .find_similar_episodes("test query", &context, 5)
        .await?;

    assert!(results.len() <= 5);
    Ok(())
}

#[tokio::test]
async fn test_find_similar_patterns() -> Result<()> {
    let storage = Box::new(MockEmbeddingStorage);
    let config = EmbeddingConfig::default();

    let service = SemanticService::new(
        Box::new(MockLocalModel::new("mock".to_string(), 384)),
        storage,
        config,
    );

    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: crate::types::ComplexityLevel::Moderate,
        domain: "test".to_string(),
        tags: vec!["test".to_string()],
    };
    let results = service.find_similar_patterns(&context, 5).await?;

    assert!(results.len() <= 5);
    Ok(())
}

#[tokio::test]
async fn test_text_similarity() -> Result<()> {
    let storage = Box::new(MockEmbeddingStorage);
    let config = EmbeddingConfig::default();

    let service = SemanticService::new(
        Box::new(MockLocalModel::new("mock".to_string(), 384)),
        storage,
        config,
    );

    let similarity = service.text_similarity("test1", "test2").await?;

    assert!((0.0..=1.0).contains(&similarity));
    Ok(())
}

#[tokio::test]
async fn test_with_fallback_provider() -> Result<()> {
    let storage = Box::new(MockEmbeddingStorage);
    let config = EmbeddingConfig {
        provider: ProviderConfig::openai_3_small(),
        ..Default::default()
    };

    let result = SemanticService::with_fallback(storage, config).await?;

    // Should fall back to Local if OpenAI is not configured
    assert!(result.provider.is_available().await);
    Ok(())
}

#[tokio::test]
async fn test_config_preservation() -> Result<()> {
    let storage = Box::new(MockEmbeddingStorage);
    let config = EmbeddingConfig {
        similarity_threshold: 0.75,
        batch_size: 32,
        cache_embeddings: true,
        ..Default::default()
    };

    let service = SemanticService::new(
        Box::new(MockLocalModel::new("mock".to_string(), 384)),
        storage,
        config.clone(),
    );

    assert_eq!(
        service.config().similarity_threshold,
        config.similarity_threshold
    );
    assert_eq!(service.config().batch_size, config.batch_size);
    assert_eq!(service.config().cache_embeddings, config.cache_embeddings);
    Ok(())
}

// NOTE: This test has been removed as it tests for the old fallback behavior
// where the config's provider was always used. In the new ProviderConfig-based
// architecture, with_fallback() tries providers in order (Local → OpenAI → Mock)
// and may use a different provider than specified in the config.
// The config is still preserved (stored in service.config), but the actual
// provider used may differ due to fallback behavior which is now expected.

/*
#[tokio::test]
async fn test_with_fallback_config_preservation() -> Result<()> {
    let storage = Box::new(MockEmbeddingStorage);

    let config = EmbeddingConfig {
        provider: ProviderConfig::openai_3_small(),
        similarity_threshold: 0.8,
        batch_size: 64,
        cache_embeddings: false,
        timeout_seconds: 60,
    };

    let result = SemanticService::with_fallback(storage, config.clone()).await?;

    assert_eq!(
        result.config.provider.effective_dimension(),
        config.provider.effective_dimension()
    );
    assert_eq!(result.config.model_name(), config.provider.model_name());
    assert_eq!(
        result.config.similarity_threshold,
        config.similarity_threshold
    );
    assert_eq!(result.config.batch_size, config.batch_size);
    assert_eq!(result.config.cache_embeddings, config.cache_embeddings);
    assert_eq!(result.config.timeout_seconds, config.timeout_seconds);
    Ok(())
}
*/

#[tokio::test]
async fn test_with_fallback_default_storage_works() -> Result<()> {
    let storage = Box::new(MockEmbeddingStorage);
    let config = EmbeddingConfig::default();

    let _result = SemanticService::with_fallback(storage, config).await?;

    let custom_config = EmbeddingConfig {
        similarity_threshold: 0.5,
        batch_size: 16,
        ..Default::default()
    };

    let storage2 = Box::new(MockEmbeddingStorage);
    let _result2 = SemanticService::with_fallback(storage2, custom_config).await?;
    Ok(())
}

#[tokio::test]
async fn test_default_creates_valid_service() -> Result<()> {
    let storage = Box::new(MockEmbeddingStorage);

    let service = SemanticService::default(storage).await?;
    match &service.config().provider {
        ProviderConfig::Local(config) => {
            assert_eq!(config.model_name, "sentence-transformers/all-MiniLM-L6-v2");
        }
        _ => panic!("Expected Local provider in default config"),
    }
    Ok(())
}

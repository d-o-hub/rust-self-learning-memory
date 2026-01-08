//! Tests for the semantic embeddings module.

use super::*;
use crate::embeddings::storage::MockEmbeddingStorage;

fn create_test_episode() -> Episode {
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: crate::types::ComplexityLevel::Moderate,
        domain: "test".to_string(),
        tags: vec!["test".to_string()],
    };

    let mut episode = Episode::new("test".to_string(), context, crate::types::TaskType::Testing);
    episode
}

fn create_test_pattern() -> Pattern {
    Pattern::new(
        "test".to_string(),
        "test pattern".to_string(),
        vec!["test".to_string()],
        "test domain".to_string(),
    )
}

#[tokio::test]
async fn test_embed_episode() {
    let storage = Box::new(MockEmbeddingStorage);
    let config = EmbeddingConfig::default();

    let result = SemanticService::new(Box::new(MockLocalModel::new()), storage, config).await;

    assert!(result.is_ok());
    let service = result.unwrap();

    let episode = create_test_episode();
    let embedding = service.embed_episode(&episode).await;

    assert!(embedding.is_ok());
    let embedding = embedding.unwrap();
    assert_eq!(embedding.len(), DEFAULT_EMBEDDING_DIM);
}

#[tokio::test]
async fn test_embed_text() {
    let storage = Box::new(MockEmbeddingStorage);
    let config = EmbeddingConfig::default();

    let result = SemanticService::new(Box::new(MockLocalModel::new()), storage, config).await;

    assert!(result.is_ok());
    let service = result.unwrap();

    let embedding = service.embed_text("test query").await;

    assert!(embedding.is_ok());
    let embedding = embedding.unwrap();
    assert_eq!(embedding.len(), DEFAULT_EMBEDDING_DIM);
}

#[tokio::test]
async fn test_similarity_search() {
    let storage = Box::new(MockEmbeddingStorage);
    let config = EmbeddingConfig::default();

    let result = SemanticService::new(Box::new(MockLocalModel::new()), storage, config).await;

    assert!(result.is_ok());
    let service = result.unwrap();

    let query = "test query".to_string();
    let episodes = vec![create_test_episode()];

    let results = service.semantic_search(&query, &episodes, 5).await;

    assert!(results.is_ok());
    let results = results.unwrap();
    assert!(results.len() <= 5);
}

#[tokio::test]
async fn test_similarity_threshold_filtering() {
    let storage = Box::new(MockEmbeddingStorage);
    let config = EmbeddingConfig {
        similarity_threshold: 0.9,
        ..Default::default()
    };

    let result = SemanticService::new(Box::new(MockLocalModel::new()), storage, config).await;

    assert!(result.is_ok());
    let service = result.unwrap();

    let query = "test query".to_string();
    let episodes = vec![create_test_episode()];

    let results = service.semantic_search(&query, &episodes, 5).await;

    assert!(results.is_ok());
    let results = results.unwrap();
    assert!(results.is_empty() || results[0].score >= 0.9);
}

#[tokio::test]
async fn test_batch_embedding() {
    let storage = Box::new(MockEmbeddingStorage);
    let config = EmbeddingConfig {
        batch_size: 2,
        ..Default::default()
    };

    let result = SemanticService::new(Box::new(MockLocalModel::new()), storage, config).await;

    assert!(result.is_ok());
    let service = result.unwrap();

    let texts = vec!["test1", "test2", "test3"];

    let results = service.embed_batch(&texts).await;

    assert!(results.is_ok());
    let results = results.unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].len(), DEFAULT_EMBEDDING_DIM);
}

#[tokio::test]
async fn test_embed_pattern() {
    let storage = Box::new(MockEmbeddingStorage);
    let config = EmbeddingConfig::default();

    let result = SemanticService::new(Box::new(MockLocalModel::new()), storage, config).await;

    assert!(result.is_ok());
    let service = result.unwrap();

    let pattern = create_test_pattern();
    let embedding = service.embed_pattern(&pattern).await;

    assert!(embedding.is_ok());
    let embedding = embedding.unwrap();
    assert_eq!(embedding.len(), DEFAULT_EMBEDDING_DIM);
}

#[tokio::test]
async fn test_search_with_diversity() {
    let storage = Box::new(MockEmbeddingStorage);
    let config = EmbeddingConfig {
        diversity_threshold: 0.5,
        ..Default::default()
    };

    let result = SemanticService::new(Box::new(MockLocalModel::new()), storage, config).await;

    assert!(result.is_ok());
    let service = result.unwrap();

    let query = "test query".to_string();
    let episodes = vec![create_test_episode(), create_test_episode()];

    let results = service.semantic_search(&query, &episodes, 5).await;

    assert!(results.is_ok());
    let results = results.unwrap();
    // Results should be diversity-filtered
    if results.len() > 1 {
        // First result should be highest score
        assert!(results[0].score >= results[1].score);
    }
}

#[tokio::test]
async fn test_with_fallback_provider() {
    let storage = Box::new(MockEmbeddingStorage);
    let config = EmbeddingConfig {
        provider: EmbeddingProviderType::OpenAI,
        ..Default::default()
    };

    let result = SemanticService::with_fallback(storage, config).await;

    // Should fall back to Local if OpenAI is not configured
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_config_preservation() {
    let storage = Box::new(MockEmbeddingStorage);
    let config = EmbeddingConfig {
        similarity_threshold: 0.75,
        batch_size: 32,
        cache_embeddings: true,
        ..Default::default()
    };

    let result =
        SemanticService::new(Box::new(MockLocalModel::new()), storage, config.clone()).await;

    assert!(result.is_ok());
    let service = result.unwrap();

    assert_eq!(
        service.config.similarity_threshold,
        config.similarity_threshold
    );
    assert_eq!(service.config.batch_size, config.batch_size);
    assert_eq!(service.config.cache_embeddings, config.cache_embeddings);
}

#[tokio::test]
async fn test_with_fallback_config_preservation() {
    let storage = Box::new(MockEmbeddingStorage);

    let config = EmbeddingConfig {
        provider: EmbeddingProviderType::Local,
        model: ModelConfig::openai_3_small(),
        similarity_threshold: 0.8,
        batch_size: 64,
        cache_embeddings: false,
        timeout_seconds: 60,
    };

    let result = SemanticService::with_fallback(storage, config.clone()).await;
    assert!(result.is_ok());

    let service = result.unwrap();

    assert_eq!(service.config.provider, config.provider);
    assert_eq!(service.config.model.model_name, config.model.model_name);
    assert_eq!(
        service.config.model.embedding_dimension,
        config.model.embedding_dimension
    );
    assert_eq!(
        service.config.similarity_threshold,
        config.similarity_threshold
    );
    assert_eq!(service.config.batch_size, config.batch_size);
    assert_eq!(service.config.cache_embeddings, config.cache_embeddings);
    assert_eq!(service.config.timeout_seconds, config.timeout_seconds);
}

#[tokio::test]
async fn test_with_fallback_default_storage_works() {
    let storage = Box::new(MockEmbeddingStorage);
    let config = EmbeddingConfig::default();

    let result = SemanticService::with_fallback(storage, config).await;
    assert!(result.is_ok());

    let custom_config = EmbeddingConfig {
        similarity_threshold: 0.5,
        batch_size: 16,
        ..Default::default()
    };

    let storage2 = Box::new(MockEmbeddingStorage);
    let result2 = SemanticService::with_fallback(storage2, custom_config).await;
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_default_creates_valid_service() {
    let storage = Box::new(MockEmbeddingStorage);

    let result = SemanticService::default(storage).await;
    if let Ok(service) = result {
        assert_eq!(service.config.provider, EmbeddingProviderType::Local);
        assert_eq!(
            service.config.model.model_name,
            "sentence-transformers/all-MiniLM-L6-v2"
        );
    }
}

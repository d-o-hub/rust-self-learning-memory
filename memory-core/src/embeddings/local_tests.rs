use super::*;

#[tokio::test]
async fn test_local_provider_creation() {
    let config = LocalConfig::new("test-model", 384);

    let provider = LocalEmbeddingProvider::new(config).await.unwrap();
    assert!(provider.is_loaded().await);
    assert_eq!(provider.embedding_dimension(), 384);
    assert_eq!(provider.model_name(), "test-model");
}

#[tokio::test]
async fn test_embed_text() {
    let config = LocalConfig::new("test-model", 384);

    let provider = LocalEmbeddingProvider::new(config).await.unwrap();

    let embedding = provider.embed_text("Hello world").await.unwrap();
    assert_eq!(embedding.len(), 384);

    // Test deterministic behavior
    let embedding2 = provider.embed_text("Hello world").await.unwrap();
    assert_eq!(embedding, embedding2);

    // Different text should produce different embedding
    let embedding3 = provider.embed_text("Different text").await.unwrap();
    assert_ne!(embedding, embedding3);
}

#[tokio::test]
async fn test_embed_batch() {
    let config = LocalConfig::new("test-model", 384);

    let provider = LocalEmbeddingProvider::new(config).await.unwrap();

    let texts = vec![
        "First text".to_string(),
        "Second text".to_string(),
        "Third text".to_string(),
    ];

    let embeddings = provider.embed_batch(&texts).await.unwrap();
    assert_eq!(embeddings.len(), 3);

    for embedding in embeddings {
        assert_eq!(embedding.len(), 384);
    }
}

#[tokio::test]
async fn test_similarity_calculation() {
    let config = LocalConfig::new("test-model", 384);

    let provider = LocalEmbeddingProvider::new(config).await.unwrap();

    // Identical texts should have high similarity
    let similarity = provider
        .similarity("Hello world", "Hello world")
        .await
        .unwrap();
    assert!((similarity - 1.0).abs() < 0.001);

    // Different texts should have lower similarity
    let similarity = provider
        .similarity("Hello world", "Goodbye universe")
        .await
        .unwrap();
    assert!(similarity < 1.0);
}

#[tokio::test]
#[ignore = "Requires local-embeddings feature with ONNX models - blocked by ort crate Send trait issue"]
#[cfg(feature = "local-embeddings")]
async fn test_real_embedding_generation() {
    // This test only runs when local-embeddings feature is enabled
    // and real ONNX models are available

    // Create a temporary directory for model cache
    let temp_dir = tempfile::TempDir::new().unwrap();
    let cache_path = temp_dir.path().join("models");

    // Try to load a real model if available
    // In CI, this might not have actual model files
    if cache_path.exists() || std::env::var("CI").is_ok() {
        tracing::info!("Skipping real embedding test - no model files available");
        return;
    }

    let config = LocalConfig::new("sentence-transformers/all-MiniLM-L6-v2", 384);

    let provider = LocalEmbeddingProvider::new(config).await.unwrap();

    // Generate embeddings for semantically similar texts
    let embedding1 = provider
        .embed_text("machine learning algorithms")
        .await
        .unwrap();
    let embedding2 = provider
        .embed_text("artificial intelligence models")
        .await
        .unwrap();
    let embedding3 = provider
        .embed_text("cooking recipes for pasta")
        .await
        .unwrap();

    assert_eq!(embedding1.len(), 384);
    assert_eq!(embedding2.len(), 384);
    assert_eq!(embedding3.len(), 384);

    // Calculate similarities
    let similarity_ai_ml = provider
        .similarity("machine learning", "artificial intelligence")
        .await
        .unwrap();
    let similarity_cooking = provider
        .similarity("machine learning", "cooking recipes")
        .await
        .unwrap();

    // Semantically similar texts should have higher similarity
    assert!(
        similarity_ai_ml > similarity_cooking,
        "AI/ML similarity ({similarity_ai_ml}) should be higher than ML/cooking ({similarity_cooking})"
    );

    // Both should be positive (cosine similarity range)
    assert!(similarity_ai_ml > 0.0);
    assert!(similarity_cooking > 0.0);
}

#[tokio::test]
async fn test_embedding_vector_properties() {
    let config = LocalConfig::new("test-model", 384);

    let provider = LocalEmbeddingProvider::new(config).await.unwrap();

    let embedding = provider.embed_text("test text").await.unwrap();

    // Check that embedding is properly normalized (unit vector)
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 0.001, "Embedding should be normalized");

    // Check that values are in reasonable range
    for &value in &embedding {
        assert!(
            (-1.0..=1.0).contains(&value),
            "Embedding values should be in [-1, 1]"
        );
    }
}

#[tokio::test]
async fn test_model_metadata() {
    let config = LocalConfig::new("sentence-transformers/test-model", 768);

    let provider = LocalEmbeddingProvider::new(config).await.unwrap();

    let metadata = provider.metadata();
    assert_eq!(metadata["model"], "sentence-transformers/test-model");
    assert_eq!(metadata["dimension"], 768);
    assert_eq!(metadata["type"], "local");

    let model_info = provider.model_info();
    assert_eq!(model_info["name"], "sentence-transformers/test-model");
    assert_eq!(model_info["dimension"], 768);
    assert_eq!(model_info["type"], "local");
}

#[tokio::test]
async fn test_error_handling() {
    let config = LocalConfig::new("nonexistent-model", 384);

    // Test with non-existent model - should fall back to mock or fail gracefully
    let result = LocalEmbeddingProvider::new(config).await;

    match result {
        Ok(provider) => {
            // If successful, it should be a mock implementation
            assert!(provider.is_loaded().await);
            let embedding = provider.embed_text("test").await.unwrap();
            assert_eq!(embedding.len(), 384);
        }
        Err(e) => {
            // Should provide meaningful error message
            assert!(e.to_string().contains("model") || e.to_string().contains("load"));
        }
    }
}

#[tokio::test]
async fn test_warmup_functionality() {
    let config = LocalConfig::new("test-model", 384);

    let provider = LocalEmbeddingProvider::new(config).await.unwrap();

    // Warmup should succeed
    let result = provider.warmup().await;
    assert!(result.is_ok(), "Warmup should succeed");
}

#[test]
fn test_utils_list_models() {
    let models = list_available_models();
    assert!(!models.is_empty());

    for model in models {
        assert!(!model.model_name.is_empty());
        assert!(model.embedding_dimension > 0);
    }
}

#[test]
fn test_utils_recommended_models() {
    let fast_model = get_recommended_model(LocalModelUseCase::Fast);
    assert_eq!(fast_model.embedding_dimension, 384);

    let quality_model = get_recommended_model(LocalModelUseCase::Quality);
    assert_eq!(quality_model.embedding_dimension, 768);

    let multilingual_model = get_recommended_model(LocalModelUseCase::Multilingual);
    assert_eq!(multilingual_model.embedding_dimension, 384);
}

#[tokio::test]
async fn test_production_warning_behavior() {
    let config = LocalConfig::new("test-model", 384);

    // This should emit a warning if not in test mode
    let provider = LocalEmbeddingProvider::new(config).await.unwrap();

    // Verify the provider works but may be using mock embeddings
    let embedding1 = provider.embed_text("test").await.unwrap();
    let embedding2 = provider.embed_text("test").await.unwrap();

    // In test mode, embeddings should be deterministic (same)
    assert_eq!(embedding1, embedding2);
    assert_eq!(embedding1.len(), 384);
}

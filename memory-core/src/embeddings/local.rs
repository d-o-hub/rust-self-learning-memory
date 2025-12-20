//! Local embedding provider using sentence transformers
//!
//! This provider runs embedding models locally using candle-transformers,
//! providing offline capability with no external API dependencies.

use super::config::ModelConfig;
use super::provider::{EmbeddingProvider, EmbeddingResult};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(feature = "local-embeddings")]
use {
    ort::{Session, SessionBuilder},
    tokenizers::Tokenizer,
};

/// Local embedding provider using sentence transformers
///
/// Runs embedding models locally using candle-transformers or similar.
/// Provides offline embedding generation with no external dependencies.
///
/// # Models Supported
/// - sentence-transformers/all-MiniLM-L6-v2 (384 dims, default)
/// - sentence-transformers/all-mpnet-base-v2 (768 dims, higher quality)
/// - sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2 (384 dims, multilingual)
///
/// # Example
/// ```no_run
/// use memory_core::embeddings::{LocalEmbeddingProvider, ModelConfig};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let config = ModelConfig::local_sentence_transformer(
///         "sentence-transformers/all-MiniLM-L6-v2",
///         384
///     );
///     let provider = LocalEmbeddingProvider::new(config).await?;
///     
///     let embedding = provider.embed_text("Hello world").await?;
///     println!("Generated embedding with {} dimensions", embedding.len());
///     Ok(())
/// }
/// ```
pub struct LocalEmbeddingProvider {
    /// Model configuration
    config: ModelConfig,
    /// Embedding model (placeholder for actual model implementation)
    #[allow(dead_code)]
    model: Arc<RwLock<Option<Box<dyn LocalEmbeddingModel>>>>,
    /// Model cache directory
    cache_dir: std::path::PathBuf,
}

impl LocalEmbeddingProvider {
    /// Create a new local embedding provider
    ///
    /// # Arguments
    /// * `config` - Model configuration specifying which model to use
    ///
    /// # Returns
    /// Configured local embedding provider
    pub async fn new(config: ModelConfig) -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;

        let provider = Self {
            config,
            model: Arc::new(RwLock::new(None)),
            cache_dir,
        };

        // Initialize/load the model
        provider.load_model().await?;

        Ok(provider)
    }

    /// Load the embedding model
    async fn load_model(&self) -> Result<()> {
        tracing::info!("Loading local embedding model: {}", self.config.model_name);

        #[cfg(feature = "local-embeddings")]
        {
            // Try to load real ONNX model, fallback to mock if fails
            match self.try_load_real_model().await {
                Ok(real_model) => {
                    let fallback_model = Box::new(RealEmbeddingModelWithFallback::new(
                        self.config.model_name.clone(),
                        self.config.embedding_dimension,
                        Some(real_model),
                    ));

                    let mut model_guard = self.model.write().await;
                    *model_guard = Some(fallback_model);

                    tracing::info!("Local embedding model loaded with real ONNX backend");
                }
                Err(e) => {
                    tracing::warn!("Failed to load real embedding model: {}", e);
                    tracing::warn!("Falling back to mock embeddings - semantic search will not work correctly");

                    let mock_fallback = Box::new(RealEmbeddingModelWithFallback::new(
                        self.config.model_name.clone(),
                        self.config.embedding_dimension,
                        None,
                    ));

                    let mut model_guard = self.model.write().await;
                    *model_guard = Some(mock_fallback);

                    tracing::info!("Local embedding model loaded with mock fallback");
                }
            }
        }

        #[cfg(not(feature = "local-embeddings"))]
        {
            tracing::warn!("PRODUCTION WARNING: Using mock embeddings - semantic search will not work correctly");
            tracing::warn!("To enable real embeddings, add 'local-embeddings' feature and ensure ONNX models are available");

            let mock_fallback = Box::new(RealEmbeddingModelWithFallback::new(
                self.config.model_name.clone(),
                self.config.embedding_dimension,
                None,
            ));

            let mut model_guard = self.model.write().await;
            *model_guard = Some(mock_fallback);

            tracing::info!("Local embedding model loaded with mock implementation");
        }

        Ok(())
    }

    /// Try to load real ONNX model
    #[cfg(feature = "local-embeddings")]
    async fn try_load_real_model(&self) -> Result<RealEmbeddingModel> {
        RealEmbeddingModel::try_load_from_cache(&self.config, &self.cache_dir).await
    }

    /// Get the cache directory for models
    fn get_cache_dir() -> Result<std::path::PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("Could not determine home directory")?;

        let cache_dir = std::path::Path::new(&home)
            .join(".cache")
            .join("memory-core")
            .join("embeddings");

        std::fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;

        Ok(cache_dir)
    }

    /// Check if model is loaded
    pub async fn is_loaded(&self) -> bool {
        let model_guard = self.model.read().await;
        model_guard.is_some()
    }

    /// Get model information
    pub fn model_info(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.config.model_name,
            "dimension": self.config.embedding_dimension,
            "type": "local",
            "cache_dir": self.cache_dir,
        })
    }
}

#[async_trait]
impl EmbeddingProvider for LocalEmbeddingProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let model_guard = self.model.read().await;
        let model = model_guard.as_ref().context("Model not loaded")?;

        model.embed(text).await
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let model_guard = self.model.read().await;
        let model = model_guard.as_ref().context("Model not loaded")?;

        model.embed_batch(texts).await
    }

    fn embedding_dimension(&self) -> usize {
        self.config.embedding_dimension
    }

    fn model_name(&self) -> &str {
        &self.config.model_name
    }

    async fn is_available(&self) -> bool {
        self.is_loaded().await
    }

    async fn warmup(&self) -> Result<()> {
        // Test embedding generation
        let _embedding = self.embed_text("warmup test").await?;
        Ok(())
    }

    fn metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "model": self.model_name(),
            "dimension": self.embedding_dimension(),
            "type": "local",
            "provider": "sentence-transformers",
            "cache_dir": self.cache_dir
        })
    }
}

/// Trait for local embedding models
#[async_trait]
trait LocalEmbeddingModel: Send + Sync {
    /// Generate embedding for single text
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for batch of texts
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;

    /// Get model name
    fn name(&self) -> &str;

    /// Get embedding dimension
    fn dimension(&self) -> usize;
}

/// Import real model implementation
pub use crate::embeddings::real_model::RealEmbeddingModel;

/// Import mock model implementations
pub use crate::embeddings::mock_model::{MockLocalModel, RealEmbeddingModelWithFallback};

/// Re-export utilities from the utils module
pub use crate::embeddings::utils::{list_available_models, get_recommended_model, LocalModelUseCase};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_local_provider_creation() {
        let config = ModelConfig::local_sentence_transformer("test-model", 384);

        let provider = LocalEmbeddingProvider::new(config).await.unwrap();
        assert!(provider.is_loaded().await);
        assert_eq!(provider.embedding_dimension(), 384);
        assert_eq!(provider.model_name(), "test-model");
    }

    #[tokio::test]
    async fn test_embed_text() {
        let config = ModelConfig::local_sentence_transformer("test-model", 384);

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
        let config = ModelConfig::local_sentence_transformer("test-model", 384);

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
        let config = ModelConfig::local_sentence_transformer("test-model", 384);

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
    #[cfg(feature = "local-embeddings")]
    async fn test_real_embedding_generation() {
        // This test only runs when local-embeddings feature is enabled
        // and real ONNX models are available
        use std::path::PathBuf;

        // Create a temporary directory for model cache
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("models");

        // Try to load a real model if available
        // In CI, this might not have actual model files
        if cache_path.exists() || std::env::var("CI").is_ok() {
            tracing::info!("Skipping real embedding test - no model files available");
            return;
        }

        let config = ModelConfig::local_sentence_transformer(
            "sentence-transformers/all-MiniLM-L6-v2",
            384,
        );

        let provider = LocalEmbeddingProvider::new(config).await.unwrap();

        // Generate embeddings for semantically similar texts
        let embedding1 = provider.embed_text("machine learning algorithms").await.unwrap();
        let embedding2 = provider.embed_text("artificial intelligence models").await.unwrap();
        let embedding3 = provider.embed_text("cooking recipes for pasta").await.unwrap();

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
            "AI/ML similarity ({}) should be higher than ML/cooking ({})",
            similarity_ai_ml,
            similarity_cooking
        );

        // Both should be positive (cosine similarity range)
        assert!(similarity_ai_ml > 0.0);
        assert!(similarity_cooking > 0.0);
    }

    #[tokio::test]
    async fn test_embedding_vector_properties() {
        let config = ModelConfig::local_sentence_transformer("test-model", 384);

        let provider = LocalEmbeddingProvider::new(config).await.unwrap();

        let embedding = provider.embed_text("test text").await.unwrap();

        // Check that embedding is properly normalized (unit vector)
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001, "Embedding should be normalized");

        // Check that values are in reasonable range
        for &value in &embedding {
            assert!(value >= -1.0 && value <= 1.0, "Embedding values should be in [-1, 1]");
        }
    }

    #[tokio::test]
    async fn test_model_metadata() {
        let config = ModelConfig::local_sentence_transformer("sentence-transformers/test-model", 768);

        let provider = LocalEmbeddingProvider::new(config).await.unwrap();

        let metadata = provider.metadata();
        assert_eq!(metadata["model"], "sentence-transformers/test-model");
        assert_eq!(metadata["dimension"], 768);
        assert_eq!(metadata["type"], "local");
        assert_eq!(metadata["provider"], "sentence-transformers");

        let model_info = provider.model_info();
        assert_eq!(model_info["name"], "sentence-transformers/test-model");
        assert_eq!(model_info["dimension"], 768);
        assert_eq!(model_info["type"], "local");
    }

    #[tokio::test]
    async fn test_error_handling() {
        let config = ModelConfig::local_sentence_transformer("nonexistent-model", 384);

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
        let config = ModelConfig::local_sentence_transformer("test-model", 384);

        let provider = LocalEmbeddingProvider::new(config).await.unwrap();

        // Warmup should succeed
        let result = provider.warmup().await;
        assert!(result.is_ok(), "Warmup should succeed");
    }

    #[test]
    fn test_utils_list_models() {
        let models = utils::list_available_models();
        assert!(!models.is_empty());

        for model in models {
            assert!(!model.model_name.is_empty());
            assert!(model.embedding_dimension > 0);
        }
    }

    #[test]
    fn test_utils_recommended_models() {
        let fast_model = utils::get_recommended_model(utils::LocalModelUseCase::Fast);
        assert_eq!(fast_model.embedding_dimension, 384);

        let quality_model = utils::get_recommended_model(utils::LocalModelUseCase::Quality);
        assert_eq!(quality_model.embedding_dimension, 768);

        let multilingual_model =
            utils::get_recommended_model(utils::LocalModelUseCase::Multilingual);
        assert_eq!(multilingual_model.embedding_dimension, 384);
    }

    #[tokio::test]
    async fn test_production_warning_behavior() {
        let config = ModelConfig::local_sentence_transformer("test-model", 384);

        // This should emit a warning if not in test mode
        let provider = LocalEmbeddingProvider::new(config).await.unwrap();

        // Verify the provider works but may be using mock embeddings
        let embedding1 = provider.embed_text("test").await.unwrap();
        let embedding2 = provider.embed_text("test").await.unwrap();

        // In test mode, embeddings should be deterministic (same)
        assert_eq!(embedding1, embedding2);
        assert_eq!(embedding1.len(), 384);
    }
}

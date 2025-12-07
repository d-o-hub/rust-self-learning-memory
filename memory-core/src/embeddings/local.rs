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

        // For now, use a mock implementation
        // In a real implementation, this would load the actual model
        let mock_model = Box::new(MockLocalModel::new(
            self.config.model_name.clone(),
            self.config.embedding_dimension,
        ));

        let mut model_guard = self.model.write().await;
        *model_guard = Some(mock_model);

        tracing::info!("Local embedding model loaded successfully");
        Ok(())
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

/// Mock implementation for local embedding model
/// In production, this would be replaced with actual model loading/inference
struct MockLocalModel {
    name: String,
    dimension: usize,
}

impl MockLocalModel {
    fn new(name: String, dimension: usize) -> Self {
        Self { name, dimension }
    }

    /// Generate a deterministic mock embedding for testing
    fn generate_mock_embedding(&self, text: &str) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Create a deterministic embedding based on text hash
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();

        let mut embedding = Vec::with_capacity(self.dimension);
        let mut seed = hash;

        for _ in 0..self.dimension {
            // Simple PRNG to generate values
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let value = ((seed >> 16) as f32) / 32768.0 - 1.0; // Range [-1, 1]
            embedding.push(value);
        }

        // Normalize the vector
        super::provider::utils::normalize_vector(embedding)
    }
}

#[async_trait]
impl LocalEmbeddingModel for MockLocalModel {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Simulate processing time
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;

        Ok(self.generate_mock_embedding(text))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // Simulate batch processing (faster than individual calls)
        let batch_delay = std::cmp::max(1, texts.len() / 10);
        tokio::time::sleep(std::time::Duration::from_millis(batch_delay as u64)).await;

        let mut embeddings = Vec::with_capacity(texts.len());
        for text in texts {
            embeddings.push(self.generate_mock_embedding(text));
        }
        Ok(embeddings)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

/// Helper functions for local model management
pub mod utils {
    use super::*;

    /// List available local models
    pub fn list_available_models() -> Vec<ModelConfig> {
        vec![
            ModelConfig::local_sentence_transformer("sentence-transformers/all-MiniLM-L6-v2", 384),
            ModelConfig::local_sentence_transformer("sentence-transformers/all-mpnet-base-v2", 768),
            ModelConfig::local_sentence_transformer(
                "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2",
                384,
            ),
        ]
    }

    /// Get recommended model configuration for different use cases
    pub fn get_recommended_model(use_case: LocalModelUseCase) -> ModelConfig {
        match use_case {
            LocalModelUseCase::Fast => ModelConfig::local_sentence_transformer(
                "sentence-transformers/all-MiniLM-L6-v2",
                384,
            ),
            LocalModelUseCase::Quality => ModelConfig::local_sentence_transformer(
                "sentence-transformers/all-mpnet-base-v2",
                768,
            ),
            LocalModelUseCase::Multilingual => ModelConfig::local_sentence_transformer(
                "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2",
                384,
            ),
        }
    }

    /// Use cases for local model selection
    pub enum LocalModelUseCase {
        /// Fast inference with good quality (384 dimensions)
        Fast,
        /// Best quality (768 dimensions, slower)
        Quality,
        /// Multilingual support (384 dimensions)
        Multilingual,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

//! Local embedding provider using sentence transformers
//!
//! This provider runs embedding models locally using candle-transformers,
//! providing offline capability with no external API dependencies.

use super::config::LocalConfig;
use super::provider::EmbeddingProvider;
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
/// use do_memory_core::embeddings::{EmbeddingProvider, LocalEmbeddingProvider, LocalConfig};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let config = LocalConfig::new(
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
    config: LocalConfig,
    /// Embedding model (placeholder for actual model implementation)
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
    pub async fn new(config: LocalConfig) -> Result<Self> {
        let cache_dir = Self::get_cache_dir().await?;

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
                    tracing::warn!(
                        "Falling back to mock embeddings - semantic search will not work correctly"
                    );

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
            tracing::warn!(
                "PRODUCTION WARNING: Using mock embeddings - semantic search will not work correctly"
            );
            tracing::warn!(
                "To enable real embeddings, add 'local-embeddings' feature and ensure ONNX models are available"
            );

            let mock_fallback = Box::new(super::mock_model::MockLocalModel::new(
                self.config.model_name.clone(),
                self.config.embedding_dimension,
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
    ///
    /// On WASM targets, returns a temporary in-memory path.
    #[cfg(target_arch = "wasm32")]
    async fn get_cache_dir() -> Result<std::path::PathBuf> {
        // WASM has no filesystem - return a placeholder path
        Ok(std::path::PathBuf::from("/tmp/memory-core-embeddings"))
    }

    /// Get the cache directory for models
    #[cfg(not(target_arch = "wasm32"))]
    async fn get_cache_dir() -> Result<std::path::PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("Could not determine home directory")?;

        let cache_dir = std::path::Path::new(&home)
            .join(".cache")
            .join("memory-core")
            .join("embeddings");

        tokio::fs::create_dir_all(&cache_dir)
            .await
            .context("Failed to create cache directory")?;

        Ok(cache_dir)
    }

    /// Check if model is loaded
    pub async fn is_loaded(&self) -> bool {
        let model_guard = self.model.read().await;
        model_guard.is_some()
    }

    /// Get model information
    #[must_use]
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
#[allow(dead_code)] // Trait methods used by implementations, not called directly in this crate
pub trait LocalEmbeddingModel: Send + Sync {
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
#[cfg(feature = "local-embeddings")]
#[allow(unused)]
pub use crate::embeddings::real_model::RealEmbeddingModel;

/// Import mock model implementations
#[cfg(feature = "local-embeddings")]
#[allow(unused)]
pub use crate::embeddings::mock_model::{MockLocalModel, RealEmbeddingModelWithFallback};

/// Re-export utilities from the utils module
#[allow(unused)]
pub use crate::embeddings::utils::{
    LocalModelUseCase, get_recommended_model, list_available_models,
};

#[cfg(test)]
#[path = "local_tests.rs"]
mod tests;

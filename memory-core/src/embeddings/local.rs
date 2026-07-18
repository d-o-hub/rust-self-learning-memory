//! Local embedding provider using sentence transformers
//!
//! This provider runs embedding models locally using candle-transformers,
//! providing offline capability with no external API dependencies.

use super::config::LocalConfig;
use super::provider::{EmbeddingHealth, EmbeddingProvider};
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
    /// Current health state (S1.5): real vs degraded-mock vs unavailable.
    health: Arc<RwLock<EmbeddingHealth>>,
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
            health: Arc::new(RwLock::new(EmbeddingHealth::Unavailable)),
        };

        // Initialize/load the model
        provider.load_model().await?;

        Ok(provider)
    }

    /// Current embedding health (S1.5).
    pub async fn health_state(&self) -> EmbeddingHealth {
        *self.health.read().await
    }

    /// S1.5b: when digest/size pins are set and a model file is present, verify it.
    async fn verify_cached_artifact_if_configured(&self) -> Result<()> {
        if self.config.expected_sha256.is_none() && self.config.max_artifact_bytes.is_none() {
            return Ok(());
        }
        // Common ONNX/model filenames under cache_dir/model_name
        let candidates = [
            self.cache_dir
                .join(&self.config.model_name)
                .join("model.onnx"),
            self.cache_dir.join(&self.config.model_name),
            std::path::PathBuf::from(&self.config.model_name),
        ];
        for path in &candidates {
            if path.is_file() {
                if let Err(e) = super::config::verify_model_artifact(
                    path,
                    self.config.expected_sha256.as_deref(),
                    self.config.max_artifact_bytes,
                ) {
                    *self.health.write().await = EmbeddingHealth::Unavailable;
                    return Err(e);
                }
                tracing::info!(
                    path = %path.display(),
                    revision = ?self.config.model_revision,
                    "Local model artifact verified (S1.5b)"
                );
                return Ok(());
            }
        }
        // Pins configured but no file yet — load path will fail closed / mock as configured
        tracing::debug!("Model integrity pins set but no local artifact found yet under cache");
        Ok(())
    }

    /// Load the embedding model
    async fn load_model(&self) -> Result<()> {
        tracing::info!("Loading local embedding model: {}", self.config.model_name);

        // S1.5b: if a pinned path exists under cache, verify digest/size first
        self.verify_cached_artifact_if_configured().await?;

        #[cfg(feature = "local-embeddings")]
        {
            // Try to load real ONNX model; mock only when explicitly allowed (S1.5).
            match self.try_load_real_model().await {
                Ok(real_model) => {
                    let fallback_model = Box::new(RealEmbeddingModelWithFallback::new(
                        self.config.model_name.clone(),
                        self.config.embedding_dimension,
                        Some(real_model),
                    ));

                    let mut model_guard = self.model.write().await;
                    *model_guard = Some(fallback_model);
                    *self.health.write().await = EmbeddingHealth::Real;

                    tracing::info!("Local embedding model loaded with real ONNX backend");
                }
                Err(e) => {
                    tracing::warn!("Failed to load real embedding model: {}", e);
                    self.install_mock_or_fail("real model load failure").await?;
                }
            }
        }

        #[cfg(not(feature = "local-embeddings"))]
        {
            self.install_mock_or_fail("local-embeddings feature disabled")
                .await?;
        }

        Ok(())
    }

    /// Install mock embeddings when allowed; otherwise leave unavailable and error (S1.5).
    async fn install_mock_or_fail(&self, reason: &str) -> Result<()> {
        if !self.config.allow_mock_fallback {
            *self.health.write().await = EmbeddingHealth::Unavailable;
            tracing::error!(
                reason,
                "Refusing mock embedding fallback (allow_mock_fallback=false); provider unavailable"
            );
            anyhow::bail!(
                "Local embedding model unavailable ({reason}); set LocalConfig::allow_mock_fallback \
                 for tests/dev or enable the local-embeddings feature with a valid model"
            );
        }

        tracing::warn!(
            reason,
            "Falling back to mock embeddings - semantic search will not work correctly"
        );

        #[cfg(feature = "local-embeddings")]
        {
            let mock_fallback = Box::new(super::mock_model::RealEmbeddingModelWithFallback::new(
                self.config.model_name.clone(),
                self.config.embedding_dimension,
                None,
            ));
            let mut model_guard = self.model.write().await;
            *model_guard = Some(mock_fallback);
        }

        #[cfg(not(feature = "local-embeddings"))]
        {
            let mock_fallback = Box::new(super::mock_model::MockLocalModel::new(
                self.config.model_name.clone(),
                self.config.embedding_dimension,
            ));
            let mut model_guard = self.model.write().await;
            *model_guard = Some(mock_fallback);
        }

        *self.health.write().await = EmbeddingHealth::DegradedMock;
        tracing::info!("Local embedding model loaded with mock fallback (degraded)");
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
            "allow_mock_fallback": self.config.allow_mock_fallback,
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
        self.health().await.is_production_ready()
    }

    async fn health(&self) -> EmbeddingHealth {
        *self.health.read().await
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
            "cache_dir": self.cache_dir,
            "allow_mock_fallback": self.config.allow_mock_fallback,
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

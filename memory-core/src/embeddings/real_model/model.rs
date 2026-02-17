//! Real embedding model implementation using ONNX runtime
//!
//! This module provides actual ONNX-based embedding model implementation
//! that runs locally when the 'local-embeddings' feature is enabled.

use crate::embeddings::config::LocalConfig;
use anyhow::Result;

#[cfg(feature = "local-embeddings")]
use anyhow::Context;

#[cfg(feature = "local-embeddings")]
use {
    ort::execution_providers::CPUExecutionProvider, ort::session::Session, tokenizers::Tokenizer,
};

#[cfg(all(feature = "local-embeddings", feature = "reqwest"))]
use crate::embeddings::real_model::download::download_model;

/// Real embedding model using ONNX runtime
#[cfg(feature = "local-embeddings")]
pub struct RealEmbeddingModel {
    name: String,
    dimension: usize,
    #[allow(dead_code)] // Used in async spawn_blocking, compiler doesn't see cross-async usage
    tokenizer: Option<Tokenizer>,
    #[allow(dead_code)] // Used in async spawn_blocking, compiler doesn't see cross-async usage
    session: std::sync::Arc<tokio::sync::Mutex<Session>>,
}

#[cfg(feature = "local-embeddings")]
impl RealEmbeddingModel {
    /// Create a new real embedding model
    pub fn new(name: String, dimension: usize, tokenizer: Tokenizer, session: Session) -> Self {
        Self {
            name,
            dimension,
            tokenizer: Some(tokenizer),
            session: std::sync::Arc::new(tokio::sync::Mutex::new(session)),
        }
    }

    /// Generate real embedding using ONNX model
    pub async fn generate_real_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // Tokenize the input text
        let tokenizer = self.tokenizer.as_ref().context("Tokenizer not available")?;

        let encoding = tokenizer
            .encode(text, false)
            .map_err(|e| anyhow::anyhow!("Failed to encode text: {e}"))?;

        let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&id| i64::from(id)).collect();
        let attention_mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&mask| i64::from(mask))
            .collect();

        // Clone session Arc for the blocking task
        let session = self.session.clone();

        // Run inference in blocking context since ONNX is synchronous
        let embedding = tokio::task::spawn_blocking(move || {
            // Prepare input tensors using ndarray
            let input_ids_array = ndarray::Array1::from_vec(input_ids).into_dyn();
            let attention_mask_array = ndarray::Array1::from_vec(attention_mask).into_dyn();

            // Convert to ORT tensor references
            let input_ids_tensor = ort::value::TensorRef::from_array_view(input_ids_array.view())?;
            let attention_mask_tensor =
                ort::value::TensorRef::from_array_view(attention_mask_array.view())?;

            // Lock the session for exclusive access
            let mut session_guard = session.blocking_lock();
            let mut outputs = session_guard.run(ort::inputs! {
                "input_ids" => input_ids_tensor,
                "attention_mask" => attention_mask_tensor
            })?;

            // Extract embeddings from output (typically last hidden state pooled)
            let output = outputs.remove("last_hidden_state").ok_or_else(|| {
                anyhow::anyhow!("Missing 'last_hidden_state' output from ONNX model")
            })?;
            let embedding_array: ndarray::ArrayViewD<f32> = output.try_extract_array()?;

            // Average pooling over sequence length
            let shape = embedding_array.shape();
            if shape.len() != 3 {
                return Err(anyhow::anyhow!("Unexpected output shape: {shape:?}"));
            }

            let batch_size = shape[0];
            let seq_length = shape[1];
            let hidden_size = shape[2];

            if batch_size != 1 {
                return Err(anyhow::anyhow!("Expected batch size 1, got {batch_size}"));
            }

            // Average pooling over sequence length
            let mut pooled_embedding = vec![0.0f32; hidden_size];
            let data = embedding_array
                .as_slice()
                .ok_or_else(|| anyhow::anyhow!("Failed to convert embedding array to slice"))?;

            // Clippy: Indexing is necessary for accumulation across sequence dimension
            #[allow(clippy::needless_range_loop)]
            for seq_idx in 0..seq_length {
                for hidden_idx in 0..hidden_size {
                    let idx = seq_idx * hidden_size + hidden_idx;
                    pooled_embedding[hidden_idx] += data[idx];
                }
            }

            // Average the pooled embedding
            for value in &mut pooled_embedding {
                *value /= seq_length as f32;
            }

            Ok(pooled_embedding)
        })
        .await
        .map_err(|e| anyhow::anyhow!("Task execution failed: {e}"))??;

        // Normalize the embedding
        let normalized_embedding = crate::embeddings::provider::utils::normalize_vector(embedding);

        Ok(normalized_embedding)
    }

    /// Try to load real ONNX model
    ///
    /// Attempts to load the model from cache. If files don't exist and
    /// the `reqwest` feature is enabled, automatically downloads them
    /// from `HuggingFace` Hub.
    #[allow(clippy::unused_async)] // Required for API compatibility with async trait methods
    pub async fn try_load_from_cache(
        config: &LocalConfig,
        cache_dir: &std::path::Path,
    ) -> Result<Self> {
        // Model file paths
        let model_name = &config.model_name;
        let sanitized_name = model_name.replace('/', "_");
        let model_path = cache_dir.join(format!("{sanitized_name}.onnx"));
        let tokenizer_path = cache_dir.join(format!("{sanitized_name}_tokenizer.json"));

        // Check if model files exist
        if !model_path.exists() || !tokenizer_path.exists() {
            #[cfg(feature = "reqwest")]
            {
                tracing::info!(
                    "Model files not found. Attempting automatic download from HuggingFace..."
                );

                if let Err(e) = download_model(model_name, cache_dir).await {
                    tracing::error!("Failed to download model: {}", e);
                    return Err(anyhow::anyhow!(
                        "Model files not found and automatic download failed: {e}\n\
                         Manual download from https://huggingface.co/{model_name} required"
                    ));
                }

                tracing::info!("Model download completed successfully");
            }

            #[cfg(not(feature = "reqwest"))]
            {
                return Err(anyhow::anyhow!(
                    "Model files not found at {} and {}\n\
                     Enable 'reqwest' feature for automatic download or manually download from https://huggingface.co/{model_name}",
                    model_path.display(),
                    tokenizer_path.display()
                ));
            }
        }

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {e}"))?;

        // Load ONNX session
        let session = Session::builder()?
            .with_execution_providers([CPUExecutionProvider::default().build()])?
            .commit_from_file(&model_path)
            .map_err(|e| anyhow::anyhow!("Failed to load ONNX model: {e}"))?;

        tracing::info!(
            "Successfully loaded real ONNX model from {path}",
            path = model_path.display()
        );

        Ok(Self::new(
            model_name.clone(),
            config.embedding_dimension,
            tokenizer,
            session,
        ))
    }

    /// Get model name
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get embedding dimension
    #[allow(dead_code)]
    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

/// Stubs for when local-embeddings feature is not enabled
#[cfg(not(feature = "local-embeddings"))]
#[allow(dead_code)]
pub struct RealEmbeddingModel {
    name: String,
    dimension: usize,
}

#[cfg(not(feature = "local-embeddings"))]
impl RealEmbeddingModel {
    /// Create a stub real embedding model
    #[allow(dead_code)]
    pub fn new(name: String, dimension: usize, _tokenizer: (), _session: ()) -> Self {
        Self { name, dimension }
    }

    /// Try to load real model (always fails without feature)
    #[allow(dead_code)]
    #[allow(clippy::unused_async)]
    pub async fn try_load_from_cache(
        _config: &LocalConfig,
        _cache_dir: &std::path::Path,
    ) -> Result<Self> {
        Err(anyhow::anyhow!(
            "Real embedding model not available - enable local-embeddings feature and ensure ONNX models are available"
        ))
    }

    /// Generate real embedding (always fails without feature)
    #[allow(dead_code)]
    #[allow(clippy::unused_async)]
    pub async fn generate_real_embedding(&self, _text: &str) -> Result<Vec<f32>> {
        Err(anyhow::anyhow!(
            "Real embedding model not available - enable local-embeddings feature"
        ))
    }

    /// Get model name
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get embedding dimension
    #[allow(dead_code)]
    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

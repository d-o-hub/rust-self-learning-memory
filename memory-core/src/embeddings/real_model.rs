//! Real embedding model implementation using ONNX runtime
//!
//! This module provides the actual ONNX-based embedding model implementation
//! that runs locally when the 'local-embeddings' feature is enabled.

use super::config::ModelConfig;
use anyhow::{Context, Result};
use async_trait::async_trait;

#[cfg(feature = "local-embeddings")]
use {
    ort::{ExecutionProvider, Session, SessionBuilder},
    tokenizers::Tokenizer,
};

/// Real embedding model using ONNX runtime
#[cfg(feature = "local-embeddings")]
pub struct RealEmbeddingModel {
    name: String,
    dimension: usize,
    #[allow(dead_code)]
    tokenizer: Option<Tokenizer>,
    #[allow(dead_code)]
    session: Session,
}

#[cfg(feature = "local-embeddings")]
impl RealEmbeddingModel {
    /// Create a new real embedding model
    pub fn new(name: String, dimension: usize, tokenizer: Tokenizer, session: Session) -> Self {
        Self {
            name,
            dimension,
            tokenizer: Some(tokenizer),
            session,
        }
    }

    /// Generate real embedding using ONNX model
    pub async fn generate_real_embedding(&self, text: &str) -> Result<Vec<f32>> {
        use ort::{ExecutionProvider, RuntimeParameters};

        // Tokenize the input text
        let tokenizer = self.tokenizer.as_ref().context("Tokenizer not available")?;

        let encoding = tokenizer
            .encode(text, false)
            .map_err(|e| anyhow::anyhow!("Failed to encode text: {}", e))?;

        let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&id| id as i64).collect();
        let attention_mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&mask| mask as i64)
            .collect();

        // Prepare input tensors
        let input_ids_tensor = ort::Tensor::new(&[input_ids.len() as u64], input_ids)?;
        let attention_mask_tensor =
            ort::Tensor::new(&[attention_mask.len() as u64], attention_mask)?;

        // Run inference in blocking context since ONNX is synchronous
        let embedding = tokio::task::spawn_blocking(move || {
            let mut outputs = self.session.run(ort::inputs! {
                "input_ids" => &input_ids_tensor,
                "attention_mask" => &attention_mask_tensor
            })?;

            // Extract embeddings from output (typically last hidden state pooled)
            let output = outputs.remove("last_hidden_state").unwrap();
            let embedding_tensor: ort::Tensor<f32> = output.try_extract()?;

            // Average pooling over sequence length
            let shape = embedding_tensor.dims();
            if shape.len() != 3 {
                return Err(anyhow::anyhow!("Unexpected output shape: {:?}", shape));
            }

            let batch_size = shape[0] as usize;
            let seq_length = shape[1] as usize;
            let hidden_size = shape[2] as usize;

            if batch_size != 1 {
                return Err(anyhow::anyhow!("Expected batch size 1, got {}", batch_size));
            }

            // Average pooling over sequence length
            let mut pooled_embedding = vec![0.0f32; hidden_size];
            let data = embedding_tensor.as_slice();

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
        .map_err(|e| anyhow::anyhow!("Task execution failed: {}", e))??;

        // Normalize the embedding
        let normalized_embedding = super::provider::utils::normalize_vector(embedding);

        Ok(normalized_embedding)
    }

    /// Try to load real ONNX model
    pub async fn try_load_from_cache(
        config: &ModelConfig,
        cache_dir: &std::path::Path,
    ) -> Result<Self> {
        use ort::ExecutionProvider;

        // Model file paths - would typically be downloaded/cache
        let model_name = &config.model_name;
        let model_path = cache_dir.join(format!("{}.onnx", model_name.replace("/", "_")));
        let tokenizer_path =
            cache_dir.join(format!("{}_tokenizer.json", model_name.replace("/", "_")));

        // Check if model files exist
        if !model_path.exists() || !tokenizer_path.exists() {
            return Err(anyhow::anyhow!(
                "Model files not found at {} and {}\n\
                 Download models from https://huggingface.co/{} or set local-embeddings feature",
                model_path.display(),
                tokenizer_path.display(),
                model_name
            ));
        }

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

        // Load ONNX session
        let session = Session::builder()?
            .with_execution_providers([ExecutionProvider::CPU().clone()])
            .commit_from_file(&model_path)
            .map_err(|e| anyhow::anyhow!("Failed to load ONNX model: {}", e))?;

        tracing::info!(
            "Successfully loaded real ONNX model from {}",
            model_path.display()
        );

        Ok(Self::new(
            model_name.clone(),
            config.embedding_dimension,
            tokenizer,
            session,
        ))
    }

    /// Get model name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get embedding dimension
    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

/// Stubs for when local-embeddings feature is not enabled
#[cfg(not(feature = "local-embeddings"))]
pub struct RealEmbeddingModel {
    name: String,
    dimension: usize,
}

#[cfg(not(feature = "local-embeddings"))]
impl RealEmbeddingModel {
    /// Create a stub real embedding model
    pub fn new(name: String, dimension: usize, _tokenizer: (), _session: ()) -> Self {
        Self { name, dimension }
    }

    /// Try to load real model (always fails without feature)
    pub async fn try_load_from_cache(
        _config: &ModelConfig,
        _cache_dir: &std::path::Path,
    ) -> Result<Self> {
        Err(anyhow::anyhow!(
            "Real embedding model not available - enable local-embeddings feature and ensure ONNX models are available"
        ))
    }

    /// Generate real embedding (always fails without feature)
    pub async fn generate_real_embedding(&self, _text: &str) -> Result<Vec<f32>> {
        Err(anyhow::anyhow!(
            "Real embedding model not available - enable local-embeddings feature"
        ))
    }

    /// Get model name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get embedding dimension
    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

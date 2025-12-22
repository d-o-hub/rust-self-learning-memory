//! Mock embedding model implementation for testing
//!
//! This module provides mock implementations for local embedding models,
//! intended primarily for testing purposes. In production, these should
//! not be used as they provide deterministic but non-semantic embeddings.

use super::cosine_similarity;
use super::provider::EmbeddingProvider;
use anyhow::Result;
use async_trait::async_trait;

/// Mock implementation for local embedding model
/// Intended for testing only - in production should not be used
pub struct MockLocalModel {
    name: String,
    dimension: usize,
}

impl MockLocalModel {
    pub fn new(name: String, dimension: usize) -> Self {
        Self { name, dimension }
    }

    /// Generate a deterministic mock embedding for testing
    pub fn generate_mock_embedding(&self, text: &str) -> Vec<f32> {
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
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            let value = ((seed >> 16) as f32) / 32768.0 - 1.0; // Range [-1, 1]
            embedding.push(value);
        }

        // Normalize the vector
        super::provider::utils::normalize_vector(embedding)
    }
}

#[async_trait]
impl EmbeddingProvider for MockLocalModel {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
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

    async fn similarity(&self, text1: &str, text2: &str) -> Result<f32> {
        let embedding1 = self.generate_mock_embedding(text1);
        let embedding2 = self.generate_mock_embedding(text2);

        Ok(cosine_similarity(&embedding1, &embedding2))
    }

    fn embedding_dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        &self.name
    }

    async fn is_available(&self) -> bool {
        true
    }

    async fn warmup(&self) -> Result<()> {
        let _ = self.embed_text("warmup test").await?;
        Ok(())
    }

    fn metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "model": self.model_name(),
            "dimension": self.embedding_dimension(),
            "type": "mock",
            "provider": "testing"
        })
    }
}

/// Real embedding model with graceful fallback to mock
pub struct RealEmbeddingModelWithFallback {
    real_model: Option<RealEmbeddingModel>,
    mock_model: MockLocalModel,
}

impl RealEmbeddingModelWithFallback {
    pub fn new(name: String, dimension: usize, real_model: Option<RealEmbeddingModel>) -> Self {
        Self {
            real_model,
            mock_model: MockLocalModel::new(name.clone(), dimension),
        }
    }

    pub async fn embed_with_fallback(&self, text: &str) -> Result<Vec<f32>> {
        if let Some(ref real_model) = self.real_model {
            match real_model.generate_real_embedding(text).await {
                Ok(embedding) => {
                    tracing::info!(
                        "Using real embedding model for text: {}",
                        text.chars().take(20).collect::<String>()
                    );
                    Ok(embedding)
                }
                Err(e) => {
                    tracing::warn!("Real embedding failed, falling back to mock: {}", e);
                    tracing::warn!("PRODUCTION WARNING: Using mock embeddings - semantic search will not work correctly!");
                    Ok(self.mock_model.generate_mock_embedding(text))
                }
            }
        } else {
            tracing::warn!("PRODUCTION WARNING: Using mock embeddings - semantic search will not work correctly!");
            tracing::warn!(
                "To enable real embeddings, add local-embeddings feature and download ONNX models"
            );
            Ok(self.mock_model.generate_mock_embedding(text))
        }
    }

    pub async fn embed_batch_with_fallback(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::with_capacity(texts.len());

        if let Some(ref real_model) = self.real_model {
            let mut real_embeddings = Vec::new();
            let mut real_failed = false;

            for text in texts {
                if let Ok(embedding) = real_model.generate_real_embedding(text).await {
                    real_embeddings.push(embedding);
                } else {
                    real_failed = true;
                    break;
                }
            }

            if !real_failed {
                tracing::info!("Using real embeddings for batch of {} texts", texts.len());
                return Ok(real_embeddings);
            }

            tracing::warn!("Real embedding batch failed, falling back to mock for all texts");
            tracing::warn!("PRODUCTION WARNING: Using mock embeddings - semantic search will not work correctly!");
        } else {
            tracing::warn!("PRODUCTION WARNING: Using mock embeddings - semantic search will not work correctly!");
            tracing::warn!(
                "To enable real embeddings, add local-embeddings feature and download ONNX models"
            );
        }

        // Fallback to mock embeddings
        for text in texts {
            embeddings.push(self.mock_model.generate_mock_embedding(text));
        }

        Ok(embeddings)
    }
}

#[async_trait]
impl EmbeddingProvider for RealEmbeddingModelWithFallback {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        self.embed_with_fallback(text).await
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        self.embed_batch_with_fallback(texts).await
    }

    async fn similarity(&self, text1: &str, text2: &str) -> Result<f32> {
        // For similarity, use the mock model for consistency
        let embedding1 = self.mock_model.generate_mock_embedding(text1);
        let embedding2 = self.mock_model.generate_mock_embedding(text2);

        Ok(cosine_similarity(&embedding1, &embedding2))
    }

    fn embedding_dimension(&self) -> usize {
        self.mock_model.dimension
    }

    fn model_name(&self) -> &str {
        &self.mock_model.name
    }

    async fn is_available(&self) -> bool {
        true
    }

    async fn warmup(&self) -> Result<()> {
        let _ = self.embed_text("warmup test").await?;
        Ok(())
    }

    fn metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "model": self.model_name(),
            "dimension": self.embedding_dimension(),
            "type": "hybrid",
            "provider": "real-with-fallback"
        })
    }
}

#[async_trait]
impl super::local::LocalEmbeddingModel for RealEmbeddingModelWithFallback {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.embed_with_fallback(text).await
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        self.embed_batch_with_fallback(texts).await
    }

    fn name(&self) -> &str {
        &self.mock_model.name
    }

    fn dimension(&self) -> usize {
        self.mock_model.dimension
    }
}

/// Real embedding model using ONNX runtime (conditional compilation)
#[cfg(feature = "local-embeddings")]
pub struct RealEmbeddingModel {
    name: String,
    dimension: usize,
    #[allow(dead_code)]
    tokenizer: Option<tokenizers::Tokenizer>,
    #[allow(dead_code)]
    session: ort::Session,
}

#[cfg(feature = "local-embeddings")]
impl RealEmbeddingModel {
    pub fn new(
        name: String,
        dimension: usize,
        tokenizer: tokenizers::Tokenizer,
        session: ort::Session,
    ) -> Self {
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
}

/// Stubs for when local-embeddings feature is not enabled
#[cfg(not(feature = "local-embeddings"))]
pub struct RealEmbeddingModel {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    dimension: usize,
}

#[cfg(not(feature = "local-embeddings"))]
impl RealEmbeddingModel {
    #[allow(dead_code)]
    pub fn new(name: String, dimension: usize, _tokenizer: (), _session: ()) -> Self {
        Self { name, dimension }
    }

    #[allow(clippy::unused_async)]
    pub async fn generate_real_embedding(&self, _text: &str) -> Result<Vec<f32>> {
        Err(anyhow::anyhow!(
            "Real embedding model not available - enable local-embeddings feature"
        ))
    }
}

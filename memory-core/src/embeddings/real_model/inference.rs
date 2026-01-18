//! Inference functionality for ONNX-based embedding models
//!
//! Provides the actual embedding generation logic using ONNX runtime
//! when the 'local-embeddings' feature is enabled.

use crate::embeddings::config::ModelConfig;
use anyhow::Result;

#[cfg(feature = "local-embeddings")]
use anyhow::Context;

#[cfg(feature = "local-embeddings")]
use {ort::session::Session, tokenizers::Tokenizer};

/// Real embedding model using ONNX runtime
#[cfg(feature = "local-embeddings")]
pub struct RealEmbeddingModel {
    name: String,
    dimension: usize,
    #[allow(dead_code)]
    tokenizer: Option<Tokenizer>,
    #[allow(dead_code)]
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
        let normalized_embedding = super::provider::utils::normalize_vector(embedding);

        Ok(normalized_embedding)
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

    /// Try to load real ONNX model from cache
    #[allow(clippy::unused_async)]
    pub async fn try_load_from_cache(
        config: &ModelConfig,
        cache_dir: &std::path::Path,
    ) -> Result<Self> {
        super::loader::try_load_model(config, cache_dir).await
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
        _config: &ModelConfig,
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_url_construction() {
        let model_name = "sentence-transformers/all-MiniLM-L6-v2";
        let sanitized = model_name.replace('/', "_");
        let base_url = format!("https://huggingface.co/{model_name}/resolve/main");

        assert_eq!(sanitized, "sentence-transformers_all-MiniLM-L6-v2");
        assert_eq!(
            base_url,
            "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main"
        );

        let files = [
            format!("{sanitized}.onnx"),
            format!("{sanitized}_tokenizer.json"),
            format!("{sanitized}_config.json"),
        ];

        assert_eq!(files[0], "sentence-transformers_all-MiniLM-L6-v2.onnx");
        assert_eq!(
            files[1],
            "sentence-transformers_all-MiniLM-L6-v2_tokenizer.json"
        );
        assert_eq!(
            files[2],
            "sentence-transformers_all-MiniLM-L6-v2_config.json"
        );
    }

    #[test]
    fn test_progress_calculation() {
        let total = 10_485_760u64; // 10 MB
        let downloaded = 5_242_880u64; // 5 MB
        let progress = (downloaded as f64 / total as f64) * 100.0;

        assert!((progress - 50.0).abs() < 0.01);

        let total_mb = total as f64 / 1_048_576.0;
        let downloaded_mb = downloaded as f64 / 1_048_576.0;

        assert!((total_mb - 10.0).abs() < 0.01);
        assert!((downloaded_mb - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_retry_backoff() {
        let max_retries = 3;
        for retry_count in 0..max_retries {
            let backoff_duration = std::time::Duration::from_millis(100 * 2_u64.pow(retry_count));
            let expected_ms = 100 * 2_u64.pow(retry_count);

            assert_eq!(backoff_duration.as_millis() as u64, expected_ms);
        }

        // Verify exponential backoff: 100ms, 200ms, 400ms
        assert_eq!(
            std::time::Duration::from_millis(100 * 2_u64.pow(0)).as_millis(),
            100
        );
        assert_eq!(
            std::time::Duration::from_millis(100 * 2_u64.pow(1)).as_millis(),
            200
        );
        assert_eq!(
            std::time::Duration::from_millis(100 * 2_u64.pow(2)).as_millis(),
            400
        );
    }

    #[test]
    fn test_speed_calculation() {
        // Test speed calculation
        let downloaded = 5_242_880u64; // 5 MB
        let elapsed = std::time::Duration::from_secs(2);
        let speed = (downloaded as f64 / 1_048_576.0) / elapsed.as_secs_f64();

        assert!((speed - 2.5).abs() < 0.01);

        // Test zero elapsed time handling
        let elapsed_zero = std::time::Duration::from_secs(0);
        let speed_zero = if elapsed_zero.as_secs_f64() > 0.0 {
            (downloaded as f64 / 1_048_576.0) / elapsed_zero.as_secs_f64()
        } else {
            0.0
        };

        assert_eq!(speed_zero, 0.0);
    }
}

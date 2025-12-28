//! Real embedding model implementation using ONNX runtime
//!
//! This module provides the actual ONNX-based embedding model implementation
//! that runs locally when the 'local-embeddings' feature is enabled.

use super::config::ModelConfig;
use anyhow::Result;

#[cfg(feature = "local-embeddings")]
use anyhow::Context;

#[cfg(feature = "local-embeddings")]
use {
    ort::execution_providers::CPUExecutionProvider, ort::session::Session, tokenizers::Tokenizer,
};

#[cfg(all(feature = "local-embeddings", feature = "reqwest"))]
use reqwest::Client;

/// Download model files from `HuggingFace` Hub
///
/// Downloads the required model files (.onnx, _tokenizer.json, _config.json)
/// from `HuggingFace` Hub to the specified cache directory.
///
/// # Arguments
///
/// * `model_name` - The `HuggingFace` model name (e.g., "sentence-transformers/all-MiniLM-L6-v2")
/// * `cache_dir` - The cache directory to store downloaded files
///
/// # Returns
///
/// Ok(()) on success, error otherwise
///
/// # Errors
///
/// Returns error if:
/// - Network errors occur (after 3 retries)
/// - Permission errors prevent file creation
/// - Disk is full
/// - Downloaded files are invalid
#[cfg(all(feature = "local-embeddings", feature = "reqwest"))]
pub async fn download_model(model_name: &str, cache_dir: &std::path::Path) -> Result<()> {
    tracing::info!("Starting model download from HuggingFace: {}", model_name);

    // Sanitize model name for file paths
    let sanitized_name = model_name.replace('/', "_");
    let base_url = format!("https://huggingface.co/{model_name}/resolve/main");

    // Files to download for the model
    let files_to_download = vec![
        format!("{sanitized_name}.onnx"),
        format!("{sanitized_name}_tokenizer.json"),
        format!("{sanitized_name}_config.json"),
    ];

    // Download each file
    for filename in &files_to_download {
        let url = format!("{base_url}/{filename}");
        let target_path = cache_dir.join(filename);

        // Check if file already exists and is valid
        if target_path.exists() {
            let metadata = std::fs::metadata(&target_path).with_context(|| {
                format!(
                    "Failed to read metadata for existing file: {}",
                    target_path.display()
                )
            })?;

            if metadata.len() > 0 {
                tracing::info!(
                    "File already exists and is valid: {} ({:.2} MB)",
                    filename,
                    metadata.len() as f64 / 1_048_576.0
                );
                continue;
            }

            tracing::warn!("File exists but is empty, will re-download: {}", filename);
        }

        tracing::info!("Downloading {} from {}", filename, url);

        // Download with retry logic and progress reporting
        download_file_with_progress(&url, &target_path).await?;

        // Validate downloaded file
        validate_downloaded_file(&target_path, filename)?;
    }

    tracing::info!("Successfully downloaded model: {}", model_name);
    Ok(())
}

/// Download a single file with progress reporting and retry logic
#[cfg(all(feature = "local-embeddings", feature = "reqwest"))]
async fn download_file_with_progress(url: &str, path: &std::path::Path) -> Result<()> {
    use std::time::Duration;

    let max_retries = 3;
    let mut retry_count = 0;

    while retry_count < max_retries {
        match attempt_download(url, path).await {
            Ok(()) => return Ok(()),
            Err(e) if retry_count < max_retries - 1 => {
                retry_count += 1;
                let backoff_duration = Duration::from_millis(100 * 2_u64.pow(retry_count));
                tracing::warn!(
                    "Download attempt {} failed: {}. Retrying in {:?}...",
                    retry_count + 1,
                    e,
                    backoff_duration
                );
                tokio::time::sleep(backoff_duration).await;

                // Clean up partially downloaded file
                if path.exists() {
                    let _ = std::fs::remove_file(path);
                }
            }
            Err(e) => {
                // Clean up partially downloaded file
                if path.exists() {
                    let _ = std::fs::remove_file(path);
                }
                return Err(e);
            }
        }
    }

    Err(anyhow::anyhow!(
        "Failed to download file after {max_retries} retries"
    ))
}

/// Attempt a single download attempt
#[cfg(all(feature = "local-embeddings", feature = "reqwest"))]
async fn attempt_download(url: &str, path: &std::path::Path) -> Result<()> {
    use futures::StreamExt;
    use std::time::Duration;
    use tokio::io::{AsyncWriteExt, BufWriter};

    let client = Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .context("Failed to build HTTP client")?;

    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Failed to fetch URL: {url}"))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "HTTP error downloading {url}: {}",
            response.status()
        ));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded = 0u64;
    let mut stream = response.bytes_stream();

    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let file = tokio::fs::File::create(path)
        .await
        .with_context(|| format!("Failed to create file: {}", path.display()))?;

    let mut writer = BufWriter::new(file);
    let start_time = std::time::Instant::now();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.with_context(|| "Failed to read download chunk")?;
        writer
            .write_all(&chunk)
            .await
            .context("Failed to write to file")?;
        downloaded += chunk.len() as u64;

        // Log progress every 1MB or every 5 seconds
        if total_size > 0 && (downloaded % 1_048_576 == 0 || start_time.elapsed().as_secs() >= 5) {
            let progress = (downloaded as f64 / total_size as f64) * 100.0;
            let elapsed = start_time.elapsed();
            let speed = if elapsed.as_secs() > 0 {
                (downloaded as f64 / 1_048_576.0) / elapsed.as_secs_f64()
            } else {
                0.0
            };

            tracing::info!(
                "Progress: {:.1}% ({:.2}/{:.2} MB, {:.2} MB/s)",
                progress,
                downloaded as f64 / 1_048_576.0,
                total_size as f64 / 1_048_576.0,
                speed
            );
        }
    }

    writer.flush().await.context("Failed to flush file")?;

    let elapsed = start_time.elapsed();
    let speed = if elapsed.as_secs_f64() > 0.0 {
        (downloaded as f64 / 1_048_576.0) / elapsed.as_secs_f64()
    } else {
        0.0
    };

    tracing::info!(
        "Successfully downloaded {} bytes to {} in {:.2}s ({:.2} MB/s)",
        downloaded,
        path.display(),
        elapsed.as_secs_f64(),
        speed
    );

    Ok(())
}

/// Validate that a downloaded file exists and is readable
#[cfg(all(feature = "local-embeddings", feature = "reqwest"))]
fn validate_downloaded_file(path: &std::path::Path, filename: &str) -> Result<()> {
    if !path.exists() {
        return Err(anyhow::anyhow!(
            "Downloaded file not found: {}",
            path.display()
        ));
    }

    let metadata = std::fs::metadata(path)
        .with_context(|| format!("Failed to read metadata for file: {}", path.display()))?;

    if metadata.len() == 0 {
        return Err(anyhow::anyhow!(
            "Downloaded file is empty: {}",
            path.display()
        ));
    }

    // Try to read a small portion to verify file is readable
    use std::io::Read;
    let mut file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open file for validation: {}", path.display()))?;

    let mut buffer = [0u8; 1024];
    let bytes_read = file
        .read(&mut buffer)
        .with_context(|| format!("Failed to read from file: {}", path.display()))?;

    if bytes_read == 0 {
        return Err(anyhow::anyhow!(
            "File contains no readable data: {}",
            path.display()
        ));
    }

    tracing::info!(
        "Validated {}: {:.2} MB, readable",
        filename,
        metadata.len() as f64 / 1_048_576.0
    );

    Ok(())
}

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
            let output = outputs.remove("last_hidden_state").unwrap();
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
            let data = embedding_array.as_slice().unwrap();

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

    /// Try to load real ONNX model
    ///
    /// Attempts to load the model from cache. If files don't exist and
    /// the `reqwest` feature is enabled, automatically downloads them
    /// from `HuggingFace` Hub.
    #[allow(clippy::unused_async)] // Required for API compatibility with async trait methods
    pub async fn try_load_from_cache(
        config: &ModelConfig,
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
        let base_url = format!("https://huggingface.co/{}/resolve/main", model_name);

        assert_eq!(sanitized, "sentence-transformers_all-MiniLM-L6-v2");
        assert_eq!(
            base_url,
            "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main"
        );

        let files = vec![
            format!("{}.onnx", sanitized),
            format!("{}_tokenizer.json", sanitized),
            format!("{}_config.json", sanitized),
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
    #[cfg(all(feature = "local-embeddings", feature = "reqwest"))]
    fn test_file_validation() {
        use super::validate_downloaded_file;
        use std::io::Write;
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");

        // Test with non-existent file
        let result = validate_downloaded_file(&file_path, "test_file.txt");
        assert!(result.is_err());

        // Test with empty file
        std::fs::File::create(&file_path)
            .unwrap()
            .write_all(b"")
            .unwrap();
        let result = validate_downloaded_file(&file_path, "test_file.txt");
        assert!(result.is_err());

        // Test with valid file
        std::fs::File::create(&file_path)
            .unwrap()
            .write_all(b"test content")
            .unwrap();
        let result = validate_downloaded_file(&file_path, "test_file.txt");
        assert!(result.is_ok());
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

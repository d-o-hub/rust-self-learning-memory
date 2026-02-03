//! Download utilities for embedding models
//!
//! Handles downloading model files from `HuggingFace` Hub.

#[cfg(feature = "local-embeddings")]
use anyhow::Context;

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
pub async fn download_model(model_name: &str, cache_dir: &std::path::Path) -> anyhow::Result<()> {
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
async fn download_file_with_progress(url: &str, path: &std::path::Path) -> anyhow::Result<()> {
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
async fn attempt_download(url: &str, path: &std::path::Path) -> anyhow::Result<()> {
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
pub fn validate_downloaded_file(path: &std::path::Path, filename: &str) -> anyhow::Result<()> {
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

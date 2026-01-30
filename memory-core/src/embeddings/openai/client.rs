//! OpenAI embedding provider client implementation.
//!
//! Contains the `OpenAIEmbeddingProvider` struct and its implementation.

#[cfg(feature = "openai")]
use super::super::config::openai::{EncodingFormat, OpenAIConfig, OpenAIEmbeddingInput};
#[cfg(feature = "openai")]
use crate::embeddings::provider::EmbeddingProvider;
#[cfg(feature = "openai")]
use anyhow::{Context, Result};
#[cfg(feature = "openai")]
use async_trait::async_trait;
#[cfg(feature = "openai")]
use std::time::Instant;

#[cfg(feature = "openai")]
/// OpenAI embedding provider
///
/// Uses OpenAI's embedding API for high-quality semantic embeddings.
/// Supports text-embedding-3.x features including custom dimensions.
///
/// # Supported Models
/// - `text-embedding-ada-002` (1536 dimensions, legacy)
/// - `text-embedding-3-small` (1536 dimensions, improved)
/// - `text-embedding-3-large` (3072 dimensions, highest quality)
///
/// # Example
/// ```no_run
/// use memory_core::embeddings::openai::{OpenAIEmbeddingProvider, OpenAIConfig};
/// use memory_core::embeddings::config::openai::OpenAIModel;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let config = OpenAIConfig::text_embedding_3_small();
///     let provider = OpenAIEmbeddingProvider::new(
///         "your-api-key".to_string(),
///         config
///     )?;
///
///     let embedding = provider.embed_text("Hello world").await?;
///     println!("Generated embedding with {} dimensions", embedding.len());
///     Ok(())
/// }
/// ```
pub struct OpenAIEmbeddingProvider {
    /// OpenAI API key
    api_key: String,
    /// OpenAI-specific configuration
    config: OpenAIConfig,
    /// HTTP client for API requests
    client: reqwest::Client,
}

#[cfg(feature = "openai")]
impl OpenAIEmbeddingProvider {
    /// Create a new OpenAI embedding provider
    ///
    /// # Arguments
    /// * `api_key` - OpenAI API key
    /// * `config` - OpenAI-specific configuration
    ///
    /// # Returns
    /// Configured OpenAI embedding provider
    pub fn new(api_key: String, config: OpenAIConfig) -> anyhow::Result<Self> {
        config.validate()?;

        let timeout_secs = config.optimization.get_timeout_seconds();

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .pool_max_idle_per_host(config.optimization.connection_pool_size)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            api_key,
            config,
            client,
        })
    }

    /// Make embedding request to OpenAI API with retry logic
    async fn request_embeddings(
        &self,
        input: OpenAIEmbeddingInput,
    ) -> Result<super::types::EmbeddingResponse> {
        use super::super::config::openai::OpenAIEmbeddingRequest;

        let url = self.config.embeddings_url();
        let max_retries = self.config.optimization.max_retries;
        let base_delay_ms = self.config.optimization.retry_delay_ms;

        // Build request with provider-specific parameters
        let request = OpenAIEmbeddingRequest {
            input,
            model: self.config.model.model_name().to_string(),
            encoding_format: Some(match self.config.encoding_format {
                EncodingFormat::Float => "float".to_string(),
                EncodingFormat::Base64 => "base64".to_string(),
            }),
            dimensions: self.config.dimensions,
        };

        let mut last_error = None;

        for attempt in 0..=max_retries {
            if attempt > 0 {
                let delay_ms = base_delay_ms * 2u64.pow(attempt - 1);
                tracing::debug!("Retry attempt {attempt}/{max_retries}, waiting {delay_ms}ms");
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
            }

            let response = match self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    tracing::warn!("Request failed: {e}");
                    last_error = Some(anyhow::Error::from(e));
                    continue;
                }
            };

            let status = response.status();

            if status.is_success() {
                let embedding_response: super::types::EmbeddingResponse = response
                    .json()
                    .await
                    .context("Failed to parse OpenAI API response")?;
                return Ok(embedding_response);
            }

            if status.as_u16() == 429 || status.is_server_error() {
                let error_text = response.text().await.unwrap_or_default();
                tracing::warn!("Retryable error {status}: {error_text}");
                last_error = Some(anyhow::anyhow!("OpenAI API error {status}: {error_text}"));
                continue;
            }

            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API error {status}: {error_text}");
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
    }

    /// Process a single batch chunk
    async fn embed_batch_chunk(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let input = OpenAIEmbeddingInput::Batch(texts.to_vec());
        let response = self.request_embeddings(input).await?;

        if response.data.len() != texts.len() {
            anyhow::bail!(
                "OpenAI API returned {} embeddings for {} texts",
                response.data.len(),
                texts.len()
            );
        }

        let mut data = response.data;
        data.sort_by_key(|item| item.index);

        let embeddings: Vec<Vec<f32>> = data.into_iter().map(|item| item.embedding).collect();

        tracing::debug!(
            "Generated {} embeddings in batch, {} tokens",
            embeddings.len(),
            response.usage.total_tokens
        );

        Ok(embeddings)
    }
}

#[cfg(feature = "openai")]
#[async_trait]
impl EmbeddingProvider for OpenAIEmbeddingProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let start_time = Instant::now();

        let input = OpenAIEmbeddingInput::Single(text.to_string());
        let response = self.request_embeddings(input).await?;

        if response.data.is_empty() {
            anyhow::bail!("OpenAI API returned no embeddings");
        }

        let embedding = response.data[0].embedding.clone();
        let generation_time = start_time.elapsed().as_millis() as u64;

        tracing::debug!(
            "Generated OpenAI embedding in {generation_time}ms, {} tokens, {} dimensions",
            response.usage.total_tokens,
            embedding.len()
        );

        Ok(embedding)
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let start_time = Instant::now();
        let max_batch_size = self.config.optimization.get_max_batch_size();

        if texts.len() <= max_batch_size {
            return self.embed_batch_chunk(texts).await;
        }

        tracing::debug!(
            "Splitting {} texts into batches of max {} items",
            texts.len(),
            max_batch_size
        );

        let mut all_embeddings = Vec::with_capacity(texts.len());

        for chunk in texts.chunks(max_batch_size) {
            let chunk_embeddings = self.embed_batch_chunk(chunk).await?;
            all_embeddings.extend(chunk_embeddings);
        }

        let generation_time = start_time.elapsed().as_millis() as u64;

        tracing::debug!(
            "Generated {} OpenAI embeddings in {generation_time}ms ({} batches)",
            all_embeddings.len(),
            (texts.len() + max_batch_size - 1) / max_batch_size
        );

        Ok(all_embeddings)
    }

    fn embedding_dimension(&self) -> usize {
        self.config.effective_dimension()
    }

    fn model_name(&self) -> &str {
        self.config.model.model_name()
    }

    async fn is_available(&self) -> bool {
        self.embed_text("test").await.is_ok()
    }

    async fn warmup(&self) -> Result<()> {
        let _embedding = self.embed_text("warmup test").await?;
        Ok(())
    }

    fn metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "model": self.model_name(),
            "dimension": self.embedding_dimension(),
            "type": "openai",
            "provider": "OpenAI",
            "encoding_format": match self.config.encoding_format {
                EncodingFormat::Float => "float",
                EncodingFormat::Base64 => "base64",
            },
            "base_url": self.config.base_url
        })
    }
}

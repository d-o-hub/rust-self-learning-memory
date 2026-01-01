//! `OpenAI` embedding provider for high-quality cloud-based embeddings

use super::config::ModelConfig;
use anyhow::Result;

#[cfg(feature = "openai")]
use {
    super::provider::EmbeddingProvider,
    anyhow::Context,
    async_trait::async_trait,
    serde::{Deserialize, Serialize},
    std::time::Instant,
};

/// `OpenAI` embedding provider
///
/// Uses `OpenAI`'s embedding API for high-quality semantic embeddings.
/// Requires an API key and internet connection.
///
/// # Supported Models
/// - `text-embedding-ada-002` (1536 dimensions, legacy)
/// - `text-embedding-3-small` (1536 dimensions, improved)
/// - `text-embedding-3-large` (3072 dimensions, highest quality)
///
/// # Example
/// ```no_run
/// use memory_core::embeddings::{OpenAIEmbeddingProvider, ModelConfig};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let config = ModelConfig::openai_3_small();
///     let provider = OpenAIEmbeddingProvider::new(
///         "your-api-key".to_string(),
///         config
///     );
///     
///     let embedding = provider.embed_text("Hello world").await?;
///     println!("Generated embedding with {} dimensions", embedding.len());
///     Ok(())
/// }
/// ```
#[cfg(feature = "openai")]
pub struct OpenAIEmbeddingProvider {
    /// `OpenAI` API key
    api_key: String,
    /// Model configuration
    config: ModelConfig,
    /// HTTP client for API requests
    client: reqwest::Client,
    /// Base URL for `OpenAI` API
    base_url: String,
}

#[cfg(feature = "openai")]
impl OpenAIEmbeddingProvider {
    /// Create a new `OpenAI` embedding provider
    ///
    /// # Arguments
    /// * `api_key` - `OpenAI` API key
    /// * `config` - Model configuration
    ///
    /// # Returns
    /// Configured `OpenAI` embedding provider
    pub fn new(api_key: String, config: ModelConfig) -> anyhow::Result<Self> {
        let timeout_secs = config.optimization.get_timeout_seconds();

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .pool_max_idle_per_host(config.optimization.connection_pool_size)
            .build()
            .context("Failed to create HTTP client")?;

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());

        Ok(Self {
            api_key,
            config,
            client,
            base_url,
        })
    }

    /// Create provider with custom base URL (for Azure `OpenAI`, etc.)
    pub fn with_custom_url(
        api_key: String,
        config: ModelConfig,
        base_url: String,
    ) -> anyhow::Result<Self> {
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
            base_url,
        })
    }

    /// Make embedding request to `OpenAI` API with retry logic
    async fn request_embeddings(&self, input: EmbeddingInput) -> Result<EmbeddingResponse> {
        let url = self.config.get_embeddings_url();
        let max_retries = self.config.optimization.max_retries;
        let base_delay_ms = self.config.optimization.retry_delay_ms;

        let request = EmbeddingRequest {
            input,
            model: self.config.model_name.clone(),
            encoding_format: Some("float".to_string()),
            dimensions: None, // Let OpenAI use default for the model
        };

        let mut last_error = None;

        for attempt in 0..=max_retries {
            if attempt > 0 {
                let delay_ms = base_delay_ms * 2u64.pow(attempt - 1); // Exponential backoff
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

            // Check for retryable errors
            if status.is_success() {
                let embedding_response: EmbeddingResponse = response
                    .json()
                    .await
                    .context("Failed to parse OpenAI API response")?;
                return Ok(embedding_response);
            }

            // Retry on rate limit (429) or server errors (5xx)
            if status.as_u16() == 429 || status.is_server_error() {
                let error_text = response.text().await.unwrap_or_default();
                tracing::warn!("Retryable error {status}: {error_text}");
                last_error = Some(anyhow::anyhow!("OpenAI API error {status}: {error_text}"));
                continue;
            }

            // Non-retryable error
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API error {status}: {error_text}");
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
    }
}

#[cfg(feature = "openai")]
#[async_trait]
impl EmbeddingProvider for OpenAIEmbeddingProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let start_time = Instant::now();

        let input = EmbeddingInput::Single(text.to_string());
        let response = self.request_embeddings(input).await?;

        if response.data.is_empty() {
            anyhow::bail!("OpenAI API returned no embeddings");
        }

        let embedding = response.data[0].embedding.clone();
        let generation_time = start_time.elapsed().as_millis() as u64;

        tracing::debug!(
            "Generated `OpenAI` embedding in {generation_time}ms, {} tokens, {} dimensions",
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

        // If texts fit in one batch, process directly
        if texts.len() <= max_batch_size {
            return self.request_batch(texts).await;
        }

        // Split into multiple batches
        tracing::debug!(
            "Splitting {} texts into batches of max {} items",
            texts.len(),
            max_batch_size
        );

        let mut all_embeddings = Vec::with_capacity(texts.len());

        for chunk in texts.chunks(max_batch_size) {
            let chunk_embeddings = self.request_batch(chunk).await?;
            all_embeddings.extend(chunk_embeddings);
        }

        let generation_time = start_time.elapsed().as_millis() as u64;

        tracing::debug!(
            "Generated {} `OpenAI` embeddings in {generation_time}ms ({} batches)",
            all_embeddings.len(),
            (texts.len() + max_batch_size - 1) / max_batch_size
        );

        Ok(all_embeddings)
    }

    /// Process a single batch request
    async fn request_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let input = EmbeddingInput::Batch(texts.to_vec());
        let response = self.request_embeddings(input).await?;

        if response.data.len() != texts.len() {
            anyhow::bail!(
                "OpenAI API returned {} embeddings for {} texts",
                response.data.len(),
                texts.len()
            );
        }

        // Sort by index to ensure correct order
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

    fn embedding_dimension(&self) -> usize {
        self.config.embedding_dimension
    }

    fn model_name(&self) -> &str {
        &self.config.model_name
    }

    async fn is_available(&self) -> bool {
        // Test with a simple request
        self.embed_text("test").await.is_ok()
    }

    async fn warmup(&self) -> Result<()> {
        // Test embedding generation to warm up the connection
        let _embedding = self.embed_text("warmup test").await?;
        Ok(())
    }

    fn metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "model": self.model_name(),
            "dimension": self.embedding_dimension(),
            "type": "openai",
            "provider": "OpenAI",
            "base_url": self.base_url
        })
    }
}

// OpenAI API request/response structures

#[cfg(feature = "openai")]
#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    input: EmbeddingInput,
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    encoding_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dimensions: Option<usize>,
}

#[cfg(feature = "openai")]
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum EmbeddingInput {
    Single(String),
    Batch(Vec<String>),
}

#[cfg(feature = "openai")]
#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
    #[allow(dead_code)] // Part of OpenAI API response, kept for future use
    model: String,
    usage: Usage,
}

#[cfg(feature = "openai")]
#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: usize,
    #[allow(dead_code)] // Part of OpenAI API response, indicates object type
    object: String, // Should be "embedding"
}

#[cfg(feature = "openai")]
#[derive(Debug, Deserialize)]
struct Usage {
    #[allow(dead_code)] // Part of OpenAI API response, not currently used
    prompt_tokens: usize,
    total_tokens: usize,
}

/// Utility functions for `OpenAI` provider
pub mod utils {
    use super::{ModelConfig, Result};

    /// Validate `OpenAI` API key format
    #[allow(dead_code)]
    pub fn validate_api_key(api_key: &str) -> Result<()> {
        if api_key.is_empty() {
            anyhow::bail!("OpenAI API key is empty");
        }

        if !api_key.starts_with("sk-") {
            anyhow::bail!("OpenAI API key should start with 'sk-'");
        }

        if api_key.len() < 20 {
            anyhow::bail!("OpenAI API key appears to be too short");
        }

        Ok(())
    }

    /// Get the appropriate model configuration for different use cases
    #[allow(dead_code)]
    pub fn get_recommended_model(use_case: OpenAIModelUseCase) -> ModelConfig {
        match use_case {
            OpenAIModelUseCase::Balanced => ModelConfig::openai_3_small(),
            OpenAIModelUseCase::Quality => ModelConfig::openai_3_large(),
            OpenAIModelUseCase::Legacy => ModelConfig::openai_ada_002(),
        }
    }

    /// Calculate approximate cost for embedding generation
    ///
    /// Based on `OpenAI`'s pricing as of 2024. Prices may change.
    #[allow(dead_code)]
    pub fn estimate_cost(num_tokens: usize, model: &str) -> f64 {
        let cost_per_million_tokens = match model {
            "text-embedding-ada-002" => 0.10,
            "text-embedding-3-small" => 0.02,
            "text-embedding-3-large" => 0.13,
            _ => 0.10, // Default to ada-002 pricing
        };

        (num_tokens as f64 / 1_000_000.0) * cost_per_million_tokens
    }

    /// Estimate token count for text (approximate)
    ///
    /// This is a rough estimate. Actual token count may differ.
    #[allow(dead_code)]
    pub fn estimate_tokens(text: &str) -> usize {
        // Rough estimate: ~1 token per 4 characters for English text
        (text.len() as f64 / 4.0).ceil() as usize
    }

    /// Use cases for `OpenAI` model selection
    #[allow(dead_code)]
    pub enum OpenAIModelUseCase {
        /// Balanced performance and cost (text-embedding-3-small)
        Balanced,
        /// Highest quality (text-embedding-3-large)
        Quality,
        /// Legacy compatibility (text-embedding-ada-002)
        Legacy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_api_key() {
        // Valid key format
        assert!(utils::validate_api_key("sk-1234567890abcdefghijklmnop").is_ok());

        // Invalid formats
        assert!(utils::validate_api_key("").is_err());
        assert!(utils::validate_api_key("invalid-key").is_err());
        assert!(utils::validate_api_key("sk-short").is_err());
    }

    #[test]
    fn test_estimate_cost() {
        // Test cost calculation for different models
        let tokens = 1000;

        let ada_cost = utils::estimate_cost(tokens, "text-embedding-ada-002");
        let small_cost = utils::estimate_cost(tokens, "text-embedding-3-small");
        let large_cost = utils::estimate_cost(tokens, "text-embedding-3-large");

        assert!(ada_cost > 0.0);
        assert!(small_cost > 0.0);
        assert!(large_cost > 0.0);

        // 3-small should be cheaper than ada-002
        assert!(small_cost < ada_cost);

        // 3-large should be most expensive
        assert!(large_cost > small_cost);
        assert!(large_cost > ada_cost);
    }

    #[test]
    fn test_estimate_tokens() {
        let text = "Hello world, this is a test sentence.";
        let tokens = utils::estimate_tokens(text);

        assert!(tokens > 0);
        assert!(tokens < text.len()); // Should be less than character count
    }

    #[test]
    fn test_recommended_models() {
        let balanced = utils::get_recommended_model(utils::OpenAIModelUseCase::Balanced);
        assert_eq!(balanced.model_name, "text-embedding-3-small");

        let quality = utils::get_recommended_model(utils::OpenAIModelUseCase::Quality);
        assert_eq!(quality.model_name, "text-embedding-3-large");

        let legacy = utils::get_recommended_model(utils::OpenAIModelUseCase::Legacy);
        assert_eq!(legacy.model_name, "text-embedding-ada-002");
    }

    #[cfg(feature = "openai")]
    #[tokio::test]
    async fn test_provider_creation() -> anyhow::Result<()> {
        let config = ModelConfig::openai_3_small();
        let provider = OpenAIEmbeddingProvider::new("sk-test-key-1234567890".to_string(), config)?;

        assert_eq!(provider.model_name(), "text-embedding-3-small");
        assert_eq!(provider.embedding_dimension(), 1536);
        assert_eq!(provider.base_url, "https://api.openai.com/v1");
        Ok(())
    }

    #[cfg(feature = "openai")]
    #[tokio::test]
    async fn test_custom_url_provider() -> anyhow::Result<()> {
        let config = ModelConfig::openai_3_small();
        let custom_url = "https://custom.openai.azure.com/v1";
        let provider = OpenAIEmbeddingProvider::with_custom_url(
            "sk-test-key-1234567890".to_string(),
            config,
            custom_url.to_string(),
        )?;

        assert_eq!(provider.base_url, custom_url);
        Ok(())
    }

    #[test]
    fn test_mistral_config() {
        let config = ModelConfig::mistral_embed();
        assert_eq!(config.model_name, "mistral-embed");
        assert_eq!(config.embedding_dimension, 1024);
        assert_eq!(
            config.base_url,
            Some("https://api.mistral.ai/v1".to_string())
        );
        assert_eq!(
            config.get_embeddings_url(),
            "https://api.mistral.ai/v1/embeddings"
        );
    }

    #[test]
    fn test_azure_openai_config() {
        let config = ModelConfig::azure_openai("my-deployment", "my-resource", "2023-05-15", 1536);
        assert_eq!(config.model_name, "my-deployment");
        assert_eq!(config.embedding_dimension, 1536);
        assert_eq!(
            config.base_url,
            Some("https://my-resource.openai.azure.com".to_string())
        );
        assert!(config.api_endpoint.is_some());
        assert!(config.get_embeddings_url().contains("my-deployment"));
        assert!(config.get_embeddings_url().contains("2023-05-15"));
    }

    #[test]
    fn test_custom_config() {
        let config = ModelConfig::custom(
            "custom-model",
            768,
            "https://api.example.com/v1",
            Some("/custom/embeddings"),
        );
        assert_eq!(config.model_name, "custom-model");
        assert_eq!(config.embedding_dimension, 768);
        assert_eq!(
            config.get_embeddings_url(),
            "https://api.example.com/v1/custom/embeddings"
        );
    }

    #[test]
    fn test_custom_config_default_endpoint() {
        let config = ModelConfig::custom("custom-model", 768, "https://api.example.com/v1", None);
        assert_eq!(
            config.get_embeddings_url(),
            "https://api.example.com/v1/embeddings"
        );
    }

    #[cfg(feature = "openai")]
    #[tokio::test]
    async fn test_mistral_provider_creation() -> anyhow::Result<()> {
        let config = ModelConfig::mistral_embed();
        let provider = OpenAIEmbeddingProvider::new("test-api-key".to_string(), config)?;

        assert_eq!(provider.model_name(), "mistral-embed");
        assert_eq!(provider.embedding_dimension(), 1024);
        assert_eq!(provider.base_url, "https://api.mistral.ai/v1");
        Ok(())
    }

    #[test]
    fn test_optimization_config_defaults() {
        use super::super::config::OptimizationConfig;

        let config = OptimizationConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_ms, 1000);
        assert_eq!(config.get_timeout_seconds(), 60);
        assert_eq!(config.get_max_batch_size(), 100);
    }

    #[test]
    fn test_optimization_config_openai() {
        use super::super::config::OptimizationConfig;

        let config = OptimizationConfig::openai();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_ms, 1000);
        assert_eq!(config.timeout_seconds, Some(60));
        assert_eq!(config.max_batch_size, Some(2048));
        assert_eq!(config.rate_limit_rpm, Some(3000));
        assert_eq!(config.rate_limit_tpm, Some(1_000_000));
        assert!(config.compression_enabled);
        assert_eq!(config.connection_pool_size, 20);
    }

    #[test]
    fn test_optimization_config_mistral() {
        use super::super::config::OptimizationConfig;

        let config = OptimizationConfig::mistral();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_ms, 500);
        assert_eq!(config.timeout_seconds, Some(30));
        assert_eq!(config.max_batch_size, Some(128));
        assert_eq!(config.rate_limit_rpm, Some(100));
        assert!(config.compression_enabled);
        assert_eq!(config.connection_pool_size, 10);
    }

    #[test]
    fn test_optimization_config_azure() {
        use super::super::config::OptimizationConfig;

        let config = OptimizationConfig::azure();
        assert_eq!(config.max_retries, 4);
        assert_eq!(config.retry_delay_ms, 2000);
        assert_eq!(config.timeout_seconds, Some(90));
        assert_eq!(config.max_batch_size, Some(2048));
        assert_eq!(config.rate_limit_rpm, Some(300));
        assert!(config.compression_enabled);
        assert_eq!(config.connection_pool_size, 15);
    }

    #[test]
    fn test_optimization_config_local() {
        use super::super::config::OptimizationConfig;

        let config = OptimizationConfig::local();
        assert_eq!(config.max_retries, 2);
        assert_eq!(config.retry_delay_ms, 100);
        assert_eq!(config.timeout_seconds, Some(10));
        assert_eq!(config.max_batch_size, Some(32));
        assert_eq!(config.rate_limit_rpm, None); // No rate limiting for local
        assert!(!config.compression_enabled);
        assert_eq!(config.connection_pool_size, 5);
    }

    #[test]
    fn test_model_config_includes_optimization() {
        let config = ModelConfig::openai_3_small();
        assert_eq!(config.optimization.max_batch_size, Some(2048));
        assert_eq!(config.optimization.timeout_seconds, Some(60));

        let mistral_config = ModelConfig::mistral_embed();
        assert_eq!(mistral_config.optimization.max_batch_size, Some(128));
        assert_eq!(mistral_config.optimization.timeout_seconds, Some(30));

        let azure_config = ModelConfig::azure_openai("dep", "res", "2023-05-15", 1536);
        assert_eq!(azure_config.optimization.max_retries, 4);
        assert_eq!(azure_config.optimization.retry_delay_ms, 2000);
    }

    #[cfg(feature = "openai")]
    #[tokio::test]
    async fn test_provider_uses_optimization_timeout() -> anyhow::Result<()> {
        let mut config = ModelConfig::openai_3_small();
        config.optimization.timeout_seconds = Some(120);

        let provider = OpenAIEmbeddingProvider::new("sk-test-key".to_string(), config)?;

        // Verify the config has the custom timeout
        assert_eq!(provider.config.optimization.timeout_seconds, Some(120));
        Ok(())
    }

    #[cfg(feature = "openai")]
    #[tokio::test]
    async fn test_provider_uses_optimization_batch_size() -> anyhow::Result<()> {
        let mut config = ModelConfig::openai_3_small();
        config.optimization.max_batch_size = Some(500);

        let provider = OpenAIEmbeddingProvider::new("sk-test-key".to_string(), config)?;

        assert_eq!(provider.config.optimization.get_max_batch_size(), 500);
        Ok(())
    }
}

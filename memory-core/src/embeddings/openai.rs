//! OpenAI embedding provider for high-quality cloud-based embeddings

use super::config::ModelConfig;
use anyhow::Result;

#[cfg(feature = "openai")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "openai")]
use std::time::Instant;

/// OpenAI embedding provider
///
/// Uses OpenAI's embedding API for high-quality semantic embeddings.
/// Requires an API key and internet connection.
///
/// # Supported Models
/// - text-embedding-ada-002 (1536 dimensions, legacy)
/// - text-embedding-3-small (1536 dimensions, improved)
/// - text-embedding-3-large (3072 dimensions, highest quality)
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
    /// OpenAI API key
    api_key: String,
    /// Model configuration
    config: ModelConfig,
    /// HTTP client for API requests
    client: reqwest::Client,
    /// Base URL for OpenAI API
    base_url: String,
}

#[cfg(feature = "openai")]
impl OpenAIEmbeddingProvider {
    /// Create a new OpenAI embedding provider
    ///
    /// # Arguments
    /// * `api_key` - OpenAI API key
    /// * `config` - Model configuration
    ///
    /// # Returns
    /// Configured OpenAI embedding provider
    pub fn new(api_key: String, config: ModelConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            api_key,
            config,
            client,
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }

    /// Create provider with custom base URL (for Azure OpenAI, etc.)
    pub fn with_custom_url(api_key: String, config: ModelConfig, base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            api_key,
            config,
            client,
            base_url,
        }
    }

    /// Make embedding request to OpenAI API
    async fn request_embeddings(&self, input: EmbeddingInput) -> Result<EmbeddingResponse> {
        let url = format!("{}/embeddings", self.base_url);

        let request = EmbeddingRequest {
            input,
            model: self.config.model_name.clone(),
            encoding_format: Some("float".to_string()),
            dimensions: None, // Let OpenAI use default for the model
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to OpenAI API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API error {}: {}", status, error_text);
        }

        let embedding_response: EmbeddingResponse = response
            .json()
            .await
            .context("Failed to parse OpenAI API response")?;

        Ok(embedding_response)
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
            "Generated OpenAI embedding in {}ms, {} tokens, {} dimensions",
            generation_time,
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

        let generation_time = start_time.elapsed().as_millis() as u64;

        tracing::debug!(
            "Generated {} OpenAI embeddings in {}ms, {} tokens",
            embeddings.len(),
            generation_time,
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
    model: String,
    usage: Usage,
}

#[cfg(feature = "openai")]
#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: usize,
    object: String, // Should be "embedding"
}

#[cfg(feature = "openai")]
#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: usize,
    total_tokens: usize,
}

/// Utility functions for OpenAI provider
pub mod utils {
    use super::*;

    /// Validate OpenAI API key format
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
    /// Based on OpenAI's pricing as of 2024. Prices may change.
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

    /// Use cases for OpenAI model selection
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
    async fn test_provider_creation() {
        let config = ModelConfig::openai_3_small();
        let provider = OpenAIEmbeddingProvider::new("sk-test-key-1234567890".to_string(), config);

        assert_eq!(provider.model_name(), "text-embedding-3-small");
        assert_eq!(provider.embedding_dimension(), 1536);
        assert_eq!(provider.base_url, "https://api.openai.com/v1");
    }

    #[cfg(feature = "openai")]
    #[tokio::test]
    async fn test_custom_url_provider() {
        let config = ModelConfig::openai_3_small();
        let custom_url = "https://custom.openai.azure.com/v1";
        let provider = OpenAIEmbeddingProvider::with_custom_url(
            "sk-test-key-1234567890".to_string(),
            config,
            custom_url.to_string(),
        );

        assert_eq!(provider.base_url, custom_url);
    }
}

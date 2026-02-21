//! Mistral embedding provider client implementation

use super::super::config::mistral::{
    MistralConfig, MistralEmbeddingInput, MistralEmbeddingRequest, MistralEmbeddingResponse,
    OutputDtype,
};
use super::super::provider::EmbeddingProvider;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::time::Instant;

/// Mistral embedding provider
///
/// Supports both mistral-embed (general text) and codestral-embed (code-specific)
/// with advanced features like custom output dimensions and data types.
pub struct MistralEmbeddingProvider {
    /// Mistral API key
    api_key: String,
    /// Mistral-specific configuration
    config: MistralConfig,
    /// HTTP client for API requests
    client: reqwest::Client,
}

impl MistralEmbeddingProvider {
    /// Create a new Mistral embedding provider
    pub fn new(api_key: String, config: MistralConfig) -> Result<Self> {
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

    /// Make embedding request to Mistral API
    async fn request_embeddings(
        &self,
        inputs: MistralEmbeddingInput,
    ) -> Result<MistralEmbeddingResponse> {
        let url = self.config.embeddings_url();
        let max_retries = self.config.optimization.max_retries;
        let base_delay_ms = self.config.optimization.retry_delay_ms;

        // Build request with Mistral-specific parameters
        let request = MistralEmbeddingRequest {
            inputs,
            model: self.config.model.model_name().to_string(),
            output_dtype: if self.config.model.supports_output_dtype() {
                Some(self.config.output_dtype.as_str().to_string())
            } else {
                None
            },
            output_dimension: if self.config.model.supports_output_dimension() {
                self.config.output_dimension
            } else {
                None
            },
        };

        let mut last_error = None;

        for attempt in 0..=max_retries {
            if attempt > 0 {
                let delay = base_delay_ms * 2_u64.pow(attempt - 1);
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                tracing::warn!("Retry attempt {} for Mistral embedding request", attempt);
            }

            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await;

            let result = self.handle_response(response).await;
            match result {
                Ok(embedding_response) => return Ok(embedding_response),
                Err((error, is_client_error)) => {
                    last_error = Some(error);
                    if is_client_error {
                        break;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            anyhow::anyhow!("Mistral embedding request failed after {max_retries} retries")
        }))
    }

    /// Handle the HTTP response and return either the embedding response or an error
    /// with a flag indicating if it's a client error (4xx) that shouldn't be retried
    async fn handle_response(
        &self,
        response: Result<reqwest::Response, reqwest::Error>,
    ) -> Result<MistralEmbeddingResponse, (anyhow::Error, bool)> {
        let resp = match response {
            Ok(r) => r,
            Err(e) => return Err((anyhow::anyhow!("HTTP request failed: {e}"), false)),
        };

        let status = resp.status();
        let response_text = match resp.text().await {
            Ok(text) => text,
            Err(e) => {
                return Err((anyhow::anyhow!("Failed to read response body: {e}"), false));
            }
        };

        if status.is_success() {
            match serde_json::from_str::<MistralEmbeddingResponse>(&response_text) {
                Ok(embedding_response) => Ok(embedding_response),
                Err(e) => Err((
                    anyhow::anyhow!(
                        "Failed to parse embedding response: {e}. Response: {response_text}"
                    ),
                    false,
                )),
            }
        } else {
            let error = anyhow::anyhow!("Mistral API error (status {status}): {response_text}");
            let is_client_error = status.is_client_error();
            Err((error, is_client_error))
        }
    }

    // Handle different output dtypes
    fn process_embedding_response(&self, data: Vec<f32>) -> Result<Vec<f32>> {
        match self.config.output_dtype {
            OutputDtype::Float => Ok(data),
            OutputDtype::Int8 | OutputDtype::Uint8 => {
                // Convert integer embeddings back to float for uniform interface
                // In production, you might want to keep as integers for efficiency
                Ok(data)
            }
            OutputDtype::Binary | OutputDtype::Ubinary => {
                // Dequantize binary embeddings
                // See: https://colab.research.google.com/github/mistralai/cookbook/blob/main/mistral/embeddings/dequantization.ipynb
                self.dequantize_binary_embeddings(data)
            }
        }
    }

    fn dequantize_binary_embeddings(&self, _packed: Vec<f32>) -> Result<Vec<f32>> {
        // Convert packed binary representation back to float embeddings
        // This is a simplified version - see Mistral's dequantization cookbook for full implementation
        anyhow::bail!("Binary dequantization not yet implemented")
    }
}

#[async_trait]
impl EmbeddingProvider for MistralEmbeddingProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let start_time = Instant::now();

        let inputs: MistralEmbeddingInput = text.to_string().into();
        let response = self.request_embeddings(inputs).await?;

        let duration = start_time.elapsed();

        tracing::debug!(
            "Mistral embedding generated in {:?} ({} tokens)",
            duration,
            response.usage.total_tokens
        );

        // Extract embedding
        let embedding = response
            .data
            .first()
            .context("Empty response from Mistral API")?
            .embedding
            .clone();

        // Process based on output dtype
        let processed = self.process_embedding_response(embedding)?;

        // Validate dimension
        let expected = self.config.expected_response_size();
        if processed.len() != expected {
            tracing::warn!(
                "Embedding dimension mismatch: expected {}, got {}",
                expected,
                processed.len()
            );
        }

        Ok(processed)
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let start_time = Instant::now();

        if texts.is_empty() {
            return Ok(vec![]);
        }

        let inputs: MistralEmbeddingInput = texts.to_vec().into();
        let response = self.request_embeddings(inputs).await?;

        let duration = start_time.elapsed();

        tracing::debug!(
            "Mistral batch embedding generated in {:?} ({} texts, {} total tokens)",
            duration,
            texts.len(),
            response.usage.total_tokens
        );

        // Extract and process embeddings
        let mut embeddings = Vec::with_capacity(response.data.len());
        for embedding_data in response.data {
            let processed = self.process_embedding_response(embedding_data.embedding)?;
            embeddings.push(processed);
        }

        // Validate count
        if embeddings.len() != texts.len() {
            tracing::warn!(
                "Embedding count mismatch: expected {}, got {}",
                texts.len(),
                embeddings.len()
            );
        }

        Ok(embeddings)
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
        let _ = self.embed_text("warmup").await?;
        Ok(())
    }

    fn metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "model": self.model_name(),
            "dimension": self.embedding_dimension(),
            "type": "mistral",
            "provider": "Mistral AI",
            "base_url": self.config.base_url,
            "output_dtype": self.config.output_dtype.as_str(),
            "output_dimension": self.config.output_dimension,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embeddings::config::{
        OptimizationConfig,
        mistral::{MistralModel, OutputDtype},
    };
    use crate::embeddings::provider::EmbeddingProvider;

    #[test]
    fn test_provider_creation() {
        let config = MistralConfig::mistral_embed();
        let result = MistralEmbeddingProvider::new("test_key".to_string(), config);
        assert!(result.is_ok());

        let provider = result.unwrap();
        assert_eq!(provider.model_name(), "mistral-embed");
        assert_eq!(provider.embedding_dimension(), 1024);
    }

    #[test]
    fn test_provider_creation_codestral() {
        let config = MistralConfig::codestral_embed()
            .with_output_dimension(512)
            .with_output_dtype(OutputDtype::Int8);
        let result = MistralEmbeddingProvider::new("test_key".to_string(), config);
        assert!(result.is_ok());

        let provider = result.unwrap();
        assert_eq!(provider.model_name(), "codestral-embed");
        assert_eq!(provider.embedding_dimension(), 512);
    }

    #[test]
    fn test_provider_creation_invalid_config() {
        let config = MistralConfig {
            model: MistralModel::MistralEmbed,
            output_dimension: Some(512),
            output_dtype: OutputDtype::Int8,
            base_url: "https://api.mistral.ai/v1".to_string(),
            optimization: OptimizationConfig::mistral(),
        };
        let result = MistralEmbeddingProvider::new("test_key".to_string(), config);
        assert!(result.is_err());
    }

    #[test]
    fn test_metadata() {
        let config = MistralConfig::codestral_embed()
            .with_output_dimension(512)
            .with_output_dtype(OutputDtype::Int8);
        let provider = MistralEmbeddingProvider::new("test_key".to_string(), config).unwrap();

        let metadata = provider.metadata();
        assert_eq!(metadata["model"], "codestral-embed");
        assert_eq!(metadata["dimension"], 512);
        assert_eq!(metadata["type"], "mistral");
        assert_eq!(metadata["provider"], "Mistral AI");
        assert_eq!(metadata["output_dtype"], "int8");
        assert_eq!(metadata["output_dimension"], 512);
    }

    #[test]
    fn test_binary_dequantization_not_implemented() {
        let config = MistralConfig::codestral_binary();
        let provider = MistralEmbeddingProvider::new("test_key".to_string(), config).unwrap();

        let test_data = vec![1.0, 2.0, 3.0];
        let result = provider.dequantize_binary_embeddings(test_data);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("not yet implemented")
        );
    }
}

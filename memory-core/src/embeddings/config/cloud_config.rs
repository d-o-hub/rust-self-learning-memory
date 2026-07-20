//! Azure OpenAI and custom HTTP embedding provider configuration.

use serde::{Deserialize, Serialize};

use super::OptimizationConfig;

/// Configuration for Azure OpenAI Service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureOpenAIConfig {
    /// Deployment name
    pub deployment_name: String,
    /// Resource name
    pub resource_name: String,
    /// API version
    pub api_version: String,
    /// Embedding dimension
    pub embedding_dimension: usize,
    /// Optimization settings
    #[serde(default)]
    pub optimization: OptimizationConfig,
}

impl AzureOpenAIConfig {
    /// Create a new Azure OpenAI config
    #[must_use]
    pub fn new(
        deployment_name: impl Into<String>,
        resource_name: impl Into<String>,
        api_version: impl Into<String>,
        dimension: usize,
    ) -> Self {
        Self {
            deployment_name: deployment_name.into(),
            resource_name: resource_name.into(),
            api_version: api_version.into(),
            embedding_dimension: dimension,
            optimization: OptimizationConfig::azure(),
        }
    }

    /// Get the Azure OpenAI endpoint URL
    #[must_use]
    pub fn endpoint_url(&self) -> String {
        format!(
            "https://{}.openai.azure.com/openai/deployments/{}/embeddings?api-version={}",
            self.resource_name, self.deployment_name, self.api_version
        )
    }

    /// Get the effective embedding dimension
    #[must_use]
    pub fn effective_dimension(&self) -> usize {
        self.embedding_dimension
    }
}

/// Configuration for custom embedding providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomConfig {
    /// Model identifier
    pub model_name: String,
    /// Embedding dimension
    pub embedding_dimension: usize,
    /// Base URL for the API
    pub base_url: String,
    /// Custom endpoint path (optional)
    #[serde(default)]
    pub api_endpoint: Option<String>,
    /// Optimization settings
    #[serde(default)]
    pub optimization: OptimizationConfig,
}

impl CustomConfig {
    /// Create a new custom config
    #[must_use]
    pub fn new(
        model_name: impl Into<String>,
        dimension: usize,
        base_url: impl Into<String>,
    ) -> Self {
        Self {
            model_name: model_name.into(),
            embedding_dimension: dimension,
            base_url: base_url.into(),
            api_endpoint: None,
            optimization: OptimizationConfig::local(),
        }
    }

    /// Set custom API endpoint
    #[must_use]
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.api_endpoint = Some(endpoint.into());
        self
    }

    /// Get the full embeddings endpoint URL
    #[must_use]
    pub fn embeddings_url(&self) -> String {
        let endpoint = self.api_endpoint.as_deref().unwrap_or("/embeddings");
        let base = self.base_url.trim_end_matches('/');

        if endpoint.starts_with('/') {
            format!("{base}{endpoint}")
        } else {
            format!("{base}/{endpoint}")
        }
    }

    /// Get the effective embedding dimension
    #[must_use]
    pub fn effective_dimension(&self) -> usize {
        self.embedding_dimension
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_azure_openai_endpoint() {
        let config = AzureOpenAIConfig::new("my-deployment", "my-resource", "2023-05-15", 1536);
        assert_eq!(
            config.endpoint_url(),
            "https://my-resource.openai.azure.com/openai/deployments/my-deployment/embeddings?api-version=2023-05-15"
        );
    }

    #[test]
    fn test_custom_config_url() {
        let config = CustomConfig::new("custom-model", 768, "https://api.example.com/v1")
            .with_endpoint("/custom-embeddings");
        assert_eq!(
            config.embeddings_url(),
            "https://api.example.com/v1/custom-embeddings"
        );
    }
}

//! Model configuration for embedding providers

use serde::{Deserialize, Serialize};

use super::optimization_config::OptimizationConfig;

/// Model configuration for embedding providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model name/identifier
    pub model_name: String,
    /// Expected embedding dimension
    pub embedding_dimension: usize,
    /// Base URL for the API endpoint (e.g., `https://api.openai.com/v1`)
    #[serde(default)]
    pub base_url: Option<String>,
    /// Custom API endpoint path (optional, for non-standard endpoints)
    #[serde(default)]
    pub api_endpoint: Option<String>,
    /// Provider-specific optimization settings
    #[serde(default)]
    pub optimization: OptimizationConfig,
    /// Model-specific parameters
    pub parameters: serde_json::Value,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            embedding_dimension: 384,
            base_url: None,
            api_endpoint: None,
            optimization: OptimizationConfig::default(),
            parameters: serde_json::json!({}),
        }
    }
}

impl ModelConfig {
    /// Create configuration for a local sentence transformer model
    #[must_use]
    pub fn local_sentence_transformer(model_name: &str, dimension: usize) -> Self {
        Self {
            model_name: model_name.to_string(),
            embedding_dimension: dimension,
            base_url: None,
            api_endpoint: None,
            optimization: OptimizationConfig::local(),
            parameters: serde_json::json!({
                "normalize": true,
                "pooling": "mean"
            }),
        }
    }

    /// Create configuration for `OpenAI` text-embedding-ada-002
    #[must_use]
    pub fn openai_ada_002() -> Self {
        Self {
            model_name: "text-embedding-ada-002".to_string(),
            embedding_dimension: 1536,
            base_url: Some("https://api.openai.com/v1".to_string()),
            api_endpoint: None,
            optimization: OptimizationConfig::openai(),
            parameters: serde_json::json!({}),
        }
    }

    /// Create configuration for `OpenAI` text-embedding-3-small
    #[must_use]
    pub fn openai_3_small() -> Self {
        Self {
            model_name: "text-embedding-3-small".to_string(),
            embedding_dimension: 1536,
            base_url: Some("https://api.openai.com/v1".to_string()),
            api_endpoint: None,
            optimization: OptimizationConfig::openai(),
            parameters: serde_json::json!({}),
        }
    }

    /// Create configuration for `OpenAI` text-embedding-3-large
    #[must_use]
    pub fn openai_3_large() -> Self {
        Self {
            model_name: "text-embedding-3-large".to_string(),
            embedding_dimension: 3072,
            base_url: Some("https://api.openai.com/v1".to_string()),
            api_endpoint: None,
            optimization: OptimizationConfig::openai(),
            parameters: serde_json::json!({}),
        }
    }

    /// Create configuration for Mistral AI's mistral-embed model
    #[must_use]
    pub fn mistral_embed() -> Self {
        Self {
            model_name: "mistral-embed".to_string(),
            embedding_dimension: 1024,
            base_url: Some("https://api.mistral.ai/v1".to_string()),
            api_endpoint: None,
            optimization: OptimizationConfig::mistral(),
            parameters: serde_json::json!({}),
        }
    }

    /// Create configuration for Azure `OpenAI` Service
    ///
    /// # Arguments
    /// * `deployment_name` - Your Azure deployment name
    /// * `resource_name` - Your Azure resource name
    /// * `api_version` - API version (e.g., "2023-05-15")
    /// * `dimension` - Embedding dimension for your model
    #[must_use]
    pub fn azure_openai(
        deployment_name: &str,
        resource_name: &str,
        api_version: &str,
        dimension: usize,
    ) -> Self {
        Self {
            model_name: deployment_name.to_string(),
            embedding_dimension: dimension,
            base_url: Some(format!("https://{resource_name}.openai.azure.com")),
            api_endpoint: Some(format!(
                "/openai/deployments/{deployment_name}/embeddings?api-version={api_version}"
            )),
            optimization: OptimizationConfig::azure(),
            parameters: serde_json::json!({}),
        }
    }

    /// Create configuration with custom base URL and endpoint
    ///
    /// # Arguments
    /// * `model_name` - Model identifier
    /// * `dimension` - Embedding dimension
    /// * `base_url` - Base URL for the API (e.g., `<https://api.example.com/v1>`)
    /// * `endpoint` - Optional custom endpoint path (defaults to "/embeddings")
    #[must_use]
    pub fn custom(
        model_name: &str,
        dimension: usize,
        base_url: &str,
        endpoint: Option<&str>,
    ) -> Self {
        Self {
            model_name: model_name.to_string(),
            embedding_dimension: dimension,
            base_url: Some(base_url.to_string()),
            api_endpoint: endpoint.map(String::from),
            optimization: OptimizationConfig::local(),
            parameters: serde_json::json!({}),
        }
    }

    /// Get the full endpoint URL for embeddings
    #[must_use]
    pub fn get_embeddings_url(&self) -> String {
        let base = self
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");
        let endpoint = self.api_endpoint.as_deref().unwrap_or("/embeddings");

        if endpoint.starts_with('/') {
            format!("{base}{endpoint}")
        } else {
            format!("{base}/{endpoint}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_config_constructors() {
        let local_config = ModelConfig::local_sentence_transformer("test-model", 512);
        assert_eq!(local_config.model_name, "test-model");
        assert_eq!(local_config.embedding_dimension, 512);
        assert!(local_config.base_url.is_none());
        assert!(local_config.api_endpoint.is_none());

        let openai_small = ModelConfig::openai_3_small();
        assert_eq!(openai_small.model_name, "text-embedding-3-small");
        assert_eq!(openai_small.embedding_dimension, 1536);

        let openai_large = ModelConfig::openai_3_large();
        assert_eq!(openai_large.model_name, "text-embedding-3-large");
        assert_eq!(openai_large.embedding_dimension, 3072);

        let openai_ada = ModelConfig::openai_ada_002();
        assert_eq!(openai_ada.model_name, "text-embedding-ada-002");
        assert_eq!(openai_ada.embedding_dimension, 1536);

        let mistral = ModelConfig::mistral_embed();
        assert_eq!(mistral.model_name, "mistral-embed");
        assert_eq!(mistral.embedding_dimension, 1024);

        let azure = ModelConfig::azure_openai("my-deployment", "my-resource", "2023-05-15", 1536);
        assert_eq!(azure.model_name, "my-deployment");
        assert_eq!(azure.embedding_dimension, 1536);

        let custom = ModelConfig::custom(
            "custom-model",
            256,
            "https://api.example.com/v1",
            Some("/custom-embeddings"),
        );
        assert_eq!(custom.model_name, "custom-model");
        assert_eq!(custom.embedding_dimension, 256);
    }

    #[test]
    fn test_model_config_get_embeddings_url() {
        let config = ModelConfig::default();
        let url = config.get_embeddings_url();
        assert_eq!(url, "https://api.openai.com/v1/embeddings");

        let config = ModelConfig {
            base_url: Some("https://custom.api.com/v1".to_string()),
            api_endpoint: None,
            ..Default::default()
        };
        let url = config.get_embeddings_url();
        assert_eq!(url, "https://custom.api.com/v1/embeddings");

        let config = ModelConfig {
            base_url: Some("https://custom.api.com/v1".to_string()),
            api_endpoint: Some("/custom-path".to_string()),
            ..Default::default()
        };
        let url = config.get_embeddings_url();
        assert_eq!(url, "https://custom.api.com/v1/custom-path");
    }
}

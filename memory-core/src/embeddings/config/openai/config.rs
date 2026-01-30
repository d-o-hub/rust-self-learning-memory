//! OpenAI-specific embedding configuration

use crate::embeddings::config::OptimizationConfig;
use serde::{Deserialize, Serialize};

/// OpenAI embedding models
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIModel {
    /// text-embedding-ada-002 (legacy, 1536 dimensions)
    Ada002,
    /// text-embedding-3-small (1536 dimensions, improved performance)
    #[default]
    TextEmbedding3Small,
    /// text-embedding-3-large (3072 dimensions, highest quality)
    TextEmbedding3Large,
}

impl OpenAIModel {
    /// Get the model name as used in API requests
    #[must_use]
    pub fn model_name(self) -> &'static str {
        match self {
            Self::Ada002 => "text-embedding-ada-002",
            Self::TextEmbedding3Small => "text-embedding-3-small",
            Self::TextEmbedding3Large => "text-embedding-3-large",
        }
    }

    /// Get the default dimension for this model
    #[must_use]
    pub fn default_dimension(self) -> usize {
        match self {
            Self::Ada002 => 1536,
            Self::TextEmbedding3Small => 1536,
            Self::TextEmbedding3Large => 3072,
        }
    }

    /// Check if this model supports the dimensions parameter
    #[must_use]
    pub fn supports_dimensions(self) -> bool {
        matches!(self, Self::TextEmbedding3Small | Self::TextEmbedding3Large)
    }

    /// Get the maximum supported dimension
    #[must_use]
    pub fn max_dimension(self) -> usize {
        match self {
            Self::Ada002 => 1536,
            Self::TextEmbedding3Small => 1536,
            Self::TextEmbedding3Large => 3072,
        }
    }
}

/// Encoding format for OpenAI embeddings
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum EncodingFormat {
    /// 32-bit floating point numbers (default)
    #[default]
    Float,
    /// Base64-encoded float32
    Base64,
}

/// Configuration for OpenAI embedding provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    /// OpenAI model to use
    #[serde(default)]
    pub model: OpenAIModel,
    /// Number of dimensions to return (only for text-embedding-3.x models)
    /// Must be <= model's max dimension
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<usize>,
    /// Encoding format for embeddings
    #[serde(default)]
    pub encoding_format: EncodingFormat,
    /// Base URL for OpenAI API
    #[serde(default = "default_openai_base_url")]
    pub base_url: String,
    /// Optimization settings
    #[serde(default)]
    pub optimization: OptimizationConfig,
}

fn default_openai_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

impl OpenAIConfig {
    /// Create a new OpenAI config with the specified model
    #[must_use]
    pub fn new(model: OpenAIModel) -> Self {
        Self {
            model,
            dimensions: None,
            encoding_format: EncodingFormat::Float,
            base_url: default_openai_base_url(),
            optimization: OptimizationConfig::openai(),
        }
    }

    /// Create config for text-embedding-ada-002
    #[must_use]
    pub fn ada_002() -> Self {
        Self::new(OpenAIModel::Ada002)
    }

    /// Create config for text-embedding-3-small
    #[must_use]
    pub fn text_embedding_3_small() -> Self {
        Self::new(OpenAIModel::TextEmbedding3Small)
    }

    /// Create config for text-embedding-3-large
    #[must_use]
    pub fn text_embedding_3_large() -> Self {
        Self::new(OpenAIModel::TextEmbedding3Large)
    }

    /// Set custom dimensions (only valid for 3.x models)
    ///
    /// # Panics
    /// Panics if dimensions > model's max dimension
    #[must_use]
    pub fn with_dimensions(mut self, dimensions: usize) -> Self {
        assert!(
            dimensions <= self.model.max_dimension(),
            "dimensions {} exceeds maximum {} for {:?}",
            dimensions,
            self.model.max_dimension(),
            self.model
        );
        self.dimensions = Some(dimensions);
        self
    }

    /// Set encoding format
    #[must_use]
    pub fn with_encoding_format(mut self, format: EncodingFormat) -> Self {
        self.encoding_format = format;
        self
    }

    /// Set custom base URL (for proxies or Azure compatibility)
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Get the effective embedding dimension
    #[must_use]
    pub fn effective_dimension(&self) -> usize {
        self.dimensions
            .unwrap_or_else(|| self.model.default_dimension())
    }

    /// Get the full embeddings endpoint URL
    #[must_use]
    pub fn embeddings_url(&self) -> String {
        format!("{}/embeddings", self.base_url.trim_end_matches('/'))
    }

    /// Validate the configuration
    ///
    /// # Errors
    /// Returns error if dimensions are invalid for the model
    pub fn validate(&self) -> anyhow::Result<()> {
        if let Some(dims) = self.dimensions {
            if !self.model.supports_dimensions() {
                anyhow::bail!("Model {:?} does not support custom dimensions", self.model);
            }
            if dims == 0 || dims > self.model.max_dimension() {
                anyhow::bail!(
                    "Dimensions must be between 1 and {}, got {}",
                    self.model.max_dimension(),
                    dims
                );
            }
        }
        Ok(())
    }
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self::text_embedding_3_small()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_model_defaults() {
        assert_eq!(OpenAIModel::Ada002.default_dimension(), 1536);
        assert_eq!(OpenAIModel::TextEmbedding3Small.default_dimension(), 1536);
        assert_eq!(OpenAIModel::TextEmbedding3Large.default_dimension(), 3072);
    }

    #[test]
    fn test_openai_model_supports_dimensions() {
        assert!(!OpenAIModel::Ada002.supports_dimensions());
        assert!(OpenAIModel::TextEmbedding3Small.supports_dimensions());
        assert!(OpenAIModel::TextEmbedding3Large.supports_dimensions());
    }

    #[test]
    fn test_openai_config_builder() {
        let config = OpenAIConfig::text_embedding_3_small()
            .with_dimensions(512)
            .with_encoding_format(EncodingFormat::Base64);

        assert_eq!(config.model, OpenAIModel::TextEmbedding3Small);
        assert_eq!(config.dimensions, Some(512));
        assert_eq!(config.encoding_format, EncodingFormat::Base64);
        assert_eq!(config.effective_dimension(), 512);
    }

    #[test]
    fn test_openai_config_default_dimension() {
        let config = OpenAIConfig::text_embedding_3_large();
        assert_eq!(config.effective_dimension(), 3072);
    }

    #[test]
    fn test_openai_config_validation() {
        let valid = OpenAIConfig::text_embedding_3_small().with_dimensions(512);
        assert!(valid.validate().is_ok());

        let invalid = OpenAIConfig::ada_002().with_dimensions(512);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_openai_config_serialization() {
        let config = OpenAIConfig::text_embedding_3_small().with_dimensions(512);

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: OpenAIConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.model, deserialized.model);
        assert_eq!(config.dimensions, deserialized.dimensions);
    }
}

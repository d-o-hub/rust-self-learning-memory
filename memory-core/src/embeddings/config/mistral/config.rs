//! Mistral-specific embedding configuration

use crate::embeddings::config::OptimizationConfig;
use serde::{Deserialize, Serialize};

/// Mistral embedding models
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum MistralModel {
    /// mistral-embed: General-purpose text embeddings (1024 dimensions)
    #[default]
    MistralEmbed,
    /// codestral-embed: Code-specific embeddings (1536 default, up to 3072)
    CodestralEmbed,
}

impl MistralModel {
    /// Get the model name as used in API requests
    #[must_use]
    pub fn model_name(self) -> &'static str {
        match self {
            Self::MistralEmbed => "mistral-embed",
            Self::CodestralEmbed => "codestral-embed",
        }
    }

    /// Get the default dimension for this model
    #[must_use]
    pub fn default_dimension(self) -> usize {
        match self {
            Self::MistralEmbed => 1024,
            Self::CodestralEmbed => 1536,
        }
    }

    /// Get the maximum supported dimension
    #[must_use]
    pub fn max_dimension(self) -> usize {
        match self {
            Self::MistralEmbed => 1024,
            Self::CodestralEmbed => 3072,
        }
    }

    /// Check if this model supports `output_dimension` parameter
    #[must_use]
    pub fn supports_output_dimension(self) -> bool {
        matches!(self, Self::CodestralEmbed)
    }

    /// Check if this model supports `output_dtype` parameter
    #[must_use]
    pub fn supports_output_dtype(self) -> bool {
        matches!(self, Self::CodestralEmbed)
    }

    /// Get the default output dtype for this model
    #[must_use]
    pub fn default_output_dtype(self) -> OutputDtype {
        OutputDtype::Float
    }
}

/// Output data type for Mistral embeddings
///
/// Allows selection of precision and format for embeddings.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputDtype {
    /// 32-bit single-precision floating-point numbers (default)
    /// Provides highest precision and retrieval accuracy
    #[default]
    Float,
    /// 8-bit signed integers (-128 to 127)
    Int8,
    /// 8-bit unsigned integers (0 to 255)
    Uint8,
    /// Bit-packed, quantized single-bit values using int8 type
    /// Length is 1/8 of `output_dimension`. Uses offset binary method.
    Binary,
    /// Bit-packed, quantized single-bit values using uint8 type
    /// Length is 1/8 of `output_dimension`.
    Ubinary,
}

impl OutputDtype {
    /// Get the string representation for API requests
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Float => "float",
            Self::Int8 => "int8",
            Self::Uint8 => "uint8",
            Self::Binary => "binary",
            Self::Ubinary => "ubinary",
        }
    }

    /// Check if this dtype uses bit-packing (returns fewer elements)
    #[must_use]
    pub fn is_bit_packed(self) -> bool {
        matches!(self, Self::Binary | Self::Ubinary)
    }

    /// Calculate the expected response size for a given output dimension
    #[must_use]
    pub fn response_size(self, output_dimension: usize) -> usize {
        if self.is_bit_packed() {
            // Bit-packed: 1/8 of dimension (rounded up)
            (output_dimension + 7) / 8
        } else {
            output_dimension
        }
    }
}

/// Configuration for Mistral embedding provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MistralConfig {
    /// Mistral model to use
    #[serde(default)]
    pub model: MistralModel,
    /// Output dimension (1-3072, codestral-embed only)
    /// If None, uses model default (1024 for mistral-embed, 1536 for codestral-embed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_dimension: Option<usize>,
    /// Output data type (codestral-embed only, defaults to float)
    #[serde(default)]
    pub output_dtype: OutputDtype,
    /// Base URL for Mistral API
    #[serde(default = "default_mistral_base_url")]
    pub base_url: String,
    /// Optimization settings
    #[serde(default)]
    pub optimization: OptimizationConfig,
}

fn default_mistral_base_url() -> String {
    "https://api.mistral.ai/v1".to_string()
}

impl MistralConfig {
    /// Create a new Mistral config with the specified model
    #[must_use]
    pub fn new(model: MistralModel) -> Self {
        Self {
            model,
            output_dimension: None,
            output_dtype: OutputDtype::Float,
            base_url: default_mistral_base_url(),
            optimization: OptimizationConfig::mistral(),
        }
    }

    /// Create config for mistral-embed (general text)
    #[must_use]
    pub fn mistral_embed() -> Self {
        Self::new(MistralModel::MistralEmbed)
    }

    /// Create config for codestral-embed (code-specific)
    #[must_use]
    pub fn codestral_embed() -> Self {
        Self::new(MistralModel::CodestralEmbed)
    }

    /// Set custom output dimension (codestral-embed only, 1-3072)
    ///
    /// # Panics
    /// Panics if dimension > 3072 or if model doesn't support custom dimensions
    #[must_use]
    pub fn with_output_dimension(mut self, dimension: usize) -> Self {
        self.output_dimension = Some(dimension);
        self
    }

    /// Set output data type (codestral-embed only)
    #[must_use]
    pub fn with_output_dtype(mut self, dtype: OutputDtype) -> Self {
        self.output_dtype = dtype;
        self
    }

    /// Set custom base URL
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Get the effective embedding dimension
    #[must_use]
    pub fn effective_dimension(&self) -> usize {
        self.output_dimension
            .unwrap_or_else(|| self.model.default_dimension())
    }

    /// Get the expected response size based on dtype
    #[must_use]
    pub fn expected_response_size(&self) -> usize {
        self.output_dtype.response_size(self.effective_dimension())
    }

    /// Get the full embeddings endpoint URL
    #[must_use]
    pub fn embeddings_url(&self) -> String {
        format!("{}/embeddings", self.base_url.trim_end_matches('/'))
    }

    /// Validate the configuration
    ///
    /// # Errors
    /// Returns error if `output_dimension` or `output_dtype` are invalid for the model
    pub fn validate(&self) -> anyhow::Result<()> {
        if let Some(dims) = self.output_dimension {
            if !self.model.supports_output_dimension() {
                anyhow::bail!(
                    "Model {:?} does not support custom output_dimension",
                    self.model
                );
            }
            if dims == 0 || dims > self.model.max_dimension() {
                anyhow::bail!(
                    "output_dimension must be between 1 and {}, got {}",
                    self.model.max_dimension(),
                    dims
                );
            }
        }

        if self.output_dtype != OutputDtype::Float && !self.model.supports_output_dtype() {
            anyhow::bail!(
                "Model {:?} does not support custom output_dtype",
                self.model
            );
        }

        Ok(())
    }

    /// Create a codestral-embed config optimized for binary embeddings
    ///
    /// This reduces storage by 32x (float32 -> 1 bit) at the cost of some accuracy.
    #[must_use]
    pub fn codestral_binary() -> Self {
        Self::codestral_embed().with_output_dtype(OutputDtype::Binary)
    }

    /// Create a codestral-embed config with reduced dimensions for efficiency
    #[must_use]
    pub fn codestral_compact(dimension: usize) -> Self {
        Self::codestral_embed().with_output_dimension(dimension)
    }
}

impl Default for MistralConfig {
    fn default() -> Self {
        Self::mistral_embed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mistral_model_properties() {
        assert_eq!(MistralModel::MistralEmbed.default_dimension(), 1024);
        assert_eq!(MistralModel::CodestralEmbed.default_dimension(), 1536);

        assert_eq!(MistralModel::MistralEmbed.max_dimension(), 1024);
        assert_eq!(MistralModel::CodestralEmbed.max_dimension(), 3072);

        assert!(!MistralModel::MistralEmbed.supports_output_dimension());
        assert!(MistralModel::CodestralEmbed.supports_output_dimension());

        assert!(!MistralModel::MistralEmbed.supports_output_dtype());
        assert!(MistralModel::CodestralEmbed.supports_output_dtype());
    }

    #[test]
    fn test_output_dtype_properties() {
        assert!(!OutputDtype::Float.is_bit_packed());
        assert!(!OutputDtype::Int8.is_bit_packed());
        assert!(!OutputDtype::Uint8.is_bit_packed());
        assert!(OutputDtype::Binary.is_bit_packed());
        assert!(OutputDtype::Ubinary.is_bit_packed());

        // Test response size calculation
        assert_eq!(OutputDtype::Float.response_size(1024), 1024);
        assert_eq!(OutputDtype::Binary.response_size(1024), 128); // 1024 / 8
        assert_eq!(OutputDtype::Binary.response_size(1000), 125); // ceil(1000 / 8)
    }

    #[test]
    fn test_mistral_config_builder() {
        let config = MistralConfig::codestral_embed()
            .with_output_dimension(512)
            .with_output_dtype(OutputDtype::Int8);

        assert_eq!(config.model, MistralModel::CodestralEmbed);
        assert_eq!(config.output_dimension, Some(512));
        assert_eq!(config.output_dtype, OutputDtype::Int8);
        assert_eq!(config.effective_dimension(), 512);
        assert_eq!(config.expected_response_size(), 512);
    }

    #[test]
    fn test_codestral_binary_config() {
        let config = MistralConfig::codestral_binary();
        assert_eq!(config.output_dtype, OutputDtype::Binary);
        assert_eq!(config.effective_dimension(), 1536);
        assert_eq!(config.expected_response_size(), 192); // 1536 / 8
    }

    #[test]
    fn test_mistral_config_validation() {
        // Valid codestral config
        let valid = MistralConfig::codestral_embed().with_output_dimension(512);
        assert!(valid.validate().is_ok());

        // Invalid: mistral-embed doesn't support custom dimensions
        let invalid = MistralConfig::mistral_embed().with_output_dimension(512);
        assert!(invalid.validate().is_err());

        // Invalid: dimension too large
        let invalid_dim = MistralConfig::codestral_embed().with_output_dimension(4000);
        assert!(invalid_dim.validate().is_err());
    }

    #[test]
    fn test_mistral_config_serialization() {
        let config = MistralConfig::codestral_embed()
            .with_output_dimension(512)
            .with_output_dtype(OutputDtype::Int8);

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: MistralConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.model, deserialized.model);
        assert_eq!(config.output_dimension, deserialized.output_dimension);
        assert_eq!(config.output_dtype, deserialized.output_dtype);
    }
}

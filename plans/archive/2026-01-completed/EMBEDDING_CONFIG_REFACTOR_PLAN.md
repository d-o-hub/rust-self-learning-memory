# Embedding Configuration Refactor Plan

## Executive Summary

This plan outlines the comprehensive refactoring of the embedding configuration system to separate OpenAI and Mistral provider-specific configurations. The current single `ModelConfig` structure will be replaced with provider-specific configurations that support the unique features of each embedding provider.

## Current State Analysis

### Existing Configuration Structure

```
memory-core/src/embeddings/config/
├── mod.rs                    # Module exports
├── model_config.rs           # Single ModelConfig for ALL providers (to be replaced)
├── optimization_config.rs    # Provider-specific optimization settings (keep)
├── provider_enum.rs          # EmbeddingProvider enum (keep)
└── embedding_config.rs       # Top-level EmbeddingConfig (update)
```

### Problems with Current Design

1. **Generic Parameters**: The current `ModelConfig` uses `parameters: serde_json::Value` to store provider-specific settings, losing type safety
2. **No Provider-Specific Features**: Cannot properly support:
   - Mistral's `output_dtype` and `output_dimension` for codestral-embed
   - OpenAI's `dimensions` parameter for text-embedding-3.x models
   - Different encoding formats per provider
3. **Unclear API**: Users cannot discover available options through IDE autocomplete
4. **Validation Issues**: No compile-time validation of provider-specific constraints

## Architecture Overview

### New Configuration Architecture

```
memory-core/src/embeddings/config/
├── mod.rs                    # Updated exports
├── provider_enum.rs          # EmbeddingProvider enum (unchanged)
├── optimization_config.rs    # OptimizationConfig (unchanged)
├── provider_config.rs        # NEW: Unified ProviderConfig enum
├── openai/
│   ├── mod.rs               # NEW: OpenAI module exports
│   ├── config.rs            # NEW: OpenAIConfig with OpenAIModel enum
│   └── types.rs             # NEW: OpenAI-specific request/response types
├── mistral/
│   ├── mod.rs               # NEW: Mistral module exports
│   ├── config.rs            # NEW: MistralConfig with MistralModel and OutputDtype enums
│   └── types.rs             # NEW: Mistral-specific request/response types
├── embedding_config.rs       # MODIFIED: Use ProviderConfig instead of ModelConfig
└── legacy/                   # DEPRECATED: Old config for reference during migration
    └── model_config.rs       # Will be removed after migration
```

### Configuration Hierarchy

```
EmbeddingConfig
├── provider: EmbeddingProvider (enum)
├── provider_config: ProviderConfig (enum)
│   ├── OpenAI(OpenAIConfig)
│   │   ├── model: OpenAIModel (Ada002 | TextEmbedding3Small | TextEmbedding3Large)
│   │   ├── dimensions: Option<usize>  # For 3.x models
│   │   ├── encoding_format: EncodingFormat (Float | Base64)
│   │   └── optimization: OptimizationConfig
│   ├── Mistral(MistralConfig)
│   │   ├── model: MistralModel (MistralEmbed | CodestralEmbed)
│   │   ├── output_dimension: Option<usize>  # 1-3072, codestral only
│   │   ├── output_dtype: OutputDtype (Float | Int8 | Uint8 | Binary | Ubinary)
│   │   └── optimization: OptimizationConfig
│   ├── Local(LocalConfig)
│   ├── AzureOpenAI(AzureOpenAIConfig)
│   └── Custom(CustomConfig)
├── similarity_threshold: f32
├── batch_size: usize
├── cache_embeddings: bool
└── timeout_seconds: u64
```

## Detailed Configuration Structures

### 1. OpenAI Configuration

#### File: `memory-core/src/embeddings/config/openai/config.rs`

```rust
//! OpenAI-specific embedding configuration

use serde::{Deserialize, Serialize};
use crate::embeddings::config::OptimizationConfig;

/// OpenAI embedding models
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OpenAIModel {
    /// text-embedding-ada-002 (legacy, 1536 dimensions)
    Ada002,
    /// text-embedding-3-small (1536 dimensions, improved performance)
    TextEmbedding3Small,
    /// text-embedding-3-large (3072 dimensions, highest quality)
    TextEmbedding3Large,
}

impl OpenAIModel {
    /// Get the model name as used in API requests
    #[must_use]
    pub fn model_name(&self) -> &'static str {
        match self {
            Self::Ada002 => "text-embedding-ada-002",
            Self::TextEmbedding3Small => "text-embedding-3-small",
            Self::TextEmbedding3Large => "text-embedding-3-large",
        }
    }

    /// Get the default dimension for this model
    #[must_use]
    pub fn default_dimension(&self) -> usize {
        match self {
            Self::Ada002 => 1536,
            Self::TextEmbedding3Small => 1536,
            Self::TextEmbedding3Large => 3072,
        }
    }

    /// Check if this model supports the dimensions parameter
    #[must_use]
    pub fn supports_dimensions(&self) -> bool {
        matches!(self, Self::TextEmbedding3Small | Self::TextEmbedding3Large)
    }

    /// Get the maximum supported dimension
    #[must_use]
    pub fn max_dimension(&self) -> usize {
        match self {
            Self::Ada002 => 1536,
            Self::TextEmbedding3Small => 1536,
            Self::TextEmbedding3Large => 3072,
        }
    }
}

impl Default for OpenAIModel {
    fn default() -> Self {
        Self::TextEmbedding3Small
    }
}

/// Encoding format for OpenAI embeddings
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EncodingFormat {
    /// 32-bit floating point numbers (default)
    Float,
    /// Base64-encoded float32
    Base64,
}

impl Default for EncodingFormat {
    fn default() -> Self {
        Self::Float
    }
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
        self.dimensions.unwrap_or_else(|| self.model.default_dimension())
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
                anyhow::bail!(
                    "Model {:?} does not support custom dimensions",
                    self.model
                );
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
        let config = OpenAIConfig::text_embedding_3_small()
            .with_dimensions(512);
        
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: OpenAIConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.model, deserialized.model);
        assert_eq!(config.dimensions, deserialized.dimensions);
    }
}
```

#### File: `memory-core/src/embeddings/config/openai/types.rs`

```rust
//! OpenAI API request/response types

use serde::{Deserialize, Serialize};

/// Input for embedding request (single text or batch)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIEmbeddingInput {
    /// Single text input
    Single(String),
    /// Batch of text inputs
    Batch(Vec<String>),
}

impl From<String> for OpenAIEmbeddingInput {
    fn from(s: String) -> Self {
        Self::Single(s)
    }
}

impl From<Vec<String>> for OpenAIEmbeddingInput {
    fn from(v: Vec<String>) -> Self {
        Self::Batch(v)
    }
}

impl From<&str> for OpenAIEmbeddingInput {
    fn from(s: &str) -> Self {
        Self::Single(s.to_string())
    }
}

impl From<Vec<&str>> for OpenAIEmbeddingInput {
    fn from(v: Vec<&str>) -> Self {
        Self::Batch(v.into_iter().map(String::from).collect())
    }
}

/// OpenAI API request structure
#[derive(Debug, Clone, Serialize)]
pub struct OpenAIEmbeddingRequest {
    /// Input text(s) to embed
    pub input: OpenAIEmbeddingInput,
    /// Model ID
    pub model: String,
    /// Encoding format (float or base64)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,
    /// Number of dimensions (for text-embedding-3.x models)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<usize>,
}

/// Individual embedding data from API response
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIEmbeddingData {
    /// The embedding vector
    pub embedding: Vec<f32>,
    /// Index in the input batch
    pub index: usize,
    /// Object type (always "embedding")
    pub object: String,
}

/// Token usage information
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIUsage {
    /// Tokens in the prompt
    pub prompt_tokens: usize,
    /// Total tokens used
    pub total_tokens: usize,
}

/// OpenAI API response structure
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIEmbeddingResponse {
    /// List of embedding data
    pub data: Vec<OpenAIEmbeddingData>,
    /// Model used
    pub model: String,
    /// Token usage
    pub usage: OpenAIUsage,
    /// Object type (always "list")
    pub object: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_input_conversions() {
        let single: OpenAIEmbeddingInput = "test".into();
        assert!(matches!(single, OpenAIEmbeddingInput::Single(_)));

        let batch: OpenAIEmbeddingInput = vec!["a", "b", "c"].into();
        assert!(matches!(batch, OpenAIEmbeddingInput::Batch(_)));
    }

    #[test]
    fn test_request_serialization() {
        let request = OpenAIEmbeddingRequest {
            input: OpenAIEmbeddingInput::Single("hello".to_string()),
            model: "text-embedding-3-small".to_string(),
            encoding_format: Some("float".to_string()),
            dimensions: Some(512),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("text-embedding-3-small"));
        assert!(json.contains("512"));
    }
}
```

#### File: `memory-core/src/embeddings/config/openai/mod.rs`

```rust
//! OpenAI-specific embedding configuration and types

pub use config::{EncodingFormat, OpenAIConfig, OpenAIModel};
pub use types::{OpenAIEmbeddingData, OpenAIEmbeddingInput, OpenAIEmbeddingRequest, OpenAIEmbeddingResponse, OpenAIUsage};

mod config;
mod types;
```

### 2. Mistral Configuration

#### File: `memory-core/src/embeddings/config/mistral/config.rs`

```rust
//! Mistral-specific embedding configuration

use serde::{Deserialize, Serialize};
use crate::embeddings::config::OptimizationConfig;

/// Mistral embedding models
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MistralModel {
    /// mistral-embed: General-purpose text embeddings (1024 dimensions)
    MistralEmbed,
    /// codestral-embed: Code-specific embeddings (1536 default, up to 3072)
    CodestralEmbed,
}

impl MistralModel {
    /// Get the model name as used in API requests
    #[must_use]
    pub fn model_name(&self) -> &'static str {
        match self {
            Self::MistralEmbed => "mistral-embed",
            Self::CodestralEmbed => "codestral-embed",
        }
    }

    /// Get the default dimension for this model
    #[must_use]
    pub fn default_dimension(&self) -> usize {
        match self {
            Self::MistralEmbed => 1024,
            Self::CodestralEmbed => 1536,
        }
    }

    /// Get the maximum supported dimension
    #[must_use]
    pub fn max_dimension(&self) -> usize {
        match self {
            Self::MistralEmbed => 1024,
            Self::CodestralEmbed => 3072,
        }
    }

    /// Check if this model supports output_dtype parameter
    #[must_use]
    pub fn supports_output_dtype(&self) -> bool {
        matches!(self, Self::CodestralEmbed)
    }

    /// Check if this model supports output_dimension parameter
    #[must_use]
    pub fn supports_output_dimension(&self) -> bool {
        matches!(self, Self::CodestralEmbed)
    }

    /// Get the default output dtype for this model
    #[must_use]
    pub fn default_output_dtype(&self) -> OutputDtype {
        OutputDtype::Float
    }
}

impl Default for MistralModel {
    fn default() -> Self {
        Self::MistralEmbed
    }
}

/// Output data type for Mistral embeddings
/// 
/// Allows selection of precision and format for embeddings.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputDtype {
    /// 32-bit single-precision floating-point numbers (default)
    /// Provides highest precision and retrieval accuracy
    Float,
    /// 8-bit signed integers (-128 to 127)
    Int8,
    /// 8-bit unsigned integers (0 to 255)
    Uint8,
    /// Bit-packed, quantized single-bit values using int8 type
    /// Length is 1/8 of output_dimension. Uses offset binary method.
    Binary,
    /// Bit-packed, quantized single-bit values using uint8 type
    /// Length is 1/8 of output_dimension.
    Ubinary,
}

impl OutputDtype {
    /// Get the string representation for API requests
    #[must_use]
    pub fn as_str(&self) -> &'static str {
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
    pub fn is_bit_packed(&self) -> bool {
        matches!(self, Self::Binary | Self::Ubinary)
    }

    /// Calculate the expected response size for a given output dimension
    #[must_use]
    pub fn response_size(&self, output_dimension: usize) -> usize {
        if self.is_bit_packed() {
            // Bit-packed: 1/8 of dimension (rounded up)
            (output_dimension + 7) / 8
        } else {
            output_dimension
        }
    }
}

impl Default for OutputDtype {
    fn default() -> Self {
        Self::Float
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
        assert!(
            self.model.supports_output_dimension(),
            "Model {:?} does not support custom output_dimension",
            self.model
        );
        assert!(
            dimension > 0 && dimension <= 3072,
            "output_dimension must be between 1 and 3072, got {}",
            dimension
        );
        self.output_dimension = Some(dimension);
        self
    }

    /// Set output data type (codestral-embed only)
    #[must_use]
    pub fn with_output_dtype(mut self, dtype: OutputDtype) -> Self {
        assert!(
            self.model.supports_output_dtype(),
            "Model {:?} does not support custom output_dtype",
            self.model
        );
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
        self.output_dimension.unwrap_or_else(|| self.model.default_dimension())
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
    /// Returns error if output_dimension or output_dtype are invalid for the model
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
```

#### File: `memory-core/src/embeddings/config/mistral/types.rs`

```rust
//! Mistral API request/response types

use serde::{Deserialize, Serialize};

/// Input for Mistral embedding request (single text or batch)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MistralEmbeddingInput {
    /// Single text input
    Single(String),
    /// Batch of text inputs
    Batch(Vec<String>),
}

impl From<String> for MistralEmbeddingInput {
    fn from(s: String) -> Self {
        Self::Single(s)
    }
}

impl From<Vec<String>> for MistralEmbeddingInput {
    fn from(v: Vec<String>) -> Self {
        Self::Batch(v)
    }
}

impl From<&str> for MistralEmbeddingInput {
    fn from(s: &str) -> Self {
        Self::Single(s.to_string())
    }
}

impl From<Vec<&str>> for MistralEmbeddingInput {
    fn from(v: Vec<&str>) -> Self {
        Self::Batch(v.into_iter().map(String::from).collect())
    }
}

/// Mistral API request structure
#[derive(Debug, Clone, Serialize)]
pub struct MistralEmbeddingRequest {
    /// Input text(s) to embed
    pub inputs: MistralEmbeddingInput,
    /// Model ID
    pub model: String,
    /// Output data type (codestral-embed only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_dtype: Option<String>,
    /// Output dimension (codestral-embed only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_dimension: Option<usize>,
}

/// Individual embedding data from Mistral API response
#[derive(Debug, Clone, Deserialize)]
pub struct MistralEmbeddingData {
    /// The embedding vector (format depends on output_dtype)
    pub embedding: Vec<f32>,
    /// Index in the input batch
    pub index: usize,
    /// Object type (always "embedding")
    pub object: String,
}

/// Token usage information
#[derive(Debug, Clone, Deserialize)]
pub struct MistralUsage {
    /// Tokens in the prompt
    pub prompt_tokens: usize,
    /// Total tokens used
    pub total_tokens: usize,
}

/// Mistral API response structure
#[derive(Debug, Clone, Deserialize)]
pub struct MistralEmbeddingResponse {
    /// List of embedding data
    pub data: Vec<MistralEmbeddingData>,
    /// Model used
    pub model: String,
    /// Token usage
    pub usage: MistralUsage,
    /// Object type (always "list")
    pub object: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mistral_embedding_input_conversions() {
        let single: MistralEmbeddingInput = "test".into();
        assert!(matches!(single, MistralEmbeddingInput::Single(_)));

        let batch: MistralEmbeddingInput = vec!["a", "b", "c"].into();
        assert!(matches!(batch, MistralEmbeddingInput::Batch(_)));
    }

    #[test]
    fn test_mistral_request_serialization() {
        let request = MistralEmbeddingRequest {
            inputs: MistralEmbeddingInput::Batch(vec!["hello".to_string(), "world".to_string()]),
            model: "codestral-embed".to_string(),
            output_dtype: Some("int8".to_string()),
            output_dimension: Some(512),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("codestral-embed"));
        assert!(json.contains("int8"));
        assert!(json.contains("512"));
    }

    #[test]
    fn test_mistral_request_serialization_without_optional() {
        let request = MistralEmbeddingRequest {
            inputs: MistralEmbeddingInput::Single("hello".to_string()),
            model: "mistral-embed".to_string(),
            output_dtype: None,
            output_dimension: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("mistral-embed"));
        assert!(!json.contains("output_dtype"));
        assert!(!json.contains("output_dimension"));
    }
}
```

#### File: `memory-core/src/embeddings/config/mistral/mod.rs`

```rust
//! Mistral-specific embedding configuration and types

pub use config::{MistralConfig, MistralModel, OutputDtype};
pub use types::{MistralEmbeddingData, MistralEmbeddingInput, MistralEmbeddingRequest, MistralEmbeddingResponse, MistralUsage};

mod config;
mod types;
```

### 3. Unified Provider Configuration

#### File: `memory-core/src/embeddings/config/provider_config.rs`

```rust
//! Unified provider configuration enum

use serde::{Deserialize, Serialize};

use super::{
    mistral::{MistralConfig, MistralModel, OutputDtype},
    openai::{EncodingFormat, OpenAIConfig, OpenAIModel},
    OptimizationConfig,
};

/// Provider-specific configuration
/// 
/// This enum wraps all provider-specific configurations, allowing type-safe
/// access to provider-specific features while maintaining a unified interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub enum ProviderConfig {
    /// Local embedding configuration
    Local(LocalConfig),
    /// OpenAI embedding configuration
    OpenAI(OpenAIConfig),
    /// Mistral AI embedding configuration
    Mistral(MistralConfig),
    /// Azure OpenAI configuration
    AzureOpenAI(AzureOpenAIConfig),
    /// Custom provider configuration
    Custom(CustomConfig),
}

/// Configuration for local embedding providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    /// Model name/path
    pub model_name: String,
    /// Embedding dimension
    pub embedding_dimension: usize,
    /// Optimization settings
    #[serde(default)]
    pub optimization: OptimizationConfig,
}

impl LocalConfig {
    /// Create a new local config
    #[must_use]
    pub fn new(model_name: impl Into<String>, dimension: usize) -> Self {
        Self {
            model_name: model_name.into(),
            embedding_dimension: dimension,
            optimization: OptimizationConfig::local(),
        }
    }

    /// Get the effective embedding dimension
    #[must_use]
    pub fn effective_dimension(&self) -> usize {
        self.embedding_dimension
    }
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self::new("sentence-transformers/all-MiniLM-L6-v2", 384)
    }
}

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
            format!("{}{}", base, endpoint)
        } else {
            format!("{}/{}", base, endpoint)
        }
    }

    /// Get the effective embedding dimension
    #[must_use]
    pub fn effective_dimension(&self) -> usize {
        self.embedding_dimension
    }
}

impl ProviderConfig {
    /// Get the effective embedding dimension for this provider
    #[must_use]
    pub fn effective_dimension(&self) -> usize {
        match self {
            Self::Local(config) => config.effective_dimension(),
            Self::OpenAI(config) => config.effective_dimension(),
            Self::Mistral(config) => config.effective_dimension(),
            Self::AzureOpenAI(config) => config.effective_dimension(),
            Self::Custom(config) => config.effective_dimension(),
        }
    }

    /// Get the optimization configuration
    #[must_use]
    pub fn optimization(&self) -> &OptimizationConfig {
        match self {
            Self::Local(config) => &config.optimization,
            Self::OpenAI(config) => &config.optimization,
            Self::Mistral(config) => &config.optimization,
            Self::AzureOpenAI(config) => &config.optimization,
            Self::Custom(config) => &config.optimization,
        }
    }

    /// Get the model name
    #[must_use]
    pub fn model_name(&self) -> String {
        match self {
            Self::Local(config) => config.model_name.clone(),
            Self::OpenAI(config) => config.model.model_name().to_string(),
            Self::Mistral(config) => config.model.model_name().to_string(),
            Self::AzureOpenAI(config) => config.deployment_name.clone(),
            Self::Custom(config) => config.model_name.clone(),
        }
    }

    /// Validate the configuration
    /// 
    /// # Errors
    /// Returns error if any provider-specific validation fails
    pub fn validate(&self) -> anyhow::Result<()> {
        match self {
            Self::OpenAI(config) => config.validate(),
            Self::Mistral(config) => config.validate(),
            _ => Ok(()),
        }
    }

    /// Create a default OpenAI configuration
    #[must_use]
    pub fn openai_default() -> Self {
        Self::OpenAI(OpenAIConfig::default())
    }

    /// Create a default Mistral configuration
    #[must_use]
    pub fn mistral_default() -> Self {
        Self::Mistral(MistralConfig::default())
    }

    /// Create a default local configuration
    #[must_use]
    pub fn local_default() -> Self {
        Self::Local(LocalConfig::default())
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self::local_default()
    }
}

// Convenience constructors for common configurations
impl ProviderConfig {
    /// OpenAI text-embedding-3-small
    #[must_use]
    pub fn openai_3_small() -> Self {
        Self::OpenAI(OpenAIConfig::text_embedding_3_small())
    }

    /// OpenAI text-embedding-3-large
    #[must_use]
    pub fn openai_3_large() -> Self {
        Self::OpenAI(OpenAIConfig::text_embedding_3_large())
    }

    /// OpenAI text-embedding-ada-002
    #[must_use]
    pub fn openai_ada_002() -> Self {
        Self::OpenAI(OpenAIConfig::ada_002())
    }

    /// Mistral mistral-embed
    #[must_use]
    pub fn mistral_embed() -> Self {
        Self::Mistral(MistralConfig::mistral_embed())
    }

    /// Mistral codestral-embed
    #[must_use]
    pub fn codestral_embed() -> Self {
        Self::Mistral(MistralConfig::codestral_embed())
    }

    /// Mistral codestral-embed with binary output
    #[must_use]
    pub fn codestral_binary() -> Self {
        Self::Mistral(MistralConfig::codestral_binary())
    }

    /// Local sentence transformer
    #[must_use]
    pub fn local_sentence_transformer(model_name: impl Into<String>, dimension: usize) -> Self {
        Self::Local(LocalConfig::new(model_name, dimension))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_config_dimensions() {
        let openai = ProviderConfig::openai_3_small();
        assert_eq!(openai.effective_dimension(), 1536);

        let mistral = ProviderConfig::mistral_embed();
        assert_eq!(mistral.effective_dimension(), 1024);

        let codestral = ProviderConfig::codestral_embed();
        assert_eq!(codestral.effective_dimension(), 1536);

        let local = ProviderConfig::local_default();
        assert_eq!(local.effective_dimension(), 384);
    }

    #[test]
    fn test_provider_config_model_names() {
        let openai = ProviderConfig::openai_3_small();
        assert_eq!(openai.model_name(), "text-embedding-3-small");

        let mistral = ProviderConfig::mistral_embed();
        assert_eq!(mistral.model_name(), "mistral-embed");

        let codestral = ProviderConfig::codestral_embed();
        assert_eq!(codestral.model_name(), "codestral-embed");
    }

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
        assert_eq!(config.embeddings_url(), "https://api.example.com/v1/custom-embeddings");
    }

    #[test]
    fn test_provider_config_serialization() {
        let config = ProviderConfig::openai_3_small();
        let json = serde_json::to_string(&config).unwrap();
        
        // Verify the tag is present
        assert!(json.contains("\"provider\":\"openai\""));
        
        let deserialized: ProviderConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.effective_dimension(), deserialized.effective_dimension());
    }

    #[test]
    fn test_mistral_config_serialization() {
        let config = ProviderConfig::Mistral(
            MistralConfig::codestral_embed()
                .with_output_dimension(512)
                .with_output_dtype(OutputDtype::Int8)
        );
        
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ProviderConfig = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            ProviderConfig::Mistral(mistral) => {
                assert_eq!(mistral.output_dimension, Some(512));
                assert_eq!(mistral.output_dtype, OutputDtype::Int8);
            }
            _ => panic!("Expected Mistral config"),
        }
    }
}
```

### 4. Updated EmbeddingConfig

#### File: `memory-core/src/embeddings/config/embedding_config.rs` (Modified)

```rust
//! Configuration for the embedding system

use serde::{Deserialize, Serialize};

use super::{provider_config::ProviderConfig, provider_enum::EmbeddingProvider};

/// Configuration for the embedding system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Embedding provider type
    pub provider: EmbeddingProvider,
    /// Provider-specific configuration
    pub provider_config: ProviderConfig,
    /// Similarity threshold for search (0.0 to 1.0)
    pub similarity_threshold: f32,
    /// Maximum batch size for embedding generation
    pub batch_size: usize,
    /// Cache embeddings to avoid regeneration
    pub cache_embeddings: bool,
    /// Timeout for embedding requests (seconds)
    pub timeout_seconds: u64,
}

impl EmbeddingConfig {
    /// Create a new embedding config with the specified provider
    #[must_use]
    pub fn new(provider: EmbeddingProvider, provider_config: ProviderConfig) -> Self {
        Self {
            provider,
            provider_config,
            similarity_threshold: 0.7,
            batch_size: 32,
            cache_embeddings: true,
            timeout_seconds: 30,
        }
    }

    /// Set similarity threshold
    #[must_use]
    pub fn with_similarity_threshold(mut self, threshold: f32) -> Self {
        self.similarity_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Set batch size
    #[must_use]
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Set cache enabled/disabled
    #[must_use]
    pub fn with_cache(mut self, enabled: bool) -> Self {
        self.cache_embeddings = enabled;
        self
    }

    /// Set timeout in seconds
    #[must_use]
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Get the effective embedding dimension
    #[must_use]
    pub fn embedding_dimension(&self) -> usize {
        self.provider_config.effective_dimension()
    }

    /// Get the model name
    #[must_use]
    pub fn model_name(&self) -> String {
        self.provider_config.model_name()
    }

    /// Validate the configuration
    /// 
    /// # Errors
    /// Returns error if provider configuration is invalid
    pub fn validate(&self) -> anyhow::Result<()> {
        // Validate provider matches provider_config type
        let config_type_matches = match (&self.provider, &self.provider_config) {
            (EmbeddingProvider::Local, ProviderConfig::Local(_)) => true,
            (EmbeddingProvider::OpenAI, ProviderConfig::OpenAI(_)) => true,
            (EmbeddingProvider::Mistral, ProviderConfig::Mistral(_)) => true,
            (EmbeddingProvider::AzureOpenAI, ProviderConfig::AzureOpenAI(_)) => true,
            (EmbeddingProvider::Custom(_), ProviderConfig::Custom(_)) => true,
            _ => false,
        };

        if !config_type_matches {
            anyhow::bail!(
                "Provider type {:?} does not match provider_config variant",
                self.provider
            );
        }

        // Validate provider-specific configuration
        self.provider_config.validate()?;

        // Validate common parameters
        if self.similarity_threshold < 0.0 || self.similarity_threshold > 1.0 {
            anyhow::bail!("similarity_threshold must be between 0.0 and 1.0");
        }

        if self.batch_size == 0 {
            anyhow::bail!("batch_size must be greater than 0");
        }

        if self.timeout_seconds == 0 {
            anyhow::bail!("timeout_seconds must be greater than 0");
        }

        Ok(())
    }

    /// Create OpenAI configuration
    #[must_use]
    pub fn openai(config: impl Into<ProviderConfig>) -> Self {
        Self::new(EmbeddingProvider::OpenAI, config.into())
    }

    /// Create Mistral configuration
    #[must_use]
    pub fn mistral(config: impl Into<ProviderConfig>) -> Self {
        Self::new(EmbeddingProvider::Mistral, config.into())
    }

    /// Create local configuration
    #[must_use]
    pub fn local(config: impl Into<ProviderConfig>) -> Self {
        Self::new(EmbeddingProvider::Local, config.into())
    }
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self::new(
            EmbeddingProvider::Local,
            ProviderConfig::local_default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embeddings::config::{
        mistral::{MistralConfig, OutputDtype},
        openai::{EncodingFormat, OpenAIConfig},
    };

    #[test]
    fn test_default_embedding_config() {
        let config = EmbeddingConfig::default();

        assert_eq!(config.provider, EmbeddingProvider::Local);
        assert_eq!(config.embedding_dimension(), 384);
        assert_eq!(config.similarity_threshold, 0.7);
        assert_eq!(config.batch_size, 32);
        assert!(config.cache_embeddings);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_openai_embedding_config() {
        let config = EmbeddingConfig::openai(OpenAIConfig::text_embedding_3_small());

        assert_eq!(config.provider, EmbeddingProvider::OpenAI);
        assert_eq!(config.embedding_dimension(), 1536);
        assert_eq!(config.model_name(), "text-embedding-3-small");
    }

    #[test]
    fn test_mistral_embedding_config() {
        let config = EmbeddingConfig::mistral(MistralConfig::mistral_embed());

        assert_eq!(config.provider, EmbeddingProvider::Mistral);
        assert_eq!(config.embedding_dimension(), 1024);
        assert_eq!(config.model_name(), "mistral-embed");
    }

    #[test]
    fn test_codestral_embedding_config() {
        let config = EmbeddingConfig::mistral(
            MistralConfig::codestral_embed()
                .with_output_dimension(512)
                .with_output_dtype(OutputDtype::Int8)
        );

        assert_eq!(config.provider, EmbeddingProvider::Mistral);
        assert_eq!(config.embedding_dimension(), 512);
        assert_eq!(config.model_name(), "codestral-embed");
    }

    #[test]
    fn test_embedding_config_validation() {
        // Valid config
        let valid = EmbeddingConfig::default();
        assert!(valid.validate().is_ok());

        // Mismatched provider and config
        let mismatched = EmbeddingConfig {
            provider: EmbeddingProvider::OpenAI,
            provider_config: ProviderConfig::local_default(),
            ..Default::default()
        };
        assert!(mismatched.validate().is_err());

        // Invalid similarity threshold
        let invalid_threshold = EmbeddingConfig::default()
            .with_similarity_threshold(1.5);
        assert!(invalid_threshold.validate().is_err());

        // Invalid batch size
        let invalid_batch = EmbeddingConfig::default()
            .with_batch_size(0);
        assert!(invalid_batch.validate().is_err());
    }

    #[test]
    fn test_embedding_config_builder() {
        let config = EmbeddingConfig::openai(OpenAIConfig::text_embedding_3_small())
            .with_similarity_threshold(0.85)
            .with_batch_size(64)
            .with_cache(false)
            .with_timeout(60);

        assert_eq!(config.similarity_threshold, 0.85);
        assert_eq!(config.batch_size, 64);
        assert!(!config.cache_embeddings);
        assert_eq!(config.timeout_seconds, 60);
    }

    #[test]
    fn test_embedding_config_serialization() {
        let config = EmbeddingConfig::mistral(
            MistralConfig::codestral_embed()
                .with_output_dimension(512)
                .with_output_dtype(OutputDtype::Binary)
        )
        .with_similarity_threshold(0.8);

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: EmbeddingConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.provider, deserialized.provider);
        assert_eq!(config.similarity_threshold, deserialized.similarity_threshold);
        assert_eq!(config.embedding_dimension(), deserialized.embedding_dimension());
    }
}
```

### 5. Updated Module Exports

#### File: `memory-core/src/embeddings/config/mod.rs` (Modified)

```rust
//! Configuration for embedding providers

// Provider-specific configurations
pub use provider_config::{
    AzureOpenAIConfig, CustomConfig, LocalConfig, ProviderConfig,
};

// Provider enums (unchanged)
pub use provider_enum::EmbeddingProvider;

// Optimization config (unchanged)
pub use optimization_config::OptimizationConfig;

// Top-level embedding config
pub use embedding_config::EmbeddingConfig;

// Provider-specific modules
pub mod mistral;
pub mod openai;

// Internal modules
mod embedding_config;
mod optimization_config;
mod provider_config;
mod provider_enum;

// Re-export commonly used types for convenience
pub use mistral::{MistralConfig, MistralModel, OutputDtype};
pub use openai::{EncodingFormat, OpenAIConfig, OpenAIModel};
```

## API Changes

### OpenAI Provider Updates

#### File: `memory-core/src/embeddings/openai/client.rs` (Modified)

```rust
//! OpenAI embedding provider client implementation.

#[cfg(feature = "openai")]
use super::super::config::openai::{OpenAIConfig, OpenAIModel, OpenAIEmbeddingRequest, OpenAIEmbeddingInput};
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
    async fn request_embeddings(&self, input: OpenAIEmbeddingInput) -> Result<super::types::OpenAIEmbeddingResponse> {
        let url = self.config.embeddings_url();
        let max_retries = self.config.optimization.max_retries;
        let base_delay_ms = self.config.optimization.retry_delay_ms;

        // Build request with provider-specific parameters
        let request = OpenAIEmbeddingRequest {
            input,
            model: self.config.model.model_name().to_string(),
            encoding_format: Some(match self.config.encoding_format {
                super::config::EncodingFormat::Float => "float".to_string(),
                super::config::EncodingFormat::Base64 => "base64".to_string(),
            }),
            dimensions: self.config.dimensions,
        };

        // ... rest of implementation similar to before, but using new types
    }

    // ... rest of implementation
}
```

### New Mistral Provider

#### File: `memory-core/src/embeddings/mistral/client.rs` (New)

```rust
//! Mistral embedding provider client implementation

#[cfg(feature = "mistral")]
use super::super::config::mistral::{MistralConfig, MistralEmbeddingInput, MistralEmbeddingRequest, OutputDtype};
#[cfg(feature = "mistral")]
use crate::embeddings::provider::EmbeddingProvider;
#[cfg(feature = "mistral")]
use anyhow::{Context, Result};
#[cfg(feature = "mistral")]
use async_trait::async_trait;
#[cfg(feature = "mistral")]
use std::time::Instant;

#[cfg(feature = "mistral")]
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

#[cfg(feature = "mistral")]
impl MistralEmbeddingProvider {
    /// Create a new Mistral embedding provider
    pub fn new(api_key: String, config: MistralConfig) -> anyhow::Result<Self> {
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
    async fn request_embeddings(&self, inputs: MistralEmbeddingInput) -> Result<super::types::MistralEmbeddingResponse> {
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

        // ... retry logic and request handling
    }

    // Handle different output dtypes
    fn process_embedding_response(&self, data: Vec<f32>) -> Result<Vec<f32>> {
        match self.config.output_dtype {
            OutputDtype::Float => Ok(data),
            OutputDtype::Int8 | OutputDtype::Uint8 => {
                // Convert integer embeddings back to float for uniform interface
                // In production, you might want to keep as integers for efficiency
                Ok(data.into_iter().map(|v| v as f32).collect())
            }
            OutputDtype::Binary | OutputDtype::Ubinary => {
                // Dequantize binary embeddings
                // See: https://colab.research.google.com/github/mistralai/cookbook/blob/main/mistral/embeddings/dequantization.ipynb
                self.dequantize_binary_embeddings(data)
            }
        }
    }

    fn dequantize_binary_embeddings(&self, packed: Vec<f32>) -> Result<Vec<f32>> {
        // Convert packed binary representation back to float embeddings
        // This is a simplified version - see Mistral's dequantization cookbook for full implementation
        anyhow::bail!("Binary dequantization not yet implemented")
    }
}

#[cfg(feature = "mistral")]
#[async_trait]
impl EmbeddingProvider for MistralEmbeddingProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        // ... implementation
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // ... implementation with batching
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
        })
    }
}
```

## Migration Strategy

Since backward compatibility is NOT required, we will perform a clean replacement:

### Phase 1: Create New Config Modules

1. **Create directory structure**:
   ```bash
   mkdir -p memory-core/src/embeddings/config/openai
   mkdir -p memory-core/src/embeddings/config/mistral
   mkdir -p memory-core/src/embeddings/mistral
   ```

2. **Create new config files**:
   - `memory-core/src/embeddings/config/openai/mod.rs`
   - `memory-core/src/embeddings/config/openai/config.rs`
   - `memory-core/src/embeddings/config/openai/types.rs`
   - `memory-core/src/embeddings/config/mistral/mod.rs`
   - `memory-core/src/embeddings/config/mistral/config.rs`
   - `memory-core/src/embeddings/config/mistral/types.rs`
   - `memory-core/src/embeddings/config/provider_config.rs`

### Phase 2: Update Existing Files

1. **Update `memory-core/src/embeddings/config/mod.rs`**:
   - Add new module exports
   - Re-export convenience types

2. **Update `memory-core/src/embeddings/config/embedding_config.rs`**:
   - Replace `ModelConfig` with `ProviderConfig`
   - Add validation logic
   - Update tests

3. **Update `memory-core/src/embeddings/openai/client.rs`**:
   - Change from `ModelConfig` to `OpenAIConfig`
   - Update request building to use new types
   - Support dimensions parameter

4. **Update `memory-core/src/embeddings/mod.rs`**:
   - Update exports
   - Update `EmbeddingService` to use new config types

### Phase 3: Remove Old Code

1. **Delete `memory-core/src/embeddings/config/model_config.rs`**:
   - After confirming all usages are migrated

2. **Update tests**:
   - `memory-core/src/embeddings/openai_tests.rs`
   - `memory-core/src/embeddings/tests.rs`
   - Any other test files using old config

### Files to Modify

| File | Changes |
|------|---------|
| `memory-core/src/embeddings/config/mod.rs` | Add new module exports |
| `memory-core/src/embeddings/config/embedding_config.rs` | Use ProviderConfig |
| `memory-core/src/embeddings/config/provider_config.rs` | **NEW FILE** |
| `memory-core/src/embeddings/config/openai/mod.rs` | **NEW FILE** |
| `memory-core/src/embeddings/config/openai/config.rs` | **NEW FILE** |
| `memory-core/src/embeddings/config/openai/types.rs` | **NEW FILE** |
| `memory-core/src/embeddings/config/mistral/mod.rs` | **NEW FILE** |
| `memory-core/src/embeddings/config/mistral/config.rs` | **NEW FILE** |
| `memory-core/src/embeddings/config/mistral/types.rs` | **NEW FILE** |
| `memory-core/src/embeddings/openai/client.rs` | Use OpenAIConfig |
| `memory-core/src/embeddings/openai/mod.rs` | Update exports |
| `memory-core/src/embeddings/mod.rs` | Update exports and service |
| `memory-core/src/embeddings/tests.rs` | Update tests |
| `memory-core/src/embeddings/openai_tests.rs` | Update tests |
| `memory-core/src/embeddings/config/model_config.rs` | **DELETE** |

## Testing Strategy

### Unit Tests

1. **Config Structure Tests**:
   ```rust
   // Test OpenAI config
   #[test]
   fn test_openai_config_validation() {
       // Valid configs
       assert!(OpenAIConfig::text_embedding_3_small().validate().is_ok());
       assert!(OpenAIConfig::text_embedding_3_small()
           .with_dimensions(512)
           .validate()
           .is_ok());
       
       // Invalid: ada-002 doesn't support dimensions
       assert!(OpenAIConfig::ada_002()
           .with_dimensions(512)
           .validate()
           .is_err());
       
       // Invalid: dimension too large
       assert!(OpenAIConfig::text_embedding_3_large()
           .with_dimensions(4000)
           .validate()
           .is_err());
   }

   // Test Mistral config
   #[test]
   fn test_mistral_codestral_features() {
       // Valid codestral with custom dimension
       let config = MistralConfig::codestral_embed()
           .with_output_dimension(512);
       assert_eq!(config.effective_dimension(), 512);
       
       // Valid codestral with binary output
       let config = MistralConfig::codestral_binary();
       assert_eq!(config.output_dtype, OutputDtype::Binary);
       assert_eq!(config.expected_response_size(), 192); // 1536 / 8
       
       // Invalid: mistral-embed doesn't support custom dimensions
       assert!(MistralConfig::mistral_embed()
           .with_output_dimension(512)
           .validate()
           .is_err());
   }
   ```

2. **ProviderConfig Enum Tests**:
   ```rust
   #[test]
   fn test_provider_config_type_safety() {
       let openai = ProviderConfig::openai_3_small();
       let mistral = ProviderConfig::mistral_embed();
       let codestral = ProviderConfig::codestral_binary();
       
       assert_eq!(openai.effective_dimension(), 1536);
       assert_eq!(mistral.effective_dimension(), 1024);
       assert_eq!(codestral.effective_dimension(), 1536);
   }
   ```

3. **Serialization Tests**:
   ```rust
   #[test]
   fn test_config_serialization_roundtrip() {
       let configs = vec![
           ProviderConfig::openai_3_small(),
           ProviderConfig::mistral_embed(),
           ProviderConfig::Mistral(
               MistralConfig::codestral_embed()
                   .with_output_dimension(512)
                   .with_output_dtype(OutputDtype::Int8)
           ),
       ];
       
       for config in configs {
           let json = serde_json::to_string(&config).unwrap();
           let deserialized: ProviderConfig = serde_json::from_str(&json).unwrap();
           assert_eq!(config.effective_dimension(), deserialized.effective_dimension());
       }
   }
   ```

### Integration Tests

1. **Provider Creation Tests**:
   ```rust
   #[tokio::test]
   async fn test_openai_provider_with_dimensions() {
       let config = OpenAIConfig::text_embedding_3_small()
           .with_dimensions(512);
       let provider = OpenAIEmbeddingProvider::new(
           "test-api-key".to_string(),
           config
       ).unwrap();
       
       assert_eq!(provider.embedding_dimension(), 512);
   }

   #[tokio::test]
   async fn test_mistral_codestral_provider() {
       let config = MistralConfig::codestral_embed()
           .with_output_dimension(256)
           .with_output_dtype(OutputDtype::Int8);
       let provider = MistralEmbeddingProvider::new(
           "test-api-key".to_string(),
           config
       ).unwrap();
       
       assert_eq!(provider.embedding_dimension(), 256);
   }
   ```

2. **Request Building Tests**:
   ```rust
   #[test]
   fn test_openai_request_with_dimensions() {
       let config = OpenAIConfig::text_embedding_3_small()
           .with_dimensions(512);
       let request = OpenAIEmbeddingRequest {
           input: "test".into(),
           model: config.model.model_name().to_string(),
           encoding_format: Some("float".to_string()),
           dimensions: config.dimensions,
       };
       
       let json = serde_json::to_string(&request).unwrap();
       assert!(json.contains("\"dimensions\":512"));
   }

   #[test]
   fn test_mistral_request_with_codestral_options() {
       let config = MistralConfig::codestral_embed()
           .with_output_dimension(512)
           .with_output_dtype(OutputDtype::Int8);
       let request = MistralEmbeddingRequest {
           inputs: vec!["test".to_string()].into(),
           model: config.model.model_name().to_string(),
           output_dtype: Some("int8".to_string()),
           output_dimension: Some(512),
       };
       
       let json = serde_json::to_string(&request).unwrap();
       assert!(json.contains("\"output_dimension\":512"));
       assert!(json.contains("\"output_dtype\":\"int8\""));
   }
   ```

### Test Cases for Codestral Features

```rust
#[cfg(test)]
mod codestral_tests {
    use super::*;

    #[test]
    fn test_codestral_output_dimensions() {
        // Test all valid dimension ranges
        for dim in [1, 128, 512, 1024, 1536, 2048, 3072] {
            let config = MistralConfig::codestral_embed()
                .with_output_dimension(dim);
            assert_eq!(config.effective_dimension(), dim);
            assert!(config.validate().is_ok());
        }
    }

    #[test]
    fn test_codestral_output_dtypes() {
        let dtypes = vec![
            (OutputDtype::Float, 1536, 1536),
            (OutputDtype::Int8, 1536, 1536),
            (OutputDtype::Uint8, 1536, 1536),
            (OutputDtype::Binary, 1536, 192),  // 1536 / 8
            (OutputDtype::Ubinary, 1536, 192), // 1536 / 8
        ];

        for (dtype, input_dim, expected_size) in dtypes {
            let config = MistralConfig::codestral_embed()
                .with_output_dimension(input_dim)
                .with_output_dtype(dtype);
            assert_eq!(config.expected_response_size(), expected_size);
        }
    }

    #[test]
    fn test_codestral_binary_response_size() {
        // Verify bit-packed size calculation
        let config = MistralConfig::codestral_binary();
        assert_eq!(config.expected_response_size(), 192); // 1536 / 8

        let config = MistralConfig::codestral_embed()
            .with_output_dimension(1000)
            .with_output_dtype(OutputDtype::Binary);
        assert_eq!(config.expected_response_size(), 125); // ceil(1000 / 8)
    }

    #[test]
    fn test_mistral_embed_constraints() {
        // mistral-embed has fixed 1024 dimensions
        let config = MistralConfig::mistral_embed();
        assert_eq!(config.effective_dimension(), 1024);
        assert!(!config.model.supports_output_dimension());
        assert!(!config.model.supports_output_dtype());

        // Attempting to set custom options should fail validation
        let invalid_dim = MistralConfig::mistral_embed()
            .with_output_dimension(512);
        assert!(invalid_dim.validate().is_err());
    }
}
```

## Implementation Phases

### Phase 1: Foundation (Week 1)

**Goal**: Create new config modules with full test coverage

**Tasks**:
1. Create directory structure
2. Implement `openai/config.rs` with all types and tests
3. Implement `openai/types.rs` with request/response types
4. Implement `mistral/config.rs` with all types and tests
5. Implement `mistral/types.rs` with request/response types
6. Implement `provider_config.rs` with unified enum
7. Run tests: `cargo test -p memory-core embeddings::config`

**Success Criteria**:
- All new config modules compile
- All unit tests pass
- Code coverage >90% for new modules

### Phase 2: OpenAI Provider Update (Week 1-2)

**Goal**: Update OpenAI provider to use new config

**Tasks**:
1. Update `openai/client.rs` to use `OpenAIConfig`
2. Update `openai/mod.rs` exports
3. Update `openai/types.rs` (or remove if using config/types.rs)
4. Update `openai_tests.rs` to use new config
5. Test with mock server

**Success Criteria**:
- OpenAI provider compiles with new config
- All OpenAI tests pass
- Dimensions parameter works correctly

### Phase 3: Mistral Provider Creation (Week 2)

**Goal**: Create new Mistral provider with full feature support

**Tasks**:
1. Create `mistral/client.rs` with `MistralEmbeddingProvider`
2. Create `mistral/mod.rs` with exports
3. Implement codestral-specific features (output_dtype, output_dimension)
4. Add comprehensive tests
5. Add integration tests

**Success Criteria**:
- Mistral provider compiles
- All Mistral tests pass
- Codestral features work correctly
- Binary/Int8 output types handled

### Phase 4: Top-Level Integration (Week 2-3)

**Goal**: Integrate new configs into embedding system

**Tasks**:
1. Update `config/mod.rs` exports
2. Update `config/embedding_config.rs` to use `ProviderConfig`
3. Update `embeddings/mod.rs` and `EmbeddingService`
4. Update `embeddings/tests.rs`
5. Update any other files using old `ModelConfig`

**Success Criteria**:
- Full crate compiles
- All tests pass
- No references to old `ModelConfig`

### Phase 5: Cleanup and Validation (Week 3)

**Goal**: Remove old code and validate everything works

**Tasks**:
1. Delete `config/model_config.rs`
2. Run full test suite: `cargo test -p memory-core`
3. Run quality gates: `./scripts/quality-gates.sh`
4. Update documentation
5. Create example usage code

**Success Criteria**:
- All tests pass
- No clippy warnings
- Code coverage maintained (>90%)
- Documentation updated

## Usage Examples

### Example 1: Basic OpenAI Usage

```rust
use memory_core::embeddings::{
    EmbeddingConfig, EmbeddingService,
    OpenAIConfig, OpenAIModel,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Use OpenAI text-embedding-3-small with default settings
    let config = EmbeddingConfig::openai(OpenAIConfig::text_embedding_3_small());
    let service = EmbeddingService::new(config).await?;
    
    let embedding = service.embed("Hello, world!").await?;
    println!("Generated {}-dimensional embedding", embedding.len());
    
    Ok(())
}
```

### Example 2: OpenAI with Custom Dimensions

```rust
use memory_core::embeddings::{
    EmbeddingConfig, EmbeddingService,
    OpenAIConfig, OpenAIModel, EncodingFormat,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Use text-embedding-3-large with reduced dimensions for efficiency
    let openai_config = OpenAIConfig::text_embedding_3_large()
        .with_dimensions(1024)  // Reduce from 3072 to 1024
        .with_encoding_format(EncodingFormat::Float);
    
    let config = EmbeddingConfig::openai(openai_config)
        .with_similarity_threshold(0.8)
        .with_batch_size(64);
    
    let service = EmbeddingService::new(config).await?;
    
    // Process batch of texts
    let texts = vec![
        "First document".to_string(),
        "Second document".to_string(),
    ];
    let embeddings = service.embed_batch(&texts).await?;
    
    Ok(())
}
```

### Example 3: Mistral Text Embeddings

```rust
use memory_core::embeddings::{
    EmbeddingConfig, EmbeddingService,
    MistralConfig, MistralModel,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Use Mistral's general-purpose text embedding model
    let config = EmbeddingConfig::mistral(MistralConfig::mistral_embed());
    let service = EmbeddingService::new(config).await?;
    
    let embedding = service.embed("Semantic search is powerful").await?;
    println!("Generated {}-dimensional embedding", embedding.len());
    // Output: Generated 1024-dimensional embedding
    
    Ok(())
}
```

### Example 4: Codestral with Custom Output

```rust
use memory_core::embeddings::{
    EmbeddingConfig, EmbeddingService,
    MistralConfig, MistralModel, OutputDtype,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Use codestral-embed with reduced dimensions and int8 quantization
    // This reduces storage by 4x compared to float32
    let mistral_config = MistralConfig::codestral_embed()
        .with_output_dimension(512)      // Reduce from 1536 to 512
        .with_output_dtype(OutputDtype::Int8);  // Use 8-bit integers
    
    let config = EmbeddingConfig::mistral(mistral_config);
    let service = EmbeddingService::new(config).await?;
    
    // Embed code snippets
    let code = r#"
        fn fibonacci(n: u32) -> u32 {
            match n {
                0 => 0,
                1 => 1,
                _ => fibonacci(n - 1) + fibonacci(n - 2),
            }
        }
    "#;
    
    let embedding = service.embed(code).await?;
    println!("Generated {}-dimensional code embedding", embedding.len());
    // Output: Generated 512-dimensional code embedding
    
    Ok(())
}
```

### Example 5: Codestral with Binary Embeddings (Maximum Compression)

```rust
use memory_core::embeddings::{
    EmbeddingConfig, EmbeddingService,
    MistralConfig, OutputDtype,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Use binary embeddings for maximum storage efficiency
    // 32x reduction compared to float32 (1 bit vs 32 bits per dimension)
    let mistral_config = MistralConfig::codestral_binary();
    
    let config = EmbeddingConfig::mistral(mistral_config);
    let service = EmbeddingService::new(config).await?;
    
    // Process large codebase
    let code_snippets = vec![
        "def hello(): pass".to_string(),
        "class World: pass".to_string(),
        // ... thousands more
    ];
    
    let embeddings = service.embed_batch(&code_snippets).await?;
    println!("Generated {} binary embeddings", embeddings.len());
    // Each embedding has 1536 dimensions but is stored as 192 bytes (1536/8)
    
    Ok(())
}
```

### Example 6: Configuration from JSON

```rust
use memory_core::embeddings::EmbeddingConfig;

fn main() -> anyhow::Result<()> {
    // Load configuration from JSON file
    let config_json = r#"{
        "provider": "mistral",
        "provider_config": {
            "provider": "mistral",
            "model": "codestral_embed",
            "output_dimension": 1024,
            "output_dtype": "int8",
            "base_url": "https://api.mistral.ai/v1"
        },
        "similarity_threshold": 0.75,
        "batch_size": 32,
        "cache_embeddings": true,
        "timeout_seconds": 30
    }"#;
    
    let config: EmbeddingConfig = serde_json::from_str(config_json)?;
    config.validate()?;
    
    println!("Loaded config for model: {}", config.model_name());
    println!("Embedding dimension: {}", config.embedding_dimension());
    
    Ok(())
}
```

## Summary

This refactoring plan provides:

1. **Type Safety**: Provider-specific configurations with compile-time validation
2. **Feature Completeness**: Full support for OpenAI dimensions and Mistral output_dtype/output_dimension
3. **Extensibility**: Easy to add new providers with the `ProviderConfig` enum pattern
4. **Testability**: Comprehensive unit and integration tests for all features
5. **Clear API**: Builder patterns and convenience constructors for ergonomic usage
6. **Documentation**: Code examples showing all major use cases

The implementation follows Rust best practices:
- Serde for serialization with proper attributes
- Builder patterns for configuration
- Newtype and enum patterns for type safety
- Comprehensive error handling with `anyhow`
- Full test coverage with edge cases
- Documentation with doc tests

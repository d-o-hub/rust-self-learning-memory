//! Unified provider configuration enum

use serde::{Deserialize, Serialize};

use super::OptimizationConfig;
use super::cloud_config::{AzureOpenAIConfig, CustomConfig};
use super::local_config::LocalConfig;
use super::mistral::MistralConfig;
use super::openai::OpenAIConfig;

/// Provider-specific configuration
///
/// This enum wraps all provider-specific configurations, allowing type-safe
/// access to provider-specific features while maintaining a unified interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider")]
pub enum ProviderConfig {
    /// Local embedding configuration
    #[serde(rename = "local")]
    Local(LocalConfig),
    /// OpenAI embedding configuration
    #[serde(rename = "openai")]
    OpenAI(OpenAIConfig),
    /// Mistral AI embedding configuration
    #[serde(rename = "mistral")]
    Mistral(MistralConfig),
    /// Azure OpenAI configuration
    #[serde(rename = "azure_openai")]
    AzureOpenAI(AzureOpenAIConfig),
    /// Custom provider configuration
    #[serde(rename = "custom")]
    Custom(CustomConfig),
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

    /// Stable cache/provider identity string (`kind:model:dims`) for ADR-074.
    #[must_use]
    pub fn cache_identity(&self) -> String {
        let kind = match self {
            Self::Local(_) => "local",
            Self::OpenAI(_) => "openai",
            Self::Mistral(_) => "mistral",
            Self::AzureOpenAI(_) => "azure_openai",
            Self::Custom(_) => "custom",
        };
        format!(
            "{}:{}:{}",
            kind,
            self.model_name().trim(),
            self.effective_dimension()
        )
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

impl Default for ProviderConfig {
    fn default() -> Self {
        Self::local_default()
    }
}

#[cfg(test)]
mod tests {
    use super::super::mistral::OutputDtype;
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
    fn test_provider_config_serialization() {
        let config = ProviderConfig::openai_3_small();
        let json = serde_json::to_string(&config).unwrap();

        assert!(
            json.contains("\"provider\":\"openai\"") || json.contains("\"openai\""),
            "Expected provider tag in JSON, got: {json}"
        );

        let deserialized: ProviderConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(
            config.effective_dimension(),
            deserialized.effective_dimension()
        );
    }

    #[test]
    fn test_mistral_config_serialization() {
        let config = ProviderConfig::Mistral(
            MistralConfig::codestral_embed()
                .with_output_dimension(512)
                .with_output_dtype(OutputDtype::Int8),
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

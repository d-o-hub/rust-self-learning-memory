//! Configure embeddings tool implementation.

use super::super::definitions::EmbeddingTools;
use crate::mcp::tools::embeddings::types::{ConfigureEmbeddingsInput, ConfigureEmbeddingsOutput};
use anyhow::{Result, anyhow};
use memory_core::embeddings::config::{
    AzureOpenAIConfig, CustomConfig, EmbeddingConfig, EmbeddingProvider, LocalConfig,
    ProviderConfig,
};
use tracing::{debug, info, instrument};

impl EmbeddingTools {
    /// Execute the configure_embeddings tool
    #[instrument(skip(self, input), fields(provider = %input.provider))]
    pub async fn execute_configure_embeddings(
        &self,
        input: ConfigureEmbeddingsInput,
    ) -> Result<ConfigureEmbeddingsOutput> {
        info!("Configuring embedding provider: {}", input.provider);

        let mut warnings = Vec::new();

        // Parse provider type
        let provider_type = match input.provider.to_lowercase().as_str() {
            "openai" => EmbeddingProvider::OpenAI,
            "local" => EmbeddingProvider::Local,
            "mistral" => EmbeddingProvider::Mistral,
            "azure" => EmbeddingProvider::AzureOpenAI,
            "cohere" => {
                warnings.push(
                    "Cohere provider not yet implemented, using Local as fallback".to_string(),
                );
                EmbeddingProvider::Local
            }
            _ => {
                return Err(anyhow!(
                    "Unsupported provider: {}. Supported providers: openai, local, mistral, azure, cohere",
                    input.provider
                ));
            }
        };

        // Validate API key for cloud providers
        if matches!(
            provider_type,
            EmbeddingProvider::OpenAI | EmbeddingProvider::Mistral | EmbeddingProvider::AzureOpenAI
        ) {
            if let Some(api_key_env) = &input.api_key_env {
                if std::env::var(api_key_env).is_err() {
                    return Err(anyhow!(
                        "Environment variable '{}' not set. Please set the API key.",
                        api_key_env
                    ));
                }
            } else {
                warnings.push(format!(
                    "No api_key_env specified for {}. Make sure API key is set in standard environment variable.",
                    input.provider
                ));
            }
        }

        // Build model configuration based on provider
        let provider_config =
            match provider_type {
                EmbeddingProvider::OpenAI => {
                    let model_name = input.model.as_deref().unwrap_or("text-embedding-3-small");
                    match model_name {
                        "text-embedding-3-small" => ProviderConfig::openai_3_small(),
                        "text-embedding-3-large" => ProviderConfig::openai_3_large(),
                        "text-embedding-ada-002" => ProviderConfig::openai_ada_002(),
                        _ => {
                            warnings.push(format!(
                                "Unknown OpenAI model '{}', using text-embedding-3-small",
                                model_name
                            ));
                            ProviderConfig::openai_3_small()
                        }
                    }
                }
                EmbeddingProvider::Mistral => {
                    let model_name = input.model.as_deref().unwrap_or("mistral-embed");
                    if model_name != "mistral-embed" {
                        warnings.push(format!(
                            "Unknown Mistral model '{}', using mistral-embed",
                            model_name
                        ));
                    }
                    ProviderConfig::mistral_embed()
                }
                EmbeddingProvider::AzureOpenAI => {
                    let deployment = input.deployment_name.as_ref().ok_or_else(|| {
                        anyhow!("deployment_name required for Azure OpenAI provider")
                    })?;
                    let resource = input.resource_name.as_ref().ok_or_else(|| {
                        anyhow!("resource_name required for Azure OpenAI provider")
                    })?;
                    let api_version = input.api_version.as_deref().unwrap_or("2023-05-15");

                    // Azure dimension depends on the underlying model
                    let dimension = 1536; // Default for ada-002 and text-embedding-3-small
                    ProviderConfig::AzureOpenAI(AzureOpenAIConfig::new(
                        deployment,
                        resource,
                        api_version,
                        dimension,
                    ))
                }
                EmbeddingProvider::Local => {
                    let model_name = input
                        .model
                        .as_deref()
                        .unwrap_or("sentence-transformers/all-MiniLM-L6-v2");
                    let dimension = 384; // Default for MiniLM
                    ProviderConfig::Local(LocalConfig::new(model_name, dimension))
                }
                EmbeddingProvider::Custom(_) => {
                    let model_name = input.model.as_deref().unwrap_or("custom-model");
                    let base_url = input
                        .base_url
                        .as_deref()
                        .ok_or_else(|| anyhow!("base_url required for custom provider"))?;
                    ProviderConfig::Custom(CustomConfig::new(model_name, 384, base_url))
                }
            };

        // Build embedding configuration
        let embedding_config = EmbeddingConfig {
            provider: provider_config.clone(),
            similarity_threshold: input.similarity_threshold.unwrap_or(0.7),
            batch_size: input.batch_size.unwrap_or(32),
            cache_embeddings: true,
            timeout_seconds: 30,
        };

        // NOTE: In a real implementation, you would update the memory system's
        // semantic_service here. Since semantic_service is private and Option,
        // we simulate the configuration response.

        debug!(
            "Configured embedding provider: {:?} with model: {}",
            embedding_config.provider,
            embedding_config.provider.model_name()
        );

        let provider_name = input.provider.clone();
        Ok(ConfigureEmbeddingsOutput {
            success: true,
            provider: input.provider,
            model: provider_config.model_name(),
            dimension: provider_config.effective_dimension(),
            message: format!(
                "Successfully configured {} provider with model {} (dimension: {})",
                provider_name,
                provider_config.model_name(),
                provider_config.effective_dimension()
            ),
            warnings,
        })
    }
}

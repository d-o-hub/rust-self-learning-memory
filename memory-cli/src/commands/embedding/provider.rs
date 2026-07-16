//! Embedding provider factory helpers for CLI commands.

use crate::config::Config;
use anyhow::Result;
#[allow(unused_imports)] // All configs are used in match statements
use do_memory_core::embeddings::{
    AzureOpenAIConfig, CustomConfig, EmbeddingProvider, LocalConfig, MistralConfig, OpenAIConfig,
};
use std::env;

/// Create embedding provider from configuration
pub(crate) async fn create_provider_from_config(
    config: &Config,
) -> Result<Box<dyn EmbeddingProvider>> {
    match config.embeddings.provider.as_str() {
        "local" => {
            #[cfg(feature = "local-embeddings")]
            {
                use do_memory_core::embeddings::LocalEmbeddingProvider;
                let model_config = LocalConfig::default();
                let provider = LocalEmbeddingProvider::new(model_config).await?;
                Ok(Box::new(provider))
            }
            #[cfg(not(feature = "local-embeddings"))]
            {
                Err(anyhow::anyhow!(
                    "Local embeddings not available. Compile with --features local-embeddings"
                ))
            }
        }
        "openai" => {
            #[cfg(feature = "openai")]
            {
                use do_memory_core::embeddings::OpenAIEmbeddingProvider;
                let api_key = get_api_key(config)?;
                let model_config = OpenAIConfig::text_embedding_3_small();
                let provider = OpenAIEmbeddingProvider::new(api_key, model_config)?;
                Ok(Box::new(provider))
            }
            #[cfg(not(feature = "openai"))]
            {
                Err(anyhow::anyhow!(
                    "OpenAI embeddings not available. Compile with --features openai"
                ))
            }
        }
        "mistral" => {
            #[cfg(feature = "mistral")]
            {
                use do_memory_core::embeddings::MistralEmbeddingProvider;
                let api_key = get_api_key(config)?;
                let model_config = MistralConfig::mistral_embed();
                let provider = MistralEmbeddingProvider::new(api_key, model_config)?;
                Ok(Box::new(provider))
            }
            #[cfg(not(feature = "mistral"))]
            {
                Err(anyhow::anyhow!(
                    "Mistral embeddings not available. Compile with --features mistral"
                ))
            }
        }
        "azure" => {
            #[cfg(feature = "openai")]
            {
                use do_memory_core::embeddings::OpenAIEmbeddingProvider;
                let api_key = get_api_key(config)?;
                // Azure configuration requires deployment, resource, and version
                let deployment = env::var("AZURE_DEPLOYMENT")
                    .unwrap_or_else(|_| config.embeddings.model.clone());
                let resource = env::var("AZURE_RESOURCE")?;
                let api_version =
                    env::var("AZURE_API_VERSION").unwrap_or_else(|_| "2023-05-15".to_string());

                let model_config = OpenAIConfig::text_embedding_3_small()
                    .with_base_url(format!(
                        "<https://{}.openai.azure.com/openai/deployments/{}>",
                        resource, deployment
                    ))
                    .with_dimensions(config.embeddings.dimension);
                // The API version needs to be appended to the URL as a query param in OpenAIEmbeddingProvider
                // but currently OpenAIConfig::embeddings_url just appends /embeddings.
                // This is a known limitation in the current CLI implementation.
                let provider = OpenAIEmbeddingProvider::new(api_key, model_config)?;
                Ok(Box::new(provider))
            }
            #[cfg(not(feature = "openai"))]
            {
                Err(anyhow::anyhow!(
                    "Azure OpenAI embeddings not available. Compile with --features openai"
                ))
            }
        }
        "custom" => {
            #[cfg(feature = "openai")]
            {
                use do_memory_core::embeddings::OpenAIEmbeddingProvider;
                let api_key = get_api_key(config).unwrap_or_else(|_| "not-needed".to_string());
                let base_url = config
                    .embeddings
                    .base_url
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("base_url required for custom provider"))?;

                let model_config = OpenAIConfig::text_embedding_3_small()
                    .with_base_url(base_url.clone())
                    .with_dimensions(config.embeddings.dimension);
                let provider = OpenAIEmbeddingProvider::new(api_key, model_config)?;
                Ok(Box::new(provider))
            }
            #[cfg(not(feature = "openai"))]
            {
                Err(anyhow::anyhow!(
                    "Custom embeddings not available. Compile with --features openai"
                ))
            }
        }
        _ => Err(anyhow::anyhow!(
            "Unknown provider: {}. Valid providers: local, openai, mistral, azure, custom",
            config.embeddings.provider
        )),
    }
}

/// Get API key from environment variable specified in config
pub fn get_api_key(config: &Config) -> Result<String> {
    let env_var = config
        .embeddings
        .api_key_env
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("api_key_env not configured"))?;

    env::var(env_var).map_err(|_| {
        anyhow::anyhow!(
            "API key environment variable '{}' not set. Please set it with: export {}=your-key",
            env_var,
            env_var
        )
    })
}

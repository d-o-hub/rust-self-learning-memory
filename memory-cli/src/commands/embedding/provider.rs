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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{
        CliConfig, Config, DatabaseConfig, EmbeddingsConfig, StorageConfig,
    };
    use serial_test::serial;

    fn base_config(provider: &str, api_key_env: Option<String>) -> Config {
        Config {
            database: DatabaseConfig {
                turso_url: None,
                turso_token: None,
                redb_path: Some(":memory:".to_string()),
                storage_mode: None,
                db_path: None,
            },
            storage: StorageConfig {
                max_episodes_cache: 100,
                cache_ttl_seconds: 3600,
                pool_size: 5,
                storage_mode: None,
            },
            cli: CliConfig {
                default_format: "human".to_string(),
                progress_bars: false,
                batch_size: 50,
            },
            embeddings: EmbeddingsConfig {
                enabled: true,
                provider: provider.to_string(),
                model: "text-embedding-3-small".to_string(),
                dimension: 1536,
                similarity_threshold: 0.7,
                batch_size: 100,
                cache_embeddings: true,
                timeout_seconds: 30,
                base_url: None,
                api_key_env,
            },
        }
    }

    #[test]
    #[serial]
    #[allow(unsafe_code)] // env mutation gated by serial_test
    fn get_api_key_reads_env_var() {
        let var = "DO_MEMORY_TEST_EMBED_API_KEY";
        // SAFETY: serial_test ensures exclusive access to this env var.
        unsafe {
            env::set_var(var, "test-secret-key");
        }
        let config = base_config("openai", Some(var.to_string()));
        let key = get_api_key(&config).expect("key present");
        assert_eq!(key, "test-secret-key");
        unsafe {
            env::remove_var(var);
        }
    }

    fn assert_provider_err(result: Result<Box<dyn EmbeddingProvider>>, must_contain: &[&str]) {
        match result {
            Ok(_) => panic!("expected provider creation to fail"),
            Err(e) => {
                let msg = e.to_string();
                assert!(
                    must_contain.iter().any(|s| msg.contains(s)),
                    "unexpected error: {msg}; expected one of {must_contain:?}"
                );
            }
        }
    }

    #[tokio::test]
    async fn create_provider_unknown_name() {
        let config = base_config("not-a-real-provider", None);
        assert_provider_err(
            create_provider_from_config(&config).await,
            &["Unknown provider"],
        );
    }

    #[tokio::test]
    async fn create_provider_openai_missing_key_env() {
        let config = base_config("openai", None);
        assert_provider_err(
            create_provider_from_config(&config).await,
            &["api_key_env", "OpenAI embeddings not available", "not set"],
        );
    }

    #[tokio::test]
    async fn create_provider_mistral_missing_feature_or_key() {
        let config = base_config("mistral", None);
        assert_provider_err(
            create_provider_from_config(&config).await,
            &["Mistral", "api_key_env", "not set"],
        );
    }

    #[tokio::test]
    async fn create_provider_azure_missing_feature_or_key() {
        let config = base_config("azure", None);
        assert_provider_err(
            create_provider_from_config(&config).await,
            &["Azure", "api_key_env", "not set"],
        );
    }

    #[tokio::test]
    async fn create_provider_custom_missing_base_url() {
        let mut config = base_config("custom", Some("UNUSED_KEY".into()));
        config.embeddings.base_url = None;
        assert_provider_err(
            create_provider_from_config(&config).await,
            &["base_url", "Custom embeddings not available"],
        );
    }

    #[tokio::test]
    async fn create_provider_local_without_or_with_feature() {
        let config = base_config("local", None);
        // May succeed with local-embeddings feature (loads mock) or fail without it.
        match create_provider_from_config(&config).await {
            Ok(_) => {}
            Err(e) => {
                let msg = e.to_string();
                assert!(
                    msg.contains("Local embeddings not available") || msg.contains("local"),
                    "unexpected: {msg}"
                );
            }
        }
    }
}

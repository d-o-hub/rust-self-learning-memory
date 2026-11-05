//! Embedding provider factory helpers for CLI commands.

use crate::config::Config;
use anyhow::Result;
use do_memory_core::embeddings::config::{MistralModel, OpenAIModel};
use do_memory_core::embeddings::{
    CustomConfig, EmbeddingProvider, LocalConfig, MistralConfig, OpenAIConfig, ProviderConfig,
};
use std::env;

/// Map CLI embedding settings onto a core [`ProviderConfig`].
///
/// This is the single source of truth for provider identity: the returned
/// config drives both the runtime provider and the embedding storage scope
/// (`cache_identity()` / `config_revision()`), so a query can never read vectors
/// written under a different model or dimension.
///
/// The second tuple element is the resolved credential, `None` for providers
/// that need none (local) and optional for OpenAI-compatible custom endpoints.
///
/// # Errors
///
/// Returns an error for an unknown provider name, an unsupported Azure
/// transport, a missing API key environment variable, a model name the provider
/// does not know, or a dimension the model cannot produce.
pub(crate) fn map_provider_config(config: &Config) -> Result<(ProviderConfig, Option<String>)> {
    let settings = &config.embeddings;
    match settings.provider.as_str() {
        "local" => {
            let local = LocalConfig::new(settings.model.as_str(), settings.dimension)
                .with_allow_mock_fallback(false);
            Ok((ProviderConfig::Local(local), None))
        }
        "openai" => {
            let model = parse_openai_model(&settings.model)?;
            let openai = openai_config(model, settings.dimension)?;
            let api_key = get_api_key(config)?;
            Ok((ProviderConfig::OpenAI(openai), Some(api_key)))
        }
        "mistral" => {
            let model = parse_mistral_model(&settings.model)?;
            let mut mistral = MistralConfig::new(model);
            if settings.dimension != model.default_dimension() {
                if !model.supports_output_dimension() {
                    anyhow::bail!(
                        "Model '{}' cannot produce {} dimensions (fixed at {})",
                        settings.model,
                        settings.dimension,
                        model.default_dimension()
                    );
                }
                if settings.dimension > model.max_dimension() {
                    anyhow::bail!(
                        "Dimension {} exceeds the maximum {} for model '{}'",
                        settings.dimension,
                        model.max_dimension(),
                        settings.model
                    );
                }
                mistral = mistral.with_output_dimension(settings.dimension);
            }
            let api_key = get_api_key(config)?;
            Ok((ProviderConfig::Mistral(mistral), Some(api_key)))
        }
        "custom" => {
            let base_url = settings
                .base_url
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("base_url required for custom provider"))?;
            let custom = CustomConfig::new(settings.model.as_str(), settings.dimension, base_url);
            let api_key = get_api_key(config).ok();
            Ok((ProviderConfig::Custom(custom), api_key))
        }
        "azure" => anyhow::bail!(
            "Azure OpenAI transport is currently unsupported due to URL query formatting limitations; use custom with an OpenAI-compatible endpoint"
        ),
        other => anyhow::bail!(
            "Unknown provider: {}. Valid providers: local, openai, mistral, custom",
            other
        ),
    }
}

/// Parse a configured OpenAI model name into the typed model enum.
fn parse_openai_model(name: &str) -> Result<OpenAIModel> {
    match name {
        "text-embedding-ada-002" => Ok(OpenAIModel::Ada002),
        "text-embedding-3-small" => Ok(OpenAIModel::TextEmbedding3Small),
        "text-embedding-3-large" => Ok(OpenAIModel::TextEmbedding3Large),
        _ => anyhow::bail!(
            "Unknown OpenAI embedding model '{}'. Valid models: text-embedding-ada-002, text-embedding-3-small, text-embedding-3-large",
            name
        ),
    }
}

/// Parse a configured Mistral model name into the typed model enum.
fn parse_mistral_model(name: &str) -> Result<MistralModel> {
    match name {
        "mistral-embed" => Ok(MistralModel::MistralEmbed),
        "codestral-embed" => Ok(MistralModel::CodestralEmbed),
        _ => anyhow::bail!(
            "Unknown Mistral embedding model '{}'. Valid models: mistral-embed, codestral-embed",
            name
        ),
    }
}

/// Build an OpenAI config for `model`, requesting `dimension` when it differs
/// from the model default.
fn openai_config(model: OpenAIModel, dimension: usize) -> Result<OpenAIConfig> {
    if dimension > model.max_dimension() {
        anyhow::bail!(
            "Dimension {} exceeds the maximum {} for model '{}'",
            dimension,
            model.max_dimension(),
            model.model_name()
        );
    }
    let config = OpenAIConfig::new(model);
    if dimension == model.default_dimension() {
        return Ok(config);
    }
    if !model.supports_dimensions() {
        anyhow::bail!(
            "Model '{}' cannot produce {} dimensions (fixed at {})",
            model.model_name(),
            dimension,
            model.default_dimension()
        );
    }
    Ok(config.with_dimensions(dimension))
}

/// Create embedding provider from configuration
pub(crate) async fn create_provider_from_config(
    config: &Config,
) -> Result<Box<dyn EmbeddingProvider>> {
    let (provider_config, api_key) = map_provider_config(config)?;

    match provider_config {
        ProviderConfig::Local(local) => {
            #[cfg(feature = "local-embeddings")]
            {
                use do_memory_core::embeddings::LocalEmbeddingProvider;
                let provider = LocalEmbeddingProvider::new(local).await?;
                Ok(Box::new(provider))
            }
            #[cfg(not(feature = "local-embeddings"))]
            {
                let _ = local;
                Err(anyhow::anyhow!(
                    "Local embeddings not available. Compile with --features local-embeddings"
                ))
            }
        }
        ProviderConfig::OpenAI(openai) => {
            #[cfg(feature = "openai")]
            {
                use do_memory_core::embeddings::OpenAIEmbeddingProvider;
                let api_key =
                    api_key.ok_or_else(|| anyhow::anyhow!("OpenAI API key not resolved"))?;
                let provider = OpenAIEmbeddingProvider::new(api_key, openai)?;
                Ok(Box::new(provider))
            }
            #[cfg(not(feature = "openai"))]
            {
                let _ = (openai, api_key);
                Err(anyhow::anyhow!(
                    "OpenAI embeddings not available. Compile with --features openai"
                ))
            }
        }
        ProviderConfig::Mistral(mistral) => {
            #[cfg(feature = "mistral")]
            {
                use do_memory_core::embeddings::MistralEmbeddingProvider;
                let api_key =
                    api_key.ok_or_else(|| anyhow::anyhow!("Mistral API key not resolved"))?;
                let provider = MistralEmbeddingProvider::new(api_key, mistral)?;
                Ok(Box::new(provider))
            }
            #[cfg(not(feature = "mistral"))]
            {
                let _ = (mistral, api_key);
                Err(anyhow::anyhow!(
                    "Mistral embeddings not available. Compile with --features mistral"
                ))
            }
        }
        ProviderConfig::Custom(custom) => {
            #[cfg(feature = "openai")]
            {
                use do_memory_core::embeddings::OpenAIEmbeddingProvider;
                // Custom endpoints speak the OpenAI embeddings wire format.
                let model = parse_openai_model(&custom.model_name)?;
                let openai = openai_config(model, custom.embedding_dimension)?
                    .with_base_url(custom.base_url);
                let api_key = api_key.unwrap_or_else(|| "not-needed".to_string());
                let provider = OpenAIEmbeddingProvider::new(api_key, openai)?;
                Ok(Box::new(provider))
            }
            #[cfg(not(feature = "openai"))]
            {
                let _ = (custom, api_key);
                Err(anyhow::anyhow!(
                    "Custom embeddings not available. Compile with --features openai"
                ))
            }
        }
        ProviderConfig::AzureOpenAI(_) => Err(anyhow::anyhow!(
            "Azure OpenAI transport is currently unsupported due to URL query formatting limitations; use custom with an OpenAI-compatible endpoint"
        )),
    }
}

/// Get API key from environment variable specified in config
pub fn get_api_key(config: &Config) -> Result<String> {
    get_api_key_with(config, |name| env::var(name))
}

/// Resolve the API key using a pluggable lookup (production uses [`env::var`]).
///
/// Injected lookup keeps unit tests free of process-wide env mutation / `unsafe`.
fn get_api_key_with<F>(config: &Config, mut lookup: F) -> Result<String>
where
    F: FnMut(&str) -> std::result::Result<String, std::env::VarError>,
{
    let env_var = config
        .embeddings
        .api_key_env
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("api_key_env not configured"))?;

    lookup(env_var).map_err(|_| {
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
    fn get_api_key_reads_via_injected_lookup() {
        let config = base_config("openai", Some("MY_KEY_ENV".to_string()));
        let key = get_api_key_with(&config, |name| {
            assert_eq!(name, "MY_KEY_ENV");
            Ok("test-secret-key".to_string())
        })
        .expect("key present");
        assert_eq!(key, "test-secret-key");
    }

    #[test]
    fn get_api_key_with_missing_lookup_value() {
        let config = base_config("openai", Some("MISSING_VAR".to_string()));
        let err = get_api_key_with(&config, |_| Err(std::env::VarError::NotPresent))
            .expect_err("missing");
        assert!(err.to_string().contains("MISSING_VAR"));
        assert!(err.to_string().contains("not set"));
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

    #[cfg(not(feature = "local-embeddings"))]
    #[tokio::test]
    async fn create_provider_local_requires_feature() {
        let config = base_config("local", None);
        assert_provider_err(
            create_provider_from_config(&config).await,
            &["Local embeddings not available"],
        );
    }

    #[test]
    fn map_provider_config_honours_model_and_dimension() {
        let mut config = base_config("custom", None);
        config.embeddings.base_url = Some("http://127.0.0.1:9/v1".to_string());
        config.embeddings.dimension = 3;

        let (mapped, credential) = map_provider_config(&config).expect("map custom");
        assert_eq!(mapped.model_name(), "text-embedding-3-small");
        assert_eq!(mapped.effective_dimension(), 3);
        assert_eq!(mapped.cache_identity(), "custom:text-embedding-3-small:3");
        assert!(credential.is_none());
    }

    #[test]
    fn map_provider_config_rejects_azure_transport() {
        let config = base_config("azure", None);
        let err = map_provider_config(&config).expect_err("azure unsupported");
        assert!(
            err.to_string()
                .contains("Azure OpenAI transport is currently unsupported"),
            "unexpected error: {err}"
        );
    }
}

//! Configure embeddings tool implementation.
//!
//! REA-2026-07-26-A5: This is the MCP activation entry point.  A success
//! response now means the provider is live in the process; subsequent
//! status/generate/query calls will use it.

use std::sync::Arc;

use super::super::definitions::EmbeddingTools;
use crate::mcp::tools::embeddings::types::{ConfigureEmbeddingsInput, ConfigureEmbeddingsOutput};
use anyhow::{Result, anyhow};
use do_memory_core::embeddings::{
    EmbeddingConfig, InMemoryEmbeddingStorage, SemanticService,
    config::{LocalConfig, ProviderConfig},
};
use tracing::{debug, info, instrument, warn};

impl EmbeddingTools {
    /// Execute the `configure_embeddings` tool.
    ///
    /// Parses and validates the request, resolves the credential, constructs
    /// the exact provider (no fallback), probes it, then atomically installs
    /// it into the memory runtime via
    /// `do_memory_core::memory::SelfLearningMemory::activate_semantic_service`.
    ///
    /// On failure the previous active provider is preserved unchanged.
    #[instrument(skip(self, input), fields(provider = %input.provider))]
    pub async fn execute_configure_embeddings(
        &self,
        input: ConfigureEmbeddingsInput,
    ) -> Result<ConfigureEmbeddingsOutput> {
        info!("Configuring embedding provider: {}", input.provider);

        let mut warnings = Vec::new();

        // ── Step 1: parse and validate provider ──────────────────────────────
        // Only local, openai, and mistral are selectable.  Azure/Custom/Cohere
        // are rejected before any credential or network work is attempted.
        let provider_config = parse_provider_config(&input, &mut warnings)?;

        // ── Step 2: resolve credential from the named env var ────────────────
        let api_key = resolve_api_key(&provider_config, &input.api_key_env, &mut warnings)?;

        // ── Step 3: build the exact provider, probe it, validate dimension ───
        // InMemoryEmbeddingStorage is used when no persistent storage backend
        // is wired.  A future ADR may add storage composition here.
        let storage = Box::new(InMemoryEmbeddingStorage::new());
        let embedding_config = EmbeddingConfig {
            provider: provider_config.clone(),
            similarity_threshold: input.similarity_threshold.unwrap_or(0.7),
            batch_size: input.batch_size.unwrap_or(32),
            cache_embeddings: true,
            timeout_seconds: 30,
        };

        let service = match SemanticService::build_exact(
            &provider_config,
            api_key,
            storage,
            embedding_config,
        )
        .await
        {
            Ok(svc) => svc,
            Err(e) => {
                warn!("Embedding provider build/probe failed: {}", e);
                return Err(anyhow!("Embedding provider activation failed: {}", e));
            }
        };

        // ── Step 4: atomically install into runtime ───────────────────────────
        let provider_identity = provider_config.cache_identity();
        let prev = self
            .memory
            .activate_semantic_service(Arc::new(service), provider_identity.clone())
            .await;

        // ── Step 5: read back the installed activation for the response ───────
        let activation = self
            .memory
            .embedding_activation()
            .await
            .expect("activation must be set immediately after activate_semantic_service");

        debug!(
            revision = activation.revision,
            identity = %activation.provider_identity,
            reindex = activation.reindex_required,
            "Embedding provider activated"
        );

        // If the provider identity changed, drop the old service explicitly so
        // its resources are freed before we return.
        drop(prev);

        let provider_name = input.provider.clone();
        Ok(ConfigureEmbeddingsOutput {
            success: true,
            provider: input.provider,
            model: provider_config.model_name(),
            dimension: provider_config.effective_dimension(),
            message: format!(
                "Activated {} provider with model {} (dimension: {}, revision: {})",
                provider_name,
                provider_config.model_name(),
                provider_config.effective_dimension(),
                activation.revision,
            ),
            warnings,
            activation_revision: Some(activation.revision),
            reindex_required: activation.reindex_required,
            provider_health: "active".to_string(),
        })
    }
}

/// Parse the provider field and build the matching `ProviderConfig`.
fn parse_provider_config(
    input: &ConfigureEmbeddingsInput,
    warnings: &mut Vec<String>,
) -> Result<ProviderConfig> {
    match input.provider.to_lowercase().as_str() {
        "openai" => {
            let model_name = input.model.as_deref().unwrap_or("text-embedding-3-small");
            let config = match model_name {
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
            };
            Ok(config)
        }

        "mistral" => {
            let model_name = input.model.as_deref().unwrap_or("mistral-embed");
            if model_name != "mistral-embed" {
                warnings.push(format!(
                    "Unknown Mistral model '{}', using mistral-embed",
                    model_name
                ));
            }
            Ok(ProviderConfig::mistral_embed())
        }

        "local" => {
            let model_name = input
                .model
                .as_deref()
                .unwrap_or("sentence-transformers/all-MiniLM-L6-v2");
            Ok(ProviderConfig::Local(LocalConfig::new(model_name, 384)))
        }

        "azure" | "azure_openai" => Err(anyhow!(
            "Azure provider is not supported. Supported providers: openai, local, mistral."
        )),
        "custom" => Err(anyhow!(
            "Custom provider is not supported. Supported providers: openai, local, mistral."
        )),
        "cohere" => Err(anyhow!(
            "Cohere provider is not implemented. Supported providers: openai, local, mistral."
        )),
        _ => Err(anyhow!(
            "Unsupported provider: {}. Supported providers: openai, local, mistral",
            input.provider
        )),
    }
}

/// Resolve the API key for cloud providers.
///
/// Returns `None` for Local (no key needed).  For cloud providers reads the
/// named environment variable when supplied; warns (but continues) when no
/// `api_key_env` is given so the caller can supply the key via a standard env
/// var that `build_exact` will reject if absent.
fn resolve_api_key(
    provider_config: &ProviderConfig,
    api_key_env: &Option<String>,
    warnings: &mut Vec<String>,
) -> Result<Option<String>> {
    match provider_config {
        ProviderConfig::Local(_) => Ok(None),

        ProviderConfig::OpenAI(_) | ProviderConfig::Mistral(_) => {
            match api_key_env {
                Some(env_var) => match std::env::var(env_var) {
                    Ok(key) => Ok(Some(key)),
                    Err(_) => Err(anyhow!(
                        "Environment variable '{}' not set. Please set the API key.",
                        env_var
                    )),
                },
                None => {
                    // No explicit env var — pass None; build_exact will reject if no
                    // key is available through the standard variable name.
                    warnings.push(format!(
                        "No api_key_env specified for {}. The provider will attempt to use its \
                         default credential environment variable.",
                        provider_config.model_name()
                    ));
                    Ok(None)
                }
            }
        }

        // AzureOpenAI and Custom are rejected before this function is reached.
        ProviderConfig::AzureOpenAI(_) | ProviderConfig::Custom(_) => Err(anyhow!(
            "Internal error: unsupported provider reached resolve_api_key"
        )),
    }
}

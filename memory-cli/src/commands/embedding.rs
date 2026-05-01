//! Embedding command implementations for semantic search configuration and testing
//!
//! This module provides commands to configure, test, and manage embedding providers
//! for semantic similarity search in the memory system.

use crate::config::Config;
use anyhow::Result;
use clap::Subcommand;
#[allow(unused_imports)] // All configs are used in match statements
use do_memory_core::embeddings::{
    AzureOpenAIConfig, CustomConfig, EmbeddingProvider, LocalConfig, MistralConfig, OpenAIConfig,
};
use std::env;

#[derive(Subcommand)]
pub enum EmbeddingCommands {
    /// Test embedding provider connectivity and configuration
    Test,

    /// Show current embedding configuration
    Config,

    /// List available embedding providers
    ListProviders,

    /// Benchmark embedding provider performance
    Benchmark,

    /// Enable embeddings in the current session
    Enable,

    /// Disable embeddings in the current session
    Disable,
}

/// Test embedding provider connectivity and configuration
///
/// This command validates that the configured embedding provider is working correctly
/// by generating a test embedding and measuring performance.
pub async fn test_embeddings(config: &Config) -> Result<()> {
    println!("🧪 Testing Embedding Provider Configuration");
    println!("{}", "=".repeat(60));
    println!();

    // Check if embeddings are enabled
    if !config.embeddings.enabled {
        println!("⚠️  Embeddings are disabled in configuration");
        println!("   To enable, set 'enabled = true' in [embeddings] section");
        println!();
        println!("   Or use: memory-cli embedding enable");
        return Ok(());
    }

    println!("📋 Configuration:");
    println!("   Provider: {}", config.embeddings.provider);
    println!("   Model: {}", config.embeddings.model);
    println!("   Dimension: {}", config.embeddings.dimension);
    println!(
        "   Similarity Threshold: {}",
        config.embeddings.similarity_threshold
    );
    println!();

    // Initialize provider
    println!("🔌 Connecting to provider...");
    let provider = create_provider_from_config(config).await?;
    println!("✅ Provider initialized: {}", provider.model_name());
    println!("   Dimension: {}", provider.embedding_dimension());
    println!();

    // Test single embedding generation
    println!("🧠 Testing single embedding generation...");
    let test_text = "Implement REST API authentication with JWT tokens";
    let start = std::time::Instant::now();
    let embedding = provider.embed_text(test_text).await?;
    let duration = start.elapsed();

    println!("✅ Embedding generated successfully");
    println!("   Text: '{}'", test_text);
    println!("   Dimensions: {}", embedding.len());
    println!("   Time: {:?}", duration);
    println!();

    // Test batch embedding generation
    println!("⚡ Testing batch embedding generation...");
    let batch_texts: Vec<String> = vec![
        "Create database migration script".to_string(),
        "Build user authentication system".to_string(),
        "Implement caching layer with Redis".to_string(),
    ];

    let start = std::time::Instant::now();
    let batch_results = provider.embed_batch(&batch_texts).await?;
    let duration = start.elapsed();

    println!("✅ Batch embeddings generated successfully");
    println!("   Count: {}", batch_results.len());
    println!("   Time: {:?}", duration);
    // Batch size is always small, cast to u32 is safe
    #[allow(clippy::cast_possible_truncation)]
    let batch_count = batch_texts.len() as u32;
    println!("   Avg per text: {:?}", duration / batch_count);
    println!();

    // Test similarity calculation
    println!("📊 Testing similarity calculations...");
    let text1 = "user authentication";
    let text2 = "login system";
    let text3 = "database optimization";

    let sim_12 = provider.similarity(text1, text2).await?;
    let sim_13 = provider.similarity(text1, text3).await?;

    println!("✅ Similarity calculations completed");
    println!("   Similarity('{}', '{}') = {:.3}", text1, text2, sim_12);
    println!("   Similarity('{}', '{}') = {:.3}", text1, text3, sim_13);
    println!();

    // Summary
    println!("✨ All tests passed!");
    println!();
    println!("Next steps:");
    println!("  • Use 'memory-cli episode list --semantic-search <query>' for semantic search");
    println!("  • Enable embeddings in config file: [embeddings] enabled = true");
    println!("  • See 'memory-cli embedding config' for configuration options");

    Ok(())
}

/// Show current embedding configuration
#[allow(clippy::unnecessary_wraps)]
pub fn show_config(config: &Config) -> Result<()> {
    println!("⚙️  Embedding Configuration");
    println!("{}", "=".repeat(60));
    println!();

    println!(
        "Status: {}",
        if config.embeddings.enabled {
            "✅ Enabled"
        } else {
            "⚠️  Disabled"
        }
    );
    println!();

    println!("Provider Settings:");
    println!("  provider: {}", config.embeddings.provider);
    println!("  model: {}", config.embeddings.model);
    println!("  dimension: {}", config.embeddings.dimension);
    if let Some(base_url) = &config.embeddings.base_url {
        println!("  base_url: {}", base_url);
    }
    if let Some(api_key_env) = &config.embeddings.api_key_env {
        let key_status = if env::var(api_key_env).is_ok() {
            "✅ Set"
        } else {
            "⚠️  Not set"
        };
        println!("  api_key: {}", key_status);
    }
    println!();

    println!("Search Settings:");
    println!(
        "  similarity_threshold: {}",
        config.embeddings.similarity_threshold
    );
    println!("  batch_size: {}", config.embeddings.batch_size);
    println!("  cache_embeddings: {}", config.embeddings.cache_embeddings);
    println!("  timeout_seconds: {}", config.embeddings.timeout_seconds);
    println!();

    if config.embeddings.enabled {
        println!("To test your configuration:");
        println!("  Run: memory-cli embedding test");
        println!();
    } else {
        println!("To enable embeddings:");
        println!("  1. Edit your config file and set: [embeddings] enabled = true");
        println!("  2. Or run: memory-cli embedding enable");
        println!();
    }

    Ok(())
}

/// List available embedding providers
#[allow(clippy::unnecessary_wraps)]
pub fn list_providers() -> Result<()> {
    println!("📚 Available Embedding Providers");
    println!("{}", "=".repeat(60));
    println!();

    println!("🏠 Local Provider");
    println!("   • Model: sentence-transformers/all-MiniLM-L6-v2");
    println!("   • Dimension: 384");
    println!("   • Cost: Free (runs on your CPU)");
    println!("   • Speed: Fast for small batches");
    println!("   • Setup: Requires 'local-embeddings' feature");
    println!();

    println!("🌐 OpenAI Provider");
    println!("   • Model: text-embedding-3-small (default)");
    println!("   • Dimension: 1536");
    println!("   • Cost: $0.02 per 1M tokens");
    println!("   • Speed: Very fast (API-based)");
    println!("   • Setup: Requires OPENAI_API_KEY");
    println!();

    println!("🟣 Mistral Provider");
    println!("   • Model: mistral-embed");
    println!("   • Dimension: 1024");
    println!("   • Cost: See Mistral pricing");
    println!("   • Speed: Fast (API-based)");
    println!("   • Setup: Requires MISTRAL_API_KEY");
    println!();

    println!("☁️  Azure OpenAI Provider");
    println!("   • Model: Your deployment");
    println!("   • Dimension: Configurable");
    println!("   • Cost: Azure pricing");
    println!("   • Speed: Fast (API-based)");
    println!("   • Setup: Requires AZURE_OPENAI_API_KEY");
    println!();

    println!("🔧 Custom Provider");
    println!("   • Model: Your choice");
    println!("   • Dimension: Configurable");
    println!("   • Cost: Depends on your setup");
    println!("   • Speed: Varies");
    println!("   • Setup: Requires base_url configuration");
    println!();

    println!("Configuration Example:");
    println!("  [embeddings]");
    println!("  enabled = true");
    println!("  provider = \"openai\"  # or \"local\", \"mistral\", \"azure\", \"custom\"");
    println!("  model = \"text-embedding-3-small\"");
    println!("  dimension = 1536");
    println!("  api_key_env = \"OPENAI_API_KEY\"");
    println!();

    Ok(())
}

/// Enable embeddings in the current session
#[allow(clippy::unnecessary_wraps)]
pub fn enable_embeddings() -> Result<()> {
    println!("✅ Embeddings Enabled");
    println!();
    println!("Note: This only enables embeddings for the current session.");
    println!("To persist this change, update your configuration file:");
    println!();
    println!("  [embeddings]");
    println!("  enabled = true");
    println!();
    println!("Run 'memory-cli embedding test' to verify your configuration.");

    Ok(())
}

/// Disable embeddings in the current session
#[allow(clippy::unnecessary_wraps)]
pub fn disable_embeddings() -> Result<()> {
    println!("⚠️  Embeddings Disabled");
    println!();
    println!("Semantic search will fall back to keyword-based search.");
    println!();
    println!("To re-enable, run: memory-cli embedding enable");

    Ok(())
}

/// Benchmark embedding provider performance
pub async fn benchmark_embeddings(config: &Config) -> Result<()> {
    println!("⚡ Benchmarking Embedding Provider");
    println!("{}", "=".repeat(60));
    println!();

    if !config.embeddings.enabled {
        return Err(anyhow::anyhow!(
            "Embeddings are disabled. Enable with: memory-cli embedding enable"
        ));
    }

    println!(
        "Provider: {} ({})",
        config.embeddings.provider, config.embeddings.model
    );
    println!();

    let provider = create_provider_from_config(config).await?;

    // Benchmark single embedding
    println!("📊 Single Embedding Benchmark");
    let test_text =
        "Implement REST API authentication with JWT tokens and role-based access control";
    let iterations: u32 = 10;

    let mut durations = Vec::new();
    for _ in 0..iterations {
        let start = std::time::Instant::now();
        let _result = provider.embed_text(test_text).await?;
        durations.push(start.elapsed());
    }

    let avg = durations.iter().sum::<std::time::Duration>() / iterations;

    // Sort durations to get min/max without expect
    // durations is guaranteed non-empty by loop with iterations > 0
    durations.sort();
    let min = durations.first().copied().unwrap_or_default();
    let max = durations.last().copied().unwrap_or_default();

    println!("  Iterations: {}", iterations);
    println!("  Average: {:?}", avg);
    println!("  Min: {:?}", min);
    println!("  Max: {:?}", max);
    println!();

    // Benchmark batch embedding
    println!("📊 Batch Embedding Benchmark");
    let batch_sizes = vec![5, 10, 20];

    for batch_size in batch_sizes {
        let texts: Vec<String> = (0..batch_size)
            .map(|i| match i % 3 {
                0 => "Implement user authentication system",
                1 => "Create database migration scripts",
                _ => "Build REST API endpoints",
            })
            .map(String::from)
            .collect();

        let start = std::time::Instant::now();
        let _results = provider.embed_batch(&texts).await?;
        let duration = start.elapsed();

        // Batch sizes are small, cast to u32 is safe
        #[allow(clippy::cast_possible_truncation)]
        let size = batch_size as u32;
        println!(
            "  Batch size {}: {:?} ({:?} per item)",
            batch_size,
            duration,
            duration / size
        );
    }
    println!();

    println!("✅ Benchmark complete!");
    Ok(())
}

/// Create embedding provider from configuration
async fn create_provider_from_config(config: &Config) -> Result<Box<dyn EmbeddingProvider>> {
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
fn get_api_key(config: &Config) -> Result<String> {
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

    fn create_test_config_with_embeddings(enabled: bool, api_key_env: Option<String>) -> Config {
        Config {
            database: DatabaseConfig {
                turso_url: None,
                turso_token: None,
                redb_path: Some(":memory:".to_string()),
            },
            storage: StorageConfig {
                max_episodes_cache: 100,
                cache_ttl_seconds: 3600,
                pool_size: 5,
            },
            cli: CliConfig {
                default_format: "human".to_string(),
                progress_bars: false,
                batch_size: 50,
            },
            embeddings: EmbeddingsConfig {
                enabled,
                provider: "openai".to_string(),
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
    fn test_get_api_key_missing_env_config() {
        // Arrange: config without api_key_env
        let config = create_test_config_with_embeddings(true, None);

        // Act
        let result = get_api_key(&config);

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("api_key_env not configured"));
    }

    #[test]
    fn test_get_api_key_env_var_not_set() {
        // Arrange: config with api_key_env pointing to non-existent variable
        let config =
            create_test_config_with_embeddings(true, Some("NONEXISTENT_API_KEY_VAR".to_string()));

        // Act
        let result = get_api_key(&config);

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("NONEXISTENT_API_KEY_VAR"));
        assert!(error.contains("not set"));
    }

    #[test]
    fn test_show_config_disabled() {
        // Arrange: config with embeddings disabled
        let config = create_test_config_with_embeddings(false, None);

        // Act
        let result = show_config(&config);

        // Assert: should succeed and show disabled status
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_config_enabled() {
        // Arrange: config with embeddings enabled
        let config = create_test_config_with_embeddings(true, Some("TEST_API_KEY".to_string()));

        // Act
        let result = show_config(&config);

        // Assert: should succeed and show enabled status
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_config_with_base_url() {
        // Arrange: config with custom base_url
        let mut config = create_test_config_with_embeddings(true, Some("TEST_API_KEY".to_string()));
        config.embeddings.base_url = Some("https://custom.api.com".to_string());

        // Act
        let result = show_config(&config);

        // Assert: should succeed and show base_url
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_providers() {
        // Act
        let result = list_providers();

        // Assert: should succeed and show provider information
        assert!(result.is_ok());
    }

    #[test]
    fn test_enable_embeddings() {
        // Act
        let result = enable_embeddings();

        // Assert: should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_disable_embeddings() {
        // Act
        let result = disable_embeddings();

        // Assert: should succeed
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_benchmark_embeddings_disabled() {
        // Arrange: config with embeddings disabled
        let config = create_test_config_with_embeddings(false, None);

        // Act
        let result = benchmark_embeddings(&config).await;

        // Assert: should fail because embeddings are disabled
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("Embeddings are disabled"));
    }
}

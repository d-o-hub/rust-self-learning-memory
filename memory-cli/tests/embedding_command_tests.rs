//! Unit tests for embedding command functions
//!
//! Tests for embedding configuration, provider listing, and benchmarking.

use do_memory_cli::commands::embedding::{
    benchmark_embeddings, disable_embeddings, enable_embeddings, get_api_key, list_providers,
    show_config,
};
use do_memory_cli::config::types::{
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

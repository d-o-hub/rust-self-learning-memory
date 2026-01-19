//! Optimization config helper method tests

use crate::embeddings::config::{ModelConfig, OptimizationConfig};

#[test]
fn test_optimization_config_helper_methods() {
    // Test get_timeout_seconds with None (should return default)
    let mut config = OptimizationConfig::default();
    assert_eq!(config.timeout_seconds, None);
    assert_eq!(config.get_timeout_seconds(), 60); // Default is 60

    // Test get_timeout_seconds with Some value
    config.timeout_seconds = Some(120);
    assert_eq!(config.get_timeout_seconds(), 120);

    // Test get_max_batch_size with None (should return default)
    config.max_batch_size = None;
    assert_eq!(config.get_max_batch_size(), 100); // Default is 100

    // Test get_max_batch_size with Some value
    config.max_batch_size = Some(256);
    assert_eq!(config.get_max_batch_size(), 256);

    // Test OpenAI profile helpers
    let openai_config = OptimizationConfig::openai();
    assert_eq!(openai_config.get_timeout_seconds(), 60);
    assert_eq!(openai_config.get_max_batch_size(), 2048);

    // Test local profile helpers
    let local_config = OptimizationConfig::local();
    assert_eq!(local_config.get_timeout_seconds(), 10);
    assert_eq!(local_config.get_max_batch_size(), 32);
}

#[test]
fn test_model_config_get_embeddings_url() {
    // Test default (no base_url, no api_endpoint)
    let config = ModelConfig::default();
    let url = config.get_embeddings_url();
    assert_eq!(url, "https://api.openai.com/v1/embeddings");

    // Test with base_url only
    let config = ModelConfig {
        base_url: Some("https://custom.api.com/v1".to_string()),
        api_endpoint: None,
        ..Default::default()
    };
    let url = config.get_embeddings_url();
    assert_eq!(url, "https://custom.api.com/v1/embeddings");

    // Test with both base_url and api_endpoint (with leading slash)
    let config = ModelConfig {
        base_url: Some("https://custom.api.com/v1".to_string()),
        api_endpoint: Some("/custom-path".to_string()),
        ..Default::default()
    };
    let url = config.get_embeddings_url();
    assert_eq!(url, "https://custom.api.com/v1/custom-path");

    // Test with both base_url and api_endpoint (without leading slash)
    let config = ModelConfig {
        base_url: Some("https://custom.api.com/v1".to_string()),
        api_endpoint: Some("custom-path".to_string()),
        ..Default::default()
    };
    let url = config.get_embeddings_url();
    assert_eq!(url, "https://custom.api.com/v1/custom-path");
}

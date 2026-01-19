//! Optimization config serialization tests

use crate::embeddings::config::OptimizationConfig;

#[test]
fn test_optimization_config_serialization() {
    // Test that default config serializes/deserializes correctly
    let config = OptimizationConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: OptimizationConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.max_retries, config.max_retries);
    assert_eq!(deserialized.retry_delay_ms, config.retry_delay_ms);
    assert_eq!(deserialized.compression_enabled, config.compression_enabled);

    // Test that openai profile serializes/deserializes correctly
    let openai_config = OptimizationConfig::openai();
    let json = serde_json::to_string(&openai_config).unwrap();
    let deserialized: OptimizationConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.timeout_seconds, openai_config.timeout_seconds);
    assert_eq!(deserialized.max_batch_size, openai_config.max_batch_size);
    assert_eq!(
        deserialized.enable_circuit_breaker,
        openai_config.enable_circuit_breaker
    );
}

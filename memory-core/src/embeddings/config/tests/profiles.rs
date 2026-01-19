//! Optimization config profile tests

use crate::embeddings::config::OptimizationConfig;

#[test]
fn test_optimization_config_profiles() {
    // Test OpenAI profile
    let openai_profile = OptimizationConfig::openai();
    assert_eq!(openai_profile.timeout_seconds, Some(60));
    assert_eq!(openai_profile.max_retries, 3);
    assert_eq!(openai_profile.max_batch_size, Some(2048));
    assert_eq!(openai_profile.rate_limit_rpm, Some(3000));
    assert!(openai_profile.compression_enabled);
    assert_eq!(openai_profile.connection_pool_size, 20);
    assert!(openai_profile.enable_circuit_breaker);
    assert!(openai_profile.circuit_breaker_config.is_some());
    assert!(openai_profile.enable_metrics);

    // Test Mistral profile
    let mistral_profile = OptimizationConfig::mistral();
    assert_eq!(mistral_profile.timeout_seconds, Some(30));
    assert_eq!(mistral_profile.max_retries, 3);
    assert_eq!(mistral_profile.max_batch_size, Some(128));
    assert_eq!(mistral_profile.retry_delay_ms, 500);
    assert_eq!(mistral_profile.rate_limit_rpm, Some(100));

    // Test Azure profile
    let azure_profile = OptimizationConfig::azure();
    assert_eq!(azure_profile.timeout_seconds, Some(90));
    assert_eq!(azure_profile.max_retries, 4);
    assert_eq!(azure_profile.retry_delay_ms, 2000);
    assert_eq!(azure_profile.rate_limit_rpm, Some(300));

    // Test local profile
    let local_profile = OptimizationConfig::local();
    assert_eq!(local_profile.timeout_seconds, Some(10));
    assert_eq!(local_profile.max_retries, 2);
    assert_eq!(local_profile.max_batch_size, Some(32));
    assert!(!local_profile.compression_enabled);
    assert!(!local_profile.enable_circuit_breaker);
    assert!(local_profile.circuit_breaker_config.is_none());
    assert_eq!(local_profile.connection_pool_size, 5);
    assert!(local_profile.enable_metrics);
}

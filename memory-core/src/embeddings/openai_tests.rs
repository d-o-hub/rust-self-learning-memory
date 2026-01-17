//! Tests for OpenAI embedding provider

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_api_key() {
        // Valid key format
        assert!(utils::validate_api_key("sk-1234567890abcdefghijklmnop").is_ok());

        // Invalid formats
        assert!(utils::validate_api_key("").is_err());
        assert!(utils::validate_api_key("invalid-key").is_err());
        assert!(utils::validate_api_key("sk-short").is_err());
    }

    #[test]
    fn test_estimate_cost() {
        // Test cost calculation for different models
        let tokens = 1000;

        let ada_cost = utils::estimate_cost(tokens, "text-embedding-ada-002");
        let small_cost = utils::estimate_cost(tokens, "text-embedding-3-small");
        let large_cost = utils::estimate_cost(tokens, "text-embedding-3-large");

        assert!(ada_cost > 0.0);
        assert!(small_cost > 0.0);
        assert!(large_cost > 0.0);

        // 3-small should be cheaper than ada-002
        assert!(small_cost < ada_cost);

        // 3-large should be most expensive
        assert!(large_cost > small_cost);
        assert!(large_cost > ada_cost);
    }

    #[test]
    fn test_estimate_tokens() {
        let text = "Hello world, this is a test sentence.";
        let tokens = utils::estimate_tokens(text);

        assert!(tokens > 0);
        assert!(tokens < text.len()); // Should be less than character count
    }

    #[test]
    fn test_recommended_models() {
        let balanced = utils::get_recommended_model(utils::OpenAIModelUseCase::Balanced);
        assert_eq!(balanced.model_name, "text-embedding-3-small");

        let quality = utils::get_recommended_model(utils::OpenAIModelUseCase::Quality);
        assert_eq!(quality.model_name, "text-embedding-3-large");

        let legacy = utils::get_recommended_model(utils::OpenAIModelUseCase::Legacy);
        assert_eq!(legacy.model_name, "text-embedding-ada-002");
    }

    #[cfg(feature = "openai")]
    #[tokio::test]
    async fn test_provider_creation() -> anyhow::Result<()> {
        let config = ModelConfig::openai_3_small();
        let provider = OpenAIEmbeddingProvider::new("sk-test-key-1234567890".to_string(), config)?;

        assert_eq!(provider.model_name(), "text-embedding-3-small");
        assert_eq!(provider.embedding_dimension(), 1536);
        assert_eq!(provider.base_url, "https://api.openai.com/v1");
        Ok(())
    }

    #[cfg(feature = "openai")]
    #[tokio::test]
    async fn test_custom_url_provider() -> anyhow::Result<()> {
        let config = ModelConfig::openai_3_small();
        let custom_url = "https://custom.openai.azure.com/v1";
        let provider = OpenAIEmbeddingProvider::with_custom_url(
            "sk-test-key-1234567890".to_string(),
            config,
            custom_url.to_string(),
        )?;

        assert_eq!(provider.base_url, custom_url);
        Ok(())
    }

    #[test]
    fn test_mistral_config() {
        let config = ModelConfig::mistral_embed();
        assert_eq!(config.model_name, "mistral-embed");
        assert_eq!(config.embedding_dimension, 1024);
        assert_eq!(
            config.base_url,
            Some("https://api.mistral.ai/v1".to_string())
        );
        assert_eq!(
            config.get_embeddings_url(),
            "https://api.mistral.ai/v1/embeddings"
        );
    }

    #[test]
    fn test_azure_openai_config() {
        let config = ModelConfig::azure_openai("my-deployment", "my-resource", "2023-05-15", 1536);
        assert_eq!(config.model_name, "my-deployment");
        assert_eq!(config.embedding_dimension, 1536);
        assert_eq!(
            config.base_url,
            Some("https://my-resource.openai.azure.com".to_string())
        );
        assert!(config.api_endpoint.is_some());
        assert!(config.get_embeddings_url().contains("my-deployment"));
        assert!(config.get_embeddings_url().contains("2023-05-15"));
    }

    #[test]
    fn test_custom_config() {
        let config = ModelConfig::custom(
            "custom-model",
            768,
            "https://api.example.com/v1",
            Some("/custom/embeddings"),
        );
        assert_eq!(config.model_name, "custom-model");
        assert_eq!(config.embedding_dimension, 768);
        assert_eq!(
            config.get_embeddings_url(),
            "https://api.example.com/v1/custom/embeddings"
        );
    }

    #[test]
    fn test_custom_config_default_endpoint() {
        let config = ModelConfig::custom("custom-model", 768, "https://api.example.com/v1", None);
        assert_eq!(
            config.get_embeddings_url(),
            "https://api.example.com/v1/embeddings"
        );
    }

    #[cfg(feature = "openai")]
    #[tokio::test]
    async fn test_mistral_provider_creation() -> anyhow::Result<()> {
        let config = ModelConfig::mistral_embed();
        let provider = OpenAIEmbeddingProvider::new("test-api-key".to_string(), config)?;

        assert_eq!(provider.model_name(), "mistral-embed");
        assert_eq!(provider.embedding_dimension(), 1024);
        assert_eq!(provider.base_url, "https://api.mistral.ai/v1");
        Ok(())
    }

    #[test]
    fn test_optimization_config_defaults() {
        use super::super::config::OptimizationConfig;

        let config = OptimizationConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_ms, 1000);
        assert_eq!(config.get_timeout_seconds(), 60);
        assert_eq!(config.get_max_batch_size(), 100);
    }

    #[test]
    fn test_optimization_config_openai() {
        use super::super::config::OptimizationConfig;

        let config = OptimizationConfig::openai();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_ms, 1000);
        assert_eq!(config.timeout_seconds, Some(60));
        assert_eq!(config.max_batch_size, Some(2048));
        assert_eq!(config.rate_limit_rpm, Some(3000));
        assert_eq!(config.rate_limit_tpm, Some(1_000_000));
        assert!(config.compression_enabled);
        assert_eq!(config.connection_pool_size, 20);
    }

    #[test]
    fn test_optimization_config_mistral() {
        use super::super::config::OptimizationConfig;

        let config = OptimizationConfig::mistral();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_ms, 500);
        assert_eq!(config.timeout_seconds, Some(30));
        assert_eq!(config.max_batch_size, Some(128));
        assert_eq!(config.rate_limit_rpm, Some(100));
        assert!(config.compression_enabled);
        assert_eq!(config.connection_pool_size, 10);
    }

    #[test]
    fn test_optimization_config_azure() {
        use super::super::config::OptimizationConfig;

        let config = OptimizationConfig::azure();
        assert_eq!(config.max_retries, 4);
        assert_eq!(config.retry_delay_ms, 2000);
        assert_eq!(config.timeout_seconds, Some(90));
        assert_eq!(config.max_batch_size, Some(2048));
        assert_eq!(config.rate_limit_rpm, Some(300));
        assert!(config.compression_enabled);
        assert_eq!(config.connection_pool_size, 15);
    }

    #[test]
    fn test_optimization_config_local() {
        use super::super::config::OptimizationConfig;

        let config = OptimizationConfig::local();
        assert_eq!(config.max_retries, 2);
        assert_eq!(config.retry_delay_ms, 100);
        assert_eq!(config.timeout_seconds, Some(10));
        assert_eq!(config.max_batch_size, Some(32));
        assert_eq!(config.rate_limit_rpm, None); // No rate limiting for local
        assert!(!config.compression_enabled);
        assert_eq!(config.connection_pool_size, 5);
    }

    #[test]
    fn test_model_config_includes_optimization() {
        let config = ModelConfig::openai_3_small();
        assert_eq!(config.optimization.max_batch_size, Some(2048));
        assert_eq!(config.optimization.timeout_seconds, Some(60));

        let mistral_config = ModelConfig::mistral_embed();
        assert_eq!(mistral_config.optimization.max_batch_size, Some(128));
        assert_eq!(mistral_config.optimization.timeout_seconds, Some(30));

        let azure_config = ModelConfig::azure_openai("dep", "res", "2023-05-15", 1536);
        assert_eq!(azure_config.optimization.max_retries, 4);
        assert_eq!(azure_config.optimization.retry_delay_ms, 2000);
    }

    #[cfg(feature = "openai")]
    #[tokio::test]
    async fn test_provider_uses_optimization_timeout() -> anyhow::Result<()> {
        let mut config = ModelConfig::openai_3_small();
        config.optimization.timeout_seconds = Some(120);

        let provider = OpenAIEmbeddingProvider::new("sk-test-key".to_string(), config)?;

        // Verify the config has the custom timeout
        assert_eq!(provider.config.optimization.timeout_seconds, Some(120));
        Ok(())
    }

    #[cfg(feature = "openai")]
    #[tokio::test]
    async fn test_provider_uses_optimization_batch_size() -> anyhow::Result<()> {
        let mut config = ModelConfig::openai_3_small();
        config.optimization.max_batch_size = Some(500);

        let provider = OpenAIEmbeddingProvider::new("sk-test-key".to_string(), config)?;

        assert_eq!(provider.config.optimization.get_max_batch_size(), 500);
        Ok(())
    }
}

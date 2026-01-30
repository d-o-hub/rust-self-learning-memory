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
        use crate::embeddings::config::openai::OpenAIConfig;

        let config = OpenAIConfig::text_embedding_3_small();
        let provider = OpenAIEmbeddingProvider::new("sk-test-key-1234567890".to_string(), config)?;

        assert_eq!(provider.model_name(), "text-embedding-3-small");
        assert_eq!(provider.embedding_dimension(), 1536);

        // Verify base_url through metadata
        let metadata = provider.metadata();
        assert_eq!(metadata["base_url"], "https://api.openai.com/v1");
        Ok(())
    }

    #[cfg(feature = "openai")]
    #[tokio::test]
    async fn test_custom_url_provider() -> anyhow::Result<()> {
        use crate::embeddings::config::openai::{EncodingFormat, OpenAIConfig};

        let config = OpenAIConfig::text_embedding_3_small()
            .with_base_url("https://custom.openai.azure.com/v1")
            .with_encoding_format(EncodingFormat::Base64);
        let provider = OpenAIEmbeddingProvider::new("sk-test-key-1234567890".to_string(), config)?;

        // Verify custom URL through metadata
        let metadata = provider.metadata();
        assert_eq!(metadata["base_url"], "https://custom.openai.azure.com/v1");
        assert_eq!(metadata["encoding_format"], "base64");
        Ok(())
    }


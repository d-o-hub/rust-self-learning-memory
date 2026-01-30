//! Test OpenAI provider with new config structure

#[cfg(test)]
mod tests {
    use memory_core::embeddings::config::openai::{EncodingFormat, OpenAIConfig, OpenAIModel};

    #[test]
    fn test_openai_config_creation() {
        // Test config creation with various models
        let ada_config = OpenAIConfig::ada_002();
        assert_eq!(ada_config.model, OpenAIModel::Ada002);
        assert_eq!(ada_config.effective_dimension(), 1536);

        let small_config = OpenAIConfig::text_embedding_3_small();
        assert_eq!(small_config.model, OpenAIModel::TextEmbedding3Small);
        assert_eq!(small_config.effective_dimension(), 1536);

        let large_config = OpenAIConfig::text_embedding_3_large();
        assert_eq!(large_config.model, OpenAIModel::TextEmbedding3Large);
        assert_eq!(large_config.effective_dimension(), 3072);
    }

    #[test]
    fn test_openai_config_with_dimensions() {
        // Test text-embedding-3-small with custom dimensions
        let config = OpenAIConfig::text_embedding_3_small().with_dimensions(512);

        assert_eq!(config.model, OpenAIModel::TextEmbedding3Small);
        assert_eq!(config.dimensions, Some(512));
        assert_eq!(config.effective_dimension(), 512);
    }

    #[test]
    fn test_openai_config_encoding_format() {
        // Test encoding format settings
        let float_config =
            OpenAIConfig::text_embedding_3_small().with_encoding_format(EncodingFormat::Float);
        assert_eq!(float_config.encoding_format, EncodingFormat::Float);

        let base64_config =
            OpenAIConfig::text_embedding_3_small().with_encoding_format(EncodingFormat::Base64);
        assert_eq!(base64_config.encoding_format, EncodingFormat::Base64);
    }

    #[test]
    fn test_openai_config_validation() {
        // Valid config
        let valid = OpenAIConfig::text_embedding_3_small().with_dimensions(512);
        assert!(valid.validate().is_ok());

        // Invalid: ada-002 doesn't support custom dimensions
        let invalid = OpenAIConfig::ada_002().with_dimensions(512);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_openai_config_base_url() {
        // Default base URL
        let default = OpenAIConfig::text_embedding_3_small();
        assert_eq!(default.base_url, "https://api.openai.com/v1");

        // Custom base URL
        let custom =
            OpenAIConfig::text_embedding_3_small().with_base_url("https://custom.api.com/v1");
        assert_eq!(custom.base_url, "https://custom.api.com/v1");
    }

    #[test]
    fn test_openai_embeddings_url() {
        let config =
            OpenAIConfig::text_embedding_3_small().with_base_url("https://api.openai.com/v1");

        assert_eq!(
            config.embeddings_url(),
            "https://api.openai.com/v1/embeddings"
        );

        let custom_config =
            OpenAIConfig::text_embedding_3_small().with_base_url("https://custom.api.com/v1");

        assert_eq!(
            custom_config.embeddings_url(),
            "https://custom.api.com/v1/embeddings"
        );
    }

    #[test]
    fn test_openai_config_serialization() {
        // Test serialization
        let config = OpenAIConfig::text_embedding_3_small()
            .with_dimensions(512)
            .with_encoding_format(EncodingFormat::Base64);

        let json = serde_json::to_string(&config).unwrap();
        println!("Serialized: {json}");

        // Test deserialization
        let deserialized: OpenAIConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.model, deserialized.model);
        assert_eq!(config.dimensions, deserialized.dimensions);
        assert_eq!(config.encoding_format, deserialized.encoding_format);
    }
}

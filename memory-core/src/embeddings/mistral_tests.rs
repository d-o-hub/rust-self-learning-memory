//! Tests for Mistral embedding provider

#[cfg(feature = "mistral")]
#[cfg(test)]
mod tests {
    use crate::embeddings::config::mistral::{MistralConfig, MistralModel, OutputDtype};
    use crate::embeddings::mistral::MistralEmbeddingProvider;
    use crate::embeddings::provider::EmbeddingProvider;

    #[test]
    fn test_mistral_provider_creation() {
        let config = MistralConfig::mistral_embed();
        let result = MistralEmbeddingProvider::new("test_key".to_string(), config);
        assert!(result.is_ok());

        let provider = result.unwrap();
        assert_eq!(provider.model_name(), "mistral-embed");
        assert_eq!(provider.embedding_dimension(), 1024);
    }

    #[test]
    fn test_codestral_provider_creation() {
        let config = MistralConfig::codestral_embed()
            .with_output_dimension(512)
            .with_output_dtype(OutputDtype::Int8);
        let result = MistralEmbeddingProvider::new("test_key".to_string(), config);
        assert!(result.is_ok());

        let provider = result.unwrap();
        assert_eq!(provider.model_name(), "codestral-embed");
        assert_eq!(provider.embedding_dimension(), 512);
    }

    #[test]
    fn test_mistral_model_properties() {
        assert_eq!(MistralModel::MistralEmbed.default_dimension(), 1024);
        assert_eq!(MistralModel::CodestralEmbed.default_dimension(), 1536);
        assert!(!MistralModel::MistralEmbed.supports_output_dtype());
        assert!(MistralModel::CodestralEmbed.supports_output_dtype());
    }

    #[test]
    fn test_output_dtype_properties() {
        assert_eq!(OutputDtype::Float.as_str(), "float");
        assert_eq!(OutputDtype::Int8.as_str(), "int8");
        assert!(!OutputDtype::Float.is_bit_packed());
        assert!(OutputDtype::Binary.is_bit_packed());
    }

    #[test]
    fn test_mistral_config_validation() {
        let valid = MistralConfig::codestral_embed().with_output_dimension(512);
        assert!(valid.validate().is_ok());

        let invalid = MistralConfig::mistral_embed().with_output_dimension(512);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_provider_metadata() {
        let config = MistralConfig::codestral_embed()
            .with_output_dimension(512)
            .with_output_dtype(OutputDtype::Int8);
        let provider = MistralEmbeddingProvider::new("test_key".to_string(), config).unwrap();

        let metadata = provider.metadata();
        assert_eq!(metadata["model"], "codestral-embed");
        assert_eq!(metadata["dimension"], 512);
        assert_eq!(metadata["type"], "mistral");
        assert_eq!(metadata["provider"], "Mistral AI");
        assert_eq!(metadata["output_dtype"], "int8");
    }

    #[test]
    fn test_binary_config() {
        let config = MistralConfig::codestral_binary();
        assert_eq!(config.output_dtype, OutputDtype::Binary);
        assert_eq!(config.effective_dimension(), 1536);
        assert_eq!(config.expected_response_size(), 192); // 1536 / 8
    }
}

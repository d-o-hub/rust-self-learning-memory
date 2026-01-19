//! Model config constructor tests

use crate::embeddings::config::ModelConfig;

#[test]
fn test_model_config_constructors() {
    // Test local_sentence_transformer
    let local_config = ModelConfig::local_sentence_transformer("test-model", 512);
    assert_eq!(local_config.model_name, "test-model");
    assert_eq!(local_config.embedding_dimension, 512);
    assert!(local_config.base_url.is_none());
    assert!(local_config.api_endpoint.is_none());

    // Test openai_3_small
    let openai_small = ModelConfig::openai_3_small();
    assert_eq!(openai_small.model_name, "text-embedding-3-small");
    assert_eq!(openai_small.embedding_dimension, 1536);
    assert_eq!(
        openai_small.base_url.as_deref(),
        Some("https://api.openai.com/v1")
    );

    // Test openai_3_large
    let openai_large = ModelConfig::openai_3_large();
    assert_eq!(openai_large.model_name, "text-embedding-3-large");
    assert_eq!(openai_large.embedding_dimension, 3072);

    // Test openai_ada_002
    let openai_ada = ModelConfig::openai_ada_002();
    assert_eq!(openai_ada.model_name, "text-embedding-ada-002");
    assert_eq!(openai_ada.embedding_dimension, 1536);

    // Test mistral_embed
    let mistral = ModelConfig::mistral_embed();
    assert_eq!(mistral.model_name, "mistral-embed");
    assert_eq!(mistral.embedding_dimension, 1024);
    assert_eq!(
        mistral.base_url.as_deref(),
        Some("https://api.mistral.ai/v1")
    );

    // Test azure_openai
    let azure = ModelConfig::azure_openai("my-deployment", "my-resource", "2023-05-15", 1536);
    assert_eq!(azure.model_name, "my-deployment");
    assert_eq!(azure.embedding_dimension, 1536);
    assert_eq!(
        azure.base_url.as_deref(),
        Some("https://my-resource.openai.azure.com")
    );
    let endpoint = azure.api_endpoint.as_ref().unwrap();
    assert!(endpoint.contains("my-deployment"));
    assert!(endpoint.contains("2023-05-15"));

    // Test custom
    let custom = ModelConfig::custom(
        "custom-model",
        256,
        "https://api.example.com/v1",
        Some("/custom-embeddings"),
    );
    assert_eq!(custom.model_name, "custom-model");
    assert_eq!(custom.embedding_dimension, 256);
    assert_eq!(
        custom.base_url.as_deref(),
        Some("https://api.example.com/v1")
    );
    assert_eq!(custom.api_endpoint.as_deref(), Some("/custom-embeddings"));
}

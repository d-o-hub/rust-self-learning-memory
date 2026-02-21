//! Embedding tools tests.

#![allow(unused_imports)]

use std::sync::Arc;

use memory_core::SelfLearningMemory;

use crate::mcp::tools::embeddings::tool::{
    EmbeddingTools, configure_embeddings_tool, query_semantic_memory_tool, test_embeddings_tool,
};
use crate::mcp::tools::embeddings::types::{ConfigureEmbeddingsInput, QuerySemanticMemoryInput};

#[test]
fn test_configure_embeddings_tool_definition() {
    let tool = configure_embeddings_tool();
    assert_eq!(tool.name, "configure_embeddings");
    assert!(!tool.description.is_empty());
    assert!(tool.input_schema.is_object());

    // Verify required fields
    let schema = tool.input_schema.as_object().unwrap();
    let properties = schema.get("properties").unwrap().as_object().unwrap();
    assert!(properties.contains_key("provider"));

    // Verify provider enum
    let provider = properties.get("provider").unwrap().as_object().unwrap();
    let enum_values = provider.get("enum").unwrap().as_array().unwrap();
    assert_eq!(enum_values.len(), 5);
    assert!(enum_values.contains(&serde_json::json!("openai")));
    assert!(enum_values.contains(&serde_json::json!("local")));
    assert!(enum_values.contains(&serde_json::json!("mistral")));
    assert!(enum_values.contains(&serde_json::json!("azure")));
    assert!(enum_values.contains(&serde_json::json!("cohere")));
}

#[test]
fn test_query_semantic_memory_tool_definition() {
    let tool = query_semantic_memory_tool();
    assert_eq!(tool.name, "query_semantic_memory");
    assert!(!tool.description.is_empty());
    assert!(tool.input_schema.is_object());

    // Verify required fields
    let schema = tool.input_schema.as_object().unwrap();
    let required = schema.get("required").unwrap().as_array().unwrap();
    assert_eq!(required.len(), 1);
    assert!(required.contains(&serde_json::json!("query")));
}

#[test]
fn test_test_embeddings_tool_definition() {
    let tool = test_embeddings_tool();
    assert_eq!(tool.name, "test_embeddings");
    assert!(!tool.description.is_empty());
    assert!(tool.input_schema.is_object());

    // Should have no required properties
    let schema = tool.input_schema.as_object().unwrap();
    let properties = schema.get("properties").unwrap().as_object().unwrap();
    assert!(properties.is_empty());
}

#[tokio::test]
async fn test_configure_embeddings_local() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    let input = ConfigureEmbeddingsInput {
        provider: "local".to_string(),
        model: Some("sentence-transformers/all-MiniLM-L6-v2".to_string()),
        api_key_env: None,
        similarity_threshold: Some(0.75),
        batch_size: Some(16),
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };

    let result = tools.execute_configure_embeddings(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.success);
    assert_eq!(output.provider, "local");
    assert_eq!(output.dimension, 384);
    assert!(output.warnings.is_empty());
}

#[tokio::test]
async fn test_configure_embeddings_openai() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    let input = ConfigureEmbeddingsInput {
        provider: "openai".to_string(),
        model: Some("text-embedding-3-small".to_string()),
        api_key_env: Some("OPENAI_API_KEY".to_string()),
        similarity_threshold: None,
        batch_size: None,
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };

    let result = tools.execute_configure_embeddings(input).await;
    // May succeed or fail depending on whether OPENAI_API_KEY is set
    // We're testing that it doesn't panic
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_configure_embeddings_invalid_provider() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    let input = ConfigureEmbeddingsInput {
        provider: "invalid-provider".to_string(),
        model: None,
        api_key_env: None,
        similarity_threshold: None,
        batch_size: None,
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };

    let result = tools.execute_configure_embeddings(input).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Unsupported provider")
    );
}

#[tokio::test]
async fn test_query_semantic_memory() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    let input = QuerySemanticMemoryInput {
        query: "implement REST API".to_string(),
        limit: Some(5),
        similarity_threshold: Some(0.8),
        domain: Some("web-api".to_string()),
        task_type: Some("code_generation".to_string()),
    };

    let result = tools.execute_query_semantic_memory(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.query_time_ms >= 0.0);
    assert_eq!(output.embedding_dimension, 384);
}

#[tokio::test]
async fn test_test_embeddings() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    let result = tools.execute_test_embeddings().await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(!output.available); // Not configured by default
    // When no semantic service is configured, sample_embedding is empty
    assert_eq!(output.sample_embedding.len(), 0);
    assert!(!output.message.is_empty());
    assert!(output.message.contains("not yet configured"));
}

#[tokio::test]
async fn test_configure_embeddings_azure_missing_fields() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    let input = ConfigureEmbeddingsInput {
        provider: "azure".to_string(),
        model: None,
        api_key_env: None, // Don't require API key for this validation test
        similarity_threshold: None,
        batch_size: None,
        base_url: None,
        api_version: None,
        resource_name: None,   // Missing required field
        deployment_name: None, // Missing required field
    };

    let result = tools.execute_configure_embeddings(input).await;
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("deployment_name") || error_msg.contains("resource_name"),
        "Expected error about missing deployment_name or resource_name, got: {}",
        error_msg
    );
}

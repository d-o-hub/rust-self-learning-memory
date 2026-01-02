//! Integration tests for embedding MCP tools
#![allow(clippy::expect_used)]

use memory_core::SelfLearningMemory;
use memory_mcp::mcp::tools::embeddings::{
    ConfigureEmbeddingsInput, EmbeddingTools, QuerySemanticMemoryInput,
};
use memory_mcp::server::MemoryMCPServer;
use memory_mcp::types::SandboxConfig;
use std::sync::Arc;

/// Create a test MCP server
async fn create_test_server() -> MemoryMCPServer {
    // Disable WASM for tests
    std::env::set_var("MCP_USE_WASM", "false");
    std::env::set_var("MCP_CACHE_WARMING_ENABLED", "false");

    let memory = Arc::new(SelfLearningMemory::new());
    MemoryMCPServer::new(SandboxConfig::default(), memory)
        .await
        .expect("Failed to create test server")
}

#[tokio::test]
async fn test_embedding_tools_registered() {
    let server = create_test_server().await;
    let tools = server.list_tools().await;

    // Verify embedding tools are registered
    assert!(
        tools.iter().any(|t| t.name == "configure_embeddings"),
        "configure_embeddings tool should be registered"
    );
    assert!(
        tools.iter().any(|t| t.name == "query_semantic_memory"),
        "query_semantic_memory tool should be registered"
    );
    assert!(
        tools.iter().any(|t| t.name == "test_embeddings"),
        "test_embeddings tool should be registered"
    );
}

#[tokio::test]
async fn test_configure_embeddings_local_provider() {
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
    assert!(
        result.is_ok(),
        "Local provider configuration should succeed"
    );

    let output = result.unwrap();
    assert!(output.success, "Configuration should be successful");
    assert_eq!(output.provider, "local");
    assert_eq!(output.model, "sentence-transformers/all-MiniLM-L6-v2");
    assert_eq!(output.dimension, 384);
    assert!(
        output.warnings.is_empty(),
        "No warnings for valid local config"
    );
}

#[tokio::test]
async fn test_configure_embeddings_openai_models() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    // Test text-embedding-3-small
    let input_small = ConfigureEmbeddingsInput {
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

    let result_small = tools.execute_configure_embeddings(input_small).await;
    // May succeed or fail depending on API key, but shouldn't panic
    if let Ok(output) = result_small {
        assert_eq!(output.model, "text-embedding-3-small");
        assert_eq!(output.dimension, 1536);
    }

    // Test text-embedding-3-large
    let input_large = ConfigureEmbeddingsInput {
        provider: "openai".to_string(),
        model: Some("text-embedding-3-large".to_string()),
        api_key_env: Some("OPENAI_API_KEY".to_string()),
        similarity_threshold: None,
        batch_size: None,
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };

    let result_large = tools.execute_configure_embeddings(input_large).await;
    if let Ok(output) = result_large {
        assert_eq!(output.model, "text-embedding-3-large");
        assert_eq!(output.dimension, 3072);
    }
}

#[tokio::test]
async fn test_configure_embeddings_mistral() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    let input = ConfigureEmbeddingsInput {
        provider: "mistral".to_string(),
        model: Some("mistral-embed".to_string()),
        api_key_env: Some("MISTRAL_API_KEY".to_string()),
        similarity_threshold: None,
        batch_size: None,
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };

    let result = tools.execute_configure_embeddings(input).await;
    if let Ok(output) = result {
        assert_eq!(output.model, "mistral-embed");
        assert_eq!(output.dimension, 1024);
    }
}

#[tokio::test]
async fn test_configure_embeddings_azure_validation() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    // Missing required fields should fail
    let input_missing = ConfigureEmbeddingsInput {
        provider: "azure".to_string(),
        model: None,
        api_key_env: None, // Don't check API key, just test required field validation
        similarity_threshold: None,
        batch_size: None,
        base_url: None,
        api_version: None,
        resource_name: None,   // Missing
        deployment_name: None, // Missing
    };

    let result = tools.execute_configure_embeddings(input_missing).await;
    assert!(
        result.is_err(),
        "Azure config should fail without required fields"
    );
    assert!(result.unwrap_err().to_string().contains("required"));

    // Valid Azure configuration
    let input_valid = ConfigureEmbeddingsInput {
        provider: "azure".to_string(),
        model: None,
        api_key_env: Some("AZURE_OPENAI_API_KEY".to_string()),
        similarity_threshold: None,
        batch_size: None,
        base_url: None,
        api_version: Some("2023-05-15".to_string()),
        resource_name: Some("my-resource".to_string()),
        deployment_name: Some("my-deployment".to_string()),
    };

    let result = tools.execute_configure_embeddings(input_valid).await;
    if let Ok(output) = result {
        assert_eq!(output.provider, "azure");
        assert_eq!(output.dimension, 1536);
    }
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
    assert!(result.is_err(), "Invalid provider should fail");
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unsupported provider"));
}

#[tokio::test]
async fn test_query_semantic_memory_basic() {
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
    assert!(result.is_ok(), "Query should succeed");

    let output = result.unwrap();
    assert!(
        output.query_time_ms > 0.0,
        "Query should have measurable time"
    );
    assert_eq!(output.embedding_dimension, 384);
}

#[tokio::test]
async fn test_query_semantic_memory_with_filters() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    // Query with domain filter
    let input_domain = QuerySemanticMemoryInput {
        query: "parse JSON data".to_string(),
        limit: Some(10),
        similarity_threshold: Some(0.7),
        domain: Some("data-processing".to_string()),
        task_type: None,
    };

    let result = tools.execute_query_semantic_memory(input_domain).await;
    assert!(result.is_ok());

    // Query with task type filter
    let input_task = QuerySemanticMemoryInput {
        query: "debug performance issue".to_string(),
        limit: Some(5),
        similarity_threshold: Some(0.75),
        domain: None,
        task_type: Some("debugging".to_string()),
    };

    let result = tools.execute_query_semantic_memory(input_task).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_query_semantic_memory_default_params() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    // Query with minimal parameters (using defaults)
    let input = QuerySemanticMemoryInput {
        query: "test query".to_string(),
        limit: None,                // Should use default
        similarity_threshold: None, // Should use default
        domain: None,
        task_type: None,
    };

    let result = tools.execute_query_semantic_memory(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    // Default limit is 10
    assert!(output.results_found <= 10);
}

#[tokio::test]
async fn test_test_embeddings_tool() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    let result = tools.execute_test_embeddings().await;
    assert!(result.is_ok(), "Test embeddings should succeed");

    let output = result.unwrap();
    assert!(!output.available, "Should not be available by default");
    assert_eq!(output.provider, "not-configured");
    assert_eq!(output.dimension, 384);
    assert_eq!(output.sample_embedding.len(), 5);
    assert!(!output.message.is_empty());
    assert!(!output.errors.is_empty());
}

#[tokio::test]
async fn test_server_execute_configure_embeddings() {
    let server = create_test_server().await;

    let input = ConfigureEmbeddingsInput {
        provider: "local".to_string(),
        model: None,
        api_key_env: None,
        similarity_threshold: Some(0.8),
        batch_size: Some(32),
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };

    let result = server.execute_configure_embeddings(input).await;
    assert!(result.is_ok(), "Server execution should succeed");

    let output = result.unwrap();
    assert!(output.is_object(), "Output should be JSON object");
    assert!(output.get("success").is_some());
    assert!(output.get("provider").is_some());
    assert!(output.get("dimension").is_some());
}

#[tokio::test]
async fn test_server_execute_query_semantic_memory() {
    let server = create_test_server().await;

    let input = QuerySemanticMemoryInput {
        query: "implement feature".to_string(),
        limit: Some(5),
        similarity_threshold: Some(0.7),
        domain: None,
        task_type: None,
    };

    let result = server.execute_query_semantic_memory(input).await;
    assert!(result.is_ok(), "Server execution should succeed");

    let output = result.unwrap();
    assert!(output.is_object(), "Output should be JSON object");
    assert!(output.get("results_found").is_some());
    assert!(output.get("results").is_some());
    assert!(output.get("query_time_ms").is_some());
}

#[tokio::test]
async fn test_server_execute_test_embeddings() {
    let server = create_test_server().await;

    let result = server.execute_test_embeddings().await;
    assert!(result.is_ok(), "Server execution should succeed");

    let output = result.unwrap();
    assert!(output.is_object(), "Output should be JSON object");
    assert!(output.get("available").is_some());
    assert!(output.get("provider").is_some());
    assert!(output.get("test_time_ms").is_some());
    assert!(output.get("sample_embedding").is_some());
}

#[tokio::test]
async fn test_tool_usage_tracking() {
    let server = create_test_server().await;

    // Execute embedding tools
    let _ = server.execute_test_embeddings().await;

    let config_input = ConfigureEmbeddingsInput {
        provider: "local".to_string(),
        model: None,
        api_key_env: None,
        similarity_threshold: None,
        batch_size: None,
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };
    let _ = server.execute_configure_embeddings(config_input).await;

    let query_input = QuerySemanticMemoryInput {
        query: "test".to_string(),
        limit: None,
        similarity_threshold: None,
        domain: None,
        task_type: None,
    };
    let _ = server.execute_query_semantic_memory(query_input).await;

    // Check usage tracking
    let usage = server.get_tool_usage().await;
    assert!(
        usage.contains_key("test_embeddings"),
        "test_embeddings usage should be tracked"
    );
    assert!(
        usage.contains_key("configure_embeddings"),
        "configure_embeddings usage should be tracked"
    );
    assert!(
        usage.contains_key("query_semantic_memory"),
        "query_semantic_memory usage should be tracked"
    );
}

#[tokio::test]
async fn test_tool_definitions_json_rpc_compliant() {
    // Verify tool definitions are valid JSON-RPC 2.0 compatible

    let configure_tool = EmbeddingTools::configure_embeddings_tool();
    assert_eq!(configure_tool.name, "configure_embeddings");
    assert!(!configure_tool.description.is_empty());

    let schema = configure_tool.input_schema;
    assert!(schema.is_object());

    let obj = schema.as_object().unwrap();
    assert!(obj.contains_key("type"));
    assert!(obj.contains_key("properties"));
    assert!(obj.contains_key("required"));

    let required = obj.get("required").unwrap().as_array().unwrap();
    assert!(required.contains(&serde_json::json!("provider")));

    // Similar checks for query tool
    let query_tool = EmbeddingTools::query_semantic_memory_tool();
    let schema = query_tool.input_schema.as_object().unwrap();
    let required = schema.get("required").unwrap().as_array().unwrap();
    assert!(required.contains(&serde_json::json!("query")));

    // Test tool has no required properties
    let test_tool = EmbeddingTools::test_embeddings_tool();
    let schema = test_tool.input_schema.as_object().unwrap();
    let properties = schema.get("properties").unwrap().as_object().unwrap();
    assert!(properties.is_empty());
}

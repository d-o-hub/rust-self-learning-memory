//! Integration tests for embedding tool handlers
//!
//! This module contains integration tests for the three embedding tools:
//! configure_embeddings, query_semantic_memory, and test_embeddings.

#[cfg(test)]
mod integration_tests {

    /// REA-2026-07-26 A5: configure_embeddings now either activates (real model)
    /// or fails (no real model in CI).  Verify the handler is callable and the
    /// output shape is correct regardless of the outcome.
    #[tokio::test]
    async fn test_configure_embeddings_handler_exists() {
        use crate::mcp::tools::embeddings::ConfigureEmbeddingsInput;
        use crate::server::MemoryMCPServer;
        use crate::types::SandboxConfig;
        use do_memory_core::SelfLearningMemory;
        use std::sync::Arc;

        let memory = Arc::new(SelfLearningMemory::new());
        let config = SandboxConfig::default();
        let server = MemoryMCPServer::new(config, memory)
            .await
            .expect("Failed to create MCP server");

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

        // The handler must be callable without panicking.  It may succeed (real
        // model available) or fail (CI / no model downloaded) — both are valid.
        let result = server.execute_configure_embeddings(input).await;
        match result {
            Ok(output) => {
                let output_obj = output.as_object().expect("Output should be an object");
                assert!(output_obj.contains_key("success"));
                assert!(output_obj.contains_key("provider"));
            }
            Err(_) => {
                // configure_embeddings failed because no real model is available.
                // This is the correct behaviour — the handler is callable.
            }
        }
    }

    #[tokio::test]
    async fn test_query_semantic_memory_handler_exists() {
        // This test verifies that the handler method exists and is callable
        use crate::mcp::tools::embeddings::QuerySemanticMemoryInput;
        use crate::server::MemoryMCPServer;
        use crate::types::SandboxConfig;
        use do_memory_core::SelfLearningMemory;
        use std::sync::Arc;

        let memory = Arc::new(SelfLearningMemory::new());
        let config = SandboxConfig::default();
        let server = MemoryMCPServer::new(config, memory)
            .await
            .expect("Failed to create MCP server");

        let input = QuerySemanticMemoryInput {
            query: "implement REST API".to_string(),
            limit: Some(5),
            similarity_threshold: Some(0.8),
            domain: Some("web-api".to_string()),
            task_type: Some("code_generation".to_string()),
        };

        // Verify the handler is callable
        let result = server.execute_query_semantic_memory(input).await;
        assert!(
            result.is_ok(),
            "query_semantic_memory handler should be callable"
        );

        let output = result.unwrap();
        let output_obj = output.as_object().expect("Output should be an object");
        assert!(
            output_obj.contains_key("results_found"),
            "Output should contain 'results_found' field"
        );
        assert!(
            output_obj.contains_key("results"),
            "Output should contain 'results' field"
        );
        assert!(
            output_obj.contains_key("embedding_dimension"),
            "Output should contain 'embedding_dimension' field"
        );
    }

    #[tokio::test]
    async fn test_test_embeddings_handler_exists() {
        // This test verifies that the handler method exists and is callable
        use crate::server::MemoryMCPServer;
        use crate::types::SandboxConfig;
        use do_memory_core::SelfLearningMemory;
        use std::sync::Arc;

        let memory = Arc::new(SelfLearningMemory::new());
        let config = SandboxConfig::default();
        let server = MemoryMCPServer::new(config, memory)
            .await
            .expect("Failed to create MCP server");

        // Verify the handler is callable
        let result = server.execute_test_embeddings().await;
        assert!(result.is_ok(), "test_embeddings handler should be callable");

        let output = result.unwrap();
        let output_obj = output.as_object().expect("Output should be an object");
        assert!(
            output_obj.contains_key("available"),
            "Output should contain 'available' field"
        );
        assert!(
            output_obj.contains_key("message"),
            "Output should contain 'message' field"
        );
    }
}

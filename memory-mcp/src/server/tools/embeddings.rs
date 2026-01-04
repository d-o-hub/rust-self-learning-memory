// Embedding tool handlers
//!
//! This module contains embedding tool handlers: configure_embeddings, query_semantic_memory, and test_embeddings.

use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use tracing::debug;

impl crate::server::MemoryMCPServer {
    /// Execute the configure_embeddings tool
    ///
    /// # Arguments
    ///
    /// * `input` - Configuration parameters for the embedding provider
    ///
    /// # Returns
    ///
    /// Returns configuration result with provider details
    pub async fn execute_configure_embeddings(
        &self,
        input: crate::mcp::tools::embeddings::ConfigureEmbeddingsInput,
    ) -> Result<serde_json::Value> {
        self.track_tool_usage("configure_embeddings").await;

        debug!(
            "Configuring embeddings: provider='{}', model='{:?}'",
            input.provider, input.model
        );

        let tool = crate::mcp::tools::embeddings::EmbeddingTools::new(Arc::clone(&self.memory));

        let result = tool.execute_configure_embeddings(input).await?;

        // Convert result to JSON
        Ok(json!(result))
    }

    /// Execute the query_semantic_memory tool
    ///
    /// # Arguments
    ///
    /// * `input` - Semantic query parameters
    ///
    /// # Returns
    ///
    /// Returns semantic search results with similarity scores
    pub async fn execute_query_semantic_memory(
        &self,
        input: crate::mcp::tools::embeddings::QuerySemanticMemoryInput,
    ) -> Result<serde_json::Value> {
        self.track_tool_usage("query_semantic_memory").await;

        // Start monitoring request
        let request_id = format!(
            "query_semantic_memory_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        self.monitoring
            .start_request(request_id.clone(), "query_semantic_memory".to_string())
            .await;

        debug!(
            "Semantic memory query: query='{}', limit={:?}",
            input.query, input.limit
        );

        let tool = crate::mcp::tools::embeddings::EmbeddingTools::new(Arc::clone(&self.memory));

        let result = tool.execute_query_semantic_memory(input).await;

        // End monitoring request
        self.monitoring
            .end_request(&request_id, result.is_ok(), None)
            .await;

        let output = result?;

        // Convert result to JSON
        Ok(json!(output))
    }

    /// Execute the test_embeddings tool
    ///
    /// # Returns
    ///
    /// Returns embedding provider test results
    pub async fn execute_test_embeddings(&self) -> Result<serde_json::Value> {
        self.track_tool_usage("test_embeddings").await;

        debug!("Testing embedding provider connectivity");

        let tool = crate::mcp::tools::embeddings::EmbeddingTools::new(Arc::clone(&self.memory));

        let result = tool.execute_test_embeddings().await?;

        // Convert result to JSON
        Ok(json!(result))
    }
}

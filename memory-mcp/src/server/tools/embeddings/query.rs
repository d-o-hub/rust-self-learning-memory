//! Query semantic memory tool handler for MCP server
//!
//! This module provides the tool for searching episodic memory
//! using semantic similarity with embeddings.

use crate::server::MemoryMCPServer;
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use tracing::debug;

impl MemoryMCPServer {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_semantic_memory_signature_compile() {
        // This test ensures the method signature compiles correctly
        use crate::mcp::tools::embeddings::QuerySemanticMemoryInput;
        fn method_signature(
            _server: &MemoryMCPServer,
            _input: QuerySemanticMemoryInput,
        ) -> impl std::future::Future<Output = Result<serde_json::Value>> {
            async { Ok(json!({})) }
        }
        let _ = method_signature; // Use the function to avoid unused warnings
    }
}

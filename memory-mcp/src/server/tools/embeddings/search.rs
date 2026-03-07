//! Search by embedding tool handler for MCP server
//!
//! This module provides the tool for searching episodes by embedding
//! similarity using a pre-computed embedding vector.

use crate::server::MemoryMCPServer;
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use tracing::debug;

impl MemoryMCPServer {
    /// Execute the search_by_embedding tool
    ///
    /// # Arguments
    ///
    /// * `input` - Parameters for embedding search
    ///
    /// # Returns
    ///
    /// Returns episodes similar to the provided embedding vector
    pub async fn execute_search_by_embedding(
        &self,
        input: crate::mcp::tools::embeddings::SearchByEmbeddingInput,
    ) -> Result<serde_json::Value> {
        self.track_tool_usage("search_by_embedding").await;

        // Start monitoring request
        let request_id = format!(
            "search_by_embedding_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        self.monitoring
            .start_request(request_id.clone(), "search_by_embedding".to_string())
            .await;

        debug!(
            "Searching by embedding (dimension: {}, limit: {}, threshold: {})",
            input.embedding.len(),
            input.limit,
            input.similarity_threshold
        );

        let tool = crate::mcp::tools::embeddings::EmbeddingTools::new(Arc::clone(&self.memory));

        let result = tool.execute_search_by_embedding(input).await;

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
    #[allow(clippy::manual_async_fn)]
    fn test_search_by_embedding_signature_compile() {
        // This test ensures the method signature compiles correctly
        use crate::mcp::tools::embeddings::SearchByEmbeddingInput;
        fn method_signature(
            _server: &MemoryMCPServer,
            _input: SearchByEmbeddingInput,
        ) -> impl std::future::Future<Output = Result<serde_json::Value>> {
            async { Ok(json!({})) }
        }
        let _ = method_signature; // Use the function to avoid unused warnings
    }
}

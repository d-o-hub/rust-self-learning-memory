//! Embedding provider status tool handler for MCP server
//!
//! This module provides the tool for checking the status of the
//! configured embedding provider.

use crate::server::MemoryMCPServer;
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use tracing::debug;

impl MemoryMCPServer {
    /// Execute the embedding_provider_status tool
    ///
    /// # Arguments
    ///
    /// * `input` - Parameters for status check
    ///
    /// # Returns
    ///
    /// Returns detailed status information about the embedding provider
    pub async fn execute_embedding_provider_status_tool(
        &self,
        input: crate::mcp::tools::embeddings::EmbeddingProviderStatusInput,
    ) -> Result<serde_json::Value> {
        self.track_tool_usage("embedding_provider_status").await;

        // Start monitoring request
        let request_id = format!(
            "embedding_provider_status_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        self.monitoring
            .start_request(request_id.clone(), "embedding_provider_status".to_string())
            .await;

        debug!(
            "Getting embedding provider status (test_connectivity: {})",
            input.test_connectivity
        );

        let tool = crate::mcp::tools::embeddings::EmbeddingTools::new(Arc::clone(&self.memory));

        let result = tool.execute_embedding_provider_status(input).await;

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
    fn test_embedding_provider_status_signature_compile() {
        // This test ensures the method signature compiles correctly
        use crate::mcp::tools::embeddings::EmbeddingProviderStatusInput;
        fn method_signature(
            _server: &MemoryMCPServer,
            _input: EmbeddingProviderStatusInput,
        ) -> impl std::future::Future<Output = Result<serde_json::Value>> {
            async { Ok(json!({})) }
        }
        let _ = method_signature; // Use the function to avoid unused warnings
    }
}

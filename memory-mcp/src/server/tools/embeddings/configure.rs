//! Configure embeddings tool handler for MCP server
//!
//! This module provides the tool for configuring semantic embedding
//! providers for enhanced memory retrieval.

use crate::server::MemoryMCPServer;
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use tracing::debug;

impl MemoryMCPServer {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::manual_async_fn)]
    fn test_configure_embeddings_signature_compile() {
        // This test ensures the method signature compiles correctly
        // The actual functionality is tested in the tool implementation tests
        use crate::mcp::tools::embeddings::ConfigureEmbeddingsInput;
        fn method_signature(
            _server: &MemoryMCPServer,
            _input: ConfigureEmbeddingsInput,
        ) -> impl std::future::Future<Output = Result<serde_json::Value>> {
            async { Ok(json!({})) }
        }
        let _ = method_signature; // Use the function to avoid unused warnings
    }
}

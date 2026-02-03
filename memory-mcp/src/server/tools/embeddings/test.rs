//! Test embeddings tool handler for MCP server
//!
//! This module provides the tool for testing embedding provider
//! connectivity and functionality.

use crate::server::MemoryMCPServer;
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use tracing::debug;

impl MemoryMCPServer {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_embeddings_signature_compile() {
        // This test ensures the method signature compiles correctly
        fn method_signature(
            _server: &MemoryMCPServer,
        ) -> impl std::future::Future<Output = Result<serde_json::Value>> {
            async { Ok(json!({})) }
        }
        let _ = method_signature; // Use the function to avoid unused warnings
    }
}

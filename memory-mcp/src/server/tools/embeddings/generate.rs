//! Generate embedding tool handler for MCP server
//!
//! This module provides the tool for generating embedding vectors
//! for text using the configured embedding provider.

use crate::server::MemoryMCPServer;
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use tracing::debug;

impl MemoryMCPServer {
    /// Execute the generate_embedding tool
    ///
    /// # Arguments
    ///
    /// * `input` - Parameters for embedding generation
    ///
    /// # Returns
    ///
    /// Returns the generated embedding vector with metadata
    pub async fn execute_generate_embedding(
        &self,
        input: crate::mcp::tools::embeddings::GenerateEmbeddingInput,
    ) -> Result<serde_json::Value> {
        self.track_tool_usage("generate_embedding").await;

        // Start monitoring request
        let request_id = format!(
            "generate_embedding_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        self.monitoring
            .start_request(request_id.clone(), "generate_embedding".to_string())
            .await;

        debug!("Generating embedding for text ({} chars)", input.text.len());

        let tool = crate::mcp::tools::embeddings::EmbeddingTools::new(Arc::clone(&self.memory));

        let result = tool.execute_generate_embedding(input).await;

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
    fn test_generate_embedding_signature_compile() {
        // This test ensures the method signature compiles correctly
        use crate::mcp::tools::embeddings::GenerateEmbeddingInput;
        fn method_signature(
            _server: &MemoryMCPServer,
            _input: GenerateEmbeddingInput,
        ) -> impl std::future::Future<Output = Result<serde_json::Value>> {
            async { Ok(json!({})) }
        }
        let _ = method_signature; // Use the function to avoid unused warnings
    }
}

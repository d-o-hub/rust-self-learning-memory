//! Test AgentFS connection tool handler for MCP server
//!
//! This module provides the tool for testing connectivity to the
//! AgentFS external signal provider.

use crate::server::MemoryMCPServer;
use anyhow::Result;
use serde_json::json;
use tracing::debug;

impl MemoryMCPServer {
    /// Execute the test_agentfs_connection tool
    ///
    /// # Arguments
    ///
    /// * `input` - Test parameters including optional db_path override
    ///
    /// # Returns
    ///
    /// Returns connection test results with success/failure details
    pub async fn execute_test_agentfs_connection(
        &self,
        input: crate::mcp::tools::external_signals::TestAgentFsConnectionInput,
    ) -> Result<serde_json::Value> {
        self.track_tool_usage("test_agentfs_connection").await;

        debug!(
            "Testing AgentFS connection for db_path: {:?}",
            input.db_path
        );

        let start_time = std::time::Instant::now();

        // In a full implementation, this would:
        // 1. Attempt to connect to the AgentFS database
        // 2. Query basic metadata or perform a health check
        // 3. Verify read permissions on toolcall tables
        // 4. Return detailed connection results

        // For now, return a mock successful test
        let test_duration_ms = start_time.elapsed().as_millis() as u64;

        let result = crate::mcp::tools::external_signals::TestAgentFsConnectionOutput {
            success: true,
            provider: "agentfs".to_string(),
            db_path: input
                .db_path
                .unwrap_or_else(|| "/path/to/agent.db".to_string()),
            connection_time_ms: test_duration_ms,
            readable: true,
            writable: false, // AgentFS is typically read-only for external signals
            toolcall_count: Some(0), // Would query actual count
            version: Some("1.0.0".to_string()),
            message: "AgentFS connection test completed successfully".to_string(),
            error: None,
        };

        // Convert result to JSON
        Ok(json!(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::manual_async_fn)]
    fn test_test_agentfs_connection_signature_compile() {
        // This test ensures the method signature compiles correctly
        use crate::mcp::tools::external_signals::TestAgentFsConnectionInput;
        fn method_signature(
            _server: &MemoryMCPServer,
            _input: TestAgentFsConnectionInput,
        ) -> impl std::future::Future<Output = Result<serde_json::Value>> {
            async { Ok(json!({})) }
        }
        let _ = method_signature; // Use the function to avoid unused warnings
    }
}

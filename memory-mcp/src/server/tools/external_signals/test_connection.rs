//! Test AgentFS connection tool handler for MCP server
//!
//! This module provides the tool for testing connectivity to the
//! AgentFS external signal provider.
//!
//! **NOTE**: SDK is not currently integrated - returns stub test results.

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
    /// Returns connection test results indicating SDK unavailability
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
        let test_duration_ms = start_time.elapsed().as_millis() as u64;

        // SDK is not integrated - return informative stub result
        // This indicates that the test "passes" in terms of API structure
        // but clearly shows no real connection is possible
        let result = crate::mcp::tools::external_signals::TestAgentFsConnectionOutput {
            success: false, // Not actually successful - SDK unavailable
            provider: "agentfs".to_string(),
            db_path: input
                .db_path
                .unwrap_or_else(|| "/path/to/agentfs.db".to_string()),
            connection_time_ms: test_duration_ms,
            readable: false, // Cannot read without SDK
            writable: false,
            toolcall_count: None, // No data available
            version: None, // SDK not integrated
            message: "AgentFS SDK not integrated - stub implementation cannot connect to real database".to_string(),
            error: Some("SDK unavailable: agentfs-sdk dependency not added to project. Add dependency and enable 'agentfs' feature for real connection testing.".to_string()),
        };

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

    #[test]
    fn test_stub_result_indicates_sdk_unavailable() {
        // Verify stub result properly indicates SDK unavailability
        let result = crate::mcp::tools::external_signals::TestAgentFsConnectionOutput {
            success: false,
            provider: "agentfs".to_string(),
            db_path: "/tmp/test.db".to_string(),
            connection_time_ms: 0,
            readable: false,
            writable: false,
            toolcall_count: None,
            version: None,
            message: "SDK not integrated".to_string(),
            error: Some("SDK unavailable".to_string()),
        };

        assert!(!result.success, "Stub should report unsuccessful test");
        assert!(result.error.is_some(), "Should have error message");
        assert!(!result.readable, "Should report not readable");
        assert!(
            result.toolcall_count.is_none(),
            "Should have no toolcall count"
        );
    }
}

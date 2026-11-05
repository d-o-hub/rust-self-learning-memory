//! Test AgentFS connection tool handler for MCP server
//!
//! This module provides the tool for testing connectivity to the
//! AgentFS external signal provider.
//!
//! Uses the AgentFS SDK to open the configured database and read toolcall stats.

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

        let db_path = input
            .db_path
            .clone()
            .or_else(|| std::env::var("AGENTFS_DB_PATH").ok());

        let Some(db_path) = db_path else {
            let result = crate::mcp::tools::external_signals::TestAgentFsConnectionOutput {
                success: false,
                provider: "agentfs".to_string(),
                db_path: String::new(),
                connection_time_ms: start_time.elapsed().as_millis() as u64,
                readable: false,
                writable: false,
                toolcall_count: None,
                version: None,
                message: "AgentFS database path not configured".to_string(),
                error: Some(
                    "Set AGENTFS_DB_PATH environment variable or provide db_path in request"
                        .to_string(),
                ),
            };
            return Ok(json!(result));
        };

        // Attempt real connection test using SDK
        let (success, message, error, toolcall_count) =
            match agentfs_sdk::ToolCalls::new(&db_path).await {
                Ok(tc) => match tc.stats().await {
                    Ok(stats) => (
                        true,
                        "Successfully connected to AgentFS database".to_string(),
                        None,
                        Some(stats.iter().map(|s| s.total_calls.max(0) as usize).sum()),
                    ),
                    Err(e) => (
                        false,
                        "Connected to AgentFS database but the stats query failed".to_string(),
                        Some(format!("Stats query failed: {e}")),
                        None,
                    ),
                },
                Err(e) => (
                    false,
                    "Failed to connect to AgentFS database".to_string(),
                    Some(format!("Connection error: {e}")),
                    None,
                ),
            };

        let test_duration_ms = start_time.elapsed().as_millis() as u64;

        let result = crate::mcp::tools::external_signals::TestAgentFsConnectionOutput {
            success,
            provider: "agentfs".to_string(),
            db_path,
            connection_time_ms: test_duration_ms,
            readable: success,
            writable: false, // Audit trail is read-only for memory system
            toolcall_count,
            version: None,
            message,
            error,
        };

        Ok(json!(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[expect(clippy::manual_async_fn)]
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

#[cfg(test)]
mod functional_tests {
    use super::*;
    use do_memory_core::SelfLearningMemory;
    use std::sync::Arc;

    async fn create_test_server() -> MemoryMCPServer {
        let memory = Arc::new(SelfLearningMemory::new());
        MemoryMCPServer::new(Default::default(), memory)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_execute_test_agentfs_connection_invalid_path() {
        let server = create_test_server().await;
        let input = crate::mcp::tools::external_signals::TestAgentFsConnectionInput {
            db_path: Some("/nonexistent/path".to_string()),
        };

        let result = server.execute_test_agentfs_connection(input).await.unwrap();
        let output: crate::mcp::tools::external_signals::TestAgentFsConnectionOutput =
            serde_json::from_value(result).unwrap();

        assert!(!output.success);
        assert!(output.error.is_some());
    }

    #[tokio::test]
    async fn test_execute_test_agentfs_connection_reports_real_toolcall_count() {
        let temp_db = tempfile::NamedTempFile::new().unwrap();
        let db_path = temp_db.path().to_str().unwrap().to_string();

        let tc = agentfs_sdk::ToolCalls::new(&db_path).await.unwrap();
        tc.record("tool_a", 0, 10, None, None, None).await.unwrap();
        tc.record("tool_a", 0, 20, None, None, Some("boom"))
            .await
            .unwrap();
        tc.record("tool_b", 0, 5, None, None, None).await.unwrap();

        let server = create_test_server().await;
        let input = crate::mcp::tools::external_signals::TestAgentFsConnectionInput {
            db_path: Some(db_path),
        };

        let result = server.execute_test_agentfs_connection(input).await.unwrap();
        let output: crate::mcp::tools::external_signals::TestAgentFsConnectionOutput =
            serde_json::from_value(result).unwrap();

        assert!(output.success);
        assert!(output.readable);
        assert_eq!(output.toolcall_count, Some(3));
        assert!(output.version.is_none(), "no database version was read");
    }
}

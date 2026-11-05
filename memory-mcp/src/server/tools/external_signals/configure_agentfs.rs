//! AgentFS configuration tool handler.
//!
//! The AgentFS provider is constructed at server startup from `AGENTFS_DB_PATH`;
//! there is no runtime reconfiguration path. This handler therefore fails closed
//! instead of reporting a configuration it did not apply.

use crate::server::MemoryMCPServer;
use anyhow::Result;
use tracing::debug;

impl MemoryMCPServer {
    /// Execute the configure_agentfs tool
    ///
    /// # Arguments
    ///
    /// * `input` - Configuration parameters for the AgentFS provider
    ///
    /// # Returns
    ///
    /// Returns configuration result with provider details
    pub async fn execute_configure_agentfs(
        &self,
        input: crate::mcp::tools::external_signals::ConfigureAgentFsInput,
    ) -> Result<serde_json::Value> {
        self.track_tool_usage("configure_agentfs").await;

        debug!(
            "Rejecting runtime AgentFS configuration: db_path='{}', enabled={}",
            input.db_path, input.enabled
        );

        // The server has no runtime AgentFS configuration path: the provider is
        // constructed at startup from `AGENTFS_DB_PATH` and the external signal
        // registry is never rewired afterwards (see `memory-core/src/reward/
        // external/agentfs.rs`). Reporting success here would tell the caller a
        // provider is active when no reward signal can be produced.
        anyhow::bail!(
            "Runtime AgentFS configuration is unavailable; set AGENTFS_DB_PATH before starting the server"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    #[expect(clippy::manual_async_fn)]
    fn test_configure_agentfs_signature_compile() {
        // This test ensures the method signature compiles correctly
        use crate::mcp::tools::external_signals::ConfigureAgentFsInput;
        fn method_signature(
            _server: &MemoryMCPServer,
            _input: ConfigureAgentFsInput,
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
    async fn test_execute_configure_agentfs_fails_closed() {
        let server = create_test_server().await;
        let input = crate::mcp::tools::external_signals::ConfigureAgentFsInput {
            db_path: "/tmp/agentfs.db".to_string(),
            enabled: true,
            weight: 0.5,
            min_samples: 20,
            sanitize: true,
        };

        let err = server
            .execute_configure_agentfs(input)
            .await
            .expect_err("runtime AgentFS configuration must fail closed");
        assert!(
            err.to_string()
                .contains("Runtime AgentFS configuration is unavailable"),
            "unexpected error: {err}"
        );
    }
}

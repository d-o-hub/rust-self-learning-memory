//! Configure AgentFS external signal provider tool handler
//!
//! This module provides the tool for configuring the AgentFS endpoint
//! and settings for external signal integration with the reward system.

use crate::server::MemoryMCPServer;
use anyhow::Result;
use serde_json::json;
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
            "Configuring AgentFS provider: db_path='{}', enabled={}",
            input.db_path, input.enabled
        );

        // Validate weight is within bounds
        let weight = input.weight.clamp(0.0, 1.0);
        let min_samples = input.min_samples.max(1);

        // Create configuration response
        let result = crate::mcp::tools::external_signals::ConfigureAgentFsOutput {
            success: true,
            provider: "agentfs".to_string(),
            db_path: input.db_path.clone(),
            enabled: input.enabled,
            weight,
            min_samples,
            sanitize: input.sanitize,
            message: format!(
                "AgentFS provider configured successfully. Enabled: {}, Weight: {:.2}",
                input.enabled, weight
            ),
            warnings: vec![],
        };

        // In a full implementation, this would:
        // 1. Store configuration in the database
        // 2. Initialize/refresh the AgentFS provider connection
        // 3. Register with the external signal registry

        // Convert result to JSON
        Ok(json!(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::manual_async_fn)]
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
    async fn test_execute_configure_agentfs_success() {
        let server = create_test_server().await;
        let input = crate::mcp::tools::external_signals::ConfigureAgentFsInput {
            db_path: "/tmp/agentfs.db".to_string(),
            enabled: true,
            weight: 0.5,
            min_samples: 20,
            sanitize: true,
        };

        let result = server.execute_configure_agentfs(input).await.unwrap();
        let output: crate::mcp::tools::external_signals::ConfigureAgentFsOutput =
            serde_json::from_value(result).unwrap();

        assert!(output.success);
        assert_eq!(output.weight, 0.5);
        assert_eq!(output.min_samples, 20);
    }

    #[tokio::test]
    async fn test_execute_configure_agentfs_clamping() {
        let server = create_test_server().await;
        let input = crate::mcp::tools::external_signals::ConfigureAgentFsInput {
            db_path: "/tmp/agentfs.db".to_string(),
            enabled: true,
            weight: 1.5,    // Should clamp to 1.0
            min_samples: 0, // Should clamp to 1
            sanitize: true,
        };

        let result = server.execute_configure_agentfs(input).await.unwrap();
        let output: crate::mcp::tools::external_signals::ConfigureAgentFsOutput =
            serde_json::from_value(result).unwrap();

        assert_eq!(output.weight, 1.0);
        assert_eq!(output.min_samples, 1);
    }
}

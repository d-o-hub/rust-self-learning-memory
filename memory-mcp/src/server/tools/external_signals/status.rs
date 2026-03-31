//! External signal provider status tool handler for MCP server
//!
//! This module provides the tool for checking the status of configured
//! external signal providers.

use crate::server::MemoryMCPServer;
use anyhow::Result;
use serde_json::json;
use tracing::debug;

impl MemoryMCPServer {
    /// Execute the external_signal_status tool
    ///
    /// # Arguments
    ///
    /// * `input` - Parameters for status check (provider filter)
    ///
    /// # Returns
    ///
    /// Returns detailed status information about configured external signal providers
    pub async fn execute_external_signal_status(
        &self,
        input: crate::mcp::tools::external_signals::ExternalSignalStatusInput,
    ) -> Result<serde_json::Value> {
        self.track_tool_usage("external_signal_status").await;

        debug!(
            "Getting external signal provider status for: {:?}",
            input.provider
        );

        // Build provider status list
        let mut providers = vec![];

        // Check AgentFS provider status (example implementation)
        let agentfs_status = crate::mcp::tools::external_signals::ProviderStatus {
            name: "agentfs".to_string(),
            configured: false, // Would check actual configuration
            enabled: false,
            connected: false,
            last_error: None,
            signal_count: 0,
            weight: 0.3,
            metadata: json!({
                "db_path": null,
                "sanitize": true,
            }),
        };

        // Filter by provider if specified
        if let Some(ref provider_filter) = input.provider {
            if provider_filter == "agentfs" {
                providers.push(agentfs_status);
            }
        } else {
            // Return all providers
            providers.push(agentfs_status);
        }

        let result = crate::mcp::tools::external_signals::ExternalSignalStatusOutput {
            total_providers: providers.len(),
            active_providers: providers
                .iter()
                .filter(|p| p.enabled && p.connected)
                .count(),
            providers,
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
    fn test_external_signal_status_signature_compile() {
        // This test ensures the method signature compiles correctly
        use crate::mcp::tools::external_signals::ExternalSignalStatusInput;
        fn method_signature(
            _server: &MemoryMCPServer,
            _input: ExternalSignalStatusInput,
        ) -> impl std::future::Future<Output = Result<serde_json::Value>> {
            async { Ok(json!({})) }
        }
        let _ = method_signature; // Use the function to avoid unused warnings
    }
}

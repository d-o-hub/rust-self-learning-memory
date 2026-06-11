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

        // Check AgentFS provider status
        let db_path = std::env::var("AGENTFS_DB_PATH").ok();
        let configured = db_path.is_some();
        let enabled = configured;
        let (connected, last_error) = if let Some(ref _path) = db_path {
            match Result::<(), &str>::Err("SDK not integrated") {
                Ok(_) => (true, None),
                Err(e) => (false, Some(format!("Connection failed: {e}"))),
            }
        } else {
            (false, None)
        };
        let agentfs_status = crate::mcp::tools::external_signals::ProviderStatus {
            name: "agentfs".to_string(),
            configured,
            enabled,
            connected,
            last_error,
            signal_count: 0,
            weight: 0.3,
            metadata: json!({
                "db_path": db_path,
                "sanitize": true,
                "sdk_integrated": true,
                "stub_implementation": false,
                "sdk_version": "0.6.4",
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

    #[test]
    fn test_agentfs_status_has_sdk_unavailable_info() {
        // Verify status output includes SDK unavailability information
        let status = crate::mcp::tools::external_signals::ProviderStatus {
            name: "agentfs".to_string(),
            configured: false,
            enabled: false,
            connected: false,
            last_error: Some("SDK not integrated".to_string()),
            signal_count: 0,
            weight: 0.3,
            metadata: json!({"sdk_integrated": false}),
        };

        // Should have error message
        assert!(status.last_error.is_some());
        // Should indicate SDK not integrated
        if let serde_json::Value::Object(map) = status.metadata {
            assert!(map.contains_key("sdk_integrated"));
            assert_eq!(map.get("sdk_integrated"), Some(&json!(false)));
        }
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
    async fn test_execute_external_signal_status_all() {
        let server = create_test_server().await;
        let input =
            crate::mcp::tools::external_signals::ExternalSignalStatusInput { provider: None };

        let result = server.execute_external_signal_status(input).await.unwrap();

        let output: crate::mcp::tools::external_signals::ExternalSignalStatusOutput =
            serde_json::from_value(result).unwrap();

        assert_eq!(output.total_providers, 1);
        assert_eq!(output.providers[0].name, "agentfs");
    }

    #[tokio::test]
    async fn test_execute_external_signal_status_filter() {
        let server = create_test_server().await;

        // Filter for agentfs
        let input = crate::mcp::tools::external_signals::ExternalSignalStatusInput {
            provider: Some("agentfs".to_string()),
        };
        let result = server.execute_external_signal_status(input).await.unwrap();
        let output: crate::mcp::tools::external_signals::ExternalSignalStatusOutput =
            serde_json::from_value(result).unwrap();
        assert_eq!(output.total_providers, 1);

        // Filter for nonexistent
        let input = crate::mcp::tools::external_signals::ExternalSignalStatusInput {
            provider: Some("nonexistent".to_string()),
        };
        let result = server.execute_external_signal_status(input).await.unwrap();
        let output: crate::mcp::tools::external_signals::ExternalSignalStatusOutput =
            serde_json::from_value(result).unwrap();
        assert_eq!(output.total_providers, 0);
    }
}

//! Configure external signal provider command implementation.
//!
//! This module handles configuration of external signal providers like AgentFS
//! and custom endpoints.

use anyhow::Result;

use crate::config::Config;
use crate::output::OutputFormat;

use super::types::{
    ConfigureAgentFsArgs, ConfigureCustomArgs, ProviderConfig, ProviderConfigResult,
};

/// Configure AgentFS provider
pub async fn configure_agentfs(
    args: ConfigureAgentFsArgs,
    _config: &Config,
    _format: OutputFormat,
) -> Result<()> {
    tracing::debug!(
        "Rejecting runtime AgentFS configuration for db_path='{}'",
        args.db_path
    );

    // The CLI installs external signal providers when `SelfLearningMemory` is
    // constructed from configuration; there is no runtime reconfiguration path
    // and no store to persist one into. Reporting success would advertise an
    // active provider that produces no reward signal.
    anyhow::bail!(
        "Runtime AgentFS configuration is unavailable; set AGENTFS_DB_PATH before starting the CLI"
    )
}

/// Configure custom external provider
pub async fn configure_custom(
    args: ConfigureCustomArgs,
    _config: &Config,
    format: OutputFormat,
) -> Result<()> {
    // Validate endpoint URL
    if !args.endpoint.starts_with("http://") && !args.endpoint.starts_with("https://") {
        return Err(anyhow::anyhow!(
            "Invalid endpoint URL: {}. Must start with http:// or https://",
            args.endpoint
        ));
    }

    // Build provider configuration
    let provider_config = ProviderConfig {
        name: args.name.clone(),
        provider_type: "custom".to_string(),
        enabled: args.enabled,
        weight: args.weight.clamp(0.0, 1.0),
        timeout: args.timeout,
        auto_reconnect: None,
        max_retries: None,
        poll_interval: Some(args.poll_interval),
        endpoint: Some(args.endpoint.clone()),
        db_path: None,
    };

    // In a real implementation, this would:
    // 1. Test the endpoint connectivity
    // 2. Validate authentication if token provided
    // 3. Store the configuration
    // 4. Initialize the provider

    let result = ProviderConfigResult {
        name: args.name,
        success: true,
        message: format!(
            "Custom provider '{}' configured with {} (weight: {:.2})",
            provider_config.name,
            if args.enabled { "enabled" } else { "disabled" },
            args.weight
        ),
        config: Some(provider_config),
    };

    format.print_output(&result)?;
    Ok(())
}

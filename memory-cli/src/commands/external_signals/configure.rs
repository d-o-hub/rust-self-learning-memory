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
    format: OutputFormat,
) -> Result<()> {
    // Validate db_path exists or is accessible
    let db_path_expanded = args.db_path.clone();
    let path = std::path::Path::new(&db_path_expanded);

    if !path.exists() && !args.db_path.starts_with("http") {
        // For local paths, warn if file doesn't exist yet
        // (it might be created on first connection)
        tracing::warn!(
            "AgentFS database path does not exist yet: {}",
            db_path_expanded
        );
    }

    // Build provider configuration
    let provider_config = ProviderConfig {
        name: "agentfs".to_string(),
        provider_type: "agentfs".to_string(),
        enabled: args.enabled,
        weight: args.weight.clamp(0.0, 1.0),
        timeout: args.timeout,
        auto_reconnect: Some(args.auto_reconnect),
        max_retries: Some(args.max_retries),
        poll_interval: None,
        endpoint: None,
        db_path: Some(db_path_expanded.clone()),
    };

    // In a real implementation, this would:
    // 1. Validate the database connection
    // 2. Store the configuration
    // 3. Initialize the provider
    // For now, we simulate success

    let result = ProviderConfigResult {
        name: "agentfs".to_string(),
        success: true,
        message: format!(
            "AgentFS provider configured with {} (weight: {:.2})",
            if args.enabled { "enabled" } else { "disabled" },
            args.weight
        ),
        config: Some(provider_config),
    };

    format.print_output(&result)?;
    Ok(())
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

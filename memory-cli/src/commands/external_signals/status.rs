//! External signal provider status command implementation.
//!
//! This module handles showing the status of configured external signal providers.

use anyhow::Result;
use chrono::Utc;

use crate::config::Config;
use crate::output::OutputFormat;

use super::types::{ExternalProviderInfo, ProviderStatus, ProviderStatusResponse};

/// Show status of external signal providers
pub async fn show_status(
    provider_name: Option<String>,
    _memory: &do_memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> Result<()> {
    // In a real implementation, this would query the memory system
    // for actual configured providers. For now, we simulate some
    // example providers to demonstrate the interface.

    let providers = match provider_name {
        Some(name) => {
            // Show specific provider
            vec![get_simulated_provider(&name)]
        }
        None => {
            // Show all providers
            get_all_simulated_providers()
        }
    };

    let total_enabled = providers.iter().filter(|p| p.enabled).count();
    let total_connected = providers
        .iter()
        .filter(|p| p.enabled && p.status == ProviderStatus::Connected)
        .count();

    let overall_status = determine_overall_status(&providers);

    let response = ProviderStatusResponse {
        providers,
        total_enabled,
        total_connected,
        overall_status,
    };

    format.print_output(&response)?;
    Ok(())
}

/// Determine overall status based on individual provider statuses
fn determine_overall_status(providers: &[ExternalProviderInfo]) -> ProviderStatus {
    if providers.is_empty() {
        return ProviderStatus::Unknown;
    }

    let enabled_providers: Vec<_> = providers.iter().filter(|p| p.enabled).collect();

    if enabled_providers.is_empty() {
        return ProviderStatus::Disconnected;
    }

    let connected_count = enabled_providers
        .iter()
        .filter(|p| p.status == ProviderStatus::Connected)
        .count();
    let error_count = enabled_providers
        .iter()
        .filter(|p| p.status == ProviderStatus::Error)
        .count();

    if connected_count == enabled_providers.len() {
        ProviderStatus::Connected
    } else if error_count > 0 && error_count == enabled_providers.len() {
        ProviderStatus::Error
    } else if connected_count > 0 {
        ProviderStatus::Degraded
    } else {
        ProviderStatus::Disconnected
    }
}

/// Get simulated provider for demonstration
fn get_simulated_provider(name: &str) -> ExternalProviderInfo {
    match name.to_lowercase().as_str() {
        "agentfs" => ExternalProviderInfo {
            name: "agentfs".to_string(),
            provider_type: "agentfs".to_string(),
            enabled: true,
            weight: 0.3,
            status: ProviderStatus::Connected,
            last_connected: Some(Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()),
            last_error: None,
            total_signals: 1_247,
            avg_latency_ms: Some(45),
        },
        "custom-api" => ExternalProviderInfo {
            name: "custom-api".to_string(),
            provider_type: "custom".to_string(),
            enabled: true,
            weight: 0.5,
            status: ProviderStatus::Degraded,
            last_connected: Some(
                (Utc::now() - chrono::Duration::minutes(5))
                    .format("%Y-%m-%d %H:%M:%S UTC")
                    .to_string(),
            ),
            last_error: Some("Connection timeout after 30s".to_string()),
            total_signals: 856,
            avg_latency_ms: Some(2_500),
        },
        _ => ExternalProviderInfo {
            name: name.to_string(),
            provider_type: "unknown".to_string(),
            enabled: false,
            weight: 0.0,
            status: ProviderStatus::Unknown,
            last_connected: None,
            last_error: Some("Provider not configured".to_string()),
            total_signals: 0,
            avg_latency_ms: None,
        },
    }
}

/// Get all simulated providers for demonstration
fn get_all_simulated_providers() -> Vec<ExternalProviderInfo> {
    vec![
        ExternalProviderInfo {
            name: "agentfs".to_string(),
            provider_type: "agentfs".to_string(),
            enabled: true,
            weight: 0.3,
            status: ProviderStatus::Connected,
            last_connected: Some(Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()),
            last_error: None,
            total_signals: 1_247,
            avg_latency_ms: Some(45),
        },
        ExternalProviderInfo {
            name: "custom-api".to_string(),
            provider_type: "custom".to_string(),
            enabled: true,
            weight: 0.5,
            status: ProviderStatus::Degraded,
            last_connected: Some(
                (Utc::now() - chrono::Duration::minutes(5))
                    .format("%Y-%m-%d %H:%M:%S UTC")
                    .to_string(),
            ),
            last_error: Some("Connection timeout after 30s".to_string()),
            total_signals: 856,
            avg_latency_ms: Some(2_500),
        },
    ]
}

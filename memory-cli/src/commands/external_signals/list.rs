//! List external signal providers command implementation.
//!
//! This module handles listing all configured external signal providers.

use anyhow::Result;

use crate::config::Config;
use crate::output::OutputFormat;

use super::types::{ProviderListItem, ProviderListResponse, ProviderStatus};

/// List all configured external signal providers
pub async fn list_providers(
    detailed: bool,
    _memory: &do_memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> Result<()> {
    // In a real implementation, this would query the memory system
    // for actual configured providers. For now, we simulate some
    // example providers to demonstrate the interface.

    let providers = if detailed {
        get_detailed_provider_list()
    } else {
        get_simple_provider_list()
    };

    let total = providers.len();
    let enabled = providers.iter().filter(|p| p.enabled).count();

    let response = ProviderListResponse {
        providers,
        total,
        enabled,
    };

    format.print_output(&response)?;
    Ok(())
}

/// Get simple list of providers
fn get_simple_provider_list() -> Vec<ProviderListItem> {
    vec![
        ProviderListItem {
            name: "agentfs".to_string(),
            provider_type: "agentfs".to_string(),
            enabled: true,
            status: ProviderStatus::Connected,
            weight: 0.3,
            config_summary: "~/.agentfs/db.sqlite".to_string(),
        },
        ProviderListItem {
            name: "custom-api".to_string(),
            provider_type: "custom".to_string(),
            enabled: true,
            status: ProviderStatus::Degraded,
            weight: 0.5,
            config_summary: "https://api.example.com/signals".to_string(),
        },
    ]
}

/// Get detailed list of providers with full configuration
fn get_detailed_provider_list() -> Vec<ProviderListItem> {
    vec![
        ProviderListItem {
            name: "agentfs".to_string(),
            provider_type: "agentfs".to_string(),
            enabled: true,
            status: ProviderStatus::Connected,
            weight: 0.3,
            config_summary: "path=~/.agentfs/db.sqlite, timeout=30s, retries=3, auto_reconnect=true, signals=1,247".to_string(),
        },
        ProviderListItem {
            name: "custom-api".to_string(),
            provider_type: "custom".to_string(),
            enabled: true,
            status: ProviderStatus::Degraded,
            weight: 0.5,
            config_summary: "endpoint=https://api.example.com/signals, timeout=30s, poll=60s, signals=856, last_error=TIMEOUT".to_string(),
        },
    ]
}

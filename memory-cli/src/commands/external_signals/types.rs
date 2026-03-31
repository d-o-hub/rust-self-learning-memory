//! External signal types and data structures.
//!
//! This module defines the data structures for external signal provider
//! configuration, status, and test results.

use clap::Subcommand;
use serde::{Deserialize, Serialize};

use crate::output::Output;

/// Commands for managing external signal providers
#[derive(Subcommand)]
pub enum ExternalSignalCommands {
    /// Configure an external signal provider
    #[command(alias = "cfg")]
    Configure {
        /// Provider type to configure
        #[command(subcommand)]
        provider: ConfigureProviderArgs,
    },
    /// Show external signal provider status
    #[command(alias = "st")]
    Status {
        /// Provider name (if omitted, shows all)
        #[arg(short, long)]
        provider: Option<String>,
    },
    /// Test connection to external signal providers
    #[command(alias = "tst")]
    Test {
        /// Provider name (if omitted, tests all)
        #[arg(short, long)]
        provider: Option<String>,
    },
    /// List all configured external signal providers
    #[command(alias = "ls")]
    List {
        /// Show detailed configuration
        #[arg(short, long)]
        detailed: bool,
    },
}

/// Configure provider subcommands
#[derive(Subcommand)]
pub enum ConfigureProviderArgs {
    /// Configure AgentFS provider
    #[command(name = "agentfs")]
    AgentFs(ConfigureAgentFsArgs),
    /// Configure a custom external signal provider
    #[command(name = "custom")]
    Custom(ConfigureCustomArgs),
}

/// Arguments for configuring AgentFS provider
#[derive(Args, Debug, Clone)]
pub struct ConfigureAgentFsArgs {
    /// Path to the AgentFS database
    #[arg(long)]
    pub db_path: String,
    /// Whether this provider is enabled
    #[arg(long, default_value_t = true)]
    pub enabled: bool,
    /// Weight for signal aggregation (0.0 to 1.0)
    #[arg(long, default_value_t = 0.3)]
    pub weight: f32,
    /// Connection timeout in seconds
    #[arg(long, default_value_t = 30)]
    pub timeout: u64,
    /// Enable automatic reconnection
    #[arg(long, default_value_t = true)]
    pub auto_reconnect: bool,
    /// Max retry attempts for failed connections
    #[arg(long, default_value_t = 3)]
    pub max_retries: u32,
}

/// Arguments for configuring a custom external provider
#[derive(Args, Debug, Clone)]
pub struct ConfigureCustomArgs {
    /// Provider name (unique identifier)
    #[arg(long)]
    pub name: String,
    /// Provider type/endpoint URL
    #[arg(long)]
    pub endpoint: String,
    /// Authentication token or API key
    #[arg(long)]
    pub token: Option<String>,
    /// Whether this provider is enabled
    #[arg(long, default_value_t = true)]
    pub enabled: bool,
    /// Weight for signal aggregation (0.0 to 1.0)
    #[arg(long, default_value_t = 0.5)]
    pub weight: f32,
    /// Connection timeout in seconds
    #[arg(long, default_value_t = 30)]
    pub timeout: u64,
    /// Polling interval in seconds (0 for no polling)
    #[arg(long, default_value_t = 60)]
    pub poll_interval: u64,
}

/// External signal provider information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExternalProviderInfo {
    pub name: String,
    pub provider_type: String,
    pub enabled: bool,
    pub weight: f32,
    pub status: ProviderStatus,
    pub last_connected: Option<String>,
    pub last_error: Option<String>,
    pub total_signals: u64,
    pub avg_latency_ms: Option<u64>,
}

/// Provider connection status
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderStatus {
    Connected,
    Disconnected,
    Degraded,
    Error,
    Unknown,
}

/// Provider configuration details
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderConfig {
    pub name: String,
    pub provider_type: String,
    pub enabled: bool,
    pub weight: f32,
    pub timeout: u64,
    pub auto_reconnect: Option<bool>,
    pub max_retries: Option<u32>,
    pub poll_interval: Option<u64>,
    pub endpoint: Option<String>,
    pub db_path: Option<String>,
}

/// Status response for all providers
#[derive(Debug, Serialize)]
pub struct ProviderStatusResponse {
    pub providers: Vec<ExternalProviderInfo>,
    pub total_enabled: usize,
    pub total_connected: usize,
    pub overall_status: ProviderStatus,
}

/// Test result for a single provider
#[derive(Debug, Serialize)]
pub struct ProviderTestResult {
    pub name: String,
    pub success: bool,
    pub status: ProviderStatus,
    pub latency_ms: u64,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Test response for all providers
#[derive(Debug, Serialize)]
pub struct ProviderTestResponse {
    pub results: Vec<ProviderTestResult>,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub duration_ms: u64,
}

/// List response for configured providers
#[derive(Debug, Serialize)]
pub struct ProviderListResponse {
    pub providers: Vec<ProviderListItem>,
    pub total: usize,
    pub enabled: usize,
}

/// Single provider in list view
#[derive(Debug, Serialize)]
pub struct ProviderListItem {
    pub name: String,
    pub provider_type: String,
    pub enabled: bool,
    pub status: ProviderStatus,
    pub weight: f32,
    pub config_summary: String,
}

impl std::fmt::Display for ProviderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderStatus::Connected => write!(f, "Connected"),
            ProviderStatus::Disconnected => write!(f, "Disconnected"),
            ProviderStatus::Degraded => write!(f, "Degraded"),
            ProviderStatus::Error => write!(f, "Error"),
            ProviderStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

impl Output for ProviderStatusResponse {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "External Signal Provider Status".bold())?;
        writeln!(writer, "{}", "─".repeat(50))?;
        writeln!(
            writer,
            "Overall: {} ({}/{} connected)",
            match self.overall_status {
                ProviderStatus::Connected => "Connected".green(),
                ProviderStatus::Disconnected => "Disconnected".red(),
                ProviderStatus::Degraded => "Degraded".yellow(),
                ProviderStatus::Error => "Error".red(),
                ProviderStatus::Unknown => "Unknown".yellow(),
            },
            self.total_connected,
            self.total_enabled
        )?;
        writeln!(writer)?;

        if self.providers.is_empty() {
            writeln!(writer, "{}", "No providers configured.".yellow())?;
            writeln!(writer)?;
            writeln!(writer, "To configure a provider:")?;
            writeln!(
                writer,
                "  memory-cli external-signal configure agentfs --db-path /path/to/agentfs.db"
            )?;
        } else {
            for provider in &self.providers {
                let status_color = match provider.status {
                    ProviderStatus::Connected => Color::Green,
                    ProviderStatus::Disconnected => Color::Red,
                    ProviderStatus::Degraded => Color::Yellow,
                    ProviderStatus::Error => Color::Red,
                    ProviderStatus::Unknown => Color::Yellow,
                };

                writeln!(writer, "{}", provider.name.bold())?;
                writeln!(
                    writer,
                    "  Type: {} | Weight: {:.2}",
                    provider.provider_type, provider.weight
                )?;
                writeln!(
                    writer,
                    "  Status: {} | Enabled: {}",
                    format!("{}", provider.status).color(status_color),
                    if provider.enabled {
                        "Yes".green()
                    } else {
                        "No".red()
                    }
                )?;

                if let Some(latency) = provider.avg_latency_ms {
                    writeln!(writer, "  Avg Latency: {}ms", latency)?;
                }
                if let Some(last) = &provider.last_connected {
                    writeln!(writer, "  Last Connected: {}", last)?;
                }
                if let Some(error) = &provider.last_error {
                    writeln!(writer, "  {}{}", "Last Error: ".red(), error.red())?;
                }
                if provider.total_signals > 0 {
                    writeln!(writer, "  Total Signals: {}", provider.total_signals)?;
                }
                writeln!(writer)?;
            }
        }

        Ok(())
    }
}

impl Output for ProviderTestResponse {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "External Signal Provider Tests".bold())?;
        writeln!(writer, "{}", "─".repeat(50))?;
        writeln!(
            writer,
            "Results: {}/{} passed in {}ms",
            self.passed.to_string().green(),
            self.total,
            self.duration_ms
        )?;
        writeln!(writer)?;

        for result in &self.results {
            let status_symbol = if result.success {
                "✅".green()
            } else {
                "❌".red()
            };

            writeln!(
                writer,
                "{} {} ({}ms)",
                status_symbol,
                result.name.bold(),
                result.latency_ms
            )?;
            writeln!(writer, "   Status: {}", result.message)?;
            Self::write_details(&result.details, &mut writer)?;
            writeln!(writer)?;
        }

        Ok(())
    }
}

impl ProviderTestResponse {
    fn write_details<W: std::io::Write>(
        details: &Option<serde_json::Value>,
        writer: &mut W,
    ) -> anyhow::Result<()> {
        use colored::*;

        if let Some(details) = details {
            if let Ok(pretty) = serde_json::to_string_pretty(details) {
                for line in pretty.lines() {
                    writeln!(writer, "   {}", line.dimmed())?;
                }
            }
        }
        Ok(())
    }
}

impl Output for ProviderListResponse {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "Configured External Signal Providers".bold())?;
        writeln!(writer, "{}", "─".repeat(50))?;
        writeln!(writer, "Total: {} | Enabled: {}", self.total, self.enabled)?;
        writeln!(writer)?;

        if self.providers.is_empty() {
            writeln!(writer, "{}", "No providers configured.".yellow())?;
            writeln!(writer)?;
            writeln!(writer, "To add a provider:")?;
            writeln!(
                writer,
                "  memory-cli external-signal configure agentfs --db-path <path>"
            )?;
            writeln!(
                writer,
                "  memory-cli external-signal configure custom --name <name> --endpoint <url>"
            )?;
        } else {
            for provider in &self.providers {
                let status_color = match provider.status {
                    ProviderStatus::Connected => Color::Green,
                    ProviderStatus::Disconnected => Color::Red,
                    ProviderStatus::Degraded => Color::Yellow,
                    ProviderStatus::Error => Color::Red,
                    ProviderStatus::Unknown => Color::Yellow,
                };

                let status_symbol = match provider.status {
                    ProviderStatus::Connected => "●".green(),
                    ProviderStatus::Disconnected => "○".red(),
                    ProviderStatus::Degraded => "◐".yellow(),
                    ProviderStatus::Error => "✗".red(),
                    ProviderStatus::Unknown => "?".yellow(),
                };

                writeln!(
                    writer,
                    "{} {} ({}) - {} - weight: {:.2}",
                    status_symbol,
                    provider.name.bold(),
                    provider.provider_type.dimmed(),
                    if provider.enabled {
                        "enabled".green()
                    } else {
                        "disabled".red()
                    },
                    provider.weight
                )?;
                writeln!(
                    writer,
                    "   {} {}",
                    "Status:".dimmed(),
                    format!("{}", provider.status).color(status_color)
                )?;
                if !provider.config_summary.is_empty() {
                    writeln!(
                        writer,
                        "   {} {}",
                        "Config:".dimmed(),
                        provider.config_summary
                    )?;
                }
            }
        }

        Ok(())
    }
}

/// Configuration result for a single provider
#[derive(Debug, Serialize)]
pub struct ProviderConfigResult {
    pub name: String,
    pub success: bool,
    pub message: String,
    pub config: Option<ProviderConfig>,
}

impl Output for ProviderConfigResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        if self.success {
            writeln!(
                writer,
                "{} {} configured successfully",
                "✅".green(),
                self.name.bold()
            )?;
            writeln!(writer, "{}", self.message)?;

            if let Some(config) = &self.config {
                writeln!(writer)?;
                writeln!(writer, "{}", "Configuration:".dimmed())?;
                writeln!(writer, "  Type: {}", config.provider_type)?;
                writeln!(writer, "  Enabled: {}", config.enabled)?;
                writeln!(writer, "  Weight: {:.2}", config.weight)?;
                writeln!(writer, "  Timeout: {}s", config.timeout)?;

                if let Some(auto_reconnect) = config.auto_reconnect {
                    writeln!(writer, "  Auto-reconnect: {}", auto_reconnect)?;
                }
                if let Some(max_retries) = config.max_retries {
                    writeln!(writer, "  Max retries: {}", max_retries)?;
                }
                if let Some(poll_interval) = config.poll_interval {
                    writeln!(writer, "  Poll interval: {}s", poll_interval)?;
                }
                if let Some(endpoint) = &config.endpoint {
                    writeln!(writer, "  Endpoint: {}", endpoint)?;
                }
                if let Some(db_path) = &config.db_path {
                    writeln!(writer, "  DB Path: {}", db_path)?;
                }
            }
        } else {
            writeln!(
                writer,
                "{} {} configuration failed",
                "❌".red(),
                self.name.bold()
            )?;
            writeln!(writer, "{}", self.message.red())?;
        }

        Ok(())
    }
}

// Import Args from clap for derive macro
use clap::Args;

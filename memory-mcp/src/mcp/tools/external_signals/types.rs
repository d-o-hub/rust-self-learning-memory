//! External signal provider types and input/output structures.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Input parameters for configuring AgentFS provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureAgentFsInput {
    /// Path to the AgentFS database file
    pub db_path: String,
    /// Whether the provider is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Weight for merging external signals (0.0-1.0)
    #[serde(default = "default_weight")]
    pub weight: f32,
    /// Minimum samples required before using external signals
    #[serde(default = "default_min_samples")]
    pub min_samples: usize,
    /// Whether to sanitize/summarize toolcall parameters for privacy
    #[serde(default = "default_sanitize")]
    pub sanitize: bool,
}

fn default_enabled() -> bool {
    true
}

fn default_weight() -> f32 {
    0.3
}

fn default_min_samples() -> usize {
    5
}

fn default_sanitize() -> bool {
    true
}

/// Output from configuring AgentFS provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureAgentFsOutput {
    /// Whether configuration was successful
    pub success: bool,
    /// Provider name
    pub provider: String,
    /// Database path configured
    pub db_path: String,
    /// Whether the provider is enabled
    pub enabled: bool,
    /// Weight used for merging
    pub weight: f32,
    /// Minimum samples required
    pub min_samples: usize,
    /// Whether sanitization is enabled
    pub sanitize: bool,
    /// Configuration message
    pub message: String,
    /// Warnings (if any)
    pub warnings: Vec<String>,
}

/// Input parameters for checking external signal provider status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSignalStatusInput {
    /// Optional provider name to filter by (e.g., "agentfs")
    pub provider: Option<String>,
}

/// Status information for a single provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    /// Provider name
    pub name: String,
    /// Whether the provider is configured
    pub configured: bool,
    /// Whether the provider is enabled
    pub enabled: bool,
    /// Whether the provider is currently connected
    pub connected: bool,
    /// Last error message (if any)
    pub last_error: Option<String>,
    /// Number of signals available from this provider
    pub signal_count: usize,
    /// Weight for signal merging
    pub weight: f32,
    /// Provider-specific metadata
    pub metadata: Value,
}

/// Output from checking external signal provider status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSignalStatusOutput {
    /// Total number of configured providers
    pub total_providers: usize,
    /// Number of active (enabled and connected) providers
    pub active_providers: usize,
    /// Status details for each provider
    pub providers: Vec<ProviderStatus>,
}

/// Input parameters for testing AgentFS connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestAgentFsConnectionInput {
    /// Optional database path override (uses configured path if not provided)
    pub db_path: Option<String>,
}

/// Output from testing AgentFS connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestAgentFsConnectionOutput {
    /// Whether the test was successful
    pub success: bool,
    /// Provider name
    pub provider: String,
    /// Database path tested
    pub db_path: String,
    /// Connection test duration in milliseconds
    pub connection_time_ms: u64,
    /// Whether the database is readable
    pub readable: bool,
    /// Whether the database is writable
    pub writable: bool,
    /// Number of toolcall records found (if accessible)
    pub toolcall_count: Option<usize>,
    /// Provider version (if available)
    pub version: Option<String>,
    /// Test status message
    pub message: String,
    /// Error message (if test failed)
    pub error: Option<String>,
}

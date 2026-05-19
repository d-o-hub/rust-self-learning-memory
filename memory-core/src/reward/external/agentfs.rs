//! AgentFS external signal provider implementation
//!
//! This module provides the AgentFS SDK integration for external signals.
//!
//! ## Current Status
//!
//! - **SDK Integrated**: Yes (v0.6.4)
//! - **Functional**: Fetches real tool statistics from AgentFS SQLite database
//! - **Error Handling**: Graceful degradation if database is unavailable
//!
//! See ADR-050 for full integration plan.

use agentfs_sdk::ToolCalls;
use async_trait::async_trait;
use std::collections::HashMap;

use super::{
    ExternalSignalError, ExternalSignalProvider, ExternalSignalSet, ProviderHealth, Result,
    ToolSignal,
};

/// Configuration for AgentFS provider
#[derive(Debug, Clone)]
pub struct AgentFsConfig {
    /// Path to AgentFS SQLite database
    pub db_path: String,
    /// Enable the provider
    pub enabled: bool,
    /// Weight for this provider's signals (0.0-1.0)
    pub external_weight: f32,
    /// Minimum sample size for correlation
    pub min_correlation_samples: usize,
    /// Sanitize parameters before storing
    pub sanitize_parameters: bool,
}

impl AgentFsConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        use std::env;

        let db_path = env::var("AGENTFS_DB_PATH").map_err(|_| {
            ExternalSignalError::ConfigMissing(
                "AGENTFS_DB_PATH environment variable not set".to_string(),
            )
        })?;

        let enabled = env::var("AGENTFS_ENABLED")
            .map(|v| v == "true")
            .unwrap_or(false);

        let external_weight: f32 = env::var("AGENTFS_WEIGHT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.3);

        let min_correlation_samples = env::var("AGENTFS_MIN_SAMPLES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        let sanitize_parameters = env::var("AGENTFS_SANITIZE")
            .map(|v| v == "true")
            .unwrap_or(true);

        Ok(Self {
            db_path,
            enabled,
            external_weight: external_weight.clamp(0.0, 1.0),
            min_correlation_samples,
            sanitize_parameters,
        })
    }
}

impl Default for AgentFsConfig {
    fn default() -> Self {
        Self {
            db_path: String::new(),
            enabled: false,
            external_weight: 0.3,
            min_correlation_samples: 10,
            sanitize_parameters: true,
        }
    }
}

/// AgentFS external signal provider
pub struct AgentFsProvider {
    config: AgentFsConfig,
}

impl AgentFsProvider {
    /// Create a new AgentFS provider with configuration
    pub fn new(config: AgentFsConfig) -> Self {
        Self { config }
    }

    /// Get the configuration
    pub fn config(&self) -> &AgentFsConfig {
        &self.config
    }

    /// Sanitize parameters for privacy
    #[allow(dead_code)] // Privacy utility for future external reward API
    fn sanitize_parameters(&self, params: &serde_json::Value) -> serde_json::Value {
        if !self.config.sanitize_parameters {
            return params.clone();
        }

        match params {
            serde_json::Value::Object(map) => {
                let mut sanitized = serde_json::Map::new();
                for (key, _) in map {
                    // Keep keys, redact values
                    sanitized.insert(
                        key.clone(),
                        serde_json::Value::String("[REDACTED]".to_string()),
                    );
                }
                serde_json::Value::Object(sanitized)
            }
            _ => serde_json::Value::String("[REDACTED]".to_string()),
        }
    }
}

#[async_trait]
impl ExternalSignalProvider for AgentFsProvider {
    fn name(&self) -> &'static str {
        "agentfs"
    }

    async fn get_signals(&self, episode: &crate::episode::Episode) -> Result<ExternalSignalSet> {
        // Disabled provider returns empty signals (graceful degradation)
        if !self.config.enabled {
            return Ok(ExternalSignalSet::empty("agentfs"));
        }

        if self.config.db_path.is_empty() {
            return Err(ExternalSignalError::ConfigMissing(
                "AgentFS database path not configured".to_string(),
            ));
        }

        let tool_signals = self.fetch_tool_stats(episode).await;

        // Calculate confidence based on sample sizes
        let total_samples: usize = tool_signals.iter().map(|t| t.sample_count).sum();

        // 100 samples = 1.0 confidence, 0 samples = 0.0
        let confidence = (total_samples as f32 / 100.0).min(1.0);

        Ok(ExternalSignalSet {
            provider: "agentfs".to_string(),
            tool_signals,
            episode_quality: None, // Could be derived from success rates if needed
            timestamp: chrono::Utc::now(),
            confidence,
        })
    }

    async fn health_check(&self) -> ProviderHealth {
        // Disabled provider is healthy (graceful degradation)
        if !self.config.enabled {
            return ProviderHealth::Healthy;
        }

        if self.config.db_path.is_empty() {
            return ProviderHealth::Unhealthy("Database path not configured".to_string());
        }

        match ToolCalls::new(&self.config.db_path).await {
            Ok(_) => ProviderHealth::Healthy,
            Err(e) => ProviderHealth::Unhealthy(format!("AgentFS connection failed: {e}")),
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.db_path.is_empty() && self.config.enabled {
            return Err(ExternalSignalError::ConfigMissing(
                "AgentFS database path required when enabled".to_string(),
            ));
        }

        if self.config.external_weight < 0.0 || self.config.external_weight > 1.0 {
            return Err(ExternalSignalError::InvalidConfig(format!(
                "Weight must be between 0.0 and 1.0, got {}",
                self.config.external_weight
            )));
        }

        Ok(())
    }
}

impl AgentFsProvider {
    /// Fetch tool statistics for an episode
    ///
    /// This method queries the AgentFS database via the SDK for toolcall statistics
    /// matching the tools used in the provided episode.
    async fn fetch_tool_stats(&self, episode: &crate::episode::Episode) -> Vec<ToolSignal> {
        let Ok(tc) = ToolCalls::new(&self.config.db_path).await else {
            return Vec::new();
        };

        // Get unique tool names from episode steps as &str for efficient lookups
        let episode_tools: std::collections::HashSet<&str> =
            episode.steps.iter().map(|s| s.tool.as_str()).collect();

        // Query all tool stats once for efficiency
        let Ok(all_stats) = tc.stats().await else {
            return Vec::new();
        };

        all_stats
            .into_iter()
            .filter(|stats| {
                episode_tools.contains(stats.name.as_str())
                    && stats.total_calls >= self.config.min_correlation_samples as i64
            })
            .map(|stats| {
                // Total calls guaranteed to be > 0 due to filter above
                let success_rate = stats.successful as f32 / stats.total_calls as f32;

                let mut metadata = HashMap::new();
                metadata.insert("failed".to_string(), serde_json::json!(stats.failed));
                metadata.insert("provider".to_string(), serde_json::json!("agentfs"));

                ToolSignal {
                    tool_name: stats.name,
                    success_rate,
                    avg_latency_ms: stats.avg_duration_ms,
                    sample_count: stats.total_calls as usize,
                    metadata,
                }
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "agentfs_tests.rs"]
mod tests;

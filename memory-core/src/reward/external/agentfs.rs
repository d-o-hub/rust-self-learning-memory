//! AgentFS external signal provider implementation
//!
//! This module integrates the AgentFS SDK (agentfs-sdk) to fetch
//! toolcall audit trails as external reward signals.

use async_trait::async_trait;
use chrono::Utc;
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
    #[allow(dead_code)]
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
        if !self.config.enabled {
            return Ok(ExternalSignalSet::empty("agentfs"));
        }

        if self.config.db_path.is_empty() {
            return Err(ExternalSignalError::ConfigMissing(
                "AgentFS database path not configured".to_string(),
            ));
        }

        // Fetch toolcall stats from AgentFS
        // Note: This is a placeholder for actual AgentFS SDK integration
        // The actual implementation would use agentfs_sdk::ToolCalls
        let tool_signals = self.fetch_tool_stats(episode);

        // Calculate confidence based on sample sizes
        let total_samples: usize = tool_signals.iter().map(|t| t.sample_count).sum();

        let confidence = if total_samples == 0 {
            0.0
        } else {
            (total_samples as f32 / 100.0).min(1.0) // Cap at 1.0 for 100+ samples
        };

        // Calculate episode quality from tool success rates
        let episode_quality = if tool_signals.is_empty() {
            None
        } else {
            let avg_success: f32 = tool_signals.iter().map(|t| t.success_rate).sum::<f32>()
                / tool_signals.len() as f32;
            Some(avg_success)
        };

        Ok(ExternalSignalSet {
            provider: "agentfs".to_string(),
            tool_signals,
            episode_quality,
            timestamp: Utc::now(),
            confidence,
        })
    }

    async fn health_check(&self) -> ProviderHealth {
        if !self.config.enabled {
            return ProviderHealth::Healthy; // Disabled is OK
        }

        if self.config.db_path.is_empty() {
            return ProviderHealth::Unhealthy("Database path not configured".to_string());
        }

        // Check if database file exists
        match tokio::fs::metadata(&self.config.db_path).await {
            Ok(metadata) => {
                if metadata.is_file() {
                    ProviderHealth::Healthy
                } else {
                    ProviderHealth::Unhealthy(format!(
                        "Path is not a file: {}",
                        self.config.db_path
                    ))
                }
            }
            Err(e) => ProviderHealth::Unhealthy(format!("Cannot access database: {}", e)),
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.config.db_path.is_empty() {
            return Err(ExternalSignalError::ConfigMissing(
                "AgentFS database path required".to_string(),
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
    /// This method correlates episode steps with AgentFS toolcall history.
    fn fetch_tool_stats(&self, episode: &crate::episode::Episode) -> Vec<ToolSignal> {
        let mut signals = Vec::new();

        // For each tool used in the episode, query AgentFS stats
        for step in &episode.steps {
            // In the actual implementation, this would query AgentFS SDK:
            // let tc = agentfs_sdk::ToolCalls::new(&self.config.db_path).await?;
            // let stats = tc.stats_for(&step.tool).await?;

            // Placeholder: Create synthetic signal for structure
            // Real implementation would fetch from AgentFS
            let signal = ToolSignal {
                tool_name: step.tool.clone(),
                success_rate: 0.85, // Placeholder - would come from AgentFS
                avg_latency_ms: step.latency_ms as f64,
                sample_count: self.config.min_correlation_samples + 10, // Placeholder
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("source".to_string(), serde_json::json!("agentfs"));
                    map.insert(
                        "episode_step".to_string(),
                        serde_json::json!(step.step_number),
                    );
                    if self.config.sanitize_parameters {
                        map.insert("sanitized".to_string(), serde_json::json!(true));
                    }
                    map
                },
            };

            signals.push(signal);
        }

        signals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env() {
        // This test requires environment variables to be set
        // In CI, use std::env::set_var for testing

        // For unit testing without env vars, test defaults
        let config = AgentFsConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.external_weight, 0.3);
    }

    #[test]
    fn test_provider_validation() {
        let config = AgentFsConfig {
            db_path: "/tmp/test.db".to_string(),
            enabled: true,
            external_weight: 0.5,
            min_correlation_samples: 10,
            sanitize_parameters: true,
        };

        let provider = AgentFsProvider::new(config);
        assert!(provider.validate_config().is_ok());
    }

    #[test]
    fn test_provider_validation_invalid_weight() {
        let config = AgentFsConfig {
            db_path: "/tmp/test.db".to_string(),
            enabled: true,
            external_weight: 1.5, // Invalid
            min_correlation_samples: 10,
            sanitize_parameters: true,
        };

        let provider = AgentFsProvider::new(config);
        assert!(provider.validate_config().is_err());
    }

    #[tokio::test]
    async fn test_disabled_provider_returns_empty() {
        let config = AgentFsConfig {
            db_path: "/tmp/test.db".to_string(),
            enabled: false,
            external_weight: 0.3,
            min_correlation_samples: 10,
            sanitize_parameters: true,
        };

        let provider = AgentFsProvider::new(config);
        let episode = crate::episode::Episode::new(
            "test".to_string(),
            crate::types::TaskContext::default(),
            crate::types::TaskType::Testing,
        );

        let signals = provider.get_signals(&episode).await.unwrap();
        assert!(signals.tool_signals.is_empty());
        assert_eq!(signals.confidence, 0.0);
    }

    #[test]
    fn test_sanitize_parameters() {
        let config = AgentFsConfig {
            db_path: "/tmp/test.db".to_string(),
            enabled: true,
            external_weight: 0.3,
            min_correlation_samples: 10,
            sanitize_parameters: true,
        };

        let provider = AgentFsProvider::new(config);

        let params = serde_json::json!({
            "query": "sensitive data",
            "api_key": "secret123",
            "limit": 10
        });

        let sanitized = provider.sanitize_parameters(&params);

        if let serde_json::Value::Object(map) = sanitized {
            assert!(map.contains_key("query"));
            assert!(map.contains_key("api_key"));
            assert!(map.contains_key("limit"));
            // All values should be redacted
            for (_, value) in map {
                assert_eq!(value, serde_json::Value::String("[REDACTED]".to_string()));
            }
        } else {
            panic!("Expected object");
        }
    }
}

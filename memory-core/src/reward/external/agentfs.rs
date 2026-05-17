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
        let tc = match ToolCalls::new(&self.config.db_path).await {
            Ok(tc) => tc,
            Err(_) => return Vec::new(),
        };

        // Get unique tool names from episode steps
        let episode_tools: std::collections::HashSet<_> =
            episode.steps.iter().map(|s| &s.tool).collect();

        // Query all tool stats once for efficiency
        let all_stats = match tc.stats().await {
            Ok(stats) => stats,
            Err(_) => return Vec::new(),
        };

        all_stats
            .into_iter()
            .filter(|stats| {
                episode_tools.contains(&stats.name)
                    && stats.total_calls >= self.config.min_correlation_samples as i64
            })
            .map(|stats| {
                let success_rate = if stats.total_calls > 0 {
                    stats.successful as f32 / stats.total_calls as f32
                } else {
                    0.5
                };

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
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = AgentFsConfig::default();
        assert!(!config.enabled, "Default config should be disabled");
        assert_eq!(config.external_weight, 0.3);
        assert!(config.db_path.is_empty(), "Default db_path should be empty");
        assert!(config.sanitize_parameters);
    }

    #[test]
    fn test_provider_creation() {
        let config = AgentFsConfig {
            db_path: "/tmp/test.db".to_string(),
            enabled: false,
            external_weight: 0.5,
            min_correlation_samples: 10,
            sanitize_parameters: true,
        };

        let provider = AgentFsProvider::new(config);
        assert_eq!(provider.name(), "agentfs");
        assert_eq!(provider.config().external_weight, 0.5);
    }

    #[test]
    fn test_config_validation_valid() {
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
    fn test_config_validation_invalid_weight() {
        let config = AgentFsConfig {
            db_path: "/tmp/test.db".to_string(),
            enabled: true,
            external_weight: 1.5, // Invalid: > 1.0
            min_correlation_samples: 10,
            sanitize_parameters: true,
        };

        let provider = AgentFsProvider::new(config);
        let result = provider.validate_config();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExternalSignalError::InvalidConfig(_)
        ));
    }

    #[test]
    fn test_config_validation_missing_path_when_enabled() {
        let config = AgentFsConfig {
            db_path: String::new(), // Empty
            enabled: true,
            external_weight: 0.3,
            min_correlation_samples: 10,
            sanitize_parameters: true,
        };

        let provider = AgentFsProvider::new(config);
        let result = provider.validate_config();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ExternalSignalError::ConfigMissing(_)
        ));
    }

    #[test]
    fn test_config_validation_disabled_no_path_required() {
        // Disabled provider doesn require db_path
        let config = AgentFsConfig {
            db_path: String::new(), // Empty OK when disabled
            enabled: false,
            external_weight: 0.3,
            min_correlation_samples: 10,
            sanitize_parameters: true,
        };

        let provider = AgentFsProvider::new(config);
        assert!(provider.validate_config().is_ok());
    }

    #[tokio::test]
    async fn test_disabled_provider_returns_empty_signals() {
        let config = AgentFsConfig {
            db_path: "/tmp/test.db".to_string(),
            enabled: false, // Disabled
            external_weight: 0.3,
            min_correlation_samples: 10,
            sanitize_parameters: true,
        };

        let provider = AgentFsProvider::new(config);
        let episode = crate::episode::Episode::new(
            "test-episode".to_string(),
            crate::types::TaskContext::default(),
            crate::types::TaskType::Testing,
        );

        let signals = provider.get_signals(&episode).await.unwrap();
        assert!(
            signals.tool_signals.is_empty(),
            "Disabled provider should return empty signals"
        );
        assert_eq!(signals.confidence, 0.0);
        assert_eq!(signals.provider, "agentfs");
    }

    #[tokio::test]
    async fn test_enabled_provider_with_invalid_path_returns_error() {
        let config = AgentFsConfig {
            db_path: "/nonexistent/path/to/db".to_string(),
            enabled: true,
            external_weight: 0.3,
            min_correlation_samples: 10,
            sanitize_parameters: true,
        };

        let provider = AgentFsProvider::new(config);
        let health = provider.health_check().await;

        // Should be unhealthy because path doesn't exist/can't connect
        assert!(matches!(health, ProviderHealth::Unhealthy(_)));
    }

    #[tokio::test]
    async fn test_health_check_disabled_returns_healthy() {
        let config = AgentFsConfig {
            db_path: String::new(),
            enabled: false,
            external_weight: 0.3,
            min_correlation_samples: 10,
            sanitize_parameters: true,
        };

        let provider = AgentFsProvider::new(config);
        let health = provider.health_check().await;
        assert!(health.is_healthy(), "Disabled provider should be healthy");
    }

    #[test]
    fn test_sanitize_parameters_object() {
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

        // Check structure preserved
        if let serde_json::Value::Object(map) = sanitized {
            assert!(map.contains_key("query"));
            assert!(map.contains_key("api_key"));
            assert!(map.contains_key("limit"));
            // All values should be redacted
            for (_, value) in map {
                assert_eq!(
                    value,
                    serde_json::Value::String("[REDACTED]".to_string()),
                    "All values should be redacted"
                );
            }
        } else {
            panic!("Expected sanitized result to be an object");
        }
    }

    #[test]
    fn test_sanitize_parameters_disabled() {
        let config = AgentFsConfig {
            db_path: "/tmp/test.db".to_string(),
            enabled: true,
            external_weight: 0.3,
            min_correlation_samples: 10,
            sanitize_parameters: false, // Disabled sanitization
        };

        let provider = AgentFsProvider::new(config);

        let params = serde_json::json!({
            "query": "sensitive data"
        });

        let sanitized = provider.sanitize_parameters(&params);

        // Should return original when sanitization disabled
        assert_eq!(sanitized, params);
    }

    #[test]
    fn test_sanitize_parameters_non_object() {
        let config = AgentFsConfig {
            db_path: "/tmp/test.db".to_string(),
            enabled: true,
            external_weight: 0.3,
            min_correlation_samples: 10,
            sanitize_parameters: true,
        };

        let provider = AgentFsProvider::new(config);

        // Test non-object value
        let params = serde_json::json!("string value");
        let sanitized = provider.sanitize_parameters(&params);
        assert_eq!(
            sanitized,
            serde_json::Value::String("[REDACTED]".to_string())
        );
    }
}


    #[tokio::test]
    async fn test_fetch_tool_stats_with_real_sdk() {
        use tempfile::NamedTempFile;
        let temp_db = NamedTempFile::new().unwrap();
        let db_path = temp_db.path().to_str().unwrap().to_string();

        // Initialize SDK and record some stats
        let tc = ToolCalls::new(&db_path).await.unwrap();
        tc.record("test_tool", 100, true, None).await.unwrap();
        tc.record("test_tool", 200, false, None).await.unwrap();
        tc.record("other_tool", 50, true, None).await.unwrap();

        let config = AgentFsConfig {
            db_path: db_path.clone(),
            enabled: true,
            external_weight: 0.3,
            min_correlation_samples: 1, // Low to match our test data
            sanitize_parameters: true,
        };

        let provider = AgentFsProvider::new(config);

        // Create an episode that uses "test_tool"
        let mut episode = crate::episode::Episode::new(
            "test-episode".to_string(),
            crate::types::TaskContext::default(),
            crate::types::TaskType::Testing,
        );
        episode.add_step(crate::episode::EpisodeStep::new(
            "test_tool".to_string(),
            serde_json::json!({}),
            crate::types::ExecutionResult::Success {
                output: "ok".to_string(),
                artifacts: vec![],
            },
            std::time::Duration::from_millis(100),
        ));

        let stats = provider.fetch_tool_stats(&episode).await;

        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].tool_name, "test_tool");
        assert_eq!(stats[0].sample_count, 2);
        assert_eq!(stats[0].success_rate, 0.5);
    }
}

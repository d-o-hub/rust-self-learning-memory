//! AgentFS external signal provider implementation
//!
//! This module provides a stub implementation for AgentFS SDK integration.
//! The actual `agentfs-sdk` crate exists on crates.io (v0.6.4) but is not
//! currently integrated as a dependency in this project.
//!
//! ## Current Status
//!
//! - **SDK Integrated**: No (stub implementation)
//! - **Functional**: Returns empty signals when enabled (no real data)
//! - **Error Handling**: Returns `SdkUnavailable` error if enabled without SDK
//!
//! ## Future Integration
//!
//! To enable full AgentFS integration:
//! 1. Add `agentfs-sdk = { version = "0.6.4", optional = true }` to Cargo.toml
//! 2. Update feature flag: `agentfs = ["dep:agentfs-sdk"]`
//! 3. Replace stub code with actual SDK calls
//!
//! See ADR-050 for full integration plan.

use async_trait::async_trait;

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

    async fn get_signals(&self, _episode: &crate::episode::Episode) -> Result<ExternalSignalSet> {
        // Disabled provider returns empty signals (graceful degradation)
        if !self.config.enabled {
            return Ok(ExternalSignalSet::empty("agentfs"));
        }

        // SDK not integrated - return error if enabled
        // This prevents misleading placeholder data from being used
        return Err(ExternalSignalError::SdkUnavailable(
            "agentfs-sdk not integrated - enable feature and add SDK dependency to use".to_string(),
        ));

        // The following code would be used when SDK is integrated:
        // if self.config.db_path.is_empty() {
        //     return Err(ExternalSignalError::ConfigMissing(
        //         "AgentFS database path not configured".to_string(),
        //     ));
        // }
        // let tool_signals = self.fetch_tool_stats(episode);
        // ... calculate confidence and quality ...
    }

    async fn health_check(&self) -> ProviderHealth {
        // Disabled provider is healthy (graceful degradation)
        if !self.config.enabled {
            return ProviderHealth::Healthy;
        }

        // SDK not integrated - report degraded status with clear message
        // This is "Degraded" not "Unhealthy" because the system still works
        // without external signals; it just lacks ground truth validation
        return ProviderHealth::Degraded(
            "SDK not integrated - stub implementation, no real signal data available".to_string(),
        );

        // The following code would be used when SDK is integrated:
        // if self.config.db_path.is_empty() {
        //     return ProviderHealth::Unhealthy("Database path not configured".to_string());
        // }
        // match tokio::fs::metadata(&self.config.db_path).await { ... }
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
    /// **STUB IMPLEMENTATION**: This method is not functional without SDK integration.
    /// When `agentfs-sdk` is integrated, this would query the AgentFS database
    /// for toolcall statistics matching the episode's tools.
    ///
    /// Returns empty vector (no real data available from stub).
    #[allow(dead_code)] // Not used until SDK integrated
    fn fetch_tool_stats(&self, _episode: &crate::episode::Episode) -> Vec<ToolSignal> {
        // Stub: No real data available without SDK
        // Real implementation would:
        // let tc = agentfs_sdk::ToolCalls::new(&self.config.db_path).await?;
        // let stats = tc.stats_for(&step.tool).await?;
        Vec::new()
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
    async fn test_enabled_provider_returns_sdk_unavailable_error() {
        let config = AgentFsConfig {
            db_path: "/tmp/test.db".to_string(),
            enabled: true, // Enabled but SDK not integrated
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

        let result = provider.get_signals(&episode).await;
        assert!(
            result.is_err(),
            "Enabled provider without SDK should return error"
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, ExternalSignalError::SdkUnavailable(_)),
            "Error should be SdkUnavailable"
        );
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

    #[tokio::test]
    async fn test_health_check_enabled_returns_degraded() {
        let config = AgentFsConfig {
            db_path: "/tmp/test.db".to_string(),
            enabled: true, // Enabled but SDK not integrated
            external_weight: 0.3,
            min_correlation_samples: 10,
            sanitize_parameters: true,
        };

        let provider = AgentFsProvider::new(config);
        let health = provider.health_check().await;

        // Degraded (not unhealthy) because system works without external signals
        assert!(
            matches!(health, ProviderHealth::Degraded(_)),
            "Enabled provider without SDK should be degraded"
        );
        assert!(health.is_operational(), "Degraded is still operational");
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

//! Provider-specific optimization configuration

use serde::{Deserialize, Serialize};

use super::super::circuit_breaker::CircuitBreakerConfig;

/// Provider-specific optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Request timeout in seconds (None = use default)
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
    /// Maximum number of retry attempts for failed requests
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Base retry delay in milliseconds (exponential backoff)
    #[serde(default = "default_retry_delay_ms")]
    pub retry_delay_ms: u64,
    /// Maximum batch size for this provider (None = use provider default)
    #[serde(default)]
    pub max_batch_size: Option<usize>,
    /// Rate limit: requests per minute (None = no limit)
    #[serde(default)]
    pub rate_limit_rpm: Option<u32>,
    /// Rate limit: tokens per minute (None = no limit)
    #[serde(default)]
    pub rate_limit_tpm: Option<u64>,
    /// Enable request compression (gzip)
    #[serde(default = "default_compression")]
    pub compression_enabled: bool,
    /// Minimum request size (bytes) to trigger compression
    #[serde(default = "default_compression_threshold")]
    pub compression_threshold_bytes: usize,
    /// Connection pool size
    #[serde(default = "default_pool_size")]
    pub connection_pool_size: usize,
    /// Enable circuit breaker
    #[serde(default)]
    pub enable_circuit_breaker: bool,
    /// Circuit breaker configuration
    #[serde(default)]
    pub circuit_breaker_config: Option<CircuitBreakerConfig>,
    /// Enable performance metrics collection
    #[serde(default = "default_enable_metrics")]
    pub enable_metrics: bool,
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_delay_ms() -> u64 {
    1000
}

fn default_compression() -> bool {
    false
}

fn default_pool_size() -> usize {
    10
}

fn default_compression_threshold() -> usize {
    1024
}

fn default_enable_metrics() -> bool {
    true
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: None,
            max_retries: default_max_retries(),
            retry_delay_ms: default_retry_delay_ms(),
            max_batch_size: None,
            rate_limit_rpm: None,
            rate_limit_tpm: None,
            compression_enabled: default_compression(),
            compression_threshold_bytes: default_compression_threshold(),
            connection_pool_size: default_pool_size(),
            enable_circuit_breaker: true,
            circuit_breaker_config: Some(CircuitBreakerConfig::default()),
            enable_metrics: default_enable_metrics(),
        }
    }
}

impl OptimizationConfig {
    /// Optimized configuration for `OpenAI`
    #[must_use]
    pub fn openai() -> Self {
        Self {
            timeout_seconds: Some(60),
            max_retries: 3,
            retry_delay_ms: 1000,
            max_batch_size: Some(2048),
            rate_limit_rpm: Some(3000),
            rate_limit_tpm: Some(1_000_000),
            compression_enabled: true,
            compression_threshold_bytes: 1024,
            connection_pool_size: 20,
            enable_circuit_breaker: true,
            circuit_breaker_config: Some(CircuitBreakerConfig {
                failure_threshold: 5,
                success_threshold: 2,
                timeout_seconds: 30,
                half_open_max_attempts: 3,
            }),
            enable_metrics: true,
        }
    }

    /// Optimized configuration for Mistral AI
    #[must_use]
    pub fn mistral() -> Self {
        Self {
            timeout_seconds: Some(30),
            max_retries: 3,
            retry_delay_ms: 500,
            max_batch_size: Some(128),
            rate_limit_rpm: Some(100),
            rate_limit_tpm: Some(100_000),
            compression_enabled: true,
            compression_threshold_bytes: 512,
            connection_pool_size: 10,
            enable_circuit_breaker: true,
            circuit_breaker_config: Some(CircuitBreakerConfig {
                failure_threshold: 3,
                success_threshold: 2,
                timeout_seconds: 20,
                half_open_max_attempts: 2,
            }),
            enable_metrics: true,
        }
    }

    /// Optimized configuration for Azure `OpenAI`
    #[must_use]
    pub fn azure() -> Self {
        Self {
            timeout_seconds: Some(90),
            max_retries: 4,
            retry_delay_ms: 2000,
            max_batch_size: Some(2048),
            rate_limit_rpm: Some(300),
            rate_limit_tpm: Some(300_000),
            compression_enabled: true,
            compression_threshold_bytes: 1024,
            connection_pool_size: 15,
            enable_circuit_breaker: true,
            circuit_breaker_config: Some(CircuitBreakerConfig {
                failure_threshold: 5,
                success_threshold: 3,
                timeout_seconds: 60,
                half_open_max_attempts: 3,
            }),
            enable_metrics: true,
        }
    }

    /// Optimized configuration for local/custom providers
    #[must_use]
    pub fn local() -> Self {
        Self {
            timeout_seconds: Some(10),
            max_retries: 2,
            retry_delay_ms: 100,
            max_batch_size: Some(32),
            rate_limit_rpm: None,
            rate_limit_tpm: None,
            compression_enabled: false,
            compression_threshold_bytes: 2048,
            connection_pool_size: 5,
            enable_circuit_breaker: false,
            circuit_breaker_config: None,
            enable_metrics: true,
        }
    }

    /// Get effective timeout (returns default if not specified)
    #[must_use]
    pub fn get_timeout_seconds(&self) -> u64 {
        self.timeout_seconds.unwrap_or(60)
    }

    /// Get effective batch size (returns default if not specified)
    #[must_use]
    pub fn get_max_batch_size(&self) -> usize {
        self.max_batch_size.unwrap_or(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_config_profiles() {
        let openai_profile = OptimizationConfig::openai();
        assert_eq!(openai_profile.timeout_seconds, Some(60));
        assert_eq!(openai_profile.max_retries, 3);
        assert_eq!(openai_profile.max_batch_size, Some(2048));

        let mistral_profile = OptimizationConfig::mistral();
        assert_eq!(mistral_profile.timeout_seconds, Some(30));
        assert_eq!(mistral_profile.max_batch_size, Some(128));

        let azure_profile = OptimizationConfig::azure();
        assert_eq!(azure_profile.timeout_seconds, Some(90));
        assert_eq!(azure_profile.max_retries, 4);

        let local_profile = OptimizationConfig::local();
        assert_eq!(local_profile.timeout_seconds, Some(10));
        assert!(!local_profile.enable_circuit_breaker);
    }

    #[test]
    fn test_optimization_config_helper_methods() {
        let mut config = OptimizationConfig::default();
        assert_eq!(config.timeout_seconds, None);
        assert_eq!(config.get_timeout_seconds(), 60);

        config.timeout_seconds = Some(120);
        assert_eq!(config.get_timeout_seconds(), 120);

        config.max_batch_size = None;
        assert_eq!(config.get_max_batch_size(), 100);

        config.max_batch_size = Some(256);
        assert_eq!(config.get_max_batch_size(), 256);
    }

    #[test]
    fn test_optimization_config_serialization() {
        let config = OptimizationConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: OptimizationConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.max_retries, config.max_retries);
        assert_eq!(deserialized.retry_delay_ms, config.retry_delay_ms);
    }
}

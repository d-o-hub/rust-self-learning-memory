//! Configuration for embedding request batching and coalescing

use serde::{Deserialize, Serialize};

/// Configuration for embedding request batching and coalescing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BatchingConfig {
    /// Enable batching and coalescing of concurrent embedding requests
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Maximum batch size for provider requests
    #[serde(default = "default_max_batch_size")]
    pub max_batch_size: usize,
    /// Maximum time in milliseconds to wait before flushing a partial batch
    #[serde(default = "default_max_wait_ms")]
    pub max_wait_ms: u64,
    /// Maximum number of concurrent in-flight requests sent to the provider
    #[serde(default = "default_max_in_flight")]
    pub max_in_flight: usize,
    /// Enable coalescing identical in-flight embedding requests
    #[serde(default = "default_coalesce_in_flight")]
    pub coalesce_in_flight: bool,
}

fn default_enabled() -> bool {
    true
}

fn default_max_batch_size() -> usize {
    64
}

fn default_max_wait_ms() -> u64 {
    10
}

fn default_max_in_flight() -> usize {
    8
}

fn default_coalesce_in_flight() -> bool {
    true
}

impl Default for BatchingConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            max_batch_size: default_max_batch_size(),
            max_wait_ms: default_max_wait_ms(),
            max_in_flight: default_max_in_flight(),
            coalesce_in_flight: default_coalesce_in_flight(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batching_config_defaults() {
        let config = BatchingConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_batch_size, 64);
        assert_eq!(config.max_wait_ms, 10);
        assert_eq!(config.max_in_flight, 8);
        assert!(config.coalesce_in_flight);
    }

    #[test]
    fn test_batching_config_serde() {
        let json_str = r#"{
            "enabled": true,
            "max_batch_size": 128,
            "max_wait_ms": 20,
            "max_in_flight": 16,
            "coalesce_in_flight": false
        }"#;
        let config: BatchingConfig = serde_json::from_str(json_str).unwrap();
        assert!(config.enabled);
        assert_eq!(config.max_batch_size, 128);
        assert_eq!(config.max_wait_ms, 20);
        assert_eq!(config.max_in_flight, 16);
        assert!(!config.coalesce_in_flight);
    }
}

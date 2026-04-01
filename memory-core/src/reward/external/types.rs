//! External signal types and data structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Normalized external signal format
///
/// This is the standard format that all external providers must
/// convert their data into. It provides a unified interface for
/// the reward system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSignalSet {
    /// Provider that generated these signals
    pub provider: String,
    /// Tool-specific signals
    pub tool_signals: Vec<ToolSignal>,
    /// Overall episode quality score (0.0-1.0)
    pub episode_quality: Option<f32>,
    /// Signal timestamp
    pub timestamp: DateTime<Utc>,
    /// Confidence in these signals (0.0-1.0)
    pub confidence: f32,
}

impl ExternalSignalSet {
    /// Create an empty signal set
    pub fn empty(provider: &str) -> Self {
        Self {
            provider: provider.to_string(),
            tool_signals: Vec::new(),
            episode_quality: None,
            timestamp: Utc::now(),
            confidence: 0.0,
        }
    }

    /// Calculate average success rate across all tool signals
    pub fn avg_success_rate(&self) -> f32 {
        if self.tool_signals.is_empty() {
            return 0.5; // Neutral if no signals
        }

        let total: f32 = self.tool_signals.iter().map(|t| t.success_rate).sum();

        total / self.tool_signals.len() as f32
    }

    /// Get total sample count across all tool signals
    pub fn total_samples(&self) -> usize {
        self.tool_signals.iter().map(|t| t.sample_count).sum()
    }
}

impl Default for ExternalSignalSet {
    fn default() -> Self {
        Self::empty("unknown")
    }
}

/// Per-tool signal data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSignal {
    /// Tool name (normalized to internal naming convention)
    pub tool_name: String,
    /// Success rate from external source (0.0-1.0)
    pub success_rate: f32,
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
    /// Sample size for statistics
    pub sample_count: usize,
    /// Additional provider-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ToolSignal {
    /// Create a new tool signal
    pub fn new(tool_name: impl Into<String>, success_rate: f32) -> Self {
        Self {
            tool_name: tool_name.into(),
            success_rate: success_rate.clamp(0.0, 1.0),
            avg_latency_ms: 0.0,
            sample_count: 0,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata
    #[must_use]
    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Configuration for external signal providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSignalConfig {
    /// Globally enable/disable external signals
    pub enabled: bool,
    /// Default weight for external signals (0.0-1.0)
    pub default_weight: f32,
    /// Provider-specific weight overrides
    pub provider_weights: HashMap<String, f32>,
    /// Minimum confidence threshold for accepting signals
    pub min_confidence: f32,
    /// Enable signal caching
    pub enable_caching: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
}

impl ExternalSignalConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> super::Result<Self> {
        use std::env;

        let enabled = env::var("EXTERNAL_SIGNALS_ENABLED")
            .map(|v| v == "true")
            .unwrap_or(false);

        let default_weight = env::var("EXTERNAL_SIGNAL_WEIGHT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.3);

        let min_confidence = env::var("EXTERNAL_SIGNAL_MIN_CONFIDENCE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.5);

        let enable_caching = env::var("EXTERNAL_SIGNAL_ENABLE_CACHING")
            .map(|v| v == "true")
            .unwrap_or(true);

        let cache_ttl_seconds = env::var("EXTERNAL_SIGNAL_CACHE_TTL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        Ok(Self {
            enabled,
            default_weight,
            provider_weights: HashMap::new(),
            min_confidence,
            enable_caching,
            cache_ttl_seconds,
        })
    }

    /// Get weight for a specific provider
    pub fn weight_for(&self, provider: &str) -> f32 {
        self.provider_weights
            .get(provider)
            .copied()
            .unwrap_or(self.default_weight)
    }
}

impl Default for ExternalSignalConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_weight: 0.3,
            provider_weights: HashMap::new(),
            min_confidence: 0.5,
            enable_caching: true,
            cache_ttl_seconds: 30,
        }
    }
}

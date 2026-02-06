//! TTL Configuration for Adaptive Cache
//!
//! This module provides configuration types for the adaptive TTL cache system,
//! including TTL bounds, adaptation policies, and memory pressure thresholds.

use std::time::Duration;

/// Default base TTL (5 minutes)
pub const DEFAULT_BASE_TTL: Duration = Duration::from_secs(300);

/// Default minimum TTL (1 minute)
pub const DEFAULT_MIN_TTL: Duration = Duration::from_secs(60);

/// Default maximum TTL (1 hour)
pub const DEFAULT_MAX_TTL: Duration = Duration::from_secs(3600);

/// Default hot threshold (accesses to be considered "hot")
pub const DEFAULT_HOT_THRESHOLD: u64 = 10;

/// Default cold threshold (accesses to be considered "cold")
pub const DEFAULT_COLD_THRESHOLD: u64 = 2;

/// Default adaptation rate (0.0 - 1.0)
pub const DEFAULT_ADAPTATION_RATE: f64 = 0.25;

/// Default cleanup interval (60 seconds)
pub const DEFAULT_CLEANUP_INTERVAL: Duration = Duration::from_secs(60);

/// Configuration for adaptive TTL cache
#[derive(Debug, Clone)]
pub struct TTLConfig {
    /// Base TTL for new entries
    pub base_ttl: Duration,

    /// Minimum TTL (entries won't be reduced below this)
    pub min_ttl: Duration,

    /// Maximum TTL (entries won't be extended beyond this)
    pub max_ttl: Duration,

    /// Hot threshold - access count above which TTL is extended
    pub hot_threshold: u64,

    /// Cold threshold - access count below which TTL is reduced
    pub cold_threshold: u64,

    /// Adaptation rate - how fast TTL changes (0.0 - 1.0)
    pub adaptation_rate: f64,

    /// Enable background cleanup task
    pub enable_background_cleanup: bool,

    /// Cleanup interval for expired entries
    pub cleanup_interval: Duration,

    /// Maximum number of entries in cache
    pub max_entries: usize,

    /// Memory pressure threshold (0.0 - 1.0)
    pub memory_pressure_threshold: f64,

    /// Enable adaptive TTL based on access patterns
    pub enable_adaptive_ttl: bool,

    /// Time window for access pattern analysis (seconds)
    pub access_window_secs: u64,
}

impl Default for TTLConfig {
    fn default() -> Self {
        Self {
            base_ttl: DEFAULT_BASE_TTL,
            min_ttl: DEFAULT_MIN_TTL,
            max_ttl: DEFAULT_MAX_TTL,
            hot_threshold: DEFAULT_HOT_THRESHOLD,
            cold_threshold: DEFAULT_COLD_THRESHOLD,
            adaptation_rate: DEFAULT_ADAPTATION_RATE,
            enable_background_cleanup: true,
            cleanup_interval: DEFAULT_CLEANUP_INTERVAL,
            max_entries: 10_000,
            memory_pressure_threshold: 0.8,
            enable_adaptive_ttl: true,
            access_window_secs: 300, // 5 minutes
        }
    }
}

impl TTLConfig {
    /// Create a new TTLConfig with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration optimized for high-hit-rate scenarios
    pub fn high_hit_rate() -> Self {
        Self {
            base_ttl: Duration::from_secs(600), // 10 minutes
            min_ttl: Duration::from_secs(120),  // 2 minutes
            max_ttl: Duration::from_secs(7200), // 2 hours
            hot_threshold: 5,                   // Lower threshold
            cold_threshold: 1,
            adaptation_rate: 0.35, // Faster adaptation
            enable_background_cleanup: true,
            cleanup_interval: Duration::from_secs(30),
            max_entries: 20_000,
            memory_pressure_threshold: 0.85,
            enable_adaptive_ttl: true,
            access_window_secs: 180, // 3 minutes
        }
    }

    /// Create a configuration optimized for memory-constrained environments
    pub fn memory_constrained() -> Self {
        Self {
            base_ttl: Duration::from_secs(180), // 3 minutes
            min_ttl: Duration::from_secs(30),   // 30 seconds
            max_ttl: Duration::from_secs(1800), // 30 minutes
            hot_threshold: 15,
            cold_threshold: 3,
            adaptation_rate: 0.2,
            enable_background_cleanup: true,
            cleanup_interval: Duration::from_secs(15),
            max_entries: 1_000,
            memory_pressure_threshold: 0.6, // Lower threshold
            enable_adaptive_ttl: true,
            access_window_secs: 120, // 2 minutes
        }
    }

    /// Create a configuration optimized for write-heavy workloads
    pub fn write_heavy() -> Self {
        Self {
            base_ttl: Duration::from_secs(120), // 2 minutes
            min_ttl: Duration::from_secs(30),   // 30 seconds
            max_ttl: Duration::from_secs(600),  // 10 minutes
            hot_threshold: 20,
            cold_threshold: 5,
            adaptation_rate: 0.15, // Slower adaptation
            enable_background_cleanup: true,
            cleanup_interval: Duration::from_secs(20),
            max_entries: 5_000,
            memory_pressure_threshold: 0.75,
            enable_adaptive_ttl: true,
            access_window_secs: 60, // 1 minute
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), TTLConfigError> {
        if self.min_ttl > self.base_ttl {
            return Err(TTLConfigError::InvalidBounds(
                "min_ttl cannot be greater than base_ttl".to_string(),
            ));
        }
        if self.base_ttl > self.max_ttl {
            return Err(TTLConfigError::InvalidBounds(
                "base_ttl cannot be greater than max_ttl".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&self.adaptation_rate) {
            return Err(TTLConfigError::InvalidAdaptationRate(self.adaptation_rate));
        }
        if !(0.0..=1.0).contains(&self.memory_pressure_threshold) {
            return Err(TTLConfigError::InvalidThreshold(
                self.memory_pressure_threshold,
            ));
        }
        if self.hot_threshold <= self.cold_threshold {
            return Err(TTLConfigError::InvalidThresholds {
                hot: self.hot_threshold,
                cold: self.cold_threshold,
            });
        }
        if self.max_entries == 0 {
            return Err(TTLConfigError::InvalidMaxEntries);
        }
        Ok(())
    }

    /// Calculate adapted TTL based on access count
    pub fn calculate_ttl(&self, current_ttl: Duration, access_count: u64) -> Duration {
        if !self.enable_adaptive_ttl {
            return self.base_ttl;
        }

        let new_ttl = if access_count >= self.hot_threshold {
            // Extend TTL for hot items
            let extension = current_ttl.mul_f64(self.adaptation_rate);
            current_ttl + extension
        } else if access_count <= self.cold_threshold {
            // Reduce TTL for cold items
            let reduction = current_ttl.mul_f64(self.adaptation_rate);
            current_ttl.saturating_sub(reduction)
        } else {
            current_ttl
        };

        // Clamp to bounds
        new_ttl.clamp(self.min_ttl, self.max_ttl)
    }

    /// Builder method to set base TTL
    pub fn with_base_ttl(mut self, ttl: Duration) -> Self {
        self.base_ttl = ttl;
        self
    }

    /// Builder method to set min TTL
    pub fn with_min_ttl(mut self, ttl: Duration) -> Self {
        self.min_ttl = ttl;
        self
    }

    /// Builder method to set max TTL
    pub fn with_max_ttl(mut self, ttl: Duration) -> Self {
        self.max_ttl = ttl;
        self
    }

    /// Builder method to set hot threshold
    pub fn with_hot_threshold(mut self, threshold: u64) -> Self {
        self.hot_threshold = threshold;
        self
    }

    /// Builder method to set cold threshold
    pub fn with_cold_threshold(mut self, threshold: u64) -> Self {
        self.cold_threshold = threshold;
        self
    }

    /// Builder method to set adaptation rate
    pub fn with_adaptation_rate(mut self, rate: f64) -> Self {
        self.adaptation_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Builder method to set max entries
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }
}

/// Errors that can occur during TTL configuration validation
#[derive(Debug, Clone, PartialEq)]
pub enum TTLConfigError {
    /// Invalid TTL bounds
    InvalidBounds(String),
    /// Invalid adaptation rate (must be 0.0 - 1.0)
    InvalidAdaptationRate(f64),
    /// Invalid threshold (must be 0.0 - 1.0)
    InvalidThreshold(f64),
    /// Invalid hot/cold thresholds (hot must be > cold)
    InvalidThresholds { hot: u64, cold: u64 },
    /// Invalid max entries (must be > 0)
    InvalidMaxEntries,
}

impl std::fmt::Display for TTLConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidBounds(msg) => write!(f, "Invalid TTL bounds: {msg}"),
            Self::InvalidAdaptationRate(rate) => {
                write!(f, "Invalid adaptation rate: {rate} (must be 0.0 - 1.0)")
            }
            Self::InvalidThreshold(threshold) => {
                write!(f, "Invalid threshold: {threshold} (must be 0.0 - 1.0)")
            }
            Self::InvalidThresholds { hot, cold } => {
                write!(f, "Invalid thresholds: hot ({hot}) must be > cold ({cold})")
            }
            Self::InvalidMaxEntries => write!(f, "Invalid max entries: must be > 0"),
        }
    }
}

impl std::error::Error for TTLConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TTLConfig::default();
        assert_eq!(config.base_ttl, DEFAULT_BASE_TTL);
        assert_eq!(config.min_ttl, DEFAULT_MIN_TTL);
        assert_eq!(config.max_ttl, DEFAULT_MAX_TTL);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_high_hit_rate_config() {
        let config = TTLConfig::high_hit_rate();
        assert!(config.validate().is_ok());
        assert_eq!(config.base_ttl, Duration::from_secs(600));
        assert!(config.max_entries > 10_000);
    }

    #[test]
    fn test_memory_constrained_config() {
        let config = TTLConfig::memory_constrained();
        assert!(config.validate().is_ok());
        assert!(config.max_entries < 2_000);
        assert!(config.memory_pressure_threshold < 0.7);
    }

    #[test]
    fn test_write_heavy_config() {
        let config = TTLConfig::write_heavy();
        assert!(config.validate().is_ok());
        assert!(config.base_ttl < Duration::from_secs(300));
    }

    #[test]
    fn test_invalid_bounds() {
        let config = TTLConfig {
            min_ttl: Duration::from_secs(600),
            base_ttl: Duration::from_secs(300),
            ..Default::default()
        };
        assert!(matches!(
            config.validate(),
            Err(TTLConfigError::InvalidBounds(_))
        ));
    }

    #[test]
    fn test_invalid_adaptation_rate() {
        let config = TTLConfig {
            adaptation_rate: 1.5,
            ..Default::default()
        };
        assert!(matches!(
            config.validate(),
            Err(TTLConfigError::InvalidAdaptationRate(1.5))
        ));
    }

    #[test]
    fn test_invalid_thresholds() {
        let config = TTLConfig {
            hot_threshold: 2,
            cold_threshold: 5,
            ..Default::default()
        };
        assert!(matches!(
            config.validate(),
            Err(TTLConfigError::InvalidThresholds { hot: 2, cold: 5 })
        ));
    }

    #[test]
    fn test_calculate_ttl_hot() {
        let config = TTLConfig::default();
        let current = Duration::from_secs(300);
        let new_ttl = config.calculate_ttl(current, 15); // Above hot threshold
        assert!(new_ttl > current);
        assert!(new_ttl <= config.max_ttl);
    }

    #[test]
    fn test_calculate_ttl_cold() {
        let config = TTLConfig::default();
        let current = Duration::from_secs(300);
        let new_ttl = config.calculate_ttl(current, 1); // Below cold threshold
        assert!(new_ttl < current);
        assert!(new_ttl >= config.min_ttl);
    }

    #[test]
    fn test_calculate_ttl_neutral() {
        let config = TTLConfig::default();
        let current = Duration::from_secs(300);
        let new_ttl = config.calculate_ttl(current, 5); // Between thresholds
        assert_eq!(new_ttl, current);
    }

    #[test]
    fn test_builder_methods() {
        let config = TTLConfig::new()
            .with_base_ttl(Duration::from_secs(600))
            .with_min_ttl(Duration::from_secs(120))
            .with_max_ttl(Duration::from_secs(7200))
            .with_hot_threshold(20)
            .with_cold_threshold(3)
            .with_adaptation_rate(0.5)
            .with_max_entries(5000);

        assert_eq!(config.base_ttl, Duration::from_secs(600));
        assert_eq!(config.min_ttl, Duration::from_secs(120));
        assert_eq!(config.max_ttl, Duration::from_secs(7200));
        assert_eq!(config.hot_threshold, 20);
        assert_eq!(config.cold_threshold, 3);
        assert_eq!(config.adaptation_rate, 0.5);
        assert_eq!(config.max_entries, 5000);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_adaptation_rate_clamping() {
        let config = TTLConfig::new().with_adaptation_rate(1.5);
        assert_eq!(config.adaptation_rate, 1.0);

        let config = TTLConfig::new().with_adaptation_rate(-0.5);
        assert_eq!(config.adaptation_rate, 0.0);
    }
}

//! Configuration for cache persistence

use std::path::PathBuf;
use std::time::Duration;

/// Persistence mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PersistenceMode {
    /// Full snapshots only
    Full,
    /// Incremental updates only
    Incremental,
    /// Hybrid: full snapshots with incremental updates
    #[default]
    Hybrid,
    /// Disabled
    Disabled,
}

/// Persistence strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PersistenceStrategy {
    /// Persist on every change (high durability, lower performance)
    Immediate,
    /// Persist at regular intervals
    #[default]
    Interval,
    /// Persist on shutdown only
    OnShutdown,
    /// Persist when cache reaches threshold
    Threshold,
}

/// Configuration for cache persistence
#[derive(Debug, Clone)]
pub struct PersistenceConfig {
    /// Whether persistence is enabled
    pub enabled: bool,
    /// Path to persistence file
    pub persistence_path: PathBuf,
    /// Persistence mode
    pub mode: PersistenceMode,
    /// Persistence strategy
    pub strategy: PersistenceStrategy,
    /// Save interval (for Interval strategy)
    pub save_interval: Duration,
    /// Minimum number of entries to trigger save
    pub min_entries_threshold: usize,
    /// Maximum number of entries to keep in snapshot
    pub max_entries: usize,
    /// Enable compression for persisted data
    pub compression_enabled: bool,
    /// Compression level (0-9, higher = more compression)
    pub compression_level: u32,
    /// Enable encryption for persisted data
    pub encryption_enabled: bool,
    /// Maximum age of snapshot before forcing full save (seconds)
    pub max_snapshot_age_secs: u64,
    /// Number of backup snapshots to keep
    pub backup_count: usize,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            persistence_path: PathBuf::from("./cache.redb.snapshot"),
            mode: PersistenceMode::default(),
            strategy: PersistenceStrategy::default(),
            save_interval: Duration::from_secs(300), // 5 minutes
            min_entries_threshold: 100,
            max_entries: 100_000,
            compression_enabled: true,
            compression_level: 6,
            encryption_enabled: false,
            max_snapshot_age_secs: 3600, // 1 hour
            backup_count: 3,
        }
    }
}

impl PersistenceConfig {
    /// Create a new configuration with custom path
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self {
            persistence_path: path.into(),
            ..Default::default()
        }
    }

    /// Disable persistence
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Enable immediate persistence strategy
    pub fn with_immediate_strategy(mut self) -> Self {
        self.strategy = PersistenceStrategy::Immediate;
        self
    }

    /// Enable interval-based persistence
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.strategy = PersistenceStrategy::Interval;
        self.save_interval = interval;
        self
    }

    /// Enable shutdown-only persistence
    pub fn with_shutdown_strategy(mut self) -> Self {
        self.strategy = PersistenceStrategy::OnShutdown;
        self
    }

    /// Set minimum entries threshold
    pub fn with_min_entries(mut self, threshold: usize) -> Self {
        self.min_entries_threshold = threshold;
        self
    }

    /// Set maximum entries
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Enable/disable compression
    pub fn with_compression(mut self, enabled: bool) -> Self {
        self.compression_enabled = enabled;
        self
    }

    /// Set compression level
    pub fn with_compression_level(mut self, level: u32) -> Self {
        self.compression_level = level.clamp(0, 9);
        self
    }

    /// Set number of backups to keep
    pub fn with_backup_count(mut self, count: usize) -> Self {
        self.backup_count = count;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PersistenceConfig::default();
        assert!(config.enabled);
        assert_eq!(config.mode, PersistenceMode::Hybrid);
        assert_eq!(config.strategy, PersistenceStrategy::Interval);
        assert_eq!(config.save_interval, Duration::from_secs(300));
        assert_eq!(config.min_entries_threshold, 100);
        assert!(config.compression_enabled);
        assert_eq!(config.compression_level, 6);
        assert!(!config.encryption_enabled);
    }

    #[test]
    fn test_disabled_config() {
        let config = PersistenceConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_builder_methods() {
        let config = PersistenceConfig::default()
            .with_immediate_strategy()
            .with_min_entries(50)
            .with_max_entries(1000)
            .with_compression(false)
            .with_compression_level(9)
            .with_backup_count(5);

        assert_eq!(config.strategy, PersistenceStrategy::Immediate);
        assert_eq!(config.min_entries_threshold, 50);
        assert_eq!(config.max_entries, 1000);
        assert!(!config.compression_enabled);
        assert_eq!(config.compression_level, 9);
        assert_eq!(config.backup_count, 5);
    }

    #[test]
    fn test_interval_config() {
        let config = PersistenceConfig::default().with_interval(Duration::from_secs(60));

        assert_eq!(config.strategy, PersistenceStrategy::Interval);
        assert_eq!(config.save_interval, Duration::from_secs(60));
    }

    #[test]
    fn test_shutdown_strategy() {
        let config = PersistenceConfig::default().with_shutdown_strategy();

        assert_eq!(config.strategy, PersistenceStrategy::OnShutdown);
    }

    #[test]
    fn test_compression_level_clamping() {
        let config = PersistenceConfig::default().with_compression_level(15);

        assert_eq!(config.compression_level, 9);

        let config = PersistenceConfig::default().with_compression_level(5);

        assert_eq!(config.compression_level, 5);
    }
}

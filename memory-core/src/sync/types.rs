//! Sync types and configuration

use std::time::Duration;

/// Synchronization state tracking
#[derive(Debug, Clone, Default)]
pub struct SyncState {
    /// Last successful sync timestamp
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    /// Number of sync operations performed
    pub sync_count: u64,
    /// Last error message if any
    pub last_error: Option<String>,
}

/// Configuration for storage synchronization
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Interval between periodic syncs
    pub sync_interval: Duration,
    /// Maximum number of items to sync in one batch
    pub batch_size: usize,
    /// Whether to sync patterns
    pub sync_patterns: bool,
    /// Whether to sync heuristics
    pub sync_heuristics: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            sync_interval: Duration::from_secs(300), // 5 minutes
            batch_size: 100,
            sync_patterns: true,
            sync_heuristics: true,
        }
    }
}

/// Synchronization statistics
#[derive(Debug, Clone, Default)]
pub struct SyncStats {
    /// Number of episodes synced
    pub episodes_synced: usize,
    /// Number of patterns synced
    pub patterns_synced: usize,
    /// Number of heuristics synced
    pub heuristics_synced: usize,
    /// Number of conflicts resolved
    pub conflicts_resolved: usize,
    /// Number of errors encountered
    pub errors: usize,
}

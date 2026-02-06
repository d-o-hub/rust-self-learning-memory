//! Types for cache persistence

use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// A single persisted cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedCacheEntry {
    /// Entry key
    pub key: String,
    /// Serialized value
    pub value: Vec<u8>,
    /// Creation timestamp (Unix seconds)
    pub created_at: u64,
    /// Number of times accessed
    pub access_count: u64,
    /// Last access timestamp (Unix seconds)
    pub last_accessed: u64,
    /// Optional TTL in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_secs: Option<u64>,
}

/// Complete cache snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSnapshot {
    /// Snapshot format version
    pub version: u32,
    /// Creation timestamp (Unix seconds)
    pub created_at: u64,
    /// Cache entries
    pub entries: Vec<PersistedCacheEntry>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl CacheSnapshot {
    /// Create a new empty snapshot
    pub fn new() -> Self {
        Self {
            version: 1,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            entries: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add an entry to the snapshot
    pub fn add_entry(mut self, entry: PersistedCacheEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if snapshot is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get total size in bytes (approximate)
    pub fn size_bytes(&self) -> usize {
        self.entries
            .iter()
            .map(|e| e.key.len() + e.value.len() + 32) // 32 bytes for metadata
            .sum()
    }
}

impl Default for CacheSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

/// Persistence statistics
#[derive(Debug, Clone, Default)]
pub struct PersistenceStats {
    /// Number of snapshots saved
    pub snapshots_saved: u64,
    /// Number of snapshots loaded
    pub snapshots_loaded: u64,
    /// Total entries saved
    pub total_entries_saved: usize,
    /// Total entries loaded
    pub total_entries_loaded: usize,
    /// Total bytes written
    pub total_bytes_written: u64,
    /// Total bytes read
    pub total_bytes_read: u64,
    /// Last save duration
    pub last_save_duration: Duration,
    /// Last load duration
    pub last_load_duration: Duration,
    /// Number of save failures
    pub save_failures: u64,
    /// Number of load failures
    pub load_failures: u64,
    /// Number of incremental updates
    pub incremental_updates: u64,
    /// Compression ratio (compressed / uncompressed)
    pub compression_ratio: f64,
}

impl PersistenceStats {
    /// Calculate average bytes per entry written
    pub fn avg_bytes_per_entry_written(&self) -> f64 {
        if self.total_entries_saved == 0 {
            0.0
        } else {
            self.total_bytes_written as f64 / self.total_entries_saved as f64
        }
    }

    /// Calculate average bytes per entry read
    pub fn avg_bytes_per_entry_read(&self) -> f64 {
        if self.total_entries_loaded == 0 {
            0.0
        } else {
            self.total_bytes_read as f64 / self.total_entries_loaded as f64
        }
    }

    /// Calculate success rate for saves
    pub fn save_success_rate(&self) -> f64 {
        let total = self.snapshots_saved + self.save_failures;
        if total == 0 {
            1.0
        } else {
            self.snapshots_saved as f64 / total as f64
        }
    }

    /// Calculate success rate for loads
    pub fn load_success_rate(&self) -> f64 {
        let total = self.snapshots_loaded + self.load_failures;
        if total == 0 {
            1.0
        } else {
            self.snapshots_loaded as f64 / total as f64
        }
    }
}

/// Incremental update record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Used in tests and future incremental persistence
pub struct IncrementalUpdate {
    /// Update sequence number
    pub sequence: u64,
    /// Timestamp of update
    pub timestamp: u64,
    /// Added or updated entries
    pub upserts: Vec<PersistedCacheEntry>,
    /// Deleted keys
    pub deletions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_snapshot_creation() {
        let snapshot = CacheSnapshot::new();
        assert_eq!(snapshot.version, 1);
        assert!(snapshot.entries.is_empty());
        assert!(snapshot.metadata.is_empty());
        assert!(snapshot.is_empty());
    }

    #[test]
    fn test_cache_snapshot_add_entry() {
        let entry = PersistedCacheEntry {
            key: "test".to_string(),
            value: vec![1, 2, 3],
            created_at: 1234567890,
            access_count: 1,
            last_accessed: 1234567890,
            ttl_secs: None,
        };

        let snapshot = CacheSnapshot::new().add_entry(entry);
        assert_eq!(snapshot.len(), 1);
        assert!(!snapshot.is_empty());
    }

    #[test]
    fn test_cache_snapshot_metadata() {
        let snapshot = CacheSnapshot::new()
            .with_metadata("version", "1.0")
            .with_metadata("source", "test");

        assert_eq!(snapshot.metadata.get("version"), Some(&"1.0".to_string()));
        assert_eq!(snapshot.metadata.get("source"), Some(&"test".to_string()));
    }

    #[test]
    fn test_persistence_stats() {
        let stats = PersistenceStats {
            snapshots_saved: 10,
            total_entries_saved: 1000,
            total_bytes_written: 50000,
            save_failures: 1,
            ..Default::default()
        };

        assert_eq!(stats.avg_bytes_per_entry_written(), 50.0);
        assert!((stats.save_success_rate() - 0.909).abs() < 0.01);
    }

    #[test]
    fn test_incremental_update() {
        let update = IncrementalUpdate {
            sequence: 1,
            timestamp: 1234567890,
            upserts: vec![],
            deletions: vec!["key1".to_string()],
        };

        assert_eq!(update.sequence, 1);
        assert_eq!(update.deletions.len(), 1);
    }
}

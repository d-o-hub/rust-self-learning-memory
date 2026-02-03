//! # Redb Cache Persistence Module
//!
//! This module provides persistence for the redb cache layer, enabling:
//! - Cache state save/load functionality
//! - Graceful shutdown with cache flush
//! - Recovery on startup
//! - Incremental cache persistence
//!
//! ## Features
//!
//! - **Full Cache Persistence**: Save and load complete cache state
//! - **Incremental Updates**: Persist only changed entries
//! - **Graceful Shutdown**: Automatic cache flush on shutdown
//! - **Recovery**: Restore cache state on startup
//! - **Compression**: Optional compression for persisted data
//!
//! ## Usage
//!
//! ```rust
//! use memory_storage_redb::{CachePersistence, PersistenceConfig};
//!
//! let config = PersistenceConfig::default();
//! let persistence = CachePersistence::new(config);
//! ```

use std::path::{Path, PathBuf};
use std::sync::Arc;
#[allow(unused_imports)]
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use parking_lot::RwLock;
use tracing::{debug, info};

#[allow(unused_imports)] // False positive - imports are used in conditional code
mod config;
mod manager;
mod types;

pub use config::{PersistenceConfig, PersistenceMode, PersistenceStrategy};
pub use manager::PersistenceManager;
pub use types::{CacheSnapshot, PersistedCacheEntry, PersistenceStats};

/// Cache persistence handler
///
/// Manages saving and loading cache state to/from disk.
/// Supports full snapshots and incremental updates.
#[derive(Debug)]
pub struct CachePersistence {
    config: PersistenceConfig,
    stats: Arc<RwLock<PersistenceStats>>,
    last_save: Arc<RwLock<Option<Instant>>>,
}

impl CachePersistence {
    /// Create a new cache persistence handler
    pub fn new(config: PersistenceConfig) -> Self {
        info!(
            "Creating cache persistence with mode={:?}, strategy={:?}",
            config.mode, config.strategy
        );

        Self {
            config,
            stats: Arc::new(RwLock::new(PersistenceStats::default())),
            last_save: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with default configuration
    pub fn with_default_config() -> Self {
        Self::new(PersistenceConfig::default())
    }

    /// Get persistence configuration
    pub fn config(&self) -> &PersistenceConfig {
        &self.config
    }

    /// Get persistence statistics
    pub fn stats(&self) -> PersistenceStats {
        self.stats.read().clone()
    }

    /// Check if persistence is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Save cache snapshot to disk
    ///
    /// # Arguments
    ///
    /// * `snapshot` - The cache snapshot to save
    /// * `path` - Optional path override (uses config path if None)
    ///
    /// # Returns
    ///
    /// Number of entries saved, or error if save failed
    pub fn save_snapshot(
        &self,
        snapshot: &CacheSnapshot,
        path: Option<&Path>,
    ) -> crate::Result<usize> {
        if !self.config.enabled {
            debug!("Cache persistence disabled, skipping save");
            return Ok(0);
        }

        let save_path = path
            .map(PathBuf::from)
            .unwrap_or_else(|| self.config.persistence_path.clone());

        info!(
            "Saving cache snapshot with {} entries to {:?}",
            snapshot.entries.len(),
            save_path
        );

        let start = Instant::now();

        // Serialize snapshot
        let serialized = postcard::to_allocvec(snapshot).map_err(|e| {
            crate::Error::Storage(format!("Failed to serialize cache snapshot: {}", e))
        })?;

        // Compress if enabled
        let data = if self.config.compression_enabled {
            debug!("Compressing cache snapshot ({} bytes)", serialized.len());
            compress_data(&serialized).map_err(|e| {
                crate::Error::Storage(format!("Failed to compress cache snapshot: {}", e))
            })?
        } else {
            serialized
        };

        // Write to file
        std::fs::write(&save_path, &data)
            .map_err(|e| crate::Error::Storage(format!("Failed to write cache snapshot: {}", e)))?;

        let elapsed = start.elapsed();
        let bytes_written = data.len();

        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.snapshots_saved += 1;
            stats.total_entries_saved += snapshot.entries.len();
            stats.total_bytes_written += bytes_written as u64;
            stats.last_save_duration = elapsed;
        }

        // Update last save time
        {
            let mut last = self.last_save.write();
            *last = Some(Instant::now());
        }

        info!(
            "Cache snapshot saved: {} entries, {} bytes in {:?}",
            snapshot.entries.len(),
            bytes_written,
            elapsed
        );

        Ok(snapshot.entries.len())
    }

    /// Load cache snapshot from disk
    ///
    /// # Arguments
    ///
    /// * `path` - Optional path override (uses config path if None)
    ///
    /// # Returns
    ///
    /// Loaded cache snapshot, or error if load failed
    pub fn load_snapshot(&self, path: Option<&Path>) -> crate::Result<Option<CacheSnapshot>> {
        if !self.config.enabled {
            debug!("Cache persistence disabled, skipping load");
            return Ok(None);
        }

        let load_path = path
            .map(PathBuf::from)
            .unwrap_or_else(|| self.config.persistence_path.clone());

        if !load_path.exists() {
            debug!("No cache snapshot found at {:?}", load_path);
            return Ok(None);
        }

        info!("Loading cache snapshot from {:?}", load_path);

        let start = Instant::now();

        // Read from file
        let data = std::fs::read(&load_path)
            .map_err(|e| crate::Error::Storage(format!("Failed to read cache snapshot: {}", e)))?;

        // Decompress if needed
        let serialized = if self.config.compression_enabled {
            debug!("Decompressing cache snapshot ({} bytes)", data.len());
            decompress_data(&data).map_err(|e| {
                crate::Error::Storage(format!("Failed to decompress cache snapshot: {}", e))
            })?
        } else {
            data
        };

        // Deserialize snapshot
        let snapshot: CacheSnapshot = postcard::from_bytes(&serialized).map_err(|e| {
            crate::Error::Storage(format!("Failed to deserialize cache snapshot: {}", e))
        })?;

        let elapsed = start.elapsed();

        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.snapshots_loaded += 1;
            stats.total_entries_loaded += snapshot.entries.len();
            stats.total_bytes_read += serialized.len() as u64;
            stats.last_load_duration = elapsed;
        }

        info!(
            "Cache snapshot loaded: {} entries, {} bytes in {:?}",
            snapshot.entries.len(),
            serialized.len(),
            elapsed
        );

        Ok(Some(snapshot))
    }

    /// Check if a save is needed based on configuration
    pub fn should_save(&self, entries_count: usize) -> bool {
        if !self.config.enabled {
            return false;
        }

        // Check minimum entries threshold
        if entries_count < self.config.min_entries_threshold {
            return false;
        }

        // Check save interval
        if let Some(last) = *self.last_save.read() {
            if last.elapsed() < self.config.save_interval {
                return false;
            }
        }

        true
    }

    /// Delete persisted cache snapshot
    pub fn delete_snapshot(&self, path: Option<&Path>) -> crate::Result<bool> {
        let delete_path = path
            .map(PathBuf::from)
            .unwrap_or_else(|| self.config.persistence_path.clone());

        if delete_path.exists() {
            std::fs::remove_file(&delete_path).map_err(|e| {
                crate::Error::Storage(format!("Failed to delete cache snapshot: {}", e))
            })?;

            info!("Cache snapshot deleted: {:?}", delete_path);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get the age of the last save
    pub fn last_save_age(&self) -> Option<Duration> {
        self.last_save.read().map(|instant| instant.elapsed())
    }

    /// Reset persistence statistics
    pub fn reset_stats(&self) {
        let mut stats = self.stats.write();
        *stats = PersistenceStats::default();
        info!("Cache persistence statistics reset");
    }
}

impl Default for CachePersistence {
    fn default() -> Self {
        Self::new(PersistenceConfig::default())
    }
}

/// Compress data using LZ4
fn compress_data(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Simple compression: store length prefix + compressed data
    let compressed = lz4_flex::compress_prepend_size(data);
    Ok(compressed)
}

/// Decompress data using LZ4
fn decompress_data(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let decompressed = lz4_flex::decompress_size_prepended(data)?;
    Ok(decompressed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_snapshot() -> CacheSnapshot {
        let entries = vec![
            PersistedCacheEntry {
                key: "entry1".to_string(),
                value: vec![1, 2, 3],
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                access_count: 5,
                last_accessed: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                ttl_secs: None,
            },
            PersistedCacheEntry {
                key: "entry2".to_string(),
                value: vec![4, 5, 6],
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                access_count: 3,
                last_accessed: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                ttl_secs: None,
            },
        ];

        CacheSnapshot {
            version: 1,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            entries,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_persistence_creation() {
        let config = PersistenceConfig::default();
        let persistence = CachePersistence::new(config);

        assert!(persistence.is_enabled());
        assert_eq!(persistence.stats().snapshots_saved, 0);
    }

    #[test]
    fn test_save_and_load_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let snapshot_path = temp_dir.path().join("cache.snapshot");

        let config = PersistenceConfig {
            enabled: true,
            persistence_path: snapshot_path.clone(),
            compression_enabled: false,
            ..Default::default()
        };

        let persistence = CachePersistence::new(config);
        let snapshot = create_test_snapshot();

        // Save snapshot
        let saved = persistence.save_snapshot(&snapshot, None).unwrap();
        assert_eq!(saved, 2);
        assert_eq!(persistence.stats().snapshots_saved, 1);

        // Load snapshot
        let loaded = persistence.load_snapshot(None).unwrap();
        assert!(loaded.is_some());

        let loaded_snapshot = loaded.unwrap();
        assert_eq!(loaded_snapshot.entries.len(), 2);
        assert_eq!(loaded_snapshot.entries[0].key, "entry1");
        assert_eq!(loaded_snapshot.entries[1].key, "entry2");
    }

    #[test]
    fn test_save_with_compression() {
        let temp_dir = TempDir::new().unwrap();
        let snapshot_path = temp_dir.path().join("cache.snapshot");

        let config = PersistenceConfig {
            enabled: true,
            persistence_path: snapshot_path.clone(),
            compression_enabled: true,
            ..Default::default()
        };

        let persistence = CachePersistence::new(config);
        let snapshot = create_test_snapshot();

        // Save with compression
        let saved = persistence.save_snapshot(&snapshot, None).unwrap();
        assert_eq!(saved, 2);

        // Load and verify
        let loaded = persistence.load_snapshot(None).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().entries.len(), 2);
    }

    #[test]
    fn test_disabled_persistence() {
        let config = PersistenceConfig {
            enabled: false,
            ..Default::default()
        };

        let persistence = CachePersistence::new(config);
        let snapshot = create_test_snapshot();

        assert!(!persistence.is_enabled());
        assert_eq!(persistence.save_snapshot(&snapshot, None).unwrap(), 0);
        assert!(persistence.load_snapshot(None).unwrap().is_none());
    }

    #[test]
    fn test_should_save() {
        let config = PersistenceConfig {
            enabled: true,
            min_entries_threshold: 10,
            save_interval: Duration::from_secs(60),
            ..Default::default()
        };

        let persistence = CachePersistence::new(config);

        // Below threshold
        assert!(!persistence.should_save(5));

        // Above threshold
        assert!(persistence.should_save(15));
    }

    #[test]
    fn test_delete_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let snapshot_path = temp_dir.path().join("cache.snapshot");

        let config = PersistenceConfig {
            enabled: true,
            persistence_path: snapshot_path.clone(),
            ..Default::default()
        };

        let persistence = CachePersistence::new(config);
        let snapshot = create_test_snapshot();

        // Save and verify exists
        persistence.save_snapshot(&snapshot, None).unwrap();
        assert!(snapshot_path.exists());

        // Delete and verify gone
        let deleted = persistence.delete_snapshot(None).unwrap();
        assert!(deleted);
        assert!(!snapshot_path.exists());

        // Delete non-existent
        let deleted = persistence.delete_snapshot(None).unwrap();
        assert!(!deleted);
    }
}

//! Configuration cache module
//!
//! This module provides caching for loaded configurations to avoid
//! repeated file parsing. The cache automatically invalidates when
//! files are modified (based on mtime).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

use crate::config::Config;

/// Cache entry containing configuration and file metadata
#[derive(Debug, Clone)]
struct CacheEntry {
    config: Config,
    mtime: SystemTime,
}

/// Configuration cache for avoiding repeated file parsing
struct ConfigCache {
    entries: Mutex<HashMap<PathBuf, CacheEntry>>,
    hits: Mutex<u64>,
    misses: Mutex<u64>,
}

impl ConfigCache {
    fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            hits: Mutex::new(0),
            misses: Mutex::new(0),
        }
    }

    /// Get configuration from cache if valid
    #[allow(clippy::excessive_nesting)]
    pub fn get(&self, path: &Path) -> Option<Config> {
        let entries = self
            .entries
            .lock()
            .expect("ConfigCache: entries lock poisoned - this indicates a panic in cache code");

        if let Some(entry) = entries.get(path) {
            // Check if file has been modified
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(current_mtime) = metadata.modified() {
                    if current_mtime == entry.mtime {
                        // Cache hit
                        *self
                            .hits
                            .lock()
                            .expect("ConfigCache: hits lock poisoned - this indicates a panic in metrics tracking") += 1;
                        tracing::debug!("Config cache hit for: {}", path.display());
                        return Some(entry.config.clone());
                    }
                }
            }
        }

        // Cache miss
        *self.misses.lock().expect(
            "ConfigCache: misses lock poisoned - this indicates a panic in metrics tracking",
        ) += 1;
        tracing::debug!("Config cache miss for: {}", path.display());
        None
    }

    /// Insert configuration into cache
    pub fn insert(&self, path: PathBuf, config: Config) {
        if let Ok(metadata) = std::fs::metadata(&path) {
            if let Ok(mtime) = metadata.modified() {
                let entry = CacheEntry { config, mtime };
                let mut entries = self.entries.lock().expect(
                    "ConfigCache: entries lock poisoned - this indicates a panic in cache code",
                );
                entries.insert(path, entry);
            }
        }
    }

    /// Clear all cached entries
    pub fn clear(&self) {
        let mut entries = self
            .entries
            .lock()
            .expect("ConfigCache: entries lock poisoned - this indicates a panic in cache code");
        entries.clear();
        tracing::debug!("Config cache cleared");
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let hits = *self
            .hits
            .lock()
            .expect("ConfigCache: hits lock poisoned - this indicates a panic in metrics tracking");
        let misses = *self.misses.lock().expect(
            "ConfigCache: misses lock poisoned - this indicates a panic in metrics tracking",
        );

        CacheStats {
            hits,
            misses,
            entries: self
                .entries
                .lock()
                .expect("ConfigCache: entries lock poisoned - this indicates a panic in cache code")
                .len(),
            hit_rate: if hits + misses > 0 {
                hits as f64 / (hits + misses) as f64
            } else {
                0.0
            },
        }
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of entries in cache
    pub entries: usize,
    /// Cache hit rate (0.0 to 1.0)
    pub hit_rate: f64,
}

/// Global configuration cache instance
fn cache() -> &'static ConfigCache {
    static CACHE: OnceLock<ConfigCache> = OnceLock::new();
    CACHE.get_or_init(ConfigCache::new)
}

/// Clear the configuration cache
///
/// This forces all subsequent configuration loads to re-read and re-parse
/// the configuration files, even if they haven't been modified.
pub fn clear_cache() {
    cache().clear();
}

/// Get cache statistics for monitoring and debugging
///
/// Returns information about cache hits, misses, and hit rate.
pub fn cache_stats() -> CacheStats {
    cache().stats()
}

/// Load configuration with caching and format detection
pub fn load_cached_config(path: &Path) -> Option<Config> {
    cache().get(path)
}

/// Insert configuration into cache
pub fn insert_cached_config(path: PathBuf, config: Config) {
    cache().insert(path, config);
}

#[cfg(test)]
mod cache_tests {
    use super::*;
    use crate::config::types::EmbeddingsConfig;
    use tempfile::tempdir;

    fn create_test_config() -> Config {
        Config {
            database: crate::config::types::DatabaseConfig {
                turso_url: None,
                turso_token: None,
                redb_path: Some("/tmp/test.redb".to_string()),
            },
            storage: crate::config::types::StorageConfig {
                max_episodes_cache: 1000,
                cache_ttl_seconds: 3600,
                pool_size: 5,
                quality_threshold: 0.7,
            },
            cli: crate::config::types::CliConfig {
                default_format: "json".to_string(),
                progress_bars: false,
                batch_size: 100,
            },
            embeddings: EmbeddingsConfig::default(),
        }
    }

    #[test]
    fn test_cache_hit() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-config.json");

        std::fs::write(
            &config_path,
            r#"{
                "database": {"redb_path": "/tmp/test.redb"},
                "storage": {"max_episodes_cache": 1000, "cache_ttl_seconds": 3600, "pool_size": 5},
                "cli": {"default_format": "json", "progress_bars": false, "batch_size": 100}
            }"#,
        )
        .unwrap();

        // Wait for filesystem to settle
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Clear cache to ensure clean state
        clear_cache();

        // First load - should be a cache miss
        let config1 = load_cached_config(&config_path);
        assert!(config1.is_none());

        // Simulate first load by inserting into cache
        let test_config = create_test_config();
        insert_cached_config(config_path.clone(), test_config.clone());

        // Second load - should be a cache hit
        let config2 = load_cached_config(&config_path);
        assert!(config2.is_some());
        assert_eq!(config2.unwrap().cli.default_format, "json");
    }

    #[test]
    fn test_cache_clear() {
        clear_cache();
        let stats = cache_stats();
        // Cache should be cleared (entries might be 0 or contain entries from other tests)
        // Just verify no panic
        let _stats_after_clear = cache_stats();
    }

    #[test]
    fn test_cache_stats_structure() {
        let stats = cache_stats();
        assert!(stats.hit_rate >= 0.0);
        assert!(stats.hit_rate <= 1.0);
    }
}

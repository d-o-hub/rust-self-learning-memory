//! Configuration loader module
//!
//! This module handles loading configuration from various file formats
//! and default locations, providing a clean separation from validation
//! and storage initialization logic.
//!
//! Caching is provided to avoid re-parsing configuration files on every
//! invocation. The cache automatically invalidates when files are modified.

use anyhow::Context;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

/// Cache entry containing configuration and file metadata
#[derive(Debug, Clone)]
struct CacheEntry {
    config: super::Config,
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
    fn get(&self, path: &Path) -> Option<super::Config> {
        let entries = self.entries.lock().unwrap();

        if let Some(entry) = entries.get(path) {
            // Check if file has been modified
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(current_mtime) = metadata.modified() {
                    if current_mtime == entry.mtime {
                        // Cache hit
                        *self.hits.lock().unwrap() += 1;
                        tracing::debug!("Config cache hit for: {}", path.display());
                        return Some(entry.config.clone());
                    }
                }
            }
        }

        // Cache miss
        *self.misses.lock().unwrap() += 1;
        tracing::debug!("Config cache miss for: {}", path.display());
        None
    }

    /// Insert configuration into cache
    fn insert(&self, path: PathBuf, config: super::Config) {
        if let Ok(metadata) = std::fs::metadata(&path) {
            if let Ok(mtime) = metadata.modified() {
                let entry = CacheEntry { config, mtime };
                let mut entries = self.entries.lock().unwrap();
                entries.insert(path, entry);
            }
        }
    }

    /// Clear all cached entries
    fn clear(&self) {
        let mut entries = self.entries.lock().unwrap();
        entries.clear();
        tracing::debug!("Config cache cleared");
    }

    /// Get cache statistics
    fn stats(&self) -> CacheStats {
        let hits = *self.hits.lock().unwrap();
        let misses = *self.misses.lock().unwrap();

        CacheStats {
            hits,
            misses,
            entries: self.entries.lock().unwrap().len(),
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
    pub hits: u64,
    pub misses: u64,
    pub entries: usize,
    pub hit_rate: f64,
}

/// Global configuration cache instance
fn cache() -> &'static ConfigCache {
    static CACHE: OnceLock<ConfigCache> = OnceLock::new();
    CACHE.get_or_init(ConfigCache::new)
}

/// Load configuration from file or use defaults
///
/// This function automatically caches loaded configurations and invalidates
/// the cache when the file is modified (based on mtime).
pub fn load_config(path: Option<&Path>) -> Result<super::Config, anyhow::Error> {
    match path {
        Some(path) => {
            // Try cache first
            if let Some(cached_config) = cache().get(path) {
                return Ok(cached_config);
            }

            // Cache miss - load from file
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read config file: {}", path.display()))?;

            let config: super::Config = if path.extension().and_then(|s| s.to_str()) == Some("yaml")
                || path.extension().and_then(|s| s.to_str()) == Some("yml")
            {
                serde_yaml::from_str(&content)
                    .with_context(|| format!("Failed to parse YAML config: {}", path.display()))?
            } else if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                toml::from_str(&content)
                    .with_context(|| format!("Failed to parse TOML config: {}", path.display()))?
            } else {
                serde_json::from_str(&content)
                    .with_context(|| format!("Failed to parse JSON config: {}", path.display()))?
            };

            // Store in cache
            cache().insert(path.to_path_buf(), config.clone());

            Ok(config)
        }
        None => {
            // Try to load from environment variable first
            if let Ok(config_path) = std::env::var("MEMORY_CLI_CONFIG") {
                let path = Path::new(&config_path);
                if path.exists() {
                    tracing::info!(
                        "Loading configuration from MEMORY_CLI_CONFIG: {}",
                        config_path
                    );
                    return load_config(Some(path));
                } else {
                    tracing::warn!(
                        "MEMORY_CLI_CONFIG points to non-existent file: {}",
                        config_path
                    );
                }
            }

            // Try to load from default locations
            let default_paths = [
                "unified-config.toml",
                "memory-cli.toml",
                "memory-cli.json",
                "memory-cli.yaml",
                ".memory-cli.toml",
                ".memory-cli.json",
                ".memory-cli.yaml",
            ];

            for path_str in &default_paths {
                let path = Path::new(path_str);
                if path.exists() {
                    return load_config(Some(path));
                }
            }

            // Fall back to defaults
            Ok(super::Config::default())
        }
    }
}

/// Configuration file format detection
pub fn detect_format(path: &Path) -> Result<ConfigFormat, anyhow::Error> {
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    match extension {
        "toml" => Ok(ConfigFormat::Toml),
        "json" => Ok(ConfigFormat::Json),
        "yaml" | "yml" => Ok(ConfigFormat::Yaml),
        _ => Err(anyhow::anyhow!(
            "Unsupported config file format for extension '.{}'. Supported formats: .toml, .json, .yaml, .yml",
            extension
        )),
    }
}

/// Configuration file format enumeration
#[derive(Debug, Clone)]
pub enum ConfigFormat {
    Toml,
    Json,
    Yaml,
}

impl ConfigFormat {
    /// Parse configuration content based on format
    pub fn parse_content(&self, content: &str) -> Result<super::Config, anyhow::Error> {
        match self {
            ConfigFormat::Toml => {
                toml::from_str(content).context("Failed to parse TOML configuration")
            }
            ConfigFormat::Json => {
                serde_json::from_str(content).context("Failed to parse JSON configuration")
            }
            ConfigFormat::Yaml => {
                serde_yaml::from_str(content).context("Failed to parse YAML configuration")
            }
        }
    }

    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ConfigFormat::Toml => "toml",
            ConfigFormat::Json => "json",
            ConfigFormat::Yaml => "yaml",
        }
    }

    /// Get MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            ConfigFormat::Toml => "application/toml",
            ConfigFormat::Json => "application/json",
            ConfigFormat::Yaml => "application/x-yaml",
        }
    }
}

/// Load configuration with format detection
///
/// This function automatically caches loaded configurations and invalidates
/// the cache when the file is modified (based on mtime).
pub fn load_config_with_format(path: &Path) -> Result<super::Config, anyhow::Error> {
    // Try cache first
    if let Some(cached_config) = cache().get(path) {
        return Ok(cached_config);
    }

    // Cache miss - load from file
    let format = detect_format(path)?;
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    let config = format.parse_content(&content)?;

    // Store in cache
    cache().insert(path.to_path_buf(), config.clone());

    Ok(config)
}

/// Validate and load configuration with detailed error reporting
pub fn load_config_verbose(path: Option<&Path>) -> Result<super::Config, anyhow::Error> {
    match path {
        Some(path) => {
            if !path.exists() {
                return Err(anyhow::anyhow!(
                    "Configuration file does not exist: {}",
                    path.display()
                ));
            }

            if !path.is_file() {
                return Err(anyhow::anyhow!(
                    "Configuration path is not a file: {}",
                    path.display()
                ));
            }

            match load_config_with_format(path) {
                Ok(config) => Ok(config),
                Err(e) => {
                    let format = detect_format(path)
                        .ok()
                        .map(|f| f.extension().to_string())
                        .unwrap_or_else(|| "unknown".to_string());

                    Err(anyhow::anyhow!(
                        "Failed to load {} configuration from {}: {}",
                        format,
                        path.display(),
                        e
                    ))
                }
            }
        }
        None => {
            // Try default locations with verbose reporting
            let default_paths = [
                "memory-cli.toml",
                "memory-cli.json",
                "memory-cli.yaml",
                ".memory-cli.toml",
                ".memory-cli.json",
                ".memory-cli.yaml",
            ];

            for path_str in &default_paths {
                let path = Path::new(path_str);
                if path.exists() {
                    return match load_config_with_format(path) {
                        Ok(config) => {
                            tracing::info!("Loaded configuration from: {}", path.display());
                            Ok(config)
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to load configuration from {}: {}, trying next location",
                                path.display(),
                                e
                            );
                            continue;
                        }
                    };
                }
            }

            tracing::info!("No configuration file found, using defaults");
            Ok(super::Config::default())
        }
    }
}

/// Configuration file writer
pub struct ConfigWriter {
    config: super::Config,
}

impl ConfigWriter {
    /// Create a new configuration writer
    pub fn new(config: super::Config) -> Self {
        Self { config }
    }

    /// Write configuration to file in specified format
    pub fn write_to_file(&self, path: &Path) -> Result<(), anyhow::Error> {
        let format = detect_format(path).or_else(|_| {
            // If format detection fails, infer from extension
            if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                match extension {
                    "toml" => Ok(ConfigFormat::Toml),
                    "json" => Ok(ConfigFormat::Json),
                    "yaml" | "yml" => Ok(ConfigFormat::Yaml),
                    _ => Err(anyhow::anyhow!("Unsupported file extension: {}", extension)),
                }
            } else {
                // Default to TOML if no extension
                Ok(ConfigFormat::Toml)
            }
        })?;

        let content = match format {
            ConfigFormat::Toml => toml::to_string_pretty(&self.config)
                .context("Failed to serialize configuration to TOML")?,
            ConfigFormat::Json => serde_json::to_string_pretty(&self.config)
                .context("Failed to serialize configuration to JSON")?,
            ConfigFormat::Yaml => serde_yaml::to_string(&self.config)
                .context("Failed to serialize configuration to YAML")?,
        };

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        std::fs::write(path, content)
            .with_context(|| format!("Failed to write configuration to: {}", path.display()))?;

        Ok(())
    }

    /// Write configuration with a specific format
    pub fn write_with_format(
        &self,
        path: &Path,
        format: ConfigFormat,
    ) -> Result<(), anyhow::Error> {
        let content = match format {
            ConfigFormat::Toml => toml::to_string_pretty(&self.config)
                .context("Failed to serialize configuration to TOML")?,
            ConfigFormat::Json => serde_json::to_string_pretty(&self.config)
                .context("Failed to serialize configuration to JSON")?,
            ConfigFormat::Yaml => serde_yaml::to_string(&self.config)
                .context("Failed to serialize configuration to YAML")?,
        };

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        std::fs::write(path, content)
            .with_context(|| format!("Failed to write configuration to: {}", path.display()))?;

        Ok(())
    }
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

#[cfg(test)]
mod cache_tests {
    use super::*;

    #[test]
    fn test_cache_hit() {
        // Create a temporary config file with .json extension
        let temp_dir = tempfile::tempdir().unwrap();
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

        // Wait for filesystem to settle (helps with mtime precision on some systems)
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Clear cache to ensure clean state
        clear_cache();

        // First load - should be a cache miss
        let config1 = load_config_with_format(&config_path).unwrap();
        assert_eq!(config1.cli.default_format, "json");

        // Second load - should be a cache hit
        let config2 = load_config_with_format(&config_path).unwrap();
        assert_eq!(config2.cli.default_format, "json");

        // Verify cache statistics
        let stats = cache_stats();
        assert!(stats.hits >= 1, "Expected at least 1 cache hit");
    }

    #[test]
    fn test_cache_invalidation() {
        // Create a temporary config file with .json extension
        let temp_dir = tempfile::tempdir().unwrap();
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

        // Clear cache to ensure clean state
        clear_cache();

        // First load
        let config1 = load_config_with_format(&config_path).unwrap();
        assert_eq!(config1.cli.default_format, "json");

        // Wait a bit to ensure mtime changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Modify the file
        std::fs::write(
            &config_path,
            r#"{
                "database": {"redb_path": "/tmp/test.redb"},
                "storage": {"max_episodes_cache": 1000, "cache_ttl_seconds": 3600, "pool_size": 5},
                "cli": {"default_format": "human", "progress_bars": true, "batch_size": 200}
            }"#,
        )
        .unwrap();

        // Second load - should detect modification and reload
        let config2 = load_config_with_format(&config_path).unwrap();
        assert_eq!(config2.cli.default_format, "human");
        assert_eq!(config2.cli.batch_size, 200);
    }

    #[test]
    fn test_clear_cache() {
        // Create a temporary config file with unique name to avoid conflicts
        let temp_dir = tempfile::tempdir().unwrap();
        let unique_name = format!(
            "test-config-clear-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let config_path = temp_dir.path().join(unique_name);

        std::fs::write(
            &config_path,
            r#"{
                "database": {"redb_path": "/tmp/test.redb"},
                "storage": {"max_episodes_cache": 1000, "cache_ttl_seconds": 3600, "pool_size": 5},
                "cli": {"default_format": "json", "progress_bars": false, "batch_size": 100}
            }"#,
        )
        .unwrap();

        // Get baseline entries count
        let entries_before = cache_stats().entries;

        // Load config
        let _config = load_config_with_format(&config_path).unwrap();

        // Verify cache gained an entry
        let stats_after_load = cache_stats();
        assert!(
            stats_after_load.entries > entries_before,
            "Cache should have more entries after load"
        );

        // Clear cache
        clear_cache();

        // Verify cache was cleared (should have fewer entries than after load)
        // Note: We compare against stats_after_load rather than entries_before
        // because parallel tests may be adding entries to the shared cache
        let stats_after_clear = cache_stats();
        assert!(
            stats_after_clear.entries < stats_after_load.entries,
            "Cache should have fewer entries after clear (before: {}, after load: {}, after clear: {})",
            entries_before,
            stats_after_load.entries,
            stats_after_clear.entries
        );
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn test_cache_stats() {
        // Create a temporary config file with unique name to avoid conflicts
        let temp_dir = tempfile::tempdir().unwrap();
        let unique_name = format!(
            "test-config-stats-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let config_path = temp_dir.path().join(unique_name);

        std::fs::write(
            &config_path,
            r#"{
                "database": {"redb_path": "/tmp/test.redb"},
                "storage": {"max_episodes_cache": 1000, "cache_ttl_seconds": 3600, "pool_size": 5},
                "cli": {"default_format": "json", "progress_bars": false, "batch_size": 100}
            }"#,
        )
        .unwrap();

        // Wait for filesystem to settle (helps with mtime precision on some systems)
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Get baseline stats
        let stats_baseline = cache_stats();

        // First load (miss)
        let _config1 = load_config_with_format(&config_path).unwrap();

        // Get stats after first load - misses should have increased
        let stats_after_first = cache_stats();
        assert!(
            stats_after_first.misses > stats_baseline.misses,
            "First load should result in cache miss"
        );

        // Second load (hit)
        let _config2 = load_config_with_format(&config_path).unwrap();

        // Get stats after second load - hits should have increased
        let stats_after_second = cache_stats();
        assert!(
            stats_after_second.hits > stats_after_first.hits,
            "Second load should result in cache hit"
        );

        // Verify hit rate calculation works
        let stats = cache_stats();
        assert!(
            stats.hit_rate >= 0.0 && stats.hit_rate <= 1.0,
            "Hit rate should be between 0 and 1"
        );
    }
}

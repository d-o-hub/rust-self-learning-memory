//! Configuration loader module
//!
//! This module handles loading configuration from various file formats
//! and default locations, providing a clean separation from validation
//! and storage initialization logic.
//!
//! Caching is provided to avoid re-parsing configuration files on every
//! invocation. The cache automatically invalidates when files are modified.

use anyhow::Context;
use std::path::Path;

mod cache;
mod env;
mod file;

pub use cache::{CacheStats, cache_stats, clear_cache};
pub use env::{
    EnvConfigInfo, get_env_config_info, load_config_from_defaults, load_config_from_env,
};
pub use file::{
    ConfigFormat, ConfigWriter, detect_format, load_config_from_file, load_config_verbose,
};

/// Load configuration from file or use defaults
///
/// This function automatically caches loaded configurations and invalidates
/// the cache when the file is modified (based on mtime).
pub fn load_config(path: Option<&Path>) -> Result<super::Config, anyhow::Error> {
    match path {
        Some(path) => {
            // Try cache first
            if let Some(cached_config) = cache::load_cached_config(path) {
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
            cache::insert_cached_config(path.to_path_buf(), config.clone());

            Ok(config)
        }
        None => {
            // Try to load from environment variable first
            if let Some((env_path, _)) = load_config_from_env() {
                return load_config(Some(&env_path));
            }

            // Try to load from default locations
            if let Some((default_path, _)) = load_config_from_defaults() {
                return load_config(Some(&default_path));
            }

            // Fall back to defaults
            Ok(super::Config::default())
        }
    }
}

/// Load configuration with format detection and caching
///
/// This function automatically caches loaded configurations and invalidates
/// the cache when the file is modified (based on mtime).
pub fn load_config_with_format(path: &Path) -> Result<super::Config, anyhow::Error> {
    // Try cache first
    if let Some(cached_config) = cache::load_cached_config(path) {
        return Ok(cached_config);
    }

    // Cache miss - load from file
    let format = detect_format(path)?;
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    let config = format.parse_content(&content)?;

    // Store in cache
    cache::insert_cached_config(path.to_path_buf(), config.clone());

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_load_config_from_nonexistent() {
        let path = Path::new("/nonexistent/path.toml");
        let result = load_config(Some(path));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_config_with_format_not_found() {
        let path = Path::new("/nonexistent/config.toml");
        let result = load_config_with_format(path);
        assert!(result.is_err());
    }

    /// Issue #829: partial configs must load via #[serde(default)] on Config sections.
    #[test]
    fn test_partial_config_loads_with_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("partial.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            r#"
[database]
redb_path = "./data/memory.redb"
storage_mode = "local"
"#
        )
        .unwrap();

        let config = load_config(Some(&path)).expect("partial config should parse");
        assert_eq!(
            config.database.redb_path.as_deref(),
            Some("./data/memory.redb")
        );
        assert_eq!(config.database.storage_mode.as_deref(), Some("local"));
        // Storage/cli sections filled from Default
        assert!(config.storage.max_episodes_cache > 0);
        assert!(config.storage.pool_size > 0);
        assert!(!config.cli.default_format.is_empty());
    }

    /// Issue #832: storage_mode under [storage] is accepted as an alias.
    #[test]
    fn test_storage_mode_alias_under_storage_section() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("alias.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            r#"
[database]
redb_path = "./data/memory.redb"

[storage]
storage_mode = "local"
max_episodes_cache = 500
"#
        )
        .unwrap();

        let mut config = load_config(Some(&path)).expect("config with alias should parse");
        assert_eq!(config.storage.storage_mode.as_deref(), Some("local"));
        config.normalize_storage_mode();
        assert_eq!(config.database.storage_mode.as_deref(), Some("local"));
        assert!(config.storage.storage_mode.is_none());
    }

    /// Database.storage_mode wins when both locations are set.
    #[test]
    fn test_database_storage_mode_takes_precedence() {
        let mut config = crate::config::Config::default();
        config.database.storage_mode = Some("remote".to_string());
        config.storage.storage_mode = Some("local".to_string());
        config.normalize_storage_mode();
        assert_eq!(config.database.storage_mode.as_deref(), Some("remote"));
        assert!(config.storage.storage_mode.is_none());
    }
}

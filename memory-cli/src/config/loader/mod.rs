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
}

//! Environment variable handling module
//!
//! This module handles loading configuration from environment variables
//! and default file locations.

use std::path::{Path, PathBuf};

use super::ConfigFormat;

/// Try to load configuration from environment variable
pub fn load_config_from_env() -> Option<(PathBuf, ConfigFormat)> {
    if let Ok(config_path) = std::env::var("MEMORY_CLI_CONFIG") {
        let path = Path::new(&config_path);
        if path.exists() {
            tracing::info!(
                "Loading configuration from MEMORY_CLI_CONFIG: {}",
                config_path
            );
            let format = detect_format_from_path(path).ok()?;
            return Some((path.to_path_buf(), format));
        } else {
            tracing::warn!(
                "MEMORY_CLI_CONFIG points to non-existent file: {}",
                config_path
            );
        }
    }
    None
}

/// Try to load configuration from default locations
pub fn load_config_from_defaults() -> Option<(PathBuf, ConfigFormat)> {
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
            if let Ok(format) = detect_format_from_path(path) {
                return Some((path.to_path_buf(), format));
            }
        }
    }
    None
}

/// Detect format from file path
fn detect_format_from_path(path: &Path) -> Result<ConfigFormat, anyhow::Error> {
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    match extension {
        "toml" => Ok(ConfigFormat::Toml),
        "json" => Ok(ConfigFormat::Json),
        "yaml" | "yml" => Ok(ConfigFormat::Yaml),
        _ => Err(anyhow::anyhow!(
            "Unsupported config file format for extension '.{}'",
            extension
        )),
    }
}

/// Get environment variable info for debugging
pub fn get_env_config_info() -> EnvConfigInfo {
    EnvConfigInfo {
        memory_cli_config: std::env::var("MEMORY_CLI_CONFIG").ok(),
        turso_url: std::env::var("TURSO_URL").ok().is_some(),
        turso_token: std::env::var("TURSO_TOKEN").ok().is_some(),
        redb_path: std::env::var("REDB_PATH").ok().is_some(),
        ci: std::env::var("CI").is_ok(),
        development: std::env::var("DEVELOPMENT").is_ok() || std::env::var("DEV").is_ok(),
    }
}

/// Information about environment configuration
#[derive(Debug, Clone)]
pub struct EnvConfigInfo {
    /// Value of MEMORY_CLI_CONFIG if set
    pub memory_cli_config: Option<String>,
    /// Whether TURSO_URL is set
    pub turso_url: bool,
    /// Whether TURSO_TOKEN is set
    pub turso_token: bool,
    /// Whether REDB_PATH is set
    pub redb_path: bool,
    /// Whether CI environment is detected
    pub ci: bool,
    /// Whether development mode is enabled
    pub development: bool,
}

impl EnvConfigInfo {
    /// Get a summary of the environment configuration
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if let Some(config) = &self.memory_cli_config {
            parts.push(format!("MEMORY_CLI_CONFIG={}", config));
        }

        if self.turso_url {
            parts.push("TURSO_URL=set".to_string());
        }

        if self.turso_token {
            parts.push("TURSO_TOKEN=set".to_string());
        }

        if self.redb_path {
            parts.push("REDB_PATH=set".to_string());
        }

        if self.ci {
            parts.push("CI=true".to_string());
        }

        if self.development {
            parts.push("DEV=true".to_string());
        }

        if parts.is_empty() {
            "No environment configuration detected".to_string()
        } else {
            parts.join(", ")
        }
    }
}

#[cfg(test)]
mod env_tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_env_config_info_summary_with_vars() {
        // Store original values
        let original_values: std::collections::HashMap<String, Option<String>> = [
            "MEMORY_CLI_CONFIG".to_string(),
            "TURSO_URL".to_string(),
            "TURSO_TOKEN".to_string(),
            "REDB_PATH".to_string(),
            "CI".to_string(),
            "DEVELOPMENT".to_string(),
            "DEV".to_string(),
        ]
        .iter()
        .map(|name| {
            let value = std::env::var(name).ok();
            (name.clone(), value)
        })
        .collect();

        // Set our test values
        // SAFETY: test-only env var manipulation
        unsafe {
            std::env::set_var("MEMORY_CLI_CONFIG", "/path/to/config.toml");
            std::env::set_var("TURSO_URL", "libsql://test.db");
            std::env::set_var("CI", "true");
        }

        // Get info and verify struct fields directly (more reliable than summary)
        let info = get_env_config_info();

        // Verify struct fields
        assert_eq!(
            info.memory_cli_config,
            Some("/path/to/config.toml".to_string()),
            "MEMORY_CLI_CONFIG should be set"
        );
        assert!(info.turso_url, "TURSO_URL should be detected");
        assert!(info.ci, "CI should be detected");

        // Also verify summary contains expected values
        let summary = info.summary();
        assert!(
            summary.contains("MEMORY_CLI_CONFIG=/path/to/config.toml"),
            "Summary: {}",
            summary
        );
        assert!(summary.contains("TURSO_URL=set"), "Summary: {}", summary);
        assert!(summary.contains("CI=true"), "Summary: {}", summary);

        // Restore original environment
        // SAFETY: test-only env var manipulation
        unsafe {
            for (name, value) in original_values {
                if let Some(v) = value {
                    std::env::set_var(name, v);
                } else {
                    std::env::remove_var(name);
                }
            }
        }
    }

    #[test]
    #[serial]
    fn test_env_config_info_summary_with_vars_different() {
        // Store original values
        let original_values: std::collections::HashMap<String, Option<String>> = [
            "MEMORY_CLI_CONFIG".to_string(),
            "TURSO_URL".to_string(),
            "TURSO_TOKEN".to_string(),
            "REDB_PATH".to_string(),
            "CI".to_string(),
            "DEVELOPMENT".to_string(),
            "DEV".to_string(),
        ]
        .iter()
        .map(|name| {
            let value = std::env::var(name).ok();
            (name.clone(), value)
        })
        .collect();

        // SAFETY: test-only env var manipulation
        unsafe {
            std::env::set_var("MEMORY_CLI_CONFIG", "/path/to/config.toml");
            std::env::set_var("TURSO_URL", "libsql://test.db");
            std::env::set_var("CI", "true");
        }

        let info = get_env_config_info();
        let summary = info.summary();

        assert!(
            summary.contains("MEMORY_CLI_CONFIG=/path/to/config.toml"),
            "Summary: {}",
            summary
        );
        assert!(summary.contains("TURSO_URL=set"), "Summary: {}", summary);
        assert!(summary.contains("CI=true"), "Summary: {}", summary);

        // Restore original environment
        // SAFETY: test-only env var manipulation
        unsafe {
            for (name, value) in original_values {
                if let Some(v) = value {
                    std::env::set_var(name, v);
                } else {
                    std::env::remove_var(name);
                }
            }
        }
    }

    #[test]
    #[serial]
    fn test_load_config_from_env_not_set() {
        // SAFETY: test-only env var manipulation
        unsafe {
            std::env::remove_var("MEMORY_CLI_CONFIG");
        }
        let result = load_config_from_env();
        assert!(result.is_none());
    }

    #[test]
    fn test_load_config_from_defaults_not_found() {
        let result = load_config_from_defaults();
        assert!(result.is_none());
    }
}

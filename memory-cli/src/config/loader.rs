//! Configuration loader module
//!
//! This module handles loading configuration from various file formats
//! and default locations, providing a clean separation from validation
//! and storage initialization logic.

use anyhow::Context;
use std::path::Path;

/// Load configuration from file or use defaults
pub fn load_config(path: Option<&Path>) -> Result<super::Config, anyhow::Error> {
    match path {
        Some(path) => {
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read config file: {}", path.display()))?;

            if path.extension().and_then(|s| s.to_str()) == Some("yaml")
                || path.extension().and_then(|s| s.to_str()) == Some("yml")
            {
                serde_yaml::from_str(&content)
                    .with_context(|| format!("Failed to parse YAML config: {}", path.display()))
            } else if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                toml::from_str(&content)
                    .with_context(|| format!("Failed to parse TOML config: {}", path.display()))
            } else {
                serde_json::from_str(&content)
                    .with_context(|| format!("Failed to parse JSON config: {}", path.display()))
            }
        }
        None => {
            // Try to load from environment variable first
            if let Ok(config_path) = std::env::var("MEMORY_CLI_CONFIG") {
                let path = Path::new(&config_path);
                if path.exists() {
                    tracing::info!("Loading configuration from MEMORY_CLI_CONFIG: {}", config_path);
                    return load_config(Some(path));
                } else {
                    tracing::warn!("MEMORY_CLI_CONFIG points to non-existent file: {}", config_path);
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
pub fn load_config_with_format(path: &Path) -> Result<super::Config, anyhow::Error> {
    let format = detect_format(path)?;
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    format.parse_content(&content)
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

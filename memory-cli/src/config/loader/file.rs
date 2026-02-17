//! Configuration file loading module
//!
//! This module handles loading configuration from files with format detection,
//! supporting TOML, JSON, and YAML formats.

use anyhow::Context;
use std::path::Path;

use crate::config::Config;

/// Configuration file format enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigFormat {
    Toml,
    Json,
    Yaml,
}

impl ConfigFormat {
    /// Parse configuration content based on format
    pub fn parse_content(&self, content: &str) -> Result<Config, anyhow::Error> {
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

    /// Serialize configuration to format-specific string
    pub fn serialize_content(&self, config: &Config) -> Result<String, anyhow::Error> {
        match self {
            ConfigFormat::Toml => {
                toml::to_string_pretty(config).context("Failed to serialize configuration to TOML")
            }
            ConfigFormat::Json => serde_json::to_string_pretty(config)
                .context("Failed to serialize configuration to JSON"),
            ConfigFormat::Yaml => {
                serde_yaml::to_string(config).context("Failed to serialize configuration to YAML")
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

/// Load configuration from file with automatic format detection
pub fn load_config_from_file(path: &Path) -> Result<Config, anyhow::Error> {
    let format = detect_format(path)?;
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    let config = format.parse_content(&content)?;

    Ok(config)
}

/// Validate and load configuration with detailed error reporting
pub fn load_config_verbose(path: Option<&Path>) -> Result<Config, anyhow::Error> {
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

            match load_config_from_file(path) {
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
                    return match load_config_from_file(path) {
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
            Ok(Config::default())
        }
    }
}

/// Configuration file writer
pub struct ConfigWriter {
    config: Config,
}

impl ConfigWriter {
    /// Create a new configuration writer
    pub fn new(config: Config) -> Self {
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

#[cfg(test)]
mod file_tests {
    use super::*;
    use crate::config::types::EmbeddingsConfig;
    use tempfile::tempdir;

    #[test]
    fn test_detect_format_toml() {
        let path = Path::new("config.toml");
        let format = detect_format(path).unwrap();
        assert_eq!(format, ConfigFormat::Toml);
    }

    #[test]
    fn test_detect_format_json() {
        let path = Path::new("config.json");
        let format = detect_format(path).unwrap();
        assert_eq!(format, ConfigFormat::Json);
    }

    #[test]
    fn test_detect_format_yaml() {
        let path = Path::new("config.yaml");
        let format = detect_format(path).unwrap();
        assert_eq!(format, ConfigFormat::Yaml);
    }

    #[test]
    fn test_detect_format_yml() {
        let path = Path::new("config.yml");
        let format = detect_format(path).unwrap();
        assert_eq!(format, ConfigFormat::Yaml);
    }

    #[test]
    fn test_detect_format_unsupported() {
        let path = Path::new("config.txt");
        let result = detect_format(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_writer_toml() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        // Create a minimal config
        let config = Config {
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
        };

        let writer = ConfigWriter::new(config);
        writer.write_to_file(&config_path).unwrap();

        // Verify file was created
        assert!(config_path.exists());

        // Verify content is valid TOML
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("redb_path"));
        assert!(content.contains("max_episodes_cache"));
    }
}

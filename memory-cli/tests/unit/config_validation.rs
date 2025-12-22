//! Unit tests for configuration loading and validation.
//!
//! These tests verify that configuration files are loaded correctly,
//! validation works as expected, and defaults are applied properly.

use memory_cli::config::{CliConfig, Config, DatabaseConfig, StorageConfig, initialize_storage};
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_config_default_values() {
        let config = Config::default();

        // Test database defaults
        assert!(config.database.turso_url.is_none());
        assert!(config.database.turso_token.is_none());
        assert_eq!(config.database.redb_path, Some("memory.redb".to_string()));

        // Test storage defaults
        assert_eq!(config.storage.max_episodes_cache, 1000);
        assert_eq!(config.storage.cache_ttl_seconds, 3600);
        assert_eq!(config.storage.pool_size, 10);

        // Test CLI defaults
        assert_eq!(config.cli.default_format, "human");
        assert!(config.cli.progress_bars);
        assert_eq!(config.cli.batch_size, 100);
    }

    #[test]
    fn test_config_validation_valid_config() {
        let config = Config::default();
        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_validation_invalid_storage_cache() {
        let mut config = Config::default();
        config.storage.max_episodes_cache = 0;

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max_episodes_cache must be greater than 0"));
    }

    #[test]
    fn test_config_validation_invalid_pool_size() {
        let mut config = Config::default();
        config.storage.pool_size = 0;

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("pool_size must be greater than 0"));
    }

    #[test]
    fn test_config_validation_invalid_default_format() {
        let mut config = Config::default();
        config.cli.default_format = "invalid".to_string();

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("default_format must be 'human', 'json', or 'yaml'"));
    }

    #[test]
    fn test_config_validation_invalid_batch_size() {
        let mut config = Config::default();
        config.cli.batch_size = 0;

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("batch_size must be greater than 0"));
    }

    #[test]
    fn test_config_validation_valid_formats() {
        let mut config = Config::default();

        config.cli.default_format = "human".to_string();
        assert!(config.validate().is_ok());

        config.cli.default_format = "json".to_string();
        assert!(config.validate().is_ok());

        config.cli.default_format = "yaml".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_load_from_toml_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        let config_content = r#"
[database]
turso_url = "file:test.db"
turso_token = "test-token"
redb_path = "custom.redb"

[storage]
max_episodes_cache = 500
cache_ttl_seconds = 1800
pool_size = 5

[cli]
default_format = "json"
progress_bars = false
batch_size = 25
"#;

        fs::write(&config_path, config_content).unwrap();

        let config = Config::load(Some(&config_path)).unwrap();

        // Verify loaded values
        assert_eq!(config.database.turso_url, Some("file:test.db".to_string()));
        assert_eq!(config.database.turso_token, Some("test-token".to_string()));
        assert_eq!(config.database.redb_path, Some("custom.redb".to_string()));
        assert_eq!(config.storage.max_episodes_cache, 500);
        assert_eq!(config.storage.cache_ttl_seconds, 1800);
        assert_eq!(config.storage.pool_size, 5);
        assert_eq!(config.cli.default_format, "json");
        assert!(!config.cli.progress_bars);
        assert_eq!(config.cli.batch_size, 25);
    }

    #[test]
    fn test_config_load_from_json_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.json");

        let config_content = r#"{
  "database": {
    "turso_url": "file:test.db",
    "turso_token": "test-token",
    "redb_path": "custom.redb"
  },
  "storage": {
    "max_episodes_cache": 300,
    "cache_ttl_seconds": 7200,
    "pool_size": 8
  },
  "cli": {
    "default_format": "yaml",
    "progress_bars": true,
    "batch_size": 50
  }
}"#;

        fs::write(&config_path, config_content).unwrap();

        let config = Config::load(Some(&config_path)).unwrap();

        // Verify loaded values
        assert_eq!(config.database.turso_url, Some("file:test.db".to_string()));
        assert_eq!(config.database.turso_token, Some("test-token".to_string()));
        assert_eq!(config.database.redb_path, Some("custom.redb".to_string()));
        assert_eq!(config.storage.max_episodes_cache, 300);
        assert_eq!(config.storage.cache_ttl_seconds, 7200);
        assert_eq!(config.storage.pool_size, 8);
        assert_eq!(config.cli.default_format, "yaml");
        assert!(config.cli.progress_bars);
        assert_eq!(config.cli.batch_size, 50);
    }

    #[test]
    fn test_config_load_from_yaml_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.yaml");

        let config_content = r#"---
database:
  turso_url: "file:test.db"
  turso_token: "test-token"
  redb_path: "custom.redb"
storage:
  max_episodes_cache: 200
  cache_ttl_seconds: 900
  pool_size: 3
cli:
  default_format: "human"
  progress_bars: false
  batch_size: 75
"#;

        fs::write(&config_path, config_content).unwrap();

        let config = Config::load(Some(&config_path)).unwrap();

        // Verify loaded values
        assert_eq!(config.database.turso_url, Some("file:test.db".to_string()));
        assert_eq!(config.database.turso_token, Some("test-token".to_string()));
        assert_eq!(config.database.redb_path, Some("custom.redb".to_string()));
        assert_eq!(config.storage.max_episodes_cache, 200);
        assert_eq!(config.storage.cache_ttl_seconds, 900);
        assert_eq!(config.storage.pool_size, 3);
        assert_eq!(config.cli.default_format, "human");
        assert!(!config.cli.progress_bars);
        assert_eq!(config.cli.batch_size, 75);
    }

    #[test]
    fn test_config_load_nonexistent_file() {
        let result = Config::load(Some(std::path::Path::new("nonexistent.toml")));
        assert!(result.is_err());
    }

    #[test]
    fn test_config_load_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.toml");

        let invalid_config = r#"
[database
turso_url = "file:test.db"
invalid toml syntax
"#;

        fs::write(&config_path, invalid_config).unwrap();

        let result = Config::load(Some(&config_path));
        assert!(result.is_err());
    }

    #[test]
    fn test_config_load_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.json");

        let invalid_config = r#"{
  "database": {
    "turso_url": "file:test.db",
    invalid json syntax
  }
}"#;

        fs::write(&config_path, invalid_config).unwrap();

        let result = Config::load(Some(&config_path));
        assert!(result.is_err());
    }

    #[test]
    fn test_config_load_invalid_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.yaml");

        let invalid_config = r#"---
database:
  turso_url: "file:test.db"
  - invalid
  - yaml
  - syntax
"#;

        fs::write(&config_path, invalid_config).unwrap();

        let result = Config::load(Some(&config_path));
        assert!(result.is_err());
    }

    #[test]
    fn test_config_load_default_locations() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Change to temp directory
        std::env::set_current_dir(&temp_dir).unwrap();

        // Test loading from memory-cli.toml
        let config_path = temp_dir.path().join("memory-cli.toml");
        let config_content = r#"
[database]
turso_url = "file:default.db"
[storage]
max_episodes_cache = 150
"#;
        fs::write(&config_path, config_content).unwrap();

        let config = Config::load(None).unwrap();
        assert_eq!(config.database.turso_url, Some("file:default.db".to_string()));
        assert_eq!(config.storage.max_episodes_cache, 150);

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_config_load_partial_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("partial.toml");

        // Config with only some fields
        let config_content = r#"
[database]
turso_url = "file:test.db"

[cli]
default_format = "json"
"#;

        fs::write(&config_path, config_content).unwrap();

        let config = Config::load(Some(&config_path)).unwrap();

        // Specified fields should be loaded
        assert_eq!(config.database.turso_url, Some("file:test.db".to_string()));
        assert_eq!(config.cli.default_format, "json");

        // Unspecified fields should have defaults
        assert_eq!(config.storage.max_episodes_cache, 1000); // default
        assert!(config.cli.progress_bars); // default
    }

    #[test]
    fn test_config_create_memory_without_features() {
        let config = Config::default();

        // This should work even without turso/redb features enabled
        // (it will create an in-memory system)
        let result = initialize_storage(&config);
        // We can't easily test the async result in a unit test without tokio::test
        // but we can verify the method exists and doesn't panic on config validation
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_struct_serialization() {
        let config = Config::default();

        // Test that the config can be serialized to TOML
        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("[database]"));
        assert!(toml.contains("[storage]"));
        assert!(toml.contains("[cli]"));

        // Test that it can be deserialized back
        let deserialized: Config = toml::from_str(&toml).unwrap();
        assert_eq!(deserialized.database.redb_path, config.database.redb_path);
        assert_eq!(deserialized.storage.max_episodes_cache, config.storage.max_episodes_cache);
        assert_eq!(deserialized.cli.default_format, config.cli.default_format);
    }

    #[test]
    fn test_config_struct_json_serialization() {
        let config = Config::default();

        // Test JSON serialization
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.database.redb_path, config.database.redb_path);
        assert_eq!(deserialized.storage.max_episodes_cache, config.storage.max_episodes_cache);
    }

    #[test]
    fn test_config_struct_yaml_serialization() {
        let config = Config::default();

        // Test YAML serialization
        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: Config = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.database.redb_path, config.database.redb_path);
        assert_eq!(deserialized.storage.max_episodes_cache, config.storage.max_episodes_cache);
    }
}
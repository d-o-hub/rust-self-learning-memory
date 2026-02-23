//! Unit tests for the new modular configuration system

use super::*;
use tempfile::TempDir;
use std::fs;
use memory_cli::config::{initialize_storage};

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_config_types_creation() {
        // Test Config creation and defaults
        let config = Config::default();
        
        assert!(config.database.turso_url.is_none());
        assert!(config.database.turso_token.is_none());
        assert_eq!(config.database.redb_path, Some("./data/memory.redb".to_string()));
        
        assert_eq!(config.storage.max_episodes_cache, 1000);
        assert_eq!(config.storage.cache_ttl_seconds, 3600);
        assert_eq!(config.storage.pool_size, 10);
        
        assert_eq!(config.cli.default_format, "!(config.cli.progresshuman");
        assert_bars);
        assert_eq!(config.cli.batch_size, 100);
    }

    #[test]
    fn test_config_presets() {
        // Test Local preset
        let local_config = ConfigPreset::Local.create_config();
        assert_eq!(local_config.storage.max_episodes_cache, 1000);
        assert!(local_config.database.redb_path.is_some());

        // Test Cloud preset
        let cloud_config = ConfigPreset::Cloud.create_config();
        assert_eq!(cloud_config.storage.max_episodes_cache, 5000);
        assert!(cloud_config.database.turso_url.is_some());

        // Test Memory preset
        let memory_config = ConfigPreset::Memory.create_config();
        assert_eq!(memory_config.storage.max_episodes_cache, 100);
        assert_eq!(memory_config.database.redb_path, Some(":memory:".to_string()));
    }

    #[test]
    fn test_validation_errors() {
        let mut config = Config::default();
        
        // Test missing database configuration
        config.database.turso_url = None;
        config.database.redb_path = None;
        
        let result = validate_config(&config);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        
        // Check that error mentions database configuration
        let has_db_error = result.errors.iter().any(|e| 
            e.field == "database" && e.message.contains("At least one database configuration")
        );
        assert!(has_db_error);
    }

    #[test]
    fn test_validation_warnings() {
        let mut config = Config::default();
        config.storage.max_episodes_cache = 100000; // Large value
        
        let result = validate_config(&config);
        assert!(result.is_valid); // Should still be valid
        assert!(!result.warnings.is_empty()); // But should have warnings
        
        // Check that warning mentions large cache size
        let has_cache_warning = result.warnings.iter().any(|w|
            w.field == "storage.max_episodes_cache" && 
            w.message.contains("Large cache size")
        );
        assert!(has_cache_warning);
    }

    #[test]
    fn test_environment_check() {
        let check = EnvironmentCheck::new();
        
        // Should have reasonable defaults
        assert!(matches!(check.recommended_preset, ConfigPreset::Local));
        
        // Should have a summary
        let summary = check.summary();
        assert!(summary.contains("Turso available"));
        assert!(summary.contains("redb available"));
        assert!(summary.contains("Recommended preset"));
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        
        // Test TOML serialization
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("[database]"));
        assert!(toml_str.contains("[storage]"));
        assert!(toml_str.contains("[cli]"));
        
        // Test deserialization
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.storage.max_episodes_cache, config.storage.max_episodes_cache);
    }

    #[test]
    fn test_config_loader() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");
        
        let config_content = r#"
[database]
turso_url = "file:test.db"
redb_path = "custom.redb"

[storage]
max_episodes_cache = 500

[cli]
default_format = "json"
"#;
        
        fs::write(&config_path, config_content).unwrap();
        
        let config = load_config(Some(&config_path)).unwrap();
        assert_eq!(config.database.turso_url, Some("file:test.db".to_string()));
        assert_eq!(config.database.redb_path, Some("custom.redb".to_string()));
        assert_eq!(config.storage.max_episodes_cache, 500);
        assert_eq!(config.cli.default_format, "json");
    }

    #[test]
    fn test_config_format_detection() {
        // Test TOML format detection
        let toml_path = std::path::Path::new("config.toml");
        assert_eq!(detect_format(toml_path).unwrap(), ConfigFormat::Toml);
        
        // Test JSON format detection
        let json_path = std::path::Path::new("config.json");
        assert_eq!(detect_format(json_path).unwrap(), ConfigFormat::Json);
        
        // Test YAML format detection
        let yaml_path = std::path::Path::new("config.yaml");
        assert_eq!(detect_format(yaml_path).unwrap(), ConfigFormat::Yaml);
        
        let yml_path = std::path::Path::new("config.yml");
        assert_eq!(detect_format(yml_path).unwrap(), ConfigFormat::Yaml);
        
        // Test unsupported format
        let txt_path = std::path::Path::new("config.txt");
        assert!(detect_format(txt_path).is_err());
    }

    #[test]
    fn test_config_writer() {
        let config = Config::default();
        let writer = create_writer(config.clone());
        
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("output.toml");
        
        writer.write_to_file(&config_path).unwrap();
        
        // Verify file was created
        assert!(config_path.exists());
        
        // Verify content can be loaded back
        let loaded_config = load_config(Some(&config_path)).unwrap();
        assert_eq!(loaded_config.storage.max_episodes_cache, config.storage.max_episodes_cache);
    }

    #[test]
    fn test_simple_config_builder() {
        let simple_config = SimpleConfig::preset(ConfigPreset::Local);
        let config = simple_config.build().unwrap();
        
        assert_eq!(config.storage.max_episodes_cache, 1000);
        assert!(config.database.redb_path.is_some());
    }

    #[test]
    fn test_simple_config_with_overrides() {
        let mut override_config = Config::default();
        override_config.storage.max_episodes_cache = 2000;
        
        let simple_config = SimpleConfig::preset(ConfigPreset::Local)
            .override_with(override_config.clone());
        let config = simple_config.build().unwrap();
        
        assert_eq!(config.storage.max_episodes_cache, 2000); // Override applied
    }

    #[test]
    fn test_quick_validation_check() {
        let config = Config::default();
        let issues = quick_validation_check(&config);
        
        // Should have no critical issues for default config
        assert!(issues.is_empty());
        
        // Test with problematic config
        let mut bad_config = Config::default();
        bad_config.storage.max_episodes_cache = 0;
        
        let bad_issues = quick_validation_check(&bad_config);
        assert!(!bad_issues.is_empty());
        
        let has_cache_issue = bad_issues.iter().any(|issue|
            issue.contains("Cache size is 0")
        );
        assert!(has_cache_issue);
    }

    #[test]
    fn test_validation_result_formatting() {
        let mut result = ValidationResult::ok();
        result.warnings.push(ValidationWarning {
            field: "test.field".to_string(),
            message: "Test warning message".to_string(),
            suggestion: Some("Test suggestion".to_string()),
        });
        
        let formatted = format_validation_result(&result);
        assert!(formatted.contains("✅ Configuration is valid"));
        assert!(formatted.contains("⚠️  Configuration has 1 warning(s):"));
        assert!(formatted.contains("Test warning message"));
    }

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError {
            field: "database.turso_url".to_string(),
            message: "Invalid URL format".to_string(),
            suggestion: Some("Use libsql:// format".to_string()),
            context: Some("Remote database access".to_string()),
        };
        
        let error_str = error.to_string();
        assert!(error_str.contains("Invalid URL format"));
        assert!(error_str.contains("Suggestion: Use libsql:// format"));
        assert!(error_str.contains("Context: Remote database access"));
    }

    #[test]
    fn test_config_summary() {
        let config = Config::default();
        let summary = get_config_summary(&config);
        
        assert!(summary.contains("Config Summary:"));
        assert!(summary.contains("Database:"));
        assert!(summary.contains("Storage:"));
        assert!(summary.contains("CLI:"));
        assert!(summary.contains("redb only")); // Default config uses redb only
    }

    #[test]
    fn test_readiness_check() {
        let check = ReadinessCheck::new();
        
        // Should have a report even if there are issues
        let report = check.report();
        assert!(!report.is_empty());
        
        // Report should contain either ready message or issues
        assert!(report.contains("✅") || report.contains("❌"));
    }

    #[test]
    fn test_template_generation() {
        let template = generate_template().unwrap();
        
        assert!(template.contains("# Memory CLI Configuration Template"));
        assert!(template.contains("[database]"));
        assert!(template.contains("[storage]"));
        assert!(template.contains("[cli]"));
    }

    #[test]
    fn test_config_migration() {
        let old_config = Config::default();
        let migrated_config = migrate_config(&old_config);
        
        // Should be identical for now since we're maintaining compatibility
        assert_eq!(migrated_config.storage.max_episodes_cache, old_config.storage.max_episodes_cache);
        assert_eq!(migrated_config.cli.default_format, old_config.cli.default_format);
    }
}

#[cfg(test)]
mod backward_compatibility_tests {
    use super::*;

    #[test]
    fn test_legacy_config_struct_compatibility() {
        // Test that the legacy Config struct is still accessible
        use config::Config;
        
        let config = Config::default();
        assert!(config.database.turso_url.is_none());
        assert_eq!(config.storage.max_episodes_cache, 1000);
    }

    #[test]
    fn test_legacy_load_function() {
        // Test that the legacy load function still works
        use config::load;
        
        // Should return default config for non-existent file
        let config = load(None).unwrap();
        assert_eq!(config.storage.max_episodes_cache, 1000);
    }

    #[test]
    fn test_legacy_validate_function() {
        // Test that the legacy validate function still works
        use config::Config;
        
        let config = Config::default();
        assert!(config.validate().is_ok());
        
        // Test with invalid config
        let mut invalid_config = Config::default();
        invalid_config.storage = 0;
.max_episodes_cache        
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_legacy_create_memory_function() {
        // Test that the legacy create_memory function signature exists
        use config::Config;
        
        let config = Config::default();
        
        // This is an async test, so we'll just verify the method exists
        // The actual async behavior is tested in integration tests
        let _future = initialize_storage(&config);
        // The future should exist and be ready to await
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_load_and_init_integration() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");
        
        // Use unique database path per test to avoid lock conflicts
        let redb_path = temp_dir.path().join("test.redb");
        let redb_str = redb_path.display().to_string().replace('\\', "/");
        
        let config_content = format!(
            r#"
[database]
redb_path = "{}"

[storage]
max_episodes_cache = 100

[cli]
default_format = "json"
"#,
            redb_str
        );
        
        std::fs::write(&config_path, config_content).unwrap();
        
        let result = load_and_init(Some(&config_path)).await;
        assert!(result.is_ok());
        
        let (config, storage_result) = result.unwrap();
        assert_eq!(config.storage.max_episodes_cache, 100);
        assert_eq!(config.cli.default_format, "json");
        
        // Storage should be initialized
        assert!(matches!(storage_result.storage_info.cache_storage, StorageType::Redb));
    }

    #[tokio::test]
    #[allow(unsafe_code)]
    async fn test_auto_configure_integration() {
        // Set up environment for testing
        // SAFETY: test-only env var manipulation
        unsafe {
            std::env::set_var("LOCAL_DATABASE_URL", "file:test.db");
        }

        let result = auto_configure().await;
        assert!(result.is_ok());

        let (config, storage_result) = result.unwrap();

        // Should successfully configure based on environment
        assert!(config.database.turso_url.is_some() || config.database.redb_path.is_some());

        // Clean up
        // SAFETY: test-only env var manipulation
        unsafe {
            std::env::remove_var("LOCAL_DATABASE_URL");
        }
    }

    #[tokio::test]
    async fn test_simple_setup_functions() {
        // Test local setup
        let result = setup_local().await;
        assert!(result.is_ok());
        
        let (config, storage) = result.unwrap();
        assert_eq!(config.storage.max_episodes_cache, 1000);
        assert!(storage.storage_info.status_messages.len() > 0);
    }

    #[tokio::test]
    async fn test_environment_check_auto_setup() {
        let check = EnvironmentCheck::new();
        let result = setup_with_environment(&check).await;
        assert!(result.is_ok());
    }
}
    #[test]
    fn test_enhanced_smart_defaults() {
        // Test that enhanced smart defaults are working
        let config = Config::default();
        
        // Verify smart defaults are being applied
        assert!(config.database.redb_path.is_some());
        
        // The defaults should be smart (not just hardcoded values)
        let redb_path = config.database.redb_path.as_ref().unwrap();
        assert!(!redb_path.is_empty());
        
        // Test that storage config has reasonable defaults
        assert!(config.storage.max_episodes_cache > 0);
        assert!(config.storage.pool_size > 0);
        assert!(config.storage.cache_ttl_seconds > 0);
        
        // Test CLI config defaults
        assert!(!config.cli.default_format.is_empty());
        assert!(config.cli.batch_size > 0);
        
        println!("Enhanced smart defaults test passed!");
    }

    #[test]
    fn test_smart_presets() {
        // Test that presets use smart defaults
        let local_config = ConfigPreset::Local.create_config();
        let cloud_config = ConfigPreset::Cloud.create_config();
        let memory_config = ConfigPreset::Memory.create_config();
        
        // All configs should have valid paths
        assert!(local_config.database.redb_path.is_some());
        assert!(cloud_config.database.redb_path.is_some());
        assert!(memory_config.database.redb_path.is_some());
        
        // Memory preset should use in-memory storage
        assert_eq!(memory_config.database.redb_path, Some(":memory:".to_string()));
        
        println!("Smart presets test passed!");
    }

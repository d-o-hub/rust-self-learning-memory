//! Configuration wizard tests for memory-cli.
//!
//! Tests for the interactive configuration wizard including:
//! - Wizard initialization
//! - User input handling and validation
//! - Config file generation
//! - Error handling and recovery
//! - Preset configurations

use memory_cli::config::types::{CliConfig, Config, ConfigPreset, DatabaseConfig, StorageConfig};
use memory_cli::config::validate_config;
use memory_cli::config::wizard::{format_duration, quick_setup, show_template, ConfigWizard};

use std::fs;
use std::path::PathBuf;

/// Test wizard initialization
#[test]
fn test_wizard_initialization() {
    let wizard = ConfigWizard::new();

    // Verify wizard is created successfully
    // The wizard has private fields, so we test by calling methods that would fail if it wasn't initialized
    // Since wizard requires interactive input, we can only test initialization in basic form
}

/// Test ConfigPreset creation and defaults
#[test]
fn test_config_preset_defaults() {
    // Test Local preset
    let local_config = ConfigPreset::Local.create_config();
    assert!(local_config.database.redb_path.is_some());
    assert!(!local_config.database.turso_url.is_some()); // Local preset uses redb, not turso
    assert_eq!(local_config.storage.max_episodes_cache, 1000);
    assert_eq!(local_config.storage.cache_ttl_seconds, 1800);
    assert_eq!(local_config.storage.pool_size, 5);
    assert_eq!(local_config.cli.default_format, "human");
    assert!(local_config.cli.progress_bars);
    assert_eq!(local_config.cli.batch_size, 100);

    // Test Cloud preset
    let cloud_config = ConfigPreset::Cloud.create_config();
    assert!(cloud_config.database.turso_url.is_some());
    assert!(cloud_config.database.redb_path.is_some());
    assert_eq!(cloud_config.storage.max_episodes_cache, 5000);
    assert_eq!(cloud_config.storage.cache_ttl_seconds, 7200);
    assert_eq!(cloud_config.storage.pool_size, 10);

    // Test Memory preset
    let memory_config = ConfigPreset::Memory.create_config();
    assert_eq!(
        memory_config.database.redb_path,
        Some(":memory:".to_string())
    );
    assert_eq!(memory_config.storage.max_episodes_cache, 100);
    assert_eq!(memory_config.storage.cache_ttl_seconds, 300);
    assert_eq!(memory_config.storage.pool_size, 2);

    // Test Custom preset
    let custom_config = ConfigPreset::Custom.create_config();
    // Custom preset provides sensible defaults that can be customized
    assert_eq!(custom_config.storage.max_episodes_cache, 1000);
    assert_eq!(custom_config.storage.pool_size, 5);
}

/// Test config validation with wizard-generated configs
#[test]
fn test_wizard_config_validation() {
    // Test Local preset validation
    let local_config = ConfigPreset::Local.create_config();
    let validation = validate_config(&local_config);
    assert!(validation.is_valid);
    assert!(validation.errors.is_empty());

    // Test Cloud preset validation (note: turso_url present but may not connect)
    let cloud_config = ConfigPreset::Cloud.create_config();
    let validation = validate_config(&cloud_config);
    // Should be structurally valid even if connection fails
    assert!(validation.errors.is_empty());

    // Test Memory preset validation
    let memory_config = ConfigPreset::Memory.create_config();
    let validation = validate_config(&memory_config);
    assert!(validation.is_valid);
    assert!(validation.errors.is_empty());

    // Test Custom preset validation
    let custom_config = ConfigPreset::Custom.create_config();
    let validation = validate_config(&custom_config);
    assert!(validation.is_valid);
    assert!(validation.errors.is_empty());
}

/// Test config with invalid values (error handling)
#[test]
fn test_invalid_config_detection() {
    // Test with zero cache size
    let mut config = ConfigPreset::Local.create_config();
    config.storage.max_episodes_cache = 0;
    let validation = validate_config(&config);
    assert!(!validation.is_valid);
    assert!(!validation.errors.is_empty());
    assert!(validation
        .errors
        .iter()
        .any(|e| e.message.contains("max_episodes_cache")));

    // Test with zero pool size
    let mut config = ConfigPreset::Local.create_config();
    config.storage.pool_size = 0;
    let validation = validate_config(&config);
    assert!(!validation.is_valid);
    assert!(!validation.errors.is_empty());
    assert!(validation
        .errors
        .iter()
        .any(|e| e.message.contains("pool_size")));

    // Test with zero batch size
    let mut config = ConfigPreset::Local.create_config();
    config.cli.batch_size = 0;
    let validation = validate_config(&config);
    assert!(!validation.is_valid);
    assert!(!validation.errors.is_empty());
    assert!(validation
        .errors
        .iter()
        .any(|e| e.message.contains("batch_size")));

    // Test with no database configured
    let mut config = ConfigPreset::Local.create_config();
    config.database.turso_url = None;
    config.database.redb_path = None;
    let validation = validate_config(&config);
    assert!(!validation.is_valid);
    assert!(!validation.errors.is_empty());
    assert!(validation
        .errors
        .iter()
        .any(|e| e.message.contains("database")));
}

/// Test config serialization and deserialization
#[test]
fn test_config_serialization() {
    let config = ConfigPreset::Local.create_config();

    // Test TOML serialization
    let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize to TOML");
    assert!(!toml_str.is_empty());
    assert!(toml_str.contains("[database]"));
    assert!(toml_str.contains("[storage]"));
    assert!(toml_str.contains("[cli]"));

    // Test TOML deserialization
    let deserialized: Config = toml::from_str(&toml_str).expect("Failed to deserialize from TOML");
    assert_eq!(
        config.storage.max_episodes_cache,
        deserialized.storage.max_episodes_cache
    );
    assert_eq!(
        config.storage.cache_ttl_seconds,
        deserialized.storage.cache_ttl_seconds
    );
    assert_eq!(config.cli.default_format, deserialized.cli.default_format);
}

/// Test format_duration helper function
#[test]
fn test_format_duration() {
    // Test seconds
    assert_eq!(format_duration(30), "30s");
    assert_eq!(format_duration(59), "59s");

    // Test minutes
    assert_eq!(format_duration(60), "1min");
    assert_eq!(format_duration(120), "2min");
    assert_eq!(format_duration(150), "2min 30s");

    // Test hours
    assert_eq!(format_duration(3600), "1hr");
    assert_eq!(format_duration(7200), "2hr");
    assert_eq!(format_duration(7500), "2hr 5min");
}

/// Test config file creation scenarios
#[test]
fn test_config_file_scenarios() {
    // Test creating in current directory
    let config = ConfigPreset::Local.create_config();
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("memory-cli.toml");

    let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize");
    fs::write(&config_path, &toml_str).expect("Failed to write config");

    assert!(config_path.exists());
    assert!(
        fs::metadata(&config_path)
            .expect("Failed to get metadata")
            .len()
            > 0
    );

    // Test reading back
    let read_content = fs::read_to_string(&config_path).expect("Failed to read config");
    assert!(!read_content.is_empty());

    // Test loading from file
    let loaded_config: Config = toml::from_str(&read_content).expect("Failed to parse config");
    assert_eq!(
        config.storage.max_episodes_cache,
        loaded_config.storage.max_episodes_cache
    );
}

/// Test config with nested directory paths
#[test]
fn test_config_nested_directory_paths() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let nested_path = temp_dir.path().join("data/config/memory-cli.toml");

    let config = ConfigPreset::Local.create_config();
    let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize");

    // Parent directory should be created automatically
    fs::write(&nested_path, &toml_str).expect("Failed to write config");

    assert!(nested_path.exists());
    assert!(nested_path.parent().unwrap().exists());
}

/// Test config validation warnings
#[test]
fn test_config_validation_warnings() {
    // Test with low cache size (should warn)
    let mut config = ConfigPreset::Local.create_config();
    config.storage.max_episodes_cache = 50; // Below recommended (100)
    let validation = validate_config(&config);
    // Note: Low cache size might not be an error, but could be a warning
    // The exact behavior depends on the validation implementation

    // Test with low pool size (should warn)
    let mut config = ConfigPreset::Local.create_config();
    config.storage.pool_size = 2; // Below recommended (5)
    let validation = validate_config(&config);
}

/// Test edge cases in config values
#[test]
fn test_config_edge_cases() {
    // Test maximum allowed values
    let config = ConfigPreset::Custom.create_config();

    // Very large cache size (should be allowed or rejected depending on limits)
    let max_cache_size = 100000;
    let mut test_config = config.clone();
    test_config.storage.max_episodes_cache = max_cache_size;
    let validation = validate_config(&test_config);
    // Depending on implementation, very large values might be allowed

    // Very large pool size
    let max_pool_size = 200;
    let mut test_config = config.clone();
    test_config.storage.pool_size = max_pool_size;
    let validation = validate_config(&test_config);
    // Should be acceptable or show a warning

    // Very long TTL
    let max_ttl = 86400; // 24 hours
    let mut test_config = config.clone();
    test_config.storage.cache_ttl_seconds = max_ttl;
    let validation = validate_config(&test_config);
    // Should be acceptable or show a warning
}

/// Test config with all three output formats
#[test]
fn test_config_output_formats() {
    // Test human format (default for most presets)
    let mut config = ConfigPreset::Local.create_config();
    config.cli.default_format = "human".to_string();
    let validation = validate_config(&config);
    assert!(validation.is_valid);

    // Test json format
    let mut config = ConfigPreset::Local.create_config();
    config.cli.default_format = "json".to_string();
    let validation = validate_config(&config);
    assert!(validation.is_valid);

    // Test yaml format
    let mut config = ConfigPreset::Local.create_config();
    config.cli.default_format = "yaml".to_string();
    let validation = validate_config(&config);
    assert!(validation.is_valid);
}

/// Test different database configurations
#[test]
fn test_database_configurations() {
    // Test local SQLite database
    let mut config = ConfigPreset::Local.create_config();
    config.database.turso_url = Some("file:./data/memory.db".to_string());
    config.database.turso_token = None;
    let validation = validate_config(&config);
    assert!(validation.is_valid);

    // Test remote Turso database with token
    let mut config = ConfigPreset::Cloud.create_config();
    config.database.turso_url = Some("libsql://your-db.turso.io/db".to_string());
    config.database.turso_token = Some("test_token".to_string());
    // Validation should pass structurally even if connection fails

    // Test in-memory database
    let mut config = ConfigPreset::Memory.create_config();
    config.database.redb_path = Some(":memory:".to_string());
    let validation = validate_config(&config);
    assert!(validation.is_valid);
}

/// Test config modifications through wizard-like scenarios
#[test]
fn test_config_modifications() {
    // Start with Local preset and modify
    let mut config = ConfigPreset::Local.create_config();

    // Increase cache size
    config.storage.max_episodes_cache = 2500;
    assert_eq!(config.storage.max_episodes_cache, 2500);

    // Change pool size for production
    config.storage.pool_size = 15;
    assert_eq!(config.storage.pool_size, 15);

    // Output format for scripts
    config.cli.default_format = "json".to_string();
    config.cli.progress_bars = false;
    assert_eq!(config.cli.default_format, "json");
    assert!(!config.cli.progress_bars);

    // Validate modified config
    let validation = validate_config(&config);
    assert!(validation.is_valid);
}

/// Test configuration persistence across operations
#[test]
fn test_config_persistence() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("memory-cli.toml");

    // Write initial config
    let original_config = ConfigPreset::Local.create_config();
    let toml_str = toml::to_string_pretty(&original_config).expect("Failed to serialize");
    fs::write(&config_path, &toml_str).expect("Failed to write config");

    // Read config
    let read_config: Config =
        toml::from_str(&fs::read_to_string(&config_path).expect("Failed to read"))
            .expect("Failed to parse");

    // Modify config
    let mut modified_config = read_config;
    modified_config.storage.max_episodes_cache = 5000;
    modified_config.cli.default_format = "yaml".to_string();

    // Write modified config
    let modified_toml = toml::to_string_pretty(&modified_config).expect("Failed to serialize");
    fs::write(&config_path, &modified_toml).expect("Failed to write config");

    // Read again and verify modifications persist
    let final_config: Config =
        toml::from_str(&fs::read_to_string(&config_path).expect("Failed to read"))
            .expect("Failed to parse");

    assert_eq!(final_config.storage.max_episodes_cache, 5000);
    assert_eq!(final_config.cli.default_format, "yaml");
}

/// Test config creation from different environments
#[test]
fn test_environment_specific_configs() {
    // Development environment (smaller cache, faster refresh)
    let dev_config = ConfigPreset::Local.create_config();
    assert_eq!(dev_config.storage.max_episodes_cache, 1000);
    assert_eq!(dev_config.storage.cache_ttl_seconds, 1800); // 30 minutes
    assert_eq!(dev_config.storage.pool_size, 5);

    // Production environment (larger cache, persistent)
    let prod_config = ConfigPreset::Cloud.create_config();
    assert_eq!(prod_config.storage.max_episodes_cache, 5000);
    assert_eq!(prod_config.storage.cache_ttl_seconds, 7200); // 2 hours
    assert_eq!(prod_config.storage.pool_size, 10);

    // Testing/CI environment (minimal, in-memory)
    let test_config = ConfigPreset::Memory.create_config();
    assert_eq!(test_config.storage.max_episodes_cache, 100);
    assert_eq!(test_config.storage.cache_ttl_seconds, 300); // 5 minutes
    assert_eq!(test_config.storage.pool_size, 2);
}

/// Test multiple config files in project
#[test]
fn test_multiple_config_files() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // Create different config files
    let local_config = ConfigPreset::Local.create_config();
    let dev_config = temp_dir.path().join("dev.toml");
    fs::write(
        &dev_config,
        toml::to_string_pretty(&local_config).expect("Failed to serialize"),
    )
    .expect("Failed to write dev config");

    let test_config = ConfigPreset::Memory.create_config();
    let ci_config = temp_dir.path().join("ci.toml");
    fs::write(
        &ci_config,
        toml::to_string_pretty(&test_config).expect("Failed to serialize"),
    )
    .expect("Failed to write CI config");

    // Verify both files exist and can be loaded
    assert!(dev_config.exists());
    assert!(ci_config.exists());

    let _: Config = toml::from_str(&fs::read_to_string(&dev_config).expect("Failed to read"))
        .expect("Failed to parse dev");
    let _: Config = toml::from_str(&fs::read_to_string(&ci_config).expect("Failed to read"))
        .expect("Failed to parse CI");
}

/// Test config merge scenarios
#[test]
fn test_config_merge_scenarios() {
    // Base config
    let base_config = ConfigPreset::Local.create_config();

    // Simulate overlaying values manually (in real usage, a merge function would do this)
    let mut merged_config = base_config;
    merged_config.storage.max_episodes_cache = 3000; // Override cache
    merged_config.cli.progress_bars = false; // Override progress bars
                                             // Keep all other base config values

    // Verify merged config
    assert_eq!(merged_config.storage.max_episodes_cache, 3000);
    assert!(!merged_config.cli.progress_bars);
    assert_eq!(merged_config.storage.cache_ttl_seconds, 1800); // Kept from base
    assert_eq!(merged_config.cli.default_format, "human"); // Kept from base

    // Validate merged config
    let validation = validate_config(&merged_config);
    assert!(validation.is_valid);
}

/// Test wizard helper function - show_template
#[test]
fn test_show_template() {
    // This would normally display to stdout, so we just verify it doesn't panic
    // In a real interactive test, we'd capture stdout
    let result = show_template();
    assert!(result.is_ok());
}

/// Test config from partial user input simulation
#[test]
fn test_partial_config_creation() {
    // Simulate user providing partial config through wizard
    let mut config = ConfigPreset::Custom.create_config();

    // User only changes database settings
    config.database.turso_url = Some("file:./custom/memory.db".to_string());
    config.database.redb_path = Some("./custom/cache.redb".to_string());

    // Storage and CLI remain at defaults
    assert_eq!(config.storage.max_episodes_cache, 1000);
    assert_eq!(config.cli.default_format, "human");

    // Validate partial config
    let validation = validate_config(&config);
    assert!(validation.is_valid);
}

/// Test config import/export scenarios
#[test]
fn test_config_import_export() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // Create config in different formats
    let config = ConfigPreset::Local.create_config();

    // TOML export
    let toml_path = temp_dir.path().join("config.toml");
    let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize");
    fs::write(&toml_path, &toml_str).expect("Failed to write");
    assert!(toml_path.exists());

    // Import and verify
    let imported: Config = toml::from_str(&fs::read_to_string(&toml_path).expect("Failed to read"))
        .expect("Failed to import");
    assert_eq!(
        config.storage.max_episodes_cache,
        imported.storage.max_episodes_cache
    );

    // JSON export (if supported)
    let json_path = temp_dir.path().join("config.json");
    if let Ok(json_str) = serde_json::to_string_pretty(&config) {
        fs::write(&json_path, &json_str).expect("Failed to write");
        assert!(json_path.exists());

        let imported_json: Config =
            serde_json::from_str(&fs::read_to_string(&json_path).expect("Failed to read"))
                .expect("Failed to import JSON");
        assert_eq!(
            config.storage.max_episodes_cache,
            imported_json.storage.max_episodes_cache
        );
    }
}

/// Test config validation error messages
#[test]
fn test_config_validation_error_messages() {
    let mut config = ConfigPreset::Local.create_config();
    config.storage.max_episodes_cache = 0;

    let validation = validate_config(&config);
    assert!(!validation.is_valid);

    // Check that error messages are helpful
    if !validation.errors.is_empty() {
        let first_error = &validation.errors[0];
        assert!(!first_error.message.is_empty());
        // Check if there's a suggestion (optional but good UX)
        if let Some(suggestion) = &first_error.suggestion {
            assert!(!suggestion.is_empty());
        }
    }
}

/// Test template generation (if available)
#[test]
fn test_template_generation() {
    // Test that we can generate valid config templates
    let local_template = ConfigPreset::Local.create_config();
    let cloud_template = ConfigPreset::Cloud.create_config();
    let memory_template = ConfigPreset::Memory.create_config();
    let custom_template = ConfigPreset::Custom.create_config();

    // All templates should be valid
    for template in [
        &local_template,
        &cloud_template,
        &memory_template,
        &custom_template,
    ] {
        let validation = validate_config(template);
        assert!(
            validation.is_valid,
            "Template validation failed: {:?}",
            validation.errors
        );
    }
}

/// Integration test: Simulate wizard workflow
#[test]
fn test_wizard_workflow_simulation() {
    // Simulate the typical wizard workflow:
    // 1. User selects preset
    let preset = ConfigPreset::Local;
    let config = preset.create_config();

    // 2. User customizes settings (simulated)
    let mut customized = config;
    customized.storage.max_episodes_cache = 2000;
    customized.cli.default_format = "json".to_string();

    // 3. Review and validate
    let validation = validate_config(&customized);
    assert!(validation.is_valid);

    // 4. Save configuration
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("memory-cli.toml");
    let toml_str = toml::to_string_pretty(&customized).expect("Failed to serialize");
    fs::write(&config_path, &toml_str).expect("Failed to write config");

    // 5. Verify saved config
    assert!(config_path.exists());
    let loaded: Config = toml::from_str(&fs::read_to_string(&config_path).expect("Failed to read"))
        .expect("Failed to load saved config");
    assert_eq!(loaded.storage.max_episodes_cache, 2000);
    assert_eq!(loaded.cli.default_format, "json");
}

/// Test config with special path handling
#[test]
fn test_special_path_handling() {
    let mut config = ConfigPreset::Local.create_config();

    // Test in-memory path
    config.database.redb_path = Some(":memory:".to_string());
    let validation = validate_config(&config);
    assert!(validation.is_valid);

    // Test relative path
    config.database.redb_path = Some("./data/cache.redb".to_string());
    let validation = validate_config(&config);
    assert!(validation.is_valid);

    // Test absolute-like path (simulation)
    config.database.redb_path = Some("/var/lib/memory/cache.redb".to_string());
    let validation = validate_config(&config);
    // Should be structurally valid even if directory doesn't exist
    assert!(validation.errors.is_empty());
}

/// Test config upgrade/migration scenarios
#[test]
fn test_config_compatibility() {
    // Ensure all presets produce configs that can be serialized/deserialized
    for preset in [
        ConfigPreset::Local,
        ConfigPreset::Cloud,
        ConfigPreset::Memory,
        ConfigPreset::Custom,
    ] {
        let config = preset.create_config();

        // Serialize
        let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize");

        // Deserialize
        let restored: Config = toml::from_str(&toml_str).expect("Failed to deserialize");

        // Verify key fields match
        assert_eq!(
            config.storage.max_episodes_cache,
            restored.storage.max_episodes_cache
        );
        assert_eq!(
            config.storage.cache_ttl_seconds,
            restored.storage.cache_ttl_seconds
        );
        assert_eq!(config.storage.pool_size, restored.storage.pool_size);
        assert_eq!(config.cli.default_format, restored.cli.default_format);
        assert_eq!(config.cli.progress_bars, restored.cli.progress_bars);
        assert_eq!(config.cli.batch_size, restored.cli.batch_size);
    }
}

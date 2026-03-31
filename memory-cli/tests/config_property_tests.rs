//! Property-based tests for configuration validation invariants
//!
//! These tests use proptest to verify that configuration validation rules
//! are consistent and produce correct results across a wide range of inputs.

use do_memory_cli::config::types::ConfigPreset;
use do_memory_cli::config::types::{
    CliConfig, Config, DatabaseConfig, EmbeddingsConfig, StorageConfig,
};
use do_memory_cli::config::validator::{
    validate_cli_config, validate_config, validate_database_config, validate_storage_config,
};
use proptest::prelude::*;

// ============================================================================
// Helper: build a Config from parts
// ============================================================================

fn make_config(db: DatabaseConfig, storage: StorageConfig, cli: CliConfig) -> Config {
    Config {
        database: db,
        storage,
        cli,
        embeddings: EmbeddingsConfig::default(),
    }
}

// ============================================================================
// Database Validation Properties
// ============================================================================

proptest! {
    /// No storage configured always produces a validation error
    #[test]
    fn no_storage_always_invalid(
        turso_token in proptest::option::of("[a-zA-Z0-9]{10,30}"),
    ) {
        let db = DatabaseConfig {
            turso_url: None,
            turso_token,
            redb_path: None,
        };

        let result = validate_database_config(&db);
        prop_assert!(!result.is_valid, "Config with no storage should be invalid");
    }

    /// Valid redb path always produces a valid database config
    #[test]
    fn valid_redb_path_is_valid(
        path in "((/tmp|/home)/[a-z]{3,10}/[a-z]{3,10}\\.redb)|:memory:",
    ) {
        let db = DatabaseConfig {
            turso_url: None,
            turso_token: None,
            redb_path: Some(path),
        };

        let result = validate_database_config(&db);
        prop_assert!(result.is_valid, "Config with valid redb path should be valid");
    }

    /// Valid libsql URL always produces a valid database config
    #[test]
    fn valid_libsql_url_is_valid(
        host in "[a-z]{3,10}",
        db_name in "[a-z]{3,10}",
    ) {
        let url = format!("libsql://{host}.turso.io/{db_name}");
        let db = DatabaseConfig {
            turso_url: Some(url),
            turso_token: Some("test-token".to_string()),
            redb_path: None,
        };

        let result = validate_database_config(&db);
        prop_assert!(result.is_valid, "Config with valid libsql URL should be valid");
    }

    /// Empty turso_url string produces validation error
    #[test]
    fn empty_turso_url_is_invalid(
        whitespace in "\\s{0,5}",
    ) {
        let db = DatabaseConfig {
            turso_url: Some(whitespace),
            turso_token: None,
            redb_path: None,
        };

        let result = validate_database_config(&db);
        prop_assert!(!result.is_valid, "Config with empty turso_url should be invalid");
    }

    /// Path traversal in file: URL always produces validation error
    #[test]
    fn path_traversal_always_rejected(
        suffix in "[a-z]{3,10}",
    ) {
        let url = format!("file:/tmp/../etc/{suffix}");
        let db = DatabaseConfig {
            turso_url: Some(url),
            turso_token: None,
            redb_path: None,
        };

        let result = validate_database_config(&db);
        prop_assert!(!result.is_valid, "Path traversal should be rejected");
    }

    /// Sensitive system paths in file: URL always produce validation error
    #[test]
    fn sensitive_paths_always_rejected(
        sensitive_prefix in prop::sample::select(vec![
            "/etc/", "/bin/", "/sbin/", "/proc/", "/dev/", "/boot/", "/root/",
        ]),
        suffix in "[a-z]{3,10}",
    ) {
        let url = format!("file:{sensitive_prefix}{suffix}");
        let db = DatabaseConfig {
            turso_url: Some(url),
            turso_token: None,
            redb_path: None,
        };

        let result = validate_database_config(&db);
        prop_assert!(!result.is_valid,
            "Sensitive path {sensitive_prefix} should be rejected");
    }
}

// ============================================================================
// Storage Validation Properties
// ============================================================================

proptest! {
    /// Zero values for any storage field produce validation errors
    #[test]
    fn zero_storage_fields_invalid(
        zero_field in 0u8..3u8,
    ) {
        let storage = StorageConfig {
            max_episodes_cache: if zero_field == 0 { 0 } else { 1000 },
            cache_ttl_seconds: if zero_field == 1 { 0 } else { 3600 },
            pool_size: if zero_field == 2 { 0 } else { 5 },
        };

        let result = validate_storage_config(&storage);
        prop_assert!(!result.is_valid,
            "Storage config with zero field {zero_field} should be invalid");
    }

    /// All positive storage values produce valid config
    #[test]
    fn positive_storage_fields_valid(
        max_cache in 1usize..10000usize,
        ttl in 1u64..86400u64,
        pool in 1usize..100usize,
    ) {
        let storage = StorageConfig {
            max_episodes_cache: max_cache,
            cache_ttl_seconds: ttl,
            pool_size: pool,
        };

        let result = validate_storage_config(&storage);
        prop_assert!(result.is_valid,
            "Storage config with all positive values should be valid");
    }

    /// Large cache sizes produce warnings but remain valid
    #[test]
    fn large_cache_produces_warning(
        cache_size in 100_001usize..500_000usize,
    ) {
        let storage = StorageConfig {
            max_episodes_cache: cache_size,
            cache_ttl_seconds: 3600,
            pool_size: 5,
        };

        let result = validate_storage_config(&storage);
        prop_assert!(result.is_valid, "Large cache should still be valid");
        prop_assert!(!result.warnings.is_empty(), "Large cache should produce warning");
    }
}

// ============================================================================
// CLI Validation Properties
// ============================================================================

proptest! {
    /// Valid output formats always pass validation
    #[test]
    fn valid_formats_accepted(
        format in prop::sample::select(vec!["human", "json", "yaml"]),
        batch_size in 1usize..10000usize,
    ) {
        let cli = CliConfig {
            default_format: format.to_string(),
            progress_bars: false,
            batch_size,
        };

        let result = validate_cli_config(&cli);
        prop_assert!(result.is_valid, "Valid format should be accepted");
    }

    /// Invalid output formats always fail validation
    #[test]
    fn invalid_formats_rejected(
        format in "[a-z]{1,10}".prop_filter("not a valid format",
            |s| s != "human" && s != "json" && s != "yaml"),
    ) {
        let cli = CliConfig {
            default_format: format,
            progress_bars: false,
            batch_size: 100,
        };

        let result = validate_cli_config(&cli);
        prop_assert!(!result.is_valid, "Invalid format should be rejected");
    }

    /// Zero batch size always fails validation
    #[test]
    fn zero_batch_size_rejected(
        format in prop::sample::select(vec!["human", "json", "yaml"]),
    ) {
        let cli = CliConfig {
            default_format: format.to_string(),
            progress_bars: false,
            batch_size: 0,
        };

        let result = validate_cli_config(&cli);
        prop_assert!(!result.is_valid, "Zero batch size should be rejected");
    }
}

// ============================================================================
// Cross-configuration Properties
// ============================================================================

proptest! {
    /// Cache size smaller than batch size produces a warning
    #[test]
    fn small_cache_vs_batch_produces_warning(
        batch_size in 100usize..1000usize,
    ) {
        let cache_size = batch_size / 2; // Smaller than batch
        let config = make_config(
            DatabaseConfig {
                turso_url: None,
                turso_token: None,
                redb_path: Some("/tmp/test.redb".to_string()),
            },
            StorageConfig {
                max_episodes_cache: cache_size,
                cache_ttl_seconds: 3600,
                pool_size: 5,
            },
            CliConfig {
                default_format: "json".to_string(),
                progress_bars: false,
                batch_size,
            },
        );

        let result = validate_config(&config);
        let has_cache_warning = result.warnings.iter().any(|w|
            w.field.contains("max_episodes_cache") && w.message.contains("smaller than batch size")
        );
        prop_assert!(has_cache_warning,
            "Cache ({cache_size}) < batch ({batch_size}) should produce warning");
    }

    /// All preset configs pass validation
    #[test]
    fn preset_configs_always_valid(
        preset_idx in 0u8..3u8,
    ) {
        let preset = match preset_idx {
            0 => ConfigPreset::Local,
            1 => ConfigPreset::Memory,
            _ => ConfigPreset::Custom,
        };

        let config = preset.create_config();
        let result = validate_config(&config);
        prop_assert!(result.is_valid,
            "Preset config should be valid, but got errors: {:?}",
            result.errors.iter().map(|e| &e.message).collect::<Vec<_>>());
    }
}

// ============================================================================
// Serialization Roundtrip Properties
// ============================================================================

proptest! {
    /// Config JSON roundtrip preserves all fields
    #[test]
    fn config_json_roundtrip(
        cache_size in 1usize..10000usize,
        ttl in 1u64..86400u64,
        pool in 1usize..100usize,
        format in prop::sample::select(vec!["human", "json", "yaml"]),
        batch in 1usize..1000usize,
        progress in proptest::bool::ANY,
    ) {
        let config = make_config(
            DatabaseConfig {
                turso_url: None,
                turso_token: None,
                redb_path: Some("/tmp/test.redb".to_string()),
            },
            StorageConfig {
                max_episodes_cache: cache_size,
                cache_ttl_seconds: ttl,
                pool_size: pool,
            },
            CliConfig {
                default_format: format.to_string(),
                progress_bars: progress,
                batch_size: batch,
            },
        );

        let json = serde_json::to_string(&config).expect("serialize to JSON");
        let deserialized: Config = serde_json::from_str(&json).expect("deserialize from JSON");

        prop_assert_eq!(config.storage.max_episodes_cache, deserialized.storage.max_episodes_cache);
        prop_assert_eq!(config.storage.cache_ttl_seconds, deserialized.storage.cache_ttl_seconds);
        prop_assert_eq!(config.storage.pool_size, deserialized.storage.pool_size);
        prop_assert_eq!(config.cli.default_format, deserialized.cli.default_format);
        prop_assert_eq!(config.cli.progress_bars, deserialized.cli.progress_bars);
        prop_assert_eq!(config.cli.batch_size, deserialized.cli.batch_size);
    }

    /// Config TOML roundtrip preserves all fields
    #[test]
    fn config_toml_roundtrip(
        cache_size in 1usize..10000usize,
        pool in 1usize..100usize,
        batch in 1usize..1000usize,
    ) {
        let config = make_config(
            DatabaseConfig {
                turso_url: None,
                turso_token: None,
                redb_path: Some("/tmp/test.redb".to_string()),
            },
            StorageConfig {
                max_episodes_cache: cache_size,
                cache_ttl_seconds: 3600,
                pool_size: pool,
            },
            CliConfig {
                default_format: "json".to_string(),
                progress_bars: false,
                batch_size: batch,
            },
        );

        let toml_str = toml::to_string(&config).expect("serialize to TOML");
        let deserialized: Config = toml::from_str(&toml_str).expect("deserialize from TOML");

        prop_assert_eq!(config.storage.max_episodes_cache, deserialized.storage.max_episodes_cache);
        prop_assert_eq!(config.storage.pool_size, deserialized.storage.pool_size);
        prop_assert_eq!(config.cli.batch_size, deserialized.cli.batch_size);
    }
}

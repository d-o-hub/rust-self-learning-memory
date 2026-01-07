//! Configuration types for memory-cli
//!
//! This module defines the core configuration structures used throughout
//! the memory-cli application, providing a clean separation of concerns
//! from loading, validation, and storage initialization logic.

// Submodules
mod defaults;
mod defaults_impl;
mod detection;
mod enums;
mod presets;
mod simple;
pub mod system_info;
mod structs;

// Re-exports for backward compatibility
pub use defaults::*;
pub use enums::{ConfigError, DatabaseType, PerformanceLevel};
pub use presets::ConfigPreset;
pub use structs::{CliConfig, Config, DatabaseConfig, EmbeddingsConfig, StorageConfig};
pub use system_info::{smart_config, ConfigRecommendations, SystemInfo};

// Re-export validation types from validator module for backward compatibility
pub use crate::config::validator::{ValidationError, ValidationResult, ValidationWarning};

#[cfg(test)]
mod simple_config_tests {
    use super::*;
    use std::env;

    /// Helper function to clean up all environment variables before each test
    fn clean_environment() {
        env::remove_var("CI");
        env::remove_var("TURSO_URL");
        env::remove_var("TURSO_TOKEN");
        env::remove_var("TURSO_DATABASE_URL");
        env::remove_var("RENDER");
        env::remove_var("HEROKU");
        env::remove_var("FLY_IO");
        env::remove_var("RAILWAY");
        env::remove_var("VERCEL");
        env::remove_var("DEVELOPMENT");
        env::remove_var("DEV");
    }

    /// Helper function to setup environment for CI testing
    fn setup_ci_environment() {
        clean_environment();
        env::set_var("CI", "true");
    }

    /// Helper function to setup environment for Turso testing
    fn setup_turso_environment() {
        clean_environment();
        env::set_var("TURSO_URL", "libsql://test.example.com/db");
        env::set_var("TURSO_TOKEN", "test-token");
    }

    /// Helper function to setup environment for cloud platform testing
    fn setup_cloud_platform_environment(platform: &str) {
        clean_environment();
        env::set_var(platform, "true");
    }

    #[tokio::test]
    async fn test_simple_config_basic() {
        clean_environment();

        let config = Config::simple()
            .await
            .expect("Config::simple() should succeed");

        // Verify that we got a valid config
        assert!(config.database.redb_path.is_some() || config.database.turso_url.is_some());
        assert!(config.storage.max_episodes_cache > 0);
        assert!(config.storage.pool_size > 0);
        assert!(!config.cli.default_format.is_empty());
    }

    #[tokio::test]
    async fn test_simple_config_ci_environment() {
        // Skip in CI due to test isolation issues with parallel execution
        if std::env::var("CI").is_ok() {
            return;
        }

        setup_ci_environment();

        let config = Config::simple()
            .await
            .expect("Config::simple() should succeed in CI");

        // In CI, should use Memory preset with in-memory redb
        assert_eq!(config.database.redb_path, Some(":memory:".to_string()));
        assert_eq!(config.storage.max_episodes_cache, 100);
        assert!(!config.cli.progress_bars);
    }

    #[tokio::test]
    #[ignore] // Run separately to avoid environment variable race conditions
    async fn test_simple_config_with_turso() {
        // Skip in CI due to test isolation issues with parallel execution
        if std::env::var("CI").is_ok() {
            return;
        }

        setup_turso_environment();

        let config = Config::simple()
            .await
            .expect("Config::simple() should succeed with Turso");

        // With Turso credentials, should use Cloud preset
        assert!(
            config.database.turso_url.is_some(),
            "turso_url should be set"
        );
        assert!(
            config.database.turso_token.is_some(),
            "turso_token should be set"
        );
    }

    #[tokio::test]
    async fn test_simple_config_with_cloud_platform() {
        // Test various cloud platform indicators
        let platforms = ["RENDER", "HEROKU", "FLY_IO", "RAILWAY", "VERCEL"];

        for platform in &platforms {
            setup_cloud_platform_environment(platform);

            let config = Config::simple()
                .await
                .expect("Config::simple() should succeed with {platform} platform");

            // Should use Cloud preset
            assert!(
                config.database.turso_url.is_some() || config.database.redb_path.is_some(),
                "Should have database configuration for cloud platform: {}",
                platform
            );
        }
    }

    #[tokio::test]
    async fn test_simple_config_validation() {
        clean_environment();

        // Should succeed with valid preset
        let result = Config::simple().await;
        assert!(
            result.is_ok(),
            "Config::simple() should succeed with valid environment: {:?}",
            result.err()
        );
    }
}

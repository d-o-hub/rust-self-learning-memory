//! Compatibility tests for the memory-cli crate.
//!
//! These tests verify that the CLI works correctly across different
//! configurations, platforms, and feature combinations.

use memory_cli::test_utils::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[cfg(test)]
mod compatibility_tests {
    use super::*;
    use assert_cmd::Command;

    #[test]
    fn test_feature_flag_compatibility() {
        // Test CLI behavior with different feature combinations
        // Since we can't easily change features at runtime, we test
        // that the CLI handles missing features gracefully

        let harness = CliHarness::new();

        // These commands should fail gracefully when features are disabled
        let feature_dependent_commands = vec![
            vec!["episode", "create", "--task", "test"],
            vec!["episode", "list"],
            vec!["pattern", "list"],
            vec!["storage", "stats"],
        ];

        for cmd in feature_dependent_commands {
            // Commands should fail with clear error messages, not crash
            let result = harness.execute(&cmd).assert().failure();

            // Should contain some indication of why it failed (features disabled)
            // The exact message depends on implementation
            let output = result.get_output();
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Should not be empty error and should be informative
            assert!(
                !stderr.trim().is_empty(),
                "Command {:?} produced empty error",
                cmd
            );
        }
    }

    #[test]
    fn test_cross_platform_path_handling() {
        let temp_dir = TempDir::new().unwrap();
        let harness = CliHarness::new();

        // Test various path formats that should work across platforms
        let test_paths = vec![
            "config.toml",
            "./config.toml",
            "subdir/config.toml",
            "path/with/forward/slashes.toml",
        ];

        for path_str in test_paths {
            let config_path = temp_dir.path().join(path_str);

            // Create parent directories if needed
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }

            let config_content = r#"
[database]
turso_url = "file:test.db"
redb_path = "test.redb"

[storage]
max_episodes_cache = 100
cache_ttl_seconds = 3600
pool_size = 5

[cli]
default_format = "json"
progress_bars = false
batch_size = 10
"#;

            fs::write(&config_path, config_content).unwrap();

            // Test that CLI can load config from this path
            let mut cmd = Command::new(std::env::var("CARGO_BIN_EXE_memory-cli").unwrap_or_else(
                |_| "/workspaces/rust-self-learning-memory/target/debug/memory-cli".to_string(),
            ));
            cmd.arg("--config").arg(&config_path);
            cmd.arg("config");

            // Should succeed in loading and displaying config
            cmd.assert().success();
        }
    }

    #[test]
    fn test_configuration_format_support() {
        let temp_dir = TempDir::new().unwrap();

        // Test different configuration formats
        let config_formats = vec![
            (
                "toml",
                r#"
[database]
turso_url = "file:test.db"
redb_path = "test.redb"

[storage]
max_episodes_cache = 100
cache_ttl_seconds = 3600
pool_size = 5

[cli]
default_format = "json"
progress_bars = false
batch_size = 10
"#,
            ),
            (
                "json",
                r#"{
  "database": {
    "turso_url": "file:test.db",
    "redb_path": "test.redb"
  },
  "storage": {
    "max_episodes_cache": 100,
    "cache_ttl_seconds": 3600,
    "pool_size": 5
  },
  "cli": {
    "default_format": "json",
    "progress_bars": false,
    "batch_size": 10
  }
}"#,
            ),
            (
                "yaml",
                r#"---
database:
  turso_url: "file:test.db"
  redb_path: "test.redb"
storage:
  max_episodes_cache: 100
  cache_ttl_seconds: 3600
  pool_size: 5
cli:
  default_format: "json"
  progress_bars: false
  batch_size: 10
"#,
            ),
        ];

        for (format_name, config_content) in config_formats {
            let config_path = temp_dir.path().join(format!("config.{}", format_name));
            fs::write(&config_path, config_content).unwrap();

            let mut cmd = Command::new(std::env::var("CARGO_BIN_EXE_memory-cli").unwrap_or_else(
                |_| "/workspaces/rust-self-learning-memory/target/debug/memory-cli".to_string(),
            ));
            cmd.arg("--config").arg(&config_path);
            cmd.arg("config");

            // Should successfully load and display config
            cmd.assert().success();

            println!("Successfully loaded {} configuration", format_name);
        }
    }

    #[test]
    fn test_environment_variable_integration() {
        // Test that environment variables are properly handled
        // (Note: The current CLI doesn't use env vars extensively, but this tests the framework)

        let harness = CliHarness::new();

        // Test with various environment configurations
        let env_scenarios = vec![
            ("empty env", vec![]),
            ("basic env", vec![("RUST_LOG", "info")]),
        ];

        for (scenario_name, env_vars) in env_scenarios {
            // Set environment variables for this test
            let mut cmd = Command::new(std::env::var("CARGO_BIN_EXE_memory-cli").unwrap_or_else(
                |_| "/workspaces/rust-self-learning-memory/target/debug/memory-cli".to_string(),
            ));
            cmd.arg("--config").arg(harness.config_path());
            cmd.arg("config");

            // Apply environment variables
            for (key, value) in env_vars {
                cmd.env(key, value);
            }

            // Should work regardless of environment
            cmd.assert().success();

            println!("Environment scenario '{}' handled correctly", scenario_name);
        }
    }

    #[test]
    fn test_backward_compatibility_config() {
        let temp_dir = TempDir::new().unwrap();

        // Test configs with missing optional fields (backward compatibility)
        let backward_compat_configs = vec![
            (
                "minimal config",
                r#"
[database]
turso_url = "file:test.db"

[storage]
max_episodes_cache = 100

[cli]
default_format = "human"
"#,
            ),
            (
                "missing optional fields",
                r#"
[database]
redb_path = "test.redb"

[storage]
max_episodes_cache = 50
cache_ttl_seconds = 1800

[cli]
default_format = "json"
"#,
            ),
        ];

        for (config_name, config_content) in backward_compat_configs {
            let config_path = temp_dir
                .path()
                .join(format!("{}.toml", config_name.replace(" ", "_")));
            fs::write(&config_path, config_content).unwrap();

            let mut cmd = Command::new(std::env::var("CARGO_BIN_EXE_memory-cli").unwrap_or_else(
                |_| "/workspaces/rust-self-learning-memory/target/debug/memory-cli".to_string(),
            ));
            cmd.arg("--config").arg(&config_path);
            cmd.arg("config");

            // Should load successfully with defaults for missing fields
            cmd.assert().success();

            println!(
                "Backward compatibility config '{}' loaded successfully",
                config_name
            );
        }
    }

    #[test]
    fn test_large_configuration_files() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("large_config.toml");

        // Create a large configuration file with many entries
        let mut config_content = String::from(
            r#"
[database]
turso_url = "file:test.db"
redb_path = "test.redb"

[storage]
max_episodes_cache = 1000
cache_ttl_seconds = 3600
pool_size = 10

[cli]
default_format = "json"
progress_bars = true
batch_size = 100
"#,
        );

        // Add many metadata-like entries
        for i in 0..1000 {
            config_content.push_str(&format!("metadata_{} = \"value_{}\"\n", i, i));
        }

        fs::write(&config_path, config_content).unwrap();

        let mut cmd = Command::new(std::env::var("CARGO_BIN_EXE_memory-cli").unwrap_or_else(
            |_| "/workspaces/rust-self-learning-memory/target/debug/memory-cli".to_string(),
        ));
        cmd.arg("--config").arg(&config_path);
        cmd.arg("config");

        // Should handle large config files without issues
        cmd.assert().success();

        println!(
            "Large configuration file ({} bytes) loaded successfully",
            fs::metadata(&config_path).unwrap().len()
        );
    }

    #[test]
    fn test_unicode_and_special_characters() {
        let harness = CliHarness::new();

        // Test with various Unicode content that should be handled properly
        let unicode_scenarios = vec![
            ("basic unicode", "test_task_🚀"),
            ("emoji only", "🔥💯✨"),
            ("mixed scripts", "Hello_世界_مرحبا"),
            ("special chars", "test-task_with.dots"),
            ("spaces", "task with spaces"),
        ];

        for (scenario_name, task_name) in unicode_scenarios {
            // Note: This will fail due to missing storage features, but tests parsing
            let result = harness.execute(&["episode", "create", "--task", task_name]);

            // Should either succeed or fail gracefully (not crash)
            let output = result.output().unwrap();

            // Should not crash - exit code can be failure due to features
            assert!(
                output.status.code().is_some(),
                "Unicode scenario '{}' caused crash",
                scenario_name
            );

            println!(
                "Unicode scenario '{}' handled correctly (exit code: {:?})",
                scenario_name,
                output.status.code()
            );
        }
    }

    #[test]
    fn test_configuration_file_encoding() {
        let temp_dir = TempDir::new().unwrap();

        // Test that UTF-8 encoded config files work
        let utf8_config = r#"
# Configuration with Unicode comments: 🚀
[database]
turso_url = "file:test.db"
# Comment with emoji: 🔥

[storage]
max_episodes_cache = 100
cache_ttl_seconds = 3600
pool_size = 5

[cli]
default_format = "json"
progress_bars = false
batch_size = 10
"#;

        let config_path = temp_dir.path().join("utf8_config.toml");
        fs::write(&config_path, utf8_config).unwrap();

        let mut cmd = Command::new(std::env::var("CARGO_BIN_EXE_memory-cli").unwrap_or_else(
            |_| "/workspaces/rust-self-learning-memory/target/debug/memory-cli".to_string(),
        ));
        cmd.arg("--config").arg(&config_path);
        cmd.arg("config");

        // Should handle UTF-8 encoded files correctly
        cmd.assert().success();

        println!("UTF-8 encoded configuration file loaded successfully");
    }

    #[test]
    fn test_relative_and_absolute_paths() {
        let temp_dir = TempDir::new().unwrap();
        let harness = CliHarness::new();

        // Test both relative and absolute config paths
        let config_path_relative = "test_config.toml";
        let config_path_absolute = temp_dir.path().join("test_config.toml");

        let config_content = r#"
[database]
turso_url = "file:test.db"
redb_path = "test.redb"

[storage]
max_episodes_cache = 100
cache_ttl_seconds = 3600
pool_size = 5

[cli]
default_format = "json"
progress_bars = false
batch_size = 10
"#;

        fs::write(&config_path_absolute, config_content).unwrap();

        // Change to temp directory for relative path test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // Test relative path
        let mut cmd_relative = Command::new(
            std::env::var("CARGO_BIN_EXE_memory-cli").unwrap_or_else(|_| {
                "/workspaces/rust-self-learning-memory/target/debug/memory-cli".to_string()
            }),
        );
        cmd_relative.arg("--config").arg(config_path_relative);
        cmd_relative.arg("config");
        cmd_relative.assert().success();

        // Test absolute path
        let mut cmd_absolute = Command::new(
            std::env::var("CARGO_BIN_EXE_memory-cli").unwrap_or_else(|_| {
                "/workspaces/rust-self-learning-memory/target/debug/memory-cli".to_string()
            }),
        );
        cmd_absolute.arg("--config").arg(&config_path_absolute);
        cmd_absolute.arg("config");
        cmd_absolute.assert().success();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        println!("Both relative and absolute configuration paths work correctly");
    }
}

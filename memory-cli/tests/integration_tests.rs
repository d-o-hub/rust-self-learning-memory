//! Integration tests for the memory-cli crate.
//!
//! These tests verify end-to-end functionality of CLI commands
//! using test harnesses and mock storage backends.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

mod common;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use memory_cli::test_utils;
    use std::time::Duration;
    use test_utils::*;

    #[test]
    fn test_cli_help_command() {
        let harness = CliHarness::new();

        harness
            .execute(["--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("memory-cli"))
            .stdout(predicate::str::contains("episode"))
            .stdout(predicate::str::contains("pattern"));
    }

    #[test]
    fn test_cli_version_command() {
        let harness = CliHarness::new();

        harness
            .execute(["--version"])
            .assert()
            .success()
            .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn test_cli_config_validation() {
        let harness = CliHarness::new();

        harness
            .execute(["config", "validate"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Configuration is valid"));
    }

    #[test]
    fn test_cli_invalid_config() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_config_path = temp_dir.path().join("invalid.toml");

        // Use unique database paths within temp directory
        let turso_db_path = temp_dir.path().join("test.db");
        let redb_path = temp_dir.path().join("test.redb");

        // Convert Windows paths to forward slashes for TOML compatibility
        let turso_db_str = turso_db_path.display().to_string().replace('\\', "/");
        let redb_str = redb_path.display().to_string().replace('\\', "/");

        let invalid_config = format!(
            r#"
[database]
turso_url = "file:{}"
redb_path = "{}"

[storage]
max_episodes_cache = 0
cache_ttl_seconds = 3600
pool_size = 5

[cli]
default_format = "human"
progress_bars = true
batch_size = 10
"#,
            turso_db_str, redb_str
        );

        fs::write(&invalid_config_path, &invalid_config).unwrap();

        #[allow(deprecated)]
        let mut cmd = Command::cargo_bin("memory-cli").expect("Failed to find memory-cli binary");
        cmd.arg("--config").arg(&invalid_config_path);
        cmd.args(["config", "validate"]); // Validate config to trigger validation

        cmd.assert()
            .success()
            .stdout(predicate::str::contains("max_episodes_cache"));
    }

    #[test]
    fn test_cli_output_formats() {
        let harness = CliHarness::new();

        // Test JSON output
        harness
            .execute(["--format", "json", "config", "show"])
            .assert()
            .success();

        // Test YAML output
        harness
            .execute(["--format", "yaml", "config", "show"])
            .assert()
            .success();

        // Test human output (default)
        harness
            .execute(["--format", "human", "config", "validate"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Configuration is valid"));
    }

    #[test]
    fn test_cli_verbose_output() {
        let harness = CliHarness::new();

        harness
            .execute(["--verbose", "config", "show"])
            .assert()
            .success();
    }

    #[test]
    fn test_cli_dry_run_mode() {
        let harness = CliHarness::new();

        // Test dry run with episode create
        harness
            .execute(["--dry-run", "episode", "create", "--task", "test task"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Would create episode"));
    }

    #[test]
    fn test_cli_completion_generation() {
        let harness = CliHarness::new();

        // Test bash completion
        harness
            .execute(["completion", "bash"])
            .assert()
            .success()
            .stdout(predicate::str::contains("memory-cli"));

        // Test zsh completion
        harness.execute(["completion", "zsh"]).assert().success();

        // Test fish completion
        harness.execute(["completion", "fish"]).assert().success();
    }

    #[test]
    fn test_cli_unknown_command() {
        let harness = CliHarness::new();

        harness
            .execute(["unknown-command"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("unrecognized"));
    }

    #[test]
    fn test_cli_invalid_format() {
        let harness = CliHarness::new();

        harness
            .execute(["--format", "invalid", "config", "show"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("invalid"));
    }

    #[test]
    fn test_cli_missing_required_args() {
        let harness = CliHarness::new();

        // Test episode view without ID
        harness.execute(["episode", "view"]).assert().failure();

        // Test episode complete without ID
        harness.execute(["episode", "complete"]).assert().failure();
    }

    #[test]
    fn test_cli_config_file_loading() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("custom.toml");

        // Use unique database paths within temp directory
        let turso_db_path = temp_dir.path().join("custom.db");
        let redb_path = temp_dir.path().join("custom.redb");

        // Convert Windows paths to forward slashes for TOML compatibility
        let turso_db_str = turso_db_path.display().to_string().replace('\\', "/");
        let redb_str = redb_path.display().to_string().replace('\\', "/");

        let custom_config = format!(
            r#"
[database]
turso_url = "file:{}"
turso_token = "custom-token"
redb_path = "{}"

[storage]
max_episodes_cache = 200
cache_ttl_seconds = 7200
pool_size = 8

[cli]
default_format = "json"
progress_bars = true
batch_size = 50
"#,
            turso_db_str, redb_str
        );

        fs::write(&config_path, &custom_config).unwrap();

        #[allow(deprecated)]
        let mut cmd = Command::cargo_bin("memory-cli").expect("Failed to find memory-cli binary");
        cmd.arg("--config").arg(&config_path);
        cmd.args(["config", "show"]);

        cmd.assert().success();
    }

    #[test]
    fn test_cli_config_default_locations() {
        let temp_dir = TempDir::new().unwrap();
        let default_config_path = temp_dir.path().join("memory-cli.toml");

        // Use unique database paths within temp directory
        let turso_db_path = temp_dir.path().join("default.db");
        let redb_path = temp_dir.path().join("default.redb");

        // Convert Windows paths to forward slashes for TOML compatibility
        let turso_db_str = turso_db_path.display().to_string().replace('\\', "/");
        let redb_str = redb_path.display().to_string().replace('\\', "/");

        let config_content = format!(
            r#"
[database]
turso_url = "file:{}"
redb_path = "{}"

[storage]
max_episodes_cache = 150
cache_ttl_seconds = 3600
pool_size = 6

[cli]
default_format = "yaml"
progress_bars = false
batch_size = 25
"#,
            turso_db_str, redb_str
        );

        fs::write(&default_config_path, &config_content).unwrap();

        // Change to temp directory to test default config loading
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        #[allow(deprecated)]
        let mut cmd = Command::cargo_bin("memory-cli").expect("Failed to find memory-cli binary");
        cmd.args(["config", "show"]);

        cmd.assert().success();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_cli_performance_baseline() {
        let harness = CliHarness::new();

        // Measure help command performance (should be very fast)
        let start = std::time::Instant::now();
        harness.execute(["--help"]).assert().success();
        let duration = start.elapsed();

        // Help should execute in under 500ms (increased for Windows compatibility)
        assert!(duration < Duration::from_millis(500));
    }

    #[test]
    fn test_cli_error_handling() {
        let harness = CliHarness::new();

        // Test invalid episode ID format
        harness
            .execute(["episode", "view", "invalid-uuid"])
            .assert()
            .failure()
            .stderr(
                predicate::str::contains("Invalid episode ID format")
                    .or(predicate::str::contains("Turso storage feature not enabled")),
            );

        // Test invalid task type
        harness
            .execute(["episode", "list", "--task-type", "invalid"])
            .assert()
            .failure();
    }

    #[test]
    fn test_cli_input_validation() {
        let harness = CliHarness::new();

        // Test with potentially dangerous input (should be handled safely)
        let dangerous_inputs = [
            "test; rm -rf /",
            "test && echo 'hacked'",
            "test | cat /etc/passwd",
            "../../../etc/passwd",
        ];

        for input in &dangerous_inputs {
            // These should fail safely, not execute dangerous commands
            harness
                .execute(["episode", "create", input])
                .assert()
                .failure(); // Should fail due to missing turso feature or validation
        }
    }

    #[test]
    fn test_cli_output_consistency() {
        let harness = CliHarness::new();

        // Test that the same command produces consistent output
        let output1 = harness.execute_and_capture(["--help"]).unwrap();
        let output2 = harness.execute_and_capture(["--help"]).unwrap();

        assert_eq!(output1, output2);
    }

    #[test]
    fn test_cli_command_structure() {
        let harness = CliHarness::new();

        // Test all main command groups are available
        let help_output = harness.execute_and_capture(["--help"]).unwrap();

        assert!(help_output.contains("episode"));
        assert!(help_output.contains("pattern"));
        assert!(help_output.contains("storage"));
        assert!(help_output.contains("completion"));
        assert!(help_output.contains("config"));
    }

    #[test]
    fn test_cli_subcommand_help() {
        let harness = CliHarness::new();

        // Test episode subcommand help
        harness
            .execute(["episode", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("create"))
            .stdout(predicate::str::contains("list"))
            .stdout(predicate::str::contains("view"));

        // Test pattern subcommand help
        harness
            .execute(["pattern", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("list"))
            .stdout(predicate::str::contains("view"));

        // Test storage subcommand help
        harness
            .execute(["storage", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("stats"))
            .stdout(predicate::str::contains("sync"));
    }
}

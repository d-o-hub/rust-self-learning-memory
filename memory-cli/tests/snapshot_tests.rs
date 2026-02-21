//! Snapshot tests for CLI output
//!
//! These tests verify that CLI output (help, version, etc.) remains
//! consistent across changes. Part of ADR-033 Phase 6.

use insta::assert_snapshot;
use memory_cli::test_utils::CliHarness;

/// Test CLI help output snapshot
#[test]
fn test_cli_help_output() {
    let harness = CliHarness::new();

    let output = harness
        .execute(["--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot!(stdout);
}

/// Test CLI version output snapshot
#[test]
fn test_cli_version_output() {
    let harness = CliHarness::new();

    let output = harness
        .execute(["--version"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot!(stdout);
}

/// Test episode subcommand help snapshot
#[test]
fn test_cli_episode_help() {
    let harness = CliHarness::new();

    let output = harness
        .execute(["episode", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot!(stdout);
}

/// Test pattern subcommand help snapshot
#[test]
fn test_cli_pattern_help() {
    let harness = CliHarness::new();

    let output = harness
        .execute(["pattern", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot!(stdout);
}

/// Test storage subcommand help snapshot
#[test]
fn test_cli_storage_help() {
    let harness = CliHarness::new();

    let output = harness
        .execute(["storage", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot!(stdout);
}

/// Test health subcommand help snapshot
#[test]
fn test_cli_health_help() {
    let harness = CliHarness::new();

    let output = harness
        .execute(["health", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot!(stdout);
}

/// Test storage stats command output format
#[test]
fn test_cli_storage_stats_output() {
    let harness = CliHarness::new();

    let output = harness
        .execute(["--dry-run", "storage", "stats"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // In dry-run mode, this should output what would be done
    assert!(
        !stdout.is_empty() || !String::from_utf8_lossy(&output.stderr).is_empty(),
        "Command should produce some output"
    );
}

/// Test config validation output
#[test]
fn test_cli_config_validate_output() {
    let harness = CliHarness::new();

    let output = harness
        .execute(["config", "validate"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Config validation should succeed with our test config
    // Verify key content is present (avoid snapshot due to timestamps)
    assert!(
        stdout.contains("✅ Configuration is valid"),
        "Should indicate valid config"
    );
    assert!(
        stdout.contains("Connectivity Status:"),
        "Should show connectivity status"
    );
}

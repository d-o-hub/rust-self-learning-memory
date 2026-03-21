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

/// Test playbook subcommand help snapshot
#[test]
fn test_cli_playbook_help() {
    let harness = CliHarness::new();
    let output = harness
        .execute(["playbook", "--help"])
        .output()
        .expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot!(stdout);
}

/// Test feedback subcommand help snapshot
#[test]
fn test_cli_feedback_help() {
    let harness = CliHarness::new();
    let output = harness
        .execute(["feedback", "--help"])
        .output()
        .expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot!(stdout);
}

/// Test CLI version output format (not snapshot-based to avoid breakage on version bumps)
#[test]
fn test_cli_version_output() {
    let harness = CliHarness::new();

    let output = harness
        .execute(["--version"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    assert!(
        trimmed.starts_with("memory-cli "),
        "Version output should start with 'memory-cli ', got: {trimmed}"
    );
    let version_part = trimmed.strip_prefix("memory-cli ").unwrap();
    assert!(
        version_part.chars().all(|c| c.is_ascii_digit() || c == '.'),
        "Version should be numeric with dots, got: {version_part}"
    );
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

/// Test backup subcommand help snapshot
#[test]
fn test_cli_backup_help() {
    let harness = CliHarness::new();
    let output = harness
        .execute(["backup", "--help"])
        .output()
        .expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot!(stdout);
}

/// Test monitor subcommand help snapshot
#[test]
fn test_cli_monitor_help() {
    let harness = CliHarness::new();
    let output = harness
        .execute(["monitor", "--help"])
        .output()
        .expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot!(stdout);
}

/// Test logs subcommand help snapshot
#[test]
fn test_cli_logs_help() {
    let harness = CliHarness::new();
    let output = harness
        .execute(["logs", "--help"])
        .output()
        .expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot!(stdout);
}

/// Test eval subcommand help snapshot
#[test]
fn test_cli_eval_help() {
    let harness = CliHarness::new();
    let output = harness
        .execute(["eval", "--help"])
        .output()
        .expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot!(stdout);
}

/// Test embedding subcommand help snapshot
#[test]
fn test_cli_embedding_help() {
    let harness = CliHarness::new();
    let output = harness
        .execute(["embedding", "--help"])
        .output()
        .expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot!(stdout);
}

/// Test tag subcommand help snapshot
#[test]
fn test_cli_tag_help() {
    let harness = CliHarness::new();
    let output = harness
        .execute(["tag", "--help"])
        .output()
        .expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot!(stdout);
}

/// Test relationship subcommand help snapshot
#[test]
fn test_cli_relationship_help() {
    let harness = CliHarness::new();
    let output = harness
        .execute(["relationship", "--help"])
        .output()
        .expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot!(stdout);
}

/// Test config subcommand help snapshot
#[test]
fn test_cli_config_help() {
    let harness = CliHarness::new();
    let output = harness
        .execute(["config", "--help"])
        .output()
        .expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot!(stdout);
}

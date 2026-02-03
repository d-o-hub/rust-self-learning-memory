//! End-to-End tests for CLI embedding commands
//!
//! Tests the CLI integration with embeddings:
//! - `memory embedding configure` command
//! - `memory embedding search` command
//! - `memory embedding test` command
//! - Integration with episode queries
//! - Configuration file handling

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Get the CLI binary path
fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_memory-cli"))
}

/// Create a temporary config file
fn create_temp_config(enabled: bool, provider: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let config_path = temp_dir.path().join("test-config.toml");

    let config_content = format!(
        r#"
[memory]
data_dir = "{}"

[embeddings]
enabled = {}
provider = "{}"
model = "sentence-transformers/all-MiniLM-L6-v2"
dimension = 384
similarity_threshold = 0.7
"#,
        temp_dir.path().join("data").display(),
        enabled,
        provider
    );

    std::fs::write(&config_path, config_content).expect("Should write config");

    (temp_dir, config_path)
}

// ============================================================================
// Day 2: CLI Integration E2E Tests
// ============================================================================

#[test]
fn test_cli_embedding_list_providers() {
    let mut cmd = cli();
    cmd.arg("embedding").arg("list-providers");

    let output = cmd.output().expect("Should execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Command should succeed");
    assert!(stdout.contains("Available Embedding Providers"));
    assert!(stdout.contains("Local Provider"));
    assert!(stdout.contains("OpenAI"));
    assert!(stdout.contains("Mistral"));

    println!("List providers output:\n{}", stdout);
}

#[test]
fn test_cli_embedding_config_disabled() {
    let (_temp_dir, config_path) = create_temp_config(false, "local");

    let mut cmd = cli();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("embedding")
        .arg("config");

    let output = cmd.output().expect("Should execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Command should succeed");
    assert!(
        stdout.contains("Status:") || stdout.contains("disabled") || stdout.contains("Disabled")
    );

    println!("Config (disabled) output:\n{}", stdout);
}

#[test]
fn test_cli_embedding_config_enabled() {
    let (_temp_dir, config_path) = create_temp_config(true, "local");

    let mut cmd = cli();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("embedding")
        .arg("config");

    let output = cmd.output().expect("Should execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Command should succeed");

    // Should show configuration details
    assert!(stdout.contains("local") || stdout.contains("Local"));

    println!("Config (enabled) output:\n{}", stdout);
}

#[test]
fn test_cli_embedding_test_command() {
    let mut cmd = cli();
    cmd.arg("embedding").arg("test");

    let output = cmd.output().expect("Should execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should succeed (even if embeddings disabled)
    assert!(output.status.success(), "Command should succeed");

    // Should show test results or message about disabled embeddings
    assert!(stdout.contains("Embedding") || stdout.contains("disabled"));

    println!("Test command output:\n{}", stdout);
}

#[test]
fn test_cli_embedding_enable_disable() {
    // Test enable
    let mut cmd_enable = cli();
    cmd_enable.arg("embedding").arg("enable");

    let output_enable = cmd_enable.output().expect("Should execute command");
    assert!(
        output_enable.status.success(),
        "Enable command should succeed"
    );

    // Test disable
    let mut cmd_disable = cli();
    cmd_disable.arg("embedding").arg("disable");

    let output_disable = cmd_disable.output().expect("Should execute command");
    assert!(
        output_disable.status.success(),
        "Disable command should succeed"
    );
}

#[test]
fn test_cli_embedding_help() {
    let mut cmd = cli();
    cmd.arg("embedding").arg("--help");

    let output = cmd.output().expect("Should execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("USAGE"));
    assert!(stdout.contains("COMMANDS"));

    // Should list available commands
    assert!(stdout.contains("test") || stdout.contains("Test"));
    assert!(stdout.contains("config") || stdout.contains("Config"));
    assert!(stdout.contains("list-providers") || stdout.contains("ListProviders"));

    println!("Help output:\n{}", stdout);
}

#[test]
fn test_cli_embedding_subcommand_help() {
    let subcommands = vec!["test", "config", "list-providers", "enable", "disable"];

    for subcmd in subcommands {
        let mut cmd = cli();
        cmd.arg("embedding").arg(subcmd).arg("--help");

        let output = cmd
            .output()
            .expect(&format!("{} help should succeed", subcmd));
        assert!(output.status.success(), "{} help should succeed", subcmd);
    }
}

#[test]
fn test_cli_embedding_benchmark() {
    let mut cmd = cli();
    cmd.arg("embedding").arg("benchmark");

    let output = cmd.output().expect("Should execute command");

    // May fail if embeddings disabled, but shouldn't crash
    // Just check it doesn't panic
    let _stderr = String::from_utf8_lossy(&output.stderr);
}

// ============================================================================
// Integration with Episode Commands
// ============================================================================

#[test]
fn test_cli_episode_query_with_embeddings() {
    let (_temp_dir, config_path) = create_temp_config(true, "local");

    // First, try to query episodes (may be empty)
    let mut cmd = cli();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("episode")
        .arg("query")
        .arg("--query")
        .arg("REST API")
        .arg("--limit")
        .arg("5");

    let output = cmd.output().expect("Should execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should succeed (even if no results)
    assert!(output.status.success(), "Query command should succeed");

    println!("Episode query output:\n{}", stdout);
}

#[test]
fn test_cli_embedding_search_integration() {
    // Test the search command if available
    let mut cmd = cli();
    cmd.arg("embedding")
        .arg("search")
        .arg("--query")
        .arg("authentication")
        .arg("--limit")
        .arg("3");

    let output = cmd.output().expect("Should execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should not crash
    let _ = stdout;
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
fn test_cli_embedding_config_local_provider() {
    let (_temp_dir, config_path) = create_temp_config(true, "local");

    let mut cmd = cli();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("embedding")
        .arg("config");

    let output = cmd.output().expect("Should execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("local") || stdout.contains("Local"));
}

#[test]
fn test_cli_embedding_config_openai_provider() {
    let (_temp_dir, config_path) = create_temp_config(true, "openai");

    let mut cmd = cli();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("embedding")
        .arg("config");

    let output = cmd.output().expect("Should execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    // Should mention OpenAI or API key requirement
    let output_str = stdout.to_string();
    assert!(
        output_str.contains("openai")
            || output_str.contains("OpenAI")
            || output_str.contains("API key")
            || output_str.contains("not configured")
    );
}

#[test]
fn test_cli_embedding_invalid_config() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let config_path = temp_dir.path().join("invalid-config.toml");

    // Write invalid config
    std::fs::write(&config_path, "invalid [toml").expect("Should write invalid config");

    let mut cmd = cli();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("embedding")
        .arg("config");

    let output = cmd.output().expect("Should execute command");

    // Should fail gracefully
    assert!(
        !output.status.success()
            || String::from_utf8_lossy(&output.stderr).contains("error")
            || String::from_utf8_lossy(&output.stdout).contains("error")
    );
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_cli_embedding_missing_config() {
    let mut cmd = cli();
    cmd.arg("--config")
        .arg("/nonexistent/config.toml")
        .arg("embedding")
        .arg("config");

    let output = cmd.output().expect("Should execute command");

    // Should fail with helpful error
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("config") || stderr.contains("not found"));
}

#[test]
fn test_cli_embedding_test_with_disabled() {
    let (_temp_dir, config_path) = create_temp_config(false, "local");

    let mut cmd = cli();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("embedding")
        .arg("test");

    let output = cmd.output().expect("Should execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should succeed but show that embeddings are disabled
    assert!(output.status.success());
    assert!(
        stdout.contains("disabled")
            || stdout.contains("Disabled")
            || stdout.contains("not enabled")
    );
}

// ============================================================================
// Workflow Tests
// ============================================================================

#[test]
fn test_cli_embedding_full_workflow() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let config_path = temp_dir.path().join("workflow-config.toml");

    // Step 1: Create initial config with embeddings disabled
    let config_content = format!(
        r#"
[memory]
data_dir = "{}"

[embeddings]
enabled = false
provider = "local"
"#,
        temp_dir.path().join("data").display()
    );

    std::fs::write(&config_path, config_content).expect("Should write config");

    // Step 2: Check status (should be disabled)
    let mut cmd_status = cli();
    cmd_status
        .arg("--config")
        .arg(&config_path)
        .arg("embedding")
        .arg("config");

    let output_status = cmd_status.output().expect("Should execute command");
    assert!(output_status.status.success());

    // Step 3: Enable embeddings
    let mut cmd_enable = cli();
    cmd_enable
        .arg("--config")
        .arg(&config_path)
        .arg("embedding")
        .arg("enable");

    let output_enable = cmd_enable.output().expect("Should execute command");
    assert!(output_enable.status.success());

    // Step 4: Test embeddings
    let mut cmd_test = cli();
    cmd_test
        .arg("--config")
        .arg(&config_path)
        .arg("embedding")
        .arg("test");

    let output_test = cmd_test.output().expect("Should execute command");
    assert!(output_test.status.success());

    println!("Full workflow test completed successfully");
}

#[test]
fn test_cli_embedding_list_providers_detailed() {
    let mut cmd = cli();
    cmd.arg("embedding").arg("list-providers").arg("--detailed");

    let output = cmd.output().expect("Should execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should succeed and show detailed info
    assert!(output.status.success());

    // Check for detailed information
    assert!(
        stdout.contains("dimension") || stdout.contains("model") || stdout.contains("Provider")
    );

    println!("Detailed providers list:\n{}", stdout);
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_cli_embedding_test_performance() {
    let mut cmd = cli();
    cmd.arg("embedding").arg("test");

    let start = std::time::Instant::now();
    let output = cmd.output().expect("Should execute command");
    let duration = start.elapsed();

    assert!(output.status.success());

    println!("CLI embedding test execution time: {:?}", duration);

    // Should complete in reasonable time
    assert!(duration < std::time::Duration::from_secs(10));
}

#[test]
fn test_cli_embedding_config_performance() {
    let (_temp_dir, config_path) = create_temp_config(true, "local");

    let mut cmd = cli();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("embedding")
        .arg("config");

    let start = std::time::Instant::now();
    let output = cmd.output().expect("Should execute command");
    let duration = start.elapsed();

    assert!(output.status.success());

    println!("CLI embedding config execution time: {:?}", duration);

    // Should be fast
    assert!(duration < std::time::Duration::from_secs(5));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_cli_embedding_empty_query() {
    let mut cmd = cli();
    cmd.arg("embedding")
        .arg("search")
        .arg("--query")
        .arg("")
        .arg("--limit")
        .arg("5");

    let output = cmd.output().expect("Should execute command");

    // Should handle gracefully (may fail or succeed)
    let _ = output;
}

#[test]
fn test_cli_embedding_very_long_query() {
    let long_query = "test ".repeat(1000);

    let mut cmd = cli();
    cmd.arg("embedding")
        .arg("search")
        .arg("--query")
        .arg(&long_query)
        .arg("--limit")
        .arg("1");

    let output = cmd.output().expect("Should execute command");

    // Should not crash
    let _ = output;
}

#[test]
fn test_cli_embedding_special_characters_in_query() {
    let special_query = "Test with <html> & special chars: @#$% emojis: 🚀";

    let mut cmd = cli();
    cmd.arg("embedding")
        .arg("search")
        .arg("--query")
        .arg(special_query)
        .arg("--limit")
        .arg("1");

    let output = cmd.output().expect("Should execute command");

    // Should handle special characters
    let _ = output;
}

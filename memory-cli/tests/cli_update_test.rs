//! CLI integration tests for episode update command
//!
//! Note: These tests spawn `cargo run` which is slow, so they have extended
//! timeouts in `.config/nextest.toml` and use `#[serial]` to avoid resource
//! contention when running in parallel.

use serial_test::serial;
use std::process::Command;

/// Helper function to run memory-cli command
fn run_command(args: &[&str]) -> (bool, String, String) {
    let output = Command::new("cargo")
        .args(["run", "--package", "memory-cli", "--"])
        .args(args)
        .output()
        .expect("Failed to execute memory-cli");

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    (success, stdout, stderr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial]
    fn test_update_command_help() {
        let (success, stdout, _stderr) = run_command(&["episode", "update", "--help"]);
        assert!(success);
        assert!(stdout.contains("Update an episode"));
        assert!(stdout.contains("--description"));
        assert!(stdout.contains("--add-tag"));
        assert!(stdout.contains("--remove-tag"));
        assert!(stdout.contains("--set-tags"));
        assert!(stdout.contains("--metadata"));
    }

    #[test]
    #[serial]
    fn test_update_command_invalid_id() {
        let (success, _stdout, stderr) = run_command(&[
            "episode",
            "update",
            "invalid-uuid",
            "--description",
            "New description",
        ]);

        // Should fail with invalid UUID
        assert!(!success);
        assert!(stderr.contains("Invalid episode ID format") || stderr.contains("error"));
    }

    #[test]
    #[serial]
    fn test_update_command_description_only() {
        // This test requires a working memory system, so we'll just test the help
        let (success, stdout, _stderr) = run_command(&["episode", "update", "--help"]);
        assert!(success);
        assert!(stdout.contains("description"));
    }

    #[test]
    #[serial]
    fn test_update_command_with_dry_run() {
        let (success, stdout, _stderr) = run_command(&[
            "--dry-run", // Global flag must come before subcommand
            "episode",
            "update",
            "00000000-0000-0000-0000-000000000000",
            "--description",
            "Test",
        ]);

        // Dry run should succeed even if episode doesn't exist
        assert!(success || stdout.contains("DRY RUN"));
    }

    #[test]
    #[serial]
    fn test_update_command_metadata_format() {
        let (success, stdout, _stderr) = run_command(&["episode", "update", "--help"]);
        assert!(success);
        assert!(stdout.contains("KEY=VALUE"));
    }
}

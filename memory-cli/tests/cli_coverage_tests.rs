//! CLI coverage tests for episode, pattern, tag, config, and health commands.
//!
//! Covers command parsing, help output, and output format validation
//! without requiring real storage backends (ACT-033, v0.1.21 sprint).

use assert_cmd::Command;
use predicates::prelude::*;

mod common;

#[cfg(test)]
mod cli_coverage_tests {
    use super::*;
    use memory_cli::test_utils;
    use test_utils::*;

    // ── Episode subcommand help ──────────────────────────────────────

    #[test]
    fn test_episode_help_shows_subcommands() {
        let harness = CliHarness::new();

        harness
            .execute(["episode", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("create"))
            .stdout(predicate::str::contains("list"))
            .stdout(predicate::str::contains("view"))
            .stdout(predicate::str::contains("complete"));
    }

    #[test]
    fn test_episode_short_help_flag() {
        let harness = CliHarness::new();

        harness
            .execute(["episode", "-h"])
            .assert()
            .success()
            .stdout(predicate::str::contains("create"))
            .stdout(predicate::str::contains("list"));
    }

    // ── Pattern subcommand help ──────────────────────────────────────

    #[test]
    fn test_pattern_help_shows_subcommands() {
        let harness = CliHarness::new();

        harness
            .execute(["pattern", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("list"))
            .stdout(predicate::str::contains("view"));
    }

    #[test]
    fn test_pattern_short_help_flag() {
        let harness = CliHarness::new();

        harness
            .execute(["pattern", "-h"])
            .assert()
            .success()
            .stdout(predicate::str::contains("list"));
    }

    // ── Tag subcommand help ──────────────────────────────────────────

    #[test]
    fn test_tag_help_shows_subcommands() {
        let harness = CliHarness::new();

        harness
            .execute(["tag", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("add"))
            .stdout(predicate::str::contains("remove"))
            .stdout(predicate::str::contains("list"))
            .stdout(predicate::str::contains("search"))
            .stdout(predicate::str::contains("stats"));
    }

    #[test]
    fn test_tag_short_help_flag() {
        let harness = CliHarness::new();

        harness
            .execute(["tag", "-h"])
            .assert()
            .success()
            .stdout(predicate::str::contains("add"))
            .stdout(predicate::str::contains("list"));
    }

    // ── Config subcommand help ───────────────────────────────────────

    #[test]
    fn test_config_help_shows_subcommands() {
        let harness = CliHarness::new();

        harness
            .execute(["config", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("show"))
            .stdout(predicate::str::contains("validate"));
    }

    #[test]
    fn test_config_short_help_flag() {
        let harness = CliHarness::new();

        harness
            .execute(["config", "-h"])
            .assert()
            .success()
            .stdout(predicate::str::contains("show"));
    }

    // ── Health subcommand help ───────────────────────────────────────

    #[test]
    fn test_health_help_shows_subcommands() {
        let harness = CliHarness::new();

        harness
            .execute(["health", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("check"))
            .stdout(predicate::str::contains("status"))
            .stdout(predicate::str::contains("monitor"));
    }

    #[test]
    fn test_health_short_help_flag() {
        let harness = CliHarness::new();

        harness
            .execute(["health", "-h"])
            .assert()
            .success()
            .stdout(predicate::str::contains("check"));
    }

    // ── Episode list with no data ────────────────────────────────────

    #[test]
    fn test_episode_list_succeeds_with_no_data() {
        let harness = CliHarness::new();

        // Should succeed or fail gracefully (no panic/crash)
        let output = harness.execute(["episode", "list"]).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Either succeeds with empty output or fails with a storage-related message
        assert!(
            output.status.success()
                || stderr.contains("storage")
                || stderr.contains("Turso")
                || stderr.contains("not enabled")
                || stderr.contains("No episodes"),
            "Unexpected output - stdout: {stdout}, stderr: {stderr}"
        );
    }

    // ── Pattern list with no data ────────────────────────────────────

    #[test]
    fn test_pattern_list_succeeds_with_no_data() {
        let harness = CliHarness::new();

        let output = harness.execute(["pattern", "list"]).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            output.status.success()
                || stderr.contains("storage")
                || stderr.contains("Turso")
                || stderr.contains("not enabled")
                || stderr.contains("No patterns"),
            "Unexpected output - stdout: {stdout}, stderr: {stderr}"
        );
    }

    // ── Invalid subcommand ───────────────────────────────────────────

    #[test]
    fn test_invalid_subcommand_fails() {
        let harness = CliHarness::new();

        harness
            .execute(["nonexistent-command"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("unrecognized"));
    }

    #[test]
    fn test_invalid_episode_subcommand_fails() {
        let harness = CliHarness::new();

        harness
            .execute(["episode", "nonexistent"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("unrecognized"));
    }

    // ── Version format ───────────────────────────────────────────────

    #[test]
    fn test_version_output_contains_package_version() {
        let harness = CliHarness::new();

        harness
            .execute(["--version"])
            .assert()
            .success()
            .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn test_version_output_contains_binary_name() {
        let harness = CliHarness::new();

        harness
            .execute(["--version"])
            .assert()
            .success()
            .stdout(predicate::str::contains("memory-cli"));
    }

    // ── Multiple help flags ──────────────────────────────────────────

    #[test]
    fn test_long_help_and_short_help_both_work() {
        let harness = CliHarness::new();

        let long_output = harness.execute_and_capture(["--help"]).unwrap();
        let short_output = harness.execute_and_capture(["-h"]).unwrap();

        // Both should contain the same key sections
        assert!(long_output.contains("episode"));
        assert!(short_output.contains("episode"));
        assert!(long_output.contains("pattern"));
        assert!(short_output.contains("pattern"));
    }

    #[test]
    fn test_help_output_lists_all_top_level_commands() {
        let harness = CliHarness::new();

        let help = harness.execute_and_capture(["--help"]).unwrap();

        assert!(help.contains("episode"));
        assert!(help.contains("pattern"));
        assert!(help.contains("config"));
        assert!(help.contains("health"));
        assert!(help.contains("tag"));
    }
}

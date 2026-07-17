//! CLI command tests for pattern, monitor, and other commands

#![allow(missing_docs)]

use do_memory_cli::test_utils::CliHarness;
use predicates::prelude::*;

#[test]
fn test_pattern_list_command() {
    let harness = CliHarness::new();

    // Should run without error, even if no patterns exist
    harness.execute(["pattern", "list"]).assert().success();

    // Test with limits
    harness
        .execute(["pattern", "list", "--limit", "5"])
        .assert()
        .success();
}

/// ADR-076: empty human-format `pattern list` prints diagnostic hints.
#[test]
fn test_pattern_list_empty_human_diagnostics() {
    let harness = CliHarness::new();

    harness
        .execute(["--format", "human", "pattern", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No durable patterns found"))
        .stdout(predicate::str::contains("episode complete"))
        .stdout(predicate::str::contains("storage sync"));
}

/// ADR-076: empty human-format `pattern search` prints diagnostic hints.
#[test]
fn test_pattern_search_empty_human_diagnostics() {
    let harness = CliHarness::new();

    harness
        .execute([
            "--format",
            "human",
            "pattern",
            "search",
            "something-unique-xyz",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("No patterns found matching query"))
        .stdout(predicate::str::contains("No durable patterns found"))
        .stdout(predicate::str::contains("episode complete"))
        .stdout(predicate::str::contains("storage sync"));
}

#[test]
fn test_pattern_view_command() {
    let harness = CliHarness::new();

    // Viewing a non-existent pattern should fail (but gracefully)
    harness
        .execute(["pattern", "view", "00000000-0000-0000-0000-000000000000"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Pattern not found"));

    // Invalid UUID should fail
    harness
        .execute(["pattern", "view", "invalid-uuid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid pattern ID format"));
}

#[test]
fn test_pattern_effectiveness_command() {
    let harness = CliHarness::new();

    harness
        .execute(["pattern", "effectiveness"])
        .assert()
        .success();
}

#[test]
fn test_logs_command() {
    let harness = CliHarness::new();

    // Logs analyze
    harness.execute(["logs", "analyze"]).assert().success();

    // Logs search
    harness
        .execute(["logs", "search", "error"])
        .assert()
        .success();

    // Logs stats
    harness.execute(["logs", "stats"]).assert().success();
}

#[test]
fn test_backup_command() {
    let harness = CliHarness::new();
    let temp_dir = harness.temp_dir();
    let backup_path = temp_dir.join("backups");
    std::fs::create_dir_all(&backup_path).unwrap();

    // List backups (empty)
    harness
        .execute(["backup", "list", "--path", backup_path.to_str().unwrap()])
        .assert()
        .success();

    // Create backup
    harness
        .execute([
            "--dry-run",
            "backup",
            "create",
            "--path",
            backup_path.to_str().unwrap(),
        ])
        .assert()
        .success();
}

#[test]
fn test_storage_command() {
    let harness = CliHarness::new();

    harness.execute(["storage", "stats"]).assert().success();
    harness.execute(["storage", "health"]).assert().success();
    harness
        .execute(["storage", "connections"])
        .assert()
        .success();
}

/// ADR-076: `storage sync` is Turso↔redb reconciliation only.
///
/// When dual backends are not configured, the command must fail with messaging
/// that distinguishes sync from pattern extraction (issue #845 residual UX).
#[tokio::test]
async fn test_storage_sync_missing_dual_backends_adr076() {
    // Arrange: memory with no Turso and no redb backends attached
    let memory = do_memory_core::SelfLearningMemory::new();
    let config = do_memory_cli::config::Config::default();

    // Act
    let result = do_memory_cli::commands::sync_storage(
        &memory,
        &config,
        do_memory_cli::output::OutputFormat::Human,
        false,
        false,
    )
    .await;

    // Assert: fail closed with ADR-076 guidance (not a silent no-op)
    assert!(result.is_err(), "sync without dual backends must fail");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("pattern extraction"),
        "must clarify sync is not pattern extraction; got: {msg}"
    );
    assert!(
        msg.contains("Turso") || msg.contains("redb") || msg.contains("reconcile"),
        "must mention dual backends / reconcile; got: {msg}"
    );
}

/// ADR-076: CLI binary path — unusable local cache yields the same dual-backend error.
#[test]
fn test_storage_sync_cli_missing_dual_backends_adr076() {
    let harness = CliHarness::new();
    let temp = harness.temp_dir();

    // Parent path is a regular file so redb cannot open → init falls back to no backends
    let blocker = temp.join("not-a-directory");
    std::fs::write(&blocker, b"block").expect("write blocker file");
    let bad_redb = blocker.join("cache.redb");
    let bad_redb_str = bad_redb.display().to_string().replace('\\', "/");

    let config_content = format!(
        r#"
[database]
redb_path = "{bad_redb_str}"

[storage]
max_episodes_cache = 100
cache_ttl_seconds = 3600
pool_size = 5

[cli]
default_format = "json"
progress_bars = false
batch_size = 10
"#
    );
    std::fs::write(harness.config_path(), config_content).expect("write config");

    harness
        .execute(["storage", "sync"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("pattern extraction").and(
                predicate::str::contains("Turso")
                    .or(predicate::str::contains("redb"))
                    .or(predicate::str::contains("reconcile")),
            ),
        );
}

#[test]
fn test_health_command() {
    let harness = CliHarness::new();

    harness.execute(["health", "check"]).assert().success();
    harness.execute(["health", "status"]).assert().success();
}

#[test]
fn test_monitor_command() {
    let harness = CliHarness::new();

    harness.execute(["monitor", "status"]).assert().success();
    harness.execute(["monitor", "metrics"]).assert().success();
}

/// ADR-075: `episode fail` is exposed as an operator subcommand.
#[test]
fn test_episode_fail_help_lists_subcommand() {
    let harness = CliHarness::new();

    harness
        .execute(["episode", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("fail"))
        .stdout(predicate::str::contains("Force-fail"));
}

/// ADR-075: dry-run for fail only prints the planned action (no success banner).
#[test]
fn test_episode_fail_dry_run() {
    let harness = CliHarness::new();
    let id = "123e4567-e89b-12d3-a456-426614174000";

    harness
        .execute(["--dry-run", "episode", "fail", id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Would complete episode"))
        .stdout(predicate::str::contains("Failure"))
        .stdout(predicate::str::contains(id));
}

/// ADR-075: invalid UUID fails without printing "Status: completed".
#[test]
fn test_episode_fail_invalid_uuid_no_success_banner() {
    let harness = CliHarness::new();

    let output = harness
        .execute(["episode", "fail", "not-a-uuid"])
        .output()
        .expect("run episode fail");

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stdout.contains("Status: completed"),
        "must not print success banner on failure path; stdout={stdout}"
    );
    assert!(
        stderr.contains("Invalid episode ID") || stdout.contains("Invalid episode ID"),
        "expected invalid ID error; stderr={stderr} stdout={stdout}"
    );
}

/// ADR-075: complete unknown id fails without printing "Status: completed".
#[test]
fn test_episode_complete_unknown_id_no_success_banner() {
    let harness = CliHarness::new();
    let id = "00000000-0000-0000-0000-000000000000";

    let output = harness
        .execute(["episode", "complete", id, "failure"])
        .output()
        .expect("run episode complete");

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("Status: completed"),
        "must not print success banner on failure path; stdout={stdout}"
    );
    assert!(
        !stdout.contains("Episode Completed"),
        "must not print completed banner on failure path; stdout={stdout}"
    );
}

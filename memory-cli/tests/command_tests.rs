use memory_cli::test_utils::CliHarness;
use predicates::prelude::*;

#[test]
fn test_pattern_list_command() {
    let harness = CliHarness::new();

    // Should run without error, even if no patterns exist
    harness
        .execute(["pattern", "list"])
        .assert()
        .success();
    
    // Test with limits
    harness
        .execute(["pattern", "list", "--limit", "5"])
        .assert()
        .success();
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
    harness
        .execute(["logs", "analyze"])
        .assert()
        .success();
        
    // Logs search
    harness
        .execute(["logs", "search", "error"])
        .assert()
        .success();

    // Logs stats
    harness
        .execute(["logs", "stats"])
        .assert()
        .success();
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
        .execute(["--dry-run", "backup", "create", "--path", backup_path.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn test_storage_command() {
    let harness = CliHarness::new();

    harness.execute(["storage", "stats"]).assert().success();
    harness.execute(["storage", "health"]).assert().success();
    harness.execute(["storage", "connections"]).assert().success();
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

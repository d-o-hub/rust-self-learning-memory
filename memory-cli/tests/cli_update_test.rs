//! CLI integration tests for episode update command.
//!
//! Invoke Cargo's built binary directly. Running `cargo run` per case made the
//! previous five tests take about 62 seconds each on this host.

#![allow(clippy::expect_used)]

use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;
use uuid::Uuid;

fn run_command(db_path: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_do-memory-cli"))
        .env("NO_COLOR", "1")
        .arg("--storage-mode")
        .arg("local")
        .arg("--db-path")
        .arg(db_path)
        .args(args)
        .output()
        .expect("Failed to execute do-memory-cli")
}

#[test]
fn test_update_persists_and_create_output_is_consistent() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let db_path = temp_dir.path().join("memory.redb");

    let json_output = run_command(
        &db_path,
        &[
            "--format",
            "json",
            "episode",
            "create",
            "--task",
            "Original task",
        ],
    );
    assert!(
        json_output.status.success(),
        "episode create failed: {}",
        String::from_utf8_lossy(&json_output.stderr)
    );
    let json_result: serde_json::Value =
        serde_json::from_slice(&json_output.stdout).expect("Create output must be JSON");
    let episode_id = json_result["episode_id"]
        .as_str()
        .expect("Create result must contain episode_id");
    assert_eq!(json_result["task"], "Original task");
    assert_eq!(json_result["status"], "created");
    assert!(json_result.get("id").is_none());

    let yaml_output = run_command(
        &db_path,
        &[
            "--format",
            "yaml",
            "episode",
            "create",
            "--task",
            "YAML task",
        ],
    );
    assert!(
        yaml_output.status.success(),
        "episode create --format yaml failed: {}",
        String::from_utf8_lossy(&yaml_output.stderr)
    );
    let yaml_result: serde_yaml::Value =
        serde_yaml::from_slice(&yaml_output.stdout).expect("Create output must be YAML");
    assert!(yaml_result["episode_id"].as_str().is_some());
    assert_eq!(yaml_result["task"].as_str(), Some("YAML task"));
    assert_eq!(yaml_result["status"].as_str(), Some("created"));
    assert!(yaml_result.get("id").is_none());

    let human_output = run_command(&db_path, &["episode", "create", "--task", "Human task"]);
    assert!(
        human_output.status.success(),
        "human-format episode create failed: {}",
        String::from_utf8_lossy(&human_output.stderr)
    );
    let human_text = String::from_utf8_lossy(&human_output.stdout);
    assert!(
        human_text.contains("Human task"),
        "Human output must include the created task"
    );
    let human_episode_id = human_text
        .split_whitespace()
        .find_map(|field| Uuid::parse_str(field).ok())
        .expect("Human output must include a valid episode ID");
    assert!(!human_episode_id.is_nil());

    let update_output = run_command(
        &db_path,
        &[
            "--format",
            "json",
            "episode",
            "update",
            episode_id,
            "--description",
            "Updated task",
        ],
    );
    assert!(
        update_output.status.success(),
        "episode update failed: {}",
        String::from_utf8_lossy(&update_output.stderr)
    );

    let view_output = run_command(
        &db_path,
        &["--format", "json", "episode", "view", episode_id],
    );
    assert!(
        view_output.status.success(),
        "episode view failed: {}",
        String::from_utf8_lossy(&view_output.stderr)
    );
    let view_result: serde_json::Value =
        serde_json::from_slice(&view_output.stdout).expect("Episode view output must be JSON");
    assert_eq!(view_result["episode_id"], episode_id);
    assert_eq!(view_result["task_description"], "Updated task");
}

#[test]
fn test_update_rejects_invalid_episode_id() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let db_path = temp_dir.path().join("memory.redb");

    let output = run_command(
        &db_path,
        &[
            "episode",
            "update",
            "invalid-uuid",
            "--description",
            "New description",
        ],
    );

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Invalid episode ID format"),
        "Expected invalid UUID error, got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

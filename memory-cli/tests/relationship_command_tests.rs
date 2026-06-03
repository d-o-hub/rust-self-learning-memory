//! Integration tests for relationship commands.
//!
//! This module provides behavior-oriented test coverage for relationship CLI core and validation logic.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use do_memory_cli::test_utils::CliHarness;
use predicates::prelude::*;
use serde_json::Value;
use uuid::Uuid;

/// Extracts the last JSON object from the given stdout bytes.
///
/// This is useful because the CLI might output log messages or ANSI escape sequences
/// before or after the actual JSON response.
fn extract_json(stdout: &[u8]) -> Value {
    let stdout_str = String::from_utf8_lossy(stdout);
    let mut depth = 0;
    let mut start = None;
    let mut last_json = None;

    for (i, &b) in stdout_str.as_bytes().iter().enumerate() {
        if b == b'{' {
            if depth == 0 {
                start = Some(i);
            }
            depth += 1;
            continue;
        }

        if b != b'}' || depth == 0 {
            continue;
        }

        depth -= 1;
        if depth != 0 {
            continue;
        }

        let Some(s) = start else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<Value>(&stdout_str[s..=i]) else {
            continue;
        };
        last_json = Some(json);
    }

    last_json.expect("Could not find JSON in output")
}

/// Tests that the relationship add command works in dry-run mode.
#[test]
fn test_relationship_add_dry_run() {
    let harness = CliHarness::new();
    let source = Uuid::new_v4().to_string();
    let target = Uuid::new_v4().to_string();
    harness
        .execute([
            "--dry-run",
            "relationship",
            "add",
            "--source",
            &source,
            "--target",
            &target,
            "--type",
            "depends-on",
            "--reason",
            "test reason",
            "--priority",
            "5",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Would create relationship:"));
}

/// Tests a full cycle of relationship operations: create episodes, add relationship,
/// info, list, find, graph, validate (success and failure), and remove.
#[test]
fn test_relationship_full_cycle() {
    let harness = CliHarness::new();

    // Create episodes
    let out1 = harness
        .execute(["--format", "json", "episode", "create", "--task", "t1"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json1 = extract_json(&out1);
    let ep1_id = json1["episode_id"]
        .as_str()
        .or_else(|| json1["id"].as_str())
        .expect("Missing episode_id")
        .to_string();

    let out2 = harness
        .execute(["--format", "json", "episode", "create", "--task", "t2"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json2 = extract_json(&out2);
    let ep2_id = json2["episode_id"]
        .as_str()
        .or_else(|| json2["id"].as_str())
        .expect("Missing episode_id")
        .to_string();

    // Add relationship
    let out_rel = harness
        .execute([
            "--format",
            "json",
            "relationship",
            "add",
            "--source",
            &ep1_id,
            "--target",
            &ep2_id,
            "--type",
            "related-to",
            "--reason",
            "logic",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let rel_id = extract_json(&out_rel)["relationship_id"]
        .as_str()
        .expect("Missing relationship_id")
        .to_string();

    // Info
    harness
        .execute(["--format", "json", "relationship", "info", &rel_id])
        .assert()
        .success()
        .stdout(predicate::str::contains(&rel_id));

    // List
    let out_list = harness
        .execute([
            "--format",
            "json",
            "relationship",
            "list",
            "--episode",
            &ep1_id,
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(
        extract_json(&out_list)["total_count"]
            .as_u64()
            .expect("Missing total_count"),
        1
    );

    // Find
    let out_find = harness
        .execute([
            "--format",
            "json",
            "relationship",
            "find",
            "--episode",
            &ep1_id,
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(
        extract_json(&out_find)["total_count"]
            .as_u64()
            .expect("Missing total_count")
            >= 1
    );

    // Graph
    harness
        .execute(["relationship", "graph", "--episode", &ep1_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Relationship Graph"));

    // Validate (no cycle)
    let out_val1 = harness
        .execute(["--format", "json", "relationship", "validate"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(!extract_json(&out_val1)["has_cycle"]
        .as_bool()
        .expect("Missing has_cycle"));

    // Create cycle
    harness
        .execute([
            "relationship",
            "add",
            "--source",
            &ep2_id,
            "--target",
            &ep1_id,
            "--type",
            "related-to",
        ])
        .assert()
        .success();

    // Validate (cycle)
    let out_val2 = harness
        .execute([
            "--format",
            "json",
            "relationship",
            "validate",
            "--type",
            "related-to",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(extract_json(&out_val2)["has_cycle"]
        .as_bool()
        .expect("Missing has_cycle"));

    // Remove
    harness
        .execute(["--format", "json", "relationship", "remove", &rel_id])
        .assert()
        .success();
}

/// Tests error handling for relationship commands with invalid inputs.
#[test]
fn test_relationship_errors() {
    let harness = CliHarness::new();
    harness
        .execute([
            "relationship",
            "add",
            "--source",
            "invalid",
            "--target",
            "invalid",
            "--type",
            "blocks",
        ])
        .assert()
        .failure();
    harness
        .execute(["relationship", "remove", &Uuid::new_v4().to_string()])
        .assert()
        .success();
    harness
        .execute(["relationship", "info", &Uuid::new_v4().to_string()])
        .assert()
        .failure();
}

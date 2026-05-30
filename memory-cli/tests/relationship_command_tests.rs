use do_memory_cli::test_utils::CliHarness;
use predicates::prelude::*;
use serde_json::Value;
use uuid::Uuid;

fn extract_json(stdout: &[u8]) -> Value {
    let stdout_str = String::from_utf8_lossy(stdout);
    let mut depth = 0;
    let mut start = None;
    let mut last_json = None;
    let bytes = stdout_str.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] == b'{' {
            if depth == 0 { start = Some(i); }
            depth += 1;
        } else if bytes[i] == b'}' {
            if depth > 0 {
                depth -= 1;
                if depth == 0 {
                    if let Some(s) = start {
                        let candidate = &stdout_str[s..=i];
                        if let Ok(json) = serde_json::from_str(candidate) {
                            last_json = Some(json);
                        }
                    }
                }
            }
        }
    }
    last_json.unwrap_or_else(|| panic!("Could not find JSON in output: {}", stdout_str))
}

#[test]
fn test_relationship_add_dry_run() {
    let harness = CliHarness::new();
    let source = Uuid::new_v4().to_string();
    let target = Uuid::new_v4().to_string();
    harness.execute([
        "--dry-run", "relationship", "add", "--source", &source, "--target", &target,
        "--type", "depends-on", "--reason", "test reason", "--priority", "5"
    ])
    .assert().success().stdout(predicate::str::contains("Would create relationship:"));
}

#[test]
fn test_relationship_full_cycle() {
    let harness = CliHarness::new();

    // Create episodes
    let out1 = harness.execute(["--format", "json", "episode", "create", "--task", "t1"]).assert().success().get_output().stdout.clone();
    let json1 = extract_json(&out1);
    let ep1_id = json1["episode_id"].as_str().or(json1["id"].as_str()).unwrap().to_string();

    let out2 = harness.execute(["--format", "json", "episode", "create", "--task", "t2"]).assert().success().get_output().stdout.clone();
    let json2 = extract_json(&out2);
    let ep2_id = json2["episode_id"].as_str().or(json2["id"].as_str()).unwrap().to_string();

    // Add relationship
    let out_rel = harness.execute([
        "--format", "json", "relationship", "add", "--source", &ep1_id, "--target", &ep2_id, "--type", "related-to", "--reason", "logic"
    ]).assert().success().get_output().stdout.clone();
    let rel_id = extract_json(&out_rel)["relationship_id"].as_str().unwrap().to_string();

    // Info
    harness.execute(["--format", "json", "relationship", "info", &rel_id]).assert().success()
        .stdout(predicate::str::contains(&rel_id));

    // List
    let out_list = harness.execute(["--format", "json", "relationship", "list", "--episode", &ep1_id]).assert().success().get_output().stdout.clone();
    assert_eq!(extract_json(&out_list)["total_count"].as_u64().unwrap(), 1);

    // Find
    let out_find = harness.execute(["--format", "json", "relationship", "find", "--episode", &ep1_id]).assert().success().get_output().stdout.clone();
    assert!(extract_json(&out_find)["total_count"].as_u64().unwrap() >= 1);

    // Graph
    harness.execute(["relationship", "graph", "--episode", &ep1_id]).assert().success()
        .stdout(predicate::str::contains("Relationship Graph"));

    // Validate (no cycle)
    let out_val1 = harness.execute(["--format", "json", "relationship", "validate"]).assert().success().get_output().stdout.clone();
    assert!(!extract_json(&out_val1)["has_cycle"].as_bool().unwrap());

    // Create cycle
    harness.execute([
        "relationship", "add", "--source", &ep2_id, "--target", &ep1_id, "--type", "related-to"
    ]).assert().success();

    // Validate (cycle)
    let out_val2 = harness.execute(["--format", "json", "relationship", "validate", "--type", "related-to"]).assert().success().get_output().stdout.clone();
    assert!(extract_json(&out_val2)["has_cycle"].as_bool().unwrap());

    // Remove
    harness.execute(["--format", "json", "relationship", "remove", &rel_id]).assert().success();
}

#[test]
fn test_relationship_errors() {
    let harness = CliHarness::new();
    harness.execute(["relationship", "add", "--source", "invalid", "--target", "invalid", "--type", "blocks"])
        .assert().failure();
    harness.execute(["relationship", "remove", &Uuid::new_v4().to_string()])
        .assert().success();
    harness.execute(["relationship", "info", &Uuid::new_v4().to_string()])
        .assert().failure();
}

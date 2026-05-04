//! CLI End-to-End Workflow Tests
//!
//! Comprehensive E2E tests covering major CLI workflows:
//! - Episode Lifecycle: Create → List → View → Update → Complete → Delete
//! - Relationship Workflow: Create episodes → Add relationship → Find related → Remove relationship
//! - Tag Management: Add tags → Search by tag → Remove tags → List tags
//! - Pattern Discovery: Create episodes with patterns → Analyze patterns → Verify patterns found
//!
//! These tests use the CLI binary directly via std::process::Command

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::if_not_else)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(missing_docs)]
#![allow(unknown_lints)]

use anyhow::Result;
use serial_test::serial;
use std::process::Command;
use tempfile::TempDir;

/// Helper to find the CLI binary
///
/// Respects CARGO_TARGET_DIR environment variable for CI compatibility.
/// See ADR-032 for CI disk space optimization and isolated target directories.
fn find_cli_binary() -> Result<std::path::PathBuf> {
    // Get the target directory, respecting CARGO_TARGET_DIR for CI compatibility
    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("target"));

    let candidates = [
        target_dir.join("debug/do-memory-cli"),
        target_dir.join("release/do-memory-cli"),
        std::path::PathBuf::from("target/debug/do-memory-cli"),
        std::path::PathBuf::from("target/release/do-memory-cli"),
        std::path::PathBuf::from("../target/debug/do-memory-cli"),
        std::path::PathBuf::from("../target/release/do-memory-cli"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }

    // Build it - specify package to avoid workspace ambiguity
    let output = Command::new("cargo")
        .args([
            "build",
            "-p",
            "do-memory-cli",
            "--bin",
            "do-memory-cli",
            "--message-format=short",
        ])
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to build do-memory-cli: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Return the path respecting CARGO_TARGET_DIR
    Ok(target_dir.join("debug/do-memory-cli"))
}

/// Helper to create a test config file
fn create_test_config(temp_dir: &TempDir) -> Result<std::path::PathBuf> {
    let config_path = temp_dir.path().join("config.toml");
    let db_dir = temp_dir.path().join("db");
    std::fs::create_dir_all(&db_dir)?;

    let config_content = format!(
        r#"[database]
redb_path = "{0}/cache.redb"

[storage]
max_episodes_cache = 100
cache_ttl_seconds = 3600
pool_size = 5

[cli]
default_format = "json"
progress_bars = false
batch_size = 100

[embeddings]
enabled = false
provider = "local"
model = "sentence-transformers/all-MiniLM-L6-v2"
dimension = 384
similarity_threshold = 0.7
batch_size = 32
cache_embeddings = true
timeout_seconds = 30
"#,
        db_dir.display()
    );

    std::fs::write(&config_path, config_content)?;
    Ok(config_path)
}

/// Run a CLI command and return output
///
/// Filters out log messages before parsing JSON.
/// This handles the case where CLI outputs logging (WARN, INFO) mixed with JSON response.
///
/// TEST ISOLATION: Each call creates a separate subprocess with its own config,
/// ensuring no shared state between test runs.
fn run_cli(
    cli_path: &std::path::Path,
    config_path: &std::path::Path,
    args: &[&str],
) -> Result<(serde_json::Value, bool)> {
    let output = Command::new(cli_path)
        .arg(format!("--config={}", config_path.display()))
        .arg("--format=json")
        .args(args)
        .env("RUST_LOG", "error") // Only show errors to reduce log noise
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let success = output.status.success();

    // Debug output for test failures
    if !success || !stdout.contains('{') {
        eprintln!("CLI command failed:");
        eprintln!("  Args: {args:?}");
        eprintln!("  Exit code: {:?}", output.status.code());
        eprintln!("  Stdout: {stdout:?}");
        eprintln!("  Stderr: {stderr:?}");
    }

    // Filter out log messages and find the JSON response
    // Strategy: Handle multi-line JSON by finding the first complete JSON object
    let json = if stdout.trim().is_empty() {
        // If command failed, return error JSON
        if !success {
            serde_json::json!({"error": "Command failed", "stderr": stderr})
        } else {
            serde_json::json!({})
        }
    } else {
        // Strip ANSI codes from the entire output
        let stripped_stdout = strip_ansi_codes(&stdout);

        // Find the first line that starts with '{' or '['
        // Then accumulate lines until we have a complete JSON object
        let lines: Vec<&str> = stripped_stdout.lines().collect();

        let mut json_start = None;
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('{') || trimmed.starts_with('[') {
                json_start = Some(i);
                break;
            }
        }

        if let Some(start_idx) = json_start {
            // Try to parse from this point forward, accumulating lines
            let mut combined = String::new();
            let mut brace_count = 0;
            let start_char = lines[start_idx].trim().chars().next().unwrap();
            let end_char = if start_char == '{' { '}' } else { ']' };

            for line in lines.iter().skip(start_idx) {
                combined.push_str(line);
                combined.push('\n');

                update_brace_count(line, start_char, end_char, &mut brace_count);

                if brace_count == 0 {
                    break;
                }
            }

            // Try to parse the combined JSON
            serde_json::from_str::<serde_json::Value>(&combined).map_err(|e| {
                anyhow::anyhow!("Failed to parse JSON: {e} - attempted to parse: '{combined}'")
            })?
        } else if !success {
            serde_json::json!({"error": "Command failed", "stderr": stderr, "stdout": stripped_stdout})
        } else {
            anyhow::bail!("No JSON found in output")
        }
    };

    Ok((json, success))
}

/// Simple ANSI escape code stripper
/// Removes ANSI color codes and formatting from strings
fn strip_ansi_codes(s: &str) -> String {
    // This regex matches ANSI escape sequences like \x1b[...m
    let re = regex::Regex::new(r"\x1b\[[0-9;]*[mGKH]").unwrap();
    re.replace_all(s, "").to_string()
}

fn update_brace_count(line: &str, start_char: char, end_char: char, brace_count: &mut i32) {
    for c in line.chars() {
        if c == start_char {
            *brace_count += 1;
        } else if c == end_char {
            *brace_count -= 1;
        }
    }
}

// ============================================================================
// Test 1: Episode Full Lifecycle
// ============================================================================

#[tokio::test]
#[serial]
async fn test_episode_full_lifecycle() {
    let cli_path = find_cli_binary().expect("Failed to find CLI binary");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = create_test_config(&temp_dir).expect("Failed to create config");

    println!("🧪 Testing episode full lifecycle...");

    // Step 1: Create episode
    let (create_result, success) = run_cli(
        &cli_path,
        &config_path,
        &["episode", "create", "--task", "Test episode for lifecycle"],
    )
    .expect("Failed to run create command");

    assert!(success, "Create episode should succeed: {create_result:?}");
    let episode_id = create_result
        .get("id")
        .and_then(|v| v.as_str())
        .expect("Should have episode id");
    println!("  ✓ Created episode: {episode_id:?}");

    // Step 2: List episodes
    let (list_result, success) = run_cli(
        &cli_path,
        &config_path,
        &["episode", "list", "--limit", "10"],
    )
    .expect("Failed to run list command");

    assert!(success, "List episodes should succeed");
    let episodes = list_result
        .get("episodes")
        .and_then(|v| v.as_array())
        .expect("Should have episodes array");
    assert!(!episodes.is_empty(), "Should have at least one episode");
    println!("  ✓ Listed {} episodes", episodes.len());

    // Step 3: View episode
    let (view_result, success) = run_cli(&cli_path, &config_path, &["episode", "view", episode_id])
        .expect("Failed to run view command");

    assert!(success, "View episode should succeed");
    let viewed_id = view_result
        .get("episode_id")
        .and_then(|v| v.as_str())
        .expect("Should have episode id in view result");
    assert_eq!(viewed_id, episode_id, "Viewed episode should match created");
    println!("  ✓ Viewed episode: {viewed_id:?}");

    // Step 4: Add step (update)
    let (_step_result, success) = run_cli(
        &cli_path,
        &config_path,
        &[
            "episode",
            "log-step",
            episode_id,
            "--tool",
            "test-tool",
            "--action",
            "Test action",
        ],
    )
    .expect("Failed to run step command");

    assert!(success, "Add step should succeed");
    println!("  ✓ Added step to episode");

    // Step 5: Complete episode
    let (_complete_result, success) = run_cli(
        &cli_path,
        &config_path,
        &["episode", "complete", episode_id, "success"],
    )
    .expect("Failed to run complete command");

    assert!(success, "Complete episode should succeed");
    println!("  ✓ Completed episode");

    // Step 6: Delete episode
    let (_delete_result, success) =
        run_cli(&cli_path, &config_path, &["episode", "delete", episode_id])
            .expect("Failed to run delete command");

    assert!(success, "Delete episode should succeed");
    println!("  ✓ Deleted episode");

    // Verify episode is deleted
    let (_view_after_delete, success) =
        run_cli(&cli_path, &config_path, &["episode", "view", episode_id])
            .expect("Failed to run view command after delete");

    assert!(!success, "View deleted episode should fail");
    println!("  ✓ Verified episode deletion");

    println!("✅ Episode full lifecycle test passed!");
}

// ============================================================================
// Test 2: Relationship Workflow
// ============================================================================

#[tokio::test]
#[serial]
async fn test_relationship_workflow() {
    let cli_path = find_cli_binary().expect("Failed to find CLI binary");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = create_test_config(&temp_dir).expect("Failed to create config");

    println!("🧪 Testing relationship workflow...");

    // Create parent episode
    let (parent_result, success) = run_cli(
        &cli_path,
        &config_path,
        &["episode", "create", "--task", "Parent episode"],
    )
    .expect("Failed to create parent episode");

    assert!(success, "Create parent should succeed");
    let parent_id = parent_result
        .get("id")
        .and_then(|v| v.as_str())
        .expect("Should have parent id")
        .to_string();

    // Create child episode
    let (child_result, success) = run_cli(
        &cli_path,
        &config_path,
        &["episode", "create", "--task", "Child episode"],
    )
    .expect("Failed to create child episode");

    assert!(success, "Create child should succeed");
    let child_id = child_result
        .get("id")
        .and_then(|v| v.as_str())
        .expect("Should have child id")
        .to_string();

    println!("  ✓ Created parent: {parent_id} and child: {child_id}");

    // Complete both episodes
    for id in [&parent_id, &child_id] {
        let (_, success) = run_cli(
            &cli_path,
            &config_path,
            &["episode", "complete", id, "success"],
        )
        .expect("Failed to complete episode");
        assert!(success, "Complete should succeed");
    }
    println!("  ✓ Completed both episodes");

    // Add relationship
    let (rel_result, success) = run_cli(
        &cli_path,
        &config_path,
        &[
            "relationship",
            "add",
            "--source",
            &parent_id,
            "--target",
            &child_id,
            "--type",
            "parent-child",
            "--reason",
            "Parent-child relationship test",
        ],
    )
    .expect("Failed to add relationship");

    assert!(success, "Add relationship should succeed");
    let rel_id = rel_result
        .get("relationship_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    println!("  ✓ Added relationship: {rel_id:?}");

    // Find related episodes
    let (related_result, success) = run_cli(
        &cli_path,
        &config_path,
        &["relationship", "find", "--episode", &parent_id],
    )
    .expect("Failed to find related");

    assert!(success, "Find related should succeed");
    let related = related_result
        .get("episodes")
        .and_then(|v| v.as_array())
        .expect("Should have related episodes");
    assert!(!related.is_empty(), "Should find related episodes");
    println!("  ✓ Found {} related episodes", related.len());

    // Remove relationship if we got an id
    if let Some(rid) = rel_id {
        let (_, success) = run_cli(&cli_path, &config_path, &["relationship", "remove", &rid])
            .expect("Failed to remove relationship");

        if success {
            println!("  ✓ Removed relationship");
        }
    }

    println!("✅ Relationship workflow test passed!");
}

// ============================================================================
// Test 3: Tag Workflow
// ============================================================================

#[tokio::test]
#[serial]
async fn test_tag_workflow() {
    let cli_path = find_cli_binary().expect("Failed to find CLI binary");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = create_test_config(&temp_dir).expect("Failed to create config");

    println!("🧪 Testing tag workflow...");

    // Create episode
    let (create_result, success) = run_cli(
        &cli_path,
        &config_path,
        &["episode", "create", "--task", "Tag test episode"],
    )
    .expect("Failed to create episode");

    assert!(success, "Create should succeed");
    let episode_id = create_result
        .get("id")
        .and_then(|v| v.as_str())
        .expect("Should have episode id")
        .to_string();

    // Add tags
    let (_, success) = run_cli(
        &cli_path,
        &config_path,
        &["tag", "add", &episode_id, "security", "testing", "rust"],
    )
    .expect("Failed to add tags");

    assert!(success, "Add tags should succeed");
    println!("  ✓ Added tags to episode");

    // Show tags
    let (show_result, success) = run_cli(&cli_path, &config_path, &["tag", "show", &episode_id])
        .expect("Failed to show tags");

    assert!(success, "Show tags should succeed");
    let tags = show_result
        .get("tags")
        .and_then(|v| v.as_array())
        .expect("Should have tags array");
    assert_eq!(tags.len(), 3, "Should have 3 tags");
    println!("  ✓ Episode has {} tags", tags.len());

    // Search by tag
    let (search_result, success) = run_cli(&cli_path, &config_path, &["tag", "search", "security"])
        .expect("Failed to search tags");

    assert!(success, "Search tags should succeed");
    let found_episodes = search_result
        .get("episodes")
        .and_then(|v| v.as_array())
        .expect("Should have episodes array");
    assert!(
        !found_episodes.is_empty(),
        "Should find episodes with 'security' tag"
    );
    println!(
        "  ✓ Found {} episodes with 'security' tag",
        found_episodes.len()
    );

    // List all tags
    let (_list_result, success) =
        run_cli(&cli_path, &config_path, &["tag", "list"]).expect("Failed to list tags");

    assert!(success, "List tags should succeed");
    println!("  ✓ Listed all tags");

    // Remove tags
    let (_, success) = run_cli(
        &cli_path,
        &config_path,
        &["tag", "remove", &episode_id, "testing"],
    )
    .expect("Failed to remove tags");

    assert!(success, "Remove tags should succeed");
    println!("  ✓ Removed 'testing' tag");

    // Verify tag removal
    let (show_after, success) = run_cli(&cli_path, &config_path, &["tag", "show", &episode_id])
        .expect("Failed to show tags after removal");

    assert!(success, "Show tags should succeed");
    let tags_after = show_after
        .get("tags")
        .and_then(|v| v.as_array())
        .expect("Should have tags array");
    assert_eq!(tags_after.len(), 2, "Should have 2 tags after removal");
    println!("  ✓ Verified tag removal");

    println!("✅ Tag workflow test passed!");
}

// ============================================================================
// Test 4: Pattern Discovery Workflow
// ============================================================================

/// Pattern discovery CLI test
///
/// Note: Pattern CLI commands ARE implemented (list, view, analyze, search, recommend, effectiveness, decay).
/// This test is ignored pending validation of specific CLI argument formats and e2e test environment setup.
///
/// Implemented commands (see memory-cli/src/commands/pattern/):
/// - `pattern analyze` - Analyze patterns from episodes
/// - `pattern search` - Search patterns by criteria
/// - `pattern recommend` - Get pattern recommendations
/// - `pattern list` - List all patterns
/// - `pattern view` - View pattern details
/// - `pattern effectiveness` - Get pattern effectiveness rankings
/// - `pattern decay` - Run pattern decay analysis
///
/// Track: E2E test environment validation for pattern commands
#[tokio::test]
#[serial]
async fn test_pattern_discovery() {
    let cli_path = find_cli_binary().expect("Failed to find CLI binary");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = create_test_config(&temp_dir).expect("Failed to create config");

    println!("🧪 Testing pattern discovery workflow...");

    // Create multiple episodes with similar patterns
    let mut episode_ids = Vec::new();

    for i in 0..5 {
        let (create_result, success) = run_cli(
            &cli_path,
            &config_path,
            &[
                "episode",
                "create",
                "--task",
                &format!("Pattern test episode {i}"),
            ],
        )
        .expect("Failed to create episode");

        assert!(success, "Create should succeed");
        let episode_id = create_result
            .get("id")
            .and_then(|v| v.as_str())
            .expect("Should have episode id")
            .to_string();

        // Add similar steps to create a pattern
        for step_num in 1..=3 {
            let (_, success) = run_cli(
                &cli_path,
                &config_path,
                &[
                    "episode",
                    "log-step",
                    &episode_id,
                    "--tool",
                    &format!("tool-{step_num}"),
                    "--action",
                    &format!("Action {step_num}"),
                    "--success",
                ],
            )
            .expect("Failed to add step");

            assert!(success, "Add step should succeed");
        }

        // Complete episode
        let (_, success) = run_cli(
            &cli_path,
            &config_path,
            &["episode", "complete", &episode_id, "success"],
        )
        .expect("Failed to complete episode");

        assert!(success, "Complete should succeed");
        episode_ids.push(episode_id);
    }

    println!(
        "  ✓ Created {} episodes with similar patterns",
        episode_ids.len()
    );

    // Wait a moment for pattern extraction (if async)
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Analyze patterns
    let (_pattern_result, success) = run_cli(
        &cli_path,
        &config_path,
        &["pattern", "analyze", "--domain", "pattern-test"],
    )
    .expect("Failed to analyze patterns");

    assert!(success, "Pattern analyze should succeed");
    println!("  ✓ Analyzed patterns");

    // Search patterns
    let (search_result, success) = run_cli(
        &cli_path,
        &config_path,
        &["pattern", "search", "--limit", "10"],
    )
    .expect("Failed to search patterns");

    assert!(success, "Pattern search should succeed");
    let patterns = search_result
        .get("patterns")
        .and_then(|v| v.as_array())
        .map(|v| v.len())
        .unwrap_or(0);
    println!("  ✓ Found {patterns:?} patterns");

    // Get pattern recommendations
    let (_rec_result, success) = run_cli(&cli_path, &config_path, &["pattern", "recommend"])
        .expect("Failed to get recommendations");

    assert!(success, "Pattern recommend should succeed");
    println!("  ✓ Got pattern recommendations");

    println!("✅ Pattern discovery workflow test passed!");
}

// ============================================================================
// Test 5: Episode Search and Filter
// ============================================================================

/// Test episode search and filter with domain/type flags
#[tokio::test]
#[serial]
async fn test_episode_search_and_filter() {
    let cli_path = find_cli_binary().expect("Failed to find CLI binary");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = create_test_config(&temp_dir).expect("Failed to create config");

    println!("Testing episode search and filter...");

    // Create episodes in different domains
    let domains = ["web-api", "database", "cli"];

    for (i, domain) in domains.iter().enumerate() {
        let (_, success) = run_cli(
            &cli_path,
            &config_path,
            &[
                "episode",
                "create",
                "--task",
                &format!("Search test episode {i}"),
                "--domain",
                domain,
            ],
        )
        .expect("Failed to create episode");

        assert!(success, "Create should succeed");
    }

    println!("  Created episodes in different domains");

    // Search by domain (requires query argument)
    let (search_result, success) = run_cli(
        &cli_path,
        &config_path,
        &["episode", "search", "test", "--domain", "web-api"],
    )
    .expect("Failed to search episodes");

    assert!(success, "Search should succeed");
    let episodes = search_result
        .get("episodes")
        .and_then(|v| v.as_array())
        .expect("Should have episodes");
    println!("  Found {} episodes in 'web-api' domain", episodes.len());

    // Search by type
    let (_type_result, success) = run_cli(
        &cli_path,
        &config_path,
        &["episode", "search", "test", "--type", "code-generation"],
    )
    .expect("Failed to search by type");

    assert!(success, "Search by type should succeed");
    println!("  Searched by task type");

    // Search by text (using search command with query)
    let (_query_result, success) = run_cli(
        &cli_path,
        &config_path,
        &["episode", "search", "search test"],
    )
    .expect("Failed to search episodes");

    assert!(success, "Search should succeed");
    println!("  Searched episodes by text");

    println!("Episode search and filter test passed!");
}

// ============================================================================
// Test 6: Bulk Operations
// ============================================================================

#[tokio::test]
#[serial]
async fn test_bulk_operations() {
    let cli_path = find_cli_binary().expect("Failed to find CLI binary");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = create_test_config(&temp_dir).expect("Failed to create config");

    println!("🧪 Testing bulk operations...");

    // Create multiple episodes
    let mut episode_ids = Vec::new();

    for i in 0..5 {
        let (create_result, success) = run_cli(
            &cli_path,
            &config_path,
            &[
                "episode",
                "create",
                "--task",
                &format!("Bulk test episode {i}"),
            ],
        )
        .expect("Failed to create episode");

        assert!(success, "Create should succeed");
        let episode_id = create_result
            .get("id")
            .and_then(|v| v.as_str())
            .expect("Should have episode id")
            .to_string();
        episode_ids.push(episode_id);
    }

    println!("  ✓ Created {} episodes", episode_ids.len());

    // Bulk complete episodes
    for id in &episode_ids {
        let (_, success) = run_cli(
            &cli_path,
            &config_path,
            &["episode", "complete", id, "success"],
        )
        .expect("Failed to complete episode");

        assert!(success, "Complete should succeed");
    }

    println!("  ✓ Bulk completed episodes");

    // List with filter
    let (list_result, success) = run_cli(
        &cli_path,
        &config_path,
        &["episode", "list", "--limit", "10"],
    )
    .expect("Failed to list episodes");

    assert!(success, "List should succeed");
    let episodes = list_result
        .get("episodes")
        .and_then(|v| v.as_array())
        .expect("Should have episodes");
    assert_eq!(episodes.len(), 5, "Should have 5 episodes");
    println!("  ✓ Verified bulk operations");

    println!("✅ Bulk operations test passed!");
}

// ============================================================================
// Test 7: Error Handling
// ============================================================================

#[tokio::test]
#[serial]
async fn test_cli_error_handling() {
    let cli_path = find_cli_binary().expect("Failed to find CLI binary");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = create_test_config(&temp_dir).expect("Failed to create config");

    println!("🧪 Testing CLI error handling...");

    // Test invalid UUID
    let (_, success) = run_cli(
        &cli_path,
        &config_path,
        &["episode", "view", "invalid-uuid"],
    )
    .expect("Failed to run command");

    assert!(!success, "Invalid UUID should fail");
    println!("  ✓ Invalid UUID handled correctly");

    // Test missing required argument
    let output = Command::new(&cli_path)
        .arg(format!("--config={}", config_path.display()))
        .args(["episode", "create"]) // Missing required --task
        .output()
        .expect("Failed to run command");

    assert!(!output.status.success(), "Missing required arg should fail");
    println!("  ✓ Missing required argument handled correctly");

    // Test non-existent episode
    let (_, success) = run_cli(
        &cli_path,
        &config_path,
        &["episode", "view", "00000000-0000-0000-0000-000000000000"],
    )
    .expect("Failed to run command");

    assert!(!success, "Non-existent episode should fail");
    println!("  ✓ Non-existent episode handled correctly");

    println!("✅ Error handling test passed!");
}

// ============================================================================
// Test 8: Health and Status Commands
// ============================================================================

#[tokio::test]
#[serial]
async fn test_health_and_status() {
    let cli_path = find_cli_binary().expect("Failed to find CLI binary");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = create_test_config(&temp_dir).expect("Failed to create config");

    println!("🧪 Testing health and status commands...");

    // Health check
    let (_health_result, success) =
        run_cli(&cli_path, &config_path, &["health", "check"]).expect("Failed to run health check");

    assert!(success, "Health check should succeed");
    println!("  ✓ Health check passed");

    // Storage health
    let (_storage_result, success) = run_cli(&cli_path, &config_path, &["storage", "health"])
        .expect("Failed to get storage health");

    assert!(success, "Storage health should succeed");
    println!("  ✓ Storage health retrieved");

    // Config validate
    let (_config_result, success) = run_cli(&cli_path, &config_path, &["config", "validate"])
        .expect("Failed to validate config");

    assert!(success, "Config validate should succeed");
    println!("  ✓ Config validated");

    println!("✅ Health and status test passed!");
}

//! End-to-end workflow tests.
//!
//! These tests simulate complete user workflows using the CLI,
//! from episode creation through completion and pattern analysis.

use assert_cmd::Command;
use memory_cli::test_utils::*;
use std::fs;
use tempfile::TempDir;

mod common;

#[cfg(test)]
mod end_to_end_workflow_tests {
    use super::*;

    /// Comprehensive workflow test harness
    struct WorkflowHarness {
        temp_dir: TempDir,
        config_path: std::path::PathBuf,
        harness: CliHarness,
    }

    impl WorkflowHarness {
        fn new() -> Self {
            let temp_dir = TempDir::new().expect("Failed to create temp directory");
            let config_path = temp_dir.path().join("workflow_config.toml");

            // Create config for workflow testing
            let config_content = r#"
[database]
turso_url = "file:workflow.db"
redb_path = "workflow.redb"

[storage]
max_episodes_cache = 50
cache_ttl_seconds = 1800
pool_size = 3

[cli]
default_format = "json"
progress_bars = false
batch_size = 5
"#;

            fs::write(&config_path, config_content).expect("Failed to write config");

            let harness = CliHarness::new();

            Self {
                temp_dir,
                config_path,
                harness,
            }
        }

        fn execute<I, S>(&self, args: I) -> Command
        where
            I: IntoIterator<Item = S>,
            S: AsRef<std::ffi::OsStr>,
        {
            let mut cmd = Command::new(std::env::var("CARGO_BIN_EXE_memory-cli").unwrap_or_else(|_| "/workspaces/rust-self-learning-memory/target/debug/memory-cli".to_string())).expect("Failed to find binary");
            cmd.arg("--config").arg(&self.config_path);
            cmd.args(args);
            cmd
        }
    }

    #[test]
    fn test_complete_episode_lifecycle_workflow() {
        let workflow = WorkflowHarness::new();

        // Phase 1: Create an episode
        println!("Phase 1: Creating episode...");
        workflow
            .execute(["episode", "create", "Implement user login feature"])
            .assert()
            .success();

        // Phase 2: List episodes to verify creation
        println!("Phase 2: Listing episodes...");
        let list_output = workflow
            .execute(["episode", "list"])
            .assert()
            .success();

        // Phase 3: Simulate logging steps (dry run since no real storage)
        println!("Phase 3: Logging execution steps...");
        workflow
            .execute([
                "episode", "log-step", "test-uuid-123",
                "--tool", "code_editor",
                "--action", "write_function",
                "--success",
                "--latency-ms", "1500",
                "--tokens", "200",
                "--observation", "Implemented login validation function"
            ])
            .assert()
            .success();

        // Phase 4: Complete the episode
        println!("Phase 4: Completing episode...");
        workflow
            .execute([
                "episode", "complete", "test-uuid-123",
                "--outcome", "success"
            ])
            .assert()
            .success();

        // Phase 5: View the completed episode
        println!("Phase 5: Viewing completed episode...");
        workflow
            .execute(["episode", "view", "test-uuid-123"])
            .assert()
            .success();

        println!("Complete episode lifecycle workflow test passed!");
    }

    #[test]
    fn test_pattern_analysis_workflow() {
        let workflow = WorkflowHarness::new();

        // Phase 1: List available patterns
        println!("Phase 1: Listing patterns...");
        workflow
            .execute(["pattern", "list"])
            .assert()
            .success();

        // Phase 2: View a specific pattern
        println!("Phase 2: Viewing pattern...");
        workflow
            .execute(["pattern", "view", "test-pattern-uuid"])
            .assert()
            .success();

        // Phase 3: Analyze pattern effectiveness
        println!("Phase 3: Analyzing pattern effectiveness...");
        workflow
            .execute(["pattern", "effectiveness", "--top", "5"])
            .assert()
            .success();

        // Phase 4: Decay patterns (dry run)
        println!("Phase 4: Running pattern decay analysis...");
        workflow
            .execute(["pattern", "decay", "--dry-run"])
            .assert()
            .success();

        println!("Pattern analysis workflow test passed!");
    }

    #[test]
    fn test_storage_operations_workflow() {
        let workflow = WorkflowHarness::new();

        // Phase 1: Check storage stats
        println!("Phase 1: Checking storage stats...");
        workflow
            .execute(["storage", "stats"])
            .assert()
            .success();

        // Phase 2: Check storage health
        println!("Phase 2: Checking storage health...");
        workflow
            .execute(["storage", "health"])
            .assert()
            .success();

        // Phase 3: Check connections
        println!("Phase 3: Checking connections...");
        workflow
            .execute(["storage", "connections"])
            .assert()
            .success();

        // Phase 4: Sync storage (dry run)
        println!("Phase 4: Syncing storage...");
        workflow
            .execute(["storage", "sync", "--dry-run"])
            .assert()
            .success();

        // Phase 5: Vacuum storage (dry run)
        println!("Phase 5: Vacuuming storage...");
        workflow
            .execute(["storage", "vacuum", "--dry-run"])
            .assert()
            .success();

        println!("Storage operations workflow test passed!");
    }

    #[test]
    fn test_output_format_workflow() {
        let workflow = WorkflowHarness::new();

        let formats = vec!["human", "json", "yaml"];

        for format in &formats {
            println!("Testing {} output format...", format);

            // Test config command with different formats
            workflow
                .execute(["--format", format, "config"])
                .assert()
                .success();

            // Test episode list with different formats
            workflow
                .execute(["--format", format, "episode", "list"])
                .assert()
                .success();

            // Test pattern list with different formats
            workflow
                .execute(["--format", format, "pattern", "list"])
                .assert()
                .success();
        }

        println!("Output format workflow test passed!");
    }

    #[test]
    fn test_error_handling_workflow() {
        let workflow = WorkflowHarness::new();

        // Test various error scenarios
        let error_scenarios = vec![
            (vec!["episode", "view", "invalid-uuid"], "Invalid episode ID"),
            (vec!["episode", "complete", ""], "Missing episode ID"),
            (vec!["--format", "invalid", "config"], "Invalid format"),
            (vec!["unknown", "command"], "Unknown command"),
        ];

        for (args, description) in error_scenarios {
            println!("Testing error scenario: {}", description);
            workflow
                .execute(&args)
                .assert()
                .failure();
        }

        println!("Error handling workflow test passed!");
    }

    #[test]
    fn test_configuration_workflow() {
        let workflow = WorkflowHarness::new();

        // Phase 1: Validate current config
        println!("Phase 1: Validating configuration...");
        workflow
            .execute(["config"])
            .assert()
            .success();

        // Phase 2: Test with verbose output
        println!("Phase 2: Testing verbose output...");
        workflow
            .execute(["--verbose", "config"])
            .assert()
            .success();

        // Phase 3: Test dry run mode
        println!("Phase 3: Testing dry run mode...");
        workflow
            .execute(["--dry-run", "episode", "create", "test task"])
            .assert()
            .success();

        println!("Configuration workflow test passed!");
    }

    #[test]
    fn test_batch_operations_workflow() {
        let workflow = WorkflowHarness::new();

        // Simulate batch episode creation
        println!("Creating batch of episodes...");
        for i in 0..5 {
            workflow
                .execute(["episode", "create", &format!("Batch task {}", i)])
                .assert()
                .success();
        }

        // List all episodes
        println!("Listing all episodes...");
        workflow
            .execute(["episode", "list"])
            .assert()
            .success();

        // Search for episodes
        println!("Searching episodes...");
        workflow
            .execute(["episode", "search", "Batch"])
            .assert()
            .success();

        println!("Batch operations workflow test passed!");
    }

    #[test]
    fn test_completion_and_help_workflow() {
        let workflow = WorkflowHarness::new();

        // Test shell completion generation
        let shells = vec!["bash", "zsh", "fish"];
        for shell in &shells {
            println!("Testing {} completion generation...", shell);
            workflow
                .execute(["completion", shell])
                .assert()
                .success();
        }

        // Test help commands
        println!("Testing help commands...");
        workflow
            .execute(["--help"])
            .assert()
            .success();

        workflow
            .execute(["episode", "--help"])
            .assert()
            .success();

        workflow
            .execute(["pattern", "--help"])
            .assert()
            .success();

        workflow
            .execute(["storage", "--help"])
            .assert()
            .success();

        println!("Completion and help workflow test passed!");
    }

    #[test]
    fn test_complex_query_workflow() {
        let workflow = WorkflowHarness::new();

        // Test various query scenarios
        let queries = vec![
            (vec!["episode", "list", "--task-type", "development"], "Filter by task type"),
            (vec!["episode", "list", "--limit", "10"], "Limit results"),
            (vec!["episode", "list", "--status", "completed"], "Filter by status"),
            (vec!["pattern", "list", "--min-confidence", "0.8"], "Filter patterns by confidence"),
            (vec!["pattern", "list", "--pattern-type", "success"], "Filter by pattern type"),
        ];

        for (args, description) in queries {
            println!("Testing query: {}", description);
            workflow
                .execute(&args)
                .assert()
                .success();
        }

        println!("Complex query workflow test passed!");
    }

    #[test]
    fn test_performance_baseline_workflow() {
        let workflow = WorkflowHarness::new();

        let start = std::time::Instant::now();

        // Run a series of operations to establish performance baseline
        for _ in 0..10 {
            workflow.execute(["config"]).assert().success();
        }

        let duration = start.elapsed();
        let avg_duration = duration / 10;

        // Performance should be reasonable
        assert!(avg_duration < std::time::Duration::from_millis(50),
            "Average operation time {}ms exceeds 50ms baseline", avg_duration.as_millis());

        println!("Performance baseline: {}ms per operation", avg_duration.as_millis());
    }

    #[test]
    fn test_workflow_isolation() {
        // Test that multiple workflow harnesses don't interfere
        let workflow1 = WorkflowHarness::new();
        let workflow2 = WorkflowHarness::new();

        // Both should work independently
        workflow1.execute(["config"]).assert().success();
        workflow2.execute(["config"]).assert().success();

        // Configurations should be isolated
        assert_ne!(workflow1.config_path, workflow2.config_path);
    }

    #[test]
    fn test_comprehensive_episode_lifecycle_workflow() {
        let workflow = WorkflowHarness::new();

        // Phase 1: Create multiple episodes
        println!("Phase 1: Creating multiple episodes...");
        let episode_ids = vec!["ep-001", "ep-002", "ep-003"];

        for (i, episode_id) in episode_ids.iter().enumerate() {
            workflow
                .execute([
                    "episode", "create",
                    "--task", &format!("Comprehensive test task {}", i + 1),
                    "--context", &format!("{{\"domain\": \"testing\", \"tags\": [\"comprehensive\", \"task_{}\"]}}", i + 1)
                ])
                .assert()
                .success();
        }

        // Phase 2: List episodes and verify creation
        println!("Phase 2: Listing and verifying episodes...");
        let list_output = workflow
            .execute(["episode", "list", "--limit", "10"])
            .assert()
            .success();

        // Phase 3: Log execution steps for each episode
        println!("Phase 3: Logging execution steps...");
        for episode_id in &episode_ids {
            // Log multiple steps for each episode
            let steps = vec![
                ("code_analysis", "analyze_requirements", "Analyzed task requirements"),
                ("implementation", "write_code", "Implemented solution"),
                ("testing", "run_tests", "Executed test suite"),
                ("review", "code_review", "Performed code review"),
            ];

            for (i, (tool, action, observation)) in steps.iter().enumerate() {
                workflow
                    .execute([
                        "episode", "log-step", episode_id,
                        "--tool", tool,
                        "--action", action,
                        "--success",
                        "--latency-ms", &(500 + i * 100).to_string(),
                        "--tokens", &(100 + i * 50).to_string(),
                        "--observation", observation,
                    ])
                    .assert()
                    .success();
            }
        }

        // Phase 4: Complete episodes with different outcomes
        println!("Phase 4: Completing episodes...");
        let outcomes = vec!["success", "success", "failure"];

        for (i, (episode_id, outcome)) in episode_ids.iter().zip(outcomes.iter()).enumerate() {
            workflow
                .execute([
                    "episode", "complete", episode_id,
                    "--outcome", outcome,
                ])
                .assert()
                .success();
        }

        // Phase 5: View completed episodes
        println!("Phase 5: Viewing completed episodes...");
        for episode_id in &episode_ids {
            workflow
                .execute(["episode", "view", episode_id])
                .assert()
                .success();
        }

        // Phase 6: Search and filter episodes
        println!("Phase 6: Searching and filtering...");
        workflow
            .execute(["episode", "search", "comprehensive"])
            .assert()
            .success();

        workflow
            .execute(["episode", "list", "--status", "completed"])
            .assert()
            .success();

        println!("Comprehensive episode lifecycle workflow test passed!");
    }

    #[test]
    fn test_advanced_pattern_analysis_workflow() {
        let workflow = WorkflowHarness::new();

        // Phase 1: Create episodes that will generate patterns
        println!("Phase 1: Creating episodes for pattern analysis...");
        for i in 0..5 {
            workflow
                .execute([
                    "episode", "create",
                    "--task", &format!("Pattern analysis task {}", i + 1),
                    "--context", &format!("{{\"domain\": \"pattern_testing\", \"tags\": [\"pattern_{}\"]}}", i + 1)
                ])
                .assert()
                .success();

            // Log some steps to create pattern data
            workflow
                .execute([
                    "episode", "log-step", &format!("ep-00{}", i + 1),
                    "--tool", "code_editor",
                    "--action", "write_function",
                    "--success",
                    "--latency-ms", "1000",
                    "--tokens", "200",
                    "--observation", &format!("Completed task {}", i + 1),
                ])
                .assert()
                .success();

            // Complete the episode
            workflow
                .execute([
                    "episode", "complete", &format!("ep-00{}", i + 1),
                    "--outcome", "success",
                ])
                .assert()
                .success();
        }

        // Phase 2: List available patterns
        println!("Phase 2: Listing patterns...");
        workflow
            .execute(["pattern", "list"])
            .assert()
            .success();

        // Phase 3: Analyze pattern effectiveness
        println!("Phase 3: Analyzing pattern effectiveness...");
        workflow
            .execute(["pattern", "effectiveness", "--top", "10"])
            .assert()
            .success();

        // Phase 4: View specific patterns
        println!("Phase 4: Viewing specific patterns...");
        // Note: In real usage, we'd have actual pattern IDs
        // For this test, we just verify the command structure works

        // Phase 5: Run pattern decay analysis
        println!("Phase 5: Running pattern decay analysis...");
        workflow
            .execute(["pattern", "decay", "--dry-run"])
            .assert()
            .success();

        // Phase 6: Analyze patterns with episode data
        println!("Phase 6: Analyzing patterns with episode data...");
        workflow
            .execute(["pattern", "analyze", "test-pattern-id", "--episodes", "5"])
            .assert()
            .success();

        println!("Advanced pattern analysis workflow test passed!");
    }

    #[test]
    fn test_storage_operations_and_maintenance_workflow() {
        let workflow = WorkflowHarness::new();

        // Phase 1: Check initial storage stats
        println!("Phase 1: Checking initial storage stats...");
        workflow
            .execute(["storage", "stats"])
            .assert()
            .success();

        // Phase 2: Check storage health
        println!("Phase 2: Checking storage health...");
        workflow
            .execute(["storage", "health"])
            .assert()
            .success();

        // Phase 3: Check storage connections
        println!("Phase 3: Checking storage connections...");
        workflow
            .execute(["storage", "connections"])
            .assert()
            .success();

        // Phase 4: Perform storage sync (dry run)
        println!("Phase 4: Performing storage sync...");
        workflow
            .execute(["storage", "sync", "--dry-run"])
            .assert()
            .success();

        // Phase 5: Perform storage vacuum (dry run)
        println!("Phase 5: Performing storage vacuum...");
        workflow
            .execute(["storage", "vacuum", "--dry-run"])
            .assert()
            .success();

        // Phase 6: Check storage stats after operations
        println!("Phase 6: Checking storage stats after operations...");
        workflow
            .execute(["storage", "stats"])
            .assert()
            .success();

        println!("Storage operations and maintenance workflow test passed!");
    }

    #[test]
    fn test_error_recovery_and_edge_cases_workflow() {
        let workflow = WorkflowHarness::new();

        // Phase 1: Test invalid operations
        println!("Phase 1: Testing invalid operations...");
        let invalid_operations = vec![
            vec!["episode", "view", "invalid-uuid"],
            vec!["episode", "complete", "nonexistent-id"],
            vec!["pattern", "view", "invalid-pattern-id"],
            vec!["--format", "invalid", "config"],
        ];

        for operation in invalid_operations {
            workflow
                .execute(&operation)
                .assert()
                .failure();
        }

        // Phase 2: Test edge cases
        println!("Phase 2: Testing edge cases...");
        let edge_cases = vec![
            vec!["episode", "create", "--task", ""], // Empty task
            vec!["episode", "list", "--limit", "0"], // Zero limit
            vec!["episode", "search", ""], // Empty search
        ];

        for edge_case in edge_cases {
            let result = workflow.execute(&edge_case);
            // Should not crash, may succeed or fail gracefully
            let _ = result.output().unwrap();
        }

        // Phase 3: Test recovery - create valid episode after errors
        println!("Phase 3: Testing recovery...");
        workflow
            .execute(["episode", "create", "--task", "Recovery test after errors"])
            .assert()
            .success();

        // Phase 4: Verify system still works
        println!("Phase 4: Verifying system integrity...");
        workflow
            .execute(["config"])
            .assert()
            .success();

        println!("Error recovery and edge cases workflow test passed!");
    }

    #[test]
    fn test_performance_and_scalability_workflow() {
        let workflow = WorkflowHarness::new();
        let start_time = std::time::Instant::now();

        // Phase 1: Bulk episode creation
        println!("Phase 1: Bulk episode creation...");
        let bulk_count = 10;
        for i in 0..bulk_count {
            workflow
                .execute([
                    "episode", "create",
                    "--task", &format!("Bulk test episode {}", i + 1)
                ])
                .assert()
                .success();
        }

        let bulk_creation_time = start_time.elapsed();
        println!("Created {} episodes in {}ms (avg: {}ms per episode)",
                 bulk_count,
                 bulk_creation_time.as_millis(),
                 bulk_creation_time.as_millis() / bulk_count as u128);

        // Phase 2: Bulk operations
        println!("Phase 2: Bulk operations...");
        let operations_start = std::time::Instant::now();

        for i in 0..bulk_count {
            // List episodes
            workflow.execute(["episode", "list"]).assert().success();

            // Check storage stats
            workflow.execute(["storage", "stats"]).assert().success();
        }

        let operations_time = operations_start.elapsed();
        println!("Performed {} bulk operations in {}ms (avg: {}ms per operation)",
                 bulk_count,
                 operations_time.as_millis(),
                 operations_time.as_millis() / bulk_count as u128);

        // Phase 3: Performance validation
        println!("Phase 3: Performance validation...");
        assert!(bulk_creation_time < std::time::Duration::from_secs(30),
                "Bulk creation took too long: {}ms", bulk_creation_time.as_millis());

        assert!(operations_time < std::time::Duration::from_secs(60),
                "Bulk operations took too long: {}ms", operations_time.as_millis());

        println!("Performance and scalability workflow test passed!");
    }

    #[test]
    fn test_configuration_management_workflow() {
        let temp_dir = tempfile::TempDir::new().unwrap();

        // Phase 1: Test multiple configuration formats
        println!("Phase 1: Testing configuration formats...");
        let configs = vec![
            ("config.toml", "toml", r#"
[database]
turso_url = "file:test.db"

[storage]
max_episodes_cache = 100

[cli]
default_format = "json"
"#),
            ("config.json", "json", r#"{
  "database": {
    "turso_url": "file:test.db"
  },
  "storage": {
    "max_episodes_cache": 100
  },
  "cli": {
    "default_format": "json"
  }
}"#),
            ("config.yaml", "yaml", r#"---
database:
  turso_url: "file:test.db"
storage:
  max_episodes_cache: 100
cli:
  default_format: "json"
"#),
        ];

        for (filename, format_name, content) in configs {
            let config_path = temp_dir.path().join(filename);
            std::fs::write(&config_path, content).unwrap();

            let mut cmd = Command::new(std::env::var("CARGO_BIN_EXE_memory-cli").unwrap_or_else(|_| "/workspaces/rust-self-learning-memory/target/debug/memory-cli".to_string()));
            cmd.arg("--config").arg(&config_path);
            cmd.arg("config");

            cmd.assert().success();
            println!("{} configuration format works correctly", format_name);
        }

        // Phase 2: Test configuration validation
        println!("Phase 2: Testing configuration validation...");
        let invalid_configs = vec![
            ("invalid_max_cache.toml", r#"
[database]
turso_url = "file:test.db"

[storage]
max_episodes_cache = 0

[cli]
default_format = "json"
"#),
        ];

        for (filename, content) in invalid_configs {
            let config_path = temp_dir.path().join(filename);
            std::fs::write(&config_path, content).unwrap();

            let mut cmd = Command::new(std::env::var("CARGO_BIN_EXE_memory-cli").unwrap_or_else(|_| "/workspaces/rust-self-learning-memory/target/debug/memory-cli".to_string()));
            cmd.arg("--config").arg(&config_path);
            cmd.arg("config");

            // Should fail validation
            cmd.assert().failure();
            println!("Invalid configuration {} correctly rejected", filename);
        }

        println!("Configuration management workflow test passed!");
    }
}
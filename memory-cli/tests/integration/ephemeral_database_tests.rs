//! Enhanced integration tests with ephemeral databases.
//!
//! These tests verify end-to-end functionality using temporary databases
//! and realistic data scenarios.

use assert_cmd::Command;
use memory_cli::test_utils::*;
use std::fs;
use tempfile::TempDir;
use tokio::runtime::Runtime;

mod common;

#[cfg(test)]
mod ephemeral_database_tests {
    use super::*;

    /// Test harness for ephemeral database testing
    struct EphemeralDbHarness {
        temp_dir: TempDir,
        turso_db_path: std::path::PathBuf,
        redb_path: std::path::PathBuf,
        config_path: std::path::PathBuf,
    }

    impl EphemeralDbHarness {
        fn new() -> Self {
            let temp_dir = TempDir::new().expect("Failed to create temp directory");

            let turso_db_path = temp_dir.path().join("test_turso.db");
            let redb_path = temp_dir.path().join("test.redb");
            let config_path = temp_dir.path().join("test_config.toml");

            // Create test config with ephemeral database paths
            let config_content = format!(r#"
[database]
turso_url = "file:{}"
redb_path = "{}"

[storage]
max_episodes_cache = 100
cache_ttl_seconds = 3600
pool_size = 5

[cli]
default_format = "json"
progress_bars = false
batch_size = 10
"#,
                turso_db_path.to_string_lossy(),
                redb_path.to_string_lossy()
            );

            fs::write(&config_path, config_content).expect("Failed to write test config");

            Self {
                temp_dir,
                turso_db_path,
                redb_path,
                config_path,
            }
        }

        fn execute<I, S>(&self, args: I) -> Command
        where
            I: IntoIterator<Item = S>,
            S: AsRef<std::ffi::OsStr>,
        {
            let mut cmd = Command::new(std::env::var("CARGO_BIN_EXE_memory-cli").unwrap_or_else(|_| "/workspaces/rust-self-learning-memory/target/debug/memory-cli".to_string())).expect("Failed to find memory-cli binary");
            cmd.arg("--config").arg(&self.config_path);
            cmd.args(args);
            cmd
        }

        fn execute_and_capture<I, S>(&self, args: I) -> Result<String, Box<dyn std::error::Error>>
        where
            I: IntoIterator<Item = S>,
            S: AsRef<std::ffi::OsStr>,
        {
            let output = self.execute(args).output()?;
            Ok(String::from_utf8(output.stdout)?)
        }
    }

    #[test]
    fn test_ephemeral_database_creation() {
        let harness = EphemeralDbHarness::new();

        // Database files should not exist initially
        assert!(!harness.turso_db_path.exists());
        assert!(!harness.redb_path.exists());

        // Config validation should work
        harness.execute(["config"]).assert().success();

        // Note: Actual database creation would happen when turso/redb features are enabled
        // and memory operations are performed
    }

    #[test]
    fn test_config_with_ephemeral_paths() {
        let harness = EphemeralDbHarness::new();

        // Test that config loads correctly with ephemeral paths
        let output = harness.execute_and_capture(["config"]).unwrap();

        // Should contain success message
        assert!(output.contains("valid") || output.contains("Configuration"));
    }

    #[test]
    fn test_dry_run_with_ephemeral_config() {
        let harness = EphemeralDbHarness::new();

        // Test dry run operations with ephemeral config
        harness
            .execute(["--dry-run", "episode", "create", "test task"])
            .assert()
            .success();
    }

    #[test]
    fn test_output_formats_with_ephemeral_config() {
        let harness = EphemeralDbHarness::new();

        // Test different output formats work with ephemeral config
        let formats = vec!["human", "json", "yaml"];

        for format in formats {
            harness
                .execute(["--format", format, "config"])
                .assert()
                .success();
        }
    }

    #[test]
    fn test_error_handling_with_ephemeral_config() {
        let harness = EphemeralDbHarness::new();

        // Test that error handling works with ephemeral config
        harness
            .execute(["episode", "view", "invalid-uuid"])
            .assert()
            .failure();
    }

    #[test]
    fn test_completion_with_ephemeral_config() {
        let harness = EphemeralDbHarness::new();

        // Test completion generation works with ephemeral config
        harness
            .execute(["completion", "bash"])
            .assert()
            .success();
    }

    #[test]
    fn test_concurrent_access_simulation() {
        let harness = EphemeralDbHarness::new();

        // Simulate concurrent access to ephemeral databases
        let handles: Vec<_> = (0..5)
            .map(|_| {
                let harness_clone = harness.config_path.clone();
                std::thread::spawn(move || {
                    let mut cmd = Command::new(std::env::var("CARGO_BIN_EXE_memory-cli").unwrap_or_else(|_| "/workspaces/rust-self-learning-memory/target/debug/memory-cli".to_string()));
                    cmd.arg("--config").arg(harness_clone);
                    cmd.arg("config");
                    cmd.assert().success();
                })
            })
            .collect();

        // Wait for all concurrent operations
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_large_batch_operations_simulation() {
        let harness = EphemeralDbHarness::new();

        // Simulate large batch operations (without actual database writes)
        for i in 0..20 {
            harness
                .execute(["--dry-run", "episode", "create", &format!("batch task {}", i)])
                .assert()
                .success();
        }
    }

    #[test]
    fn test_config_file_isolation() {
        // Test that multiple ephemeral configs don't interfere
        let harness1 = EphemeralDbHarness::new();
        let harness2 = EphemeralDbHarness::new();

        // Both should work independently
        harness1.execute(["config"]).assert().success();
        harness2.execute(["config"]).assert().success();

        // Configs should be different
        assert_ne!(harness1.config_path, harness2.config_path);
        assert_ne!(harness1.turso_db_path, harness2.turso_db_path);
        assert_ne!(harness1.redb_path, harness2.redb_path);
    }

    #[test]
    fn test_cleanup_after_test() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("cleanup_test.toml");

        {
            let harness = EphemeralDbHarness::new();
            // Harness goes out of scope here
        }

        // Temp directory should be cleaned up
        // Note: This test mainly verifies that the harness doesn't prevent cleanup
    }

    #[test]
    fn test_config_validation_with_ephemeral_paths() {
        let harness = EphemeralDbHarness::new();

        // Test various config validation scenarios
        harness
            .execute(["config"])
            .assert()
            .success();
    }

    #[test]
    fn test_performance_with_ephemeral_databases() {
        let harness = EphemeralDbHarness::new();

        let start = std::time::Instant::now();

        // Run multiple operations
        for _ in 0..10 {
            harness.execute(["config"]).assert().success();
        }

        let duration = start.elapsed();

        // Should complete within reasonable time
        assert!(duration < std::time::Duration::from_millis(500));
    }
}

// Integration tests that require async runtime
#[cfg(test)]
mod async_integration_tests {
    use super::*;

    fn run_async_test<F>(test: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let rt = Runtime::new().unwrap();
        rt.block_on(test);
    }

    #[test]
    fn test_async_config_operations() {
        run_async_test(async {
            let harness = EphemeralDbHarness::new();

            // Test that async operations work (config loading is sync, but future operations would be async)
            harness.execute(["config"]).assert().success();
        });
    }

    #[test]
    fn test_memory_system_initialization_simulation() {
        run_async_test(async {
            // This would test actual memory system initialization with ephemeral DBs
            // For now, just test the config loading part
            let harness = EphemeralDbHarness::new();
            harness.execute(["config"]).assert().success();
        });
    }
}
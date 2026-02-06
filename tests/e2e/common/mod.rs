//! E2E Test Utilities for CLI and MCP Integration
//!
//! This module provides shared utilities for end-to-end testing:
//! - CLI invocation helpers using `assert_cmd`
//! - MCP client setup and communication
//! - Temporary directory management for test isolation
//! - Common test data generators

#![allow(clippy::unwrap_used, clippy::expect_used)]

use anyhow::Result;
use memory_core::{SelfLearningMemory, TaskOutcome, TaskType};
use memory_storage_redb::RedbStorage;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use uuid::Uuid;

pub use super::common::*;

/// Test configuration for E2E tests
pub struct TestConfig {
    pub temp_dir: TempDir,
    pub cli_path: PathBuf,
    pub mcp_server_path: PathBuf,
}

impl TestConfig {
    /// Create a new test configuration with temporary directories
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;

        // Find CLI binary path
        let cli_path = find_binary("memory-cli")?;
        let mcp_server_path = find_binary("memory-mcp-server")?;

        Ok(Self {
            temp_dir,
            cli_path,
            mcp_server_path,
        })
    }

    /// Get the path to the temporary database directory
    pub fn db_dir(&self) -> PathBuf {
        self.temp_dir.path().join("db")
    }

    /// Get the path to the temporary config file
    pub fn config_path(&self) -> PathBuf {
        self.temp_dir.path().join("config.yaml")
    }

    /// Create a minimal config file for testing
    pub fn create_test_config(&self) -> Result<PathBuf> {
        let config_path = self.config_path();
        let db_dir = self.db_dir();
        std::fs::create_dir_all(&db_dir)?;

        let config_content = format!(
            r#"storage:
  turso_path: {}
  cache_path: {}
  enable_compression: true

cli:
  default_output_format: json
  verbose: false
"#,
            db_dir.join("turso.redb").display(),
            db_dir.join("cache.redb").display()
        );

        std::fs::write(&config_path, config_content)?;
        Ok(config_path)
    }
}

/// Find a binary in the target directory
fn find_binary(name: &str) -> Result<PathBuf> {
    // Try multiple locations
    let candidates = [
        PathBuf::from(format!("target/debug/{}", name)),
        PathBuf::from(format!("target/release/{}", name)),
        PathBuf::from(format!("../target/debug/{}", name)),
        PathBuf::from(format!("../target/release/{}", name)),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }

    // Try to find via cargo
    let output = Command::new("cargo")
        .args(["build", "--bin", name, "--message-format=short"])
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to build {}: {}",
            name,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Try again after building
    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }

    anyhow::bail!("Could not find or build binary: {}", name)
}

/// CLI command builder for testing
pub struct CliCommand {
    config: TestConfig,
    args: Vec<String>,
}

impl CliCommand {
    /// Create a new CLI command
    pub fn new(config: &TestConfig) -> Self {
        Self {
            config: TestConfig {
                temp_dir: tempfile::tempdir().expect("Failed to create temp dir"),
                cli_path: config.cli_path.clone(),
                mcp_server_path: config.mcp_server_path.clone(),
            },
            args: Vec::new(),
        }
    }

    /// Add an argument
    pub fn arg<S: AsRef<str>>(mut self, arg: S) -> Self {
        self.args.push(arg.as_ref().to_string());
        self
    }

    /// Add multiple arguments
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for arg in args {
            self.args.push(arg.as_ref().to_string());
        }
        self
    }

    /// Set the config file
    pub fn with_config(mut self, config_path: &PathBuf) -> Self {
        self.args
            .insert(0, format!("--config={}", config_path.display()));
        self
    }

    /// Set JSON output format
    pub fn json(mut self) -> Self {
        self.args.insert(0, "--format=json".to_string());
        self
    }

    /// Execute the command and return output
    pub fn execute(self) -> Result<CommandOutput> {
        let mut cmd = Command::new(&self.config.cli_path);
        cmd.args(&self.args);
        cmd.current_dir(self.config.temp_dir.path());

        let output = cmd.output()?;

        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            status: output.status,
        })
    }
}

/// Output from a command execution
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: std::process::ExitStatus,
}

impl CommandOutput {
    /// Check if the command succeeded
    pub fn success(&self) -> bool {
        self.status.success()
    }

    /// Parse stdout as JSON
    pub fn json(&self) -> Result<serde_json::Value> {
        Ok(serde_json::from_str(&self.stdout)?)
    }

    /// Get the exit code
    pub fn exit_code(&self) -> Option<i32> {
        self.status.code()
    }
}

/// Helper to create a memory instance with storage for integration tests
pub async fn setup_test_memory() -> Result<(Arc<SelfLearningMemory>, TempDir)> {
    let dir = TempDir::new()?;
    let turso_path = dir.path().join("test_turso.redb");
    let cache_path = dir.path().join("test_cache.redb");

    let turso_storage = RedbStorage::new(&turso_path)
        .await
        .expect("Failed to create turso storage");
    let cache_storage = RedbStorage::new(&cache_path)
        .await
        .expect("Failed to create cache storage");

    let memory = Arc::new(SelfLearningMemory::with_storage(
        Default::default(),
        Arc::new(turso_storage),
        Arc::new(cache_storage),
    ));

    Ok((memory, dir))
}

/// Helper to create a completed episode
pub async fn create_completed_episode(
    memory: &Arc<SelfLearningMemory>,
    description: &str,
    domain: &str,
    task_type: TaskType,
) -> Result<Uuid> {
    let id = memory
        .create_episode(description.to_string(), domain.to_string(), task_type)
        .await?;

    memory
        .complete_episode(
            id,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await?;

    Ok(id)
}

/// Wait for a condition with timeout
pub async fn wait_for<F, Fut>(mut condition: F, timeout_ms: u64) -> Result<()>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = Instant::now();
    let timeout = Duration::from_millis(timeout_ms);

    while start.elapsed() < timeout {
        if condition().await {
            return Ok(());
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    anyhow::bail!("Timeout waiting for condition")
}

/// Generate test episode data
pub fn generate_test_episodes(count: usize) -> Vec<TestEpisodeData> {
    (0..count)
        .map(|i| TestEpisodeData {
            description: format!("Test episode {}", i),
            domain: format!("domain-{}", i % 3),
            task_type: match i % 4 {
                0 => TaskType::CodeGeneration,
                1 => TaskType::Testing,
                2 => TaskType::Debugging,
                _ => TaskType::Analysis,
            },
        })
        .collect()
}

/// Test episode data structure
pub struct TestEpisodeData {
    pub description: String,
    pub domain: String,
    pub task_type: TaskType,
}

/// MCP Server process handle for integration tests
pub struct McpServerProcess {
    pub process: std::process::Child,
    pub stdin: std::process::ChildStdin,
    pub stdout: std::process::ChildStdout,
}

impl McpServerProcess {
    /// Start the MCP server process
    pub fn start(config: &TestConfig) -> Result<Self> {
        let mut cmd = Command::new(&config.mcp_server_path);
        cmd.current_dir(config.temp_dir.path())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut process = cmd.spawn()?;
        let stdin = process.stdin.take().expect("Failed to get stdin");
        let stdout = process.stdout.take().expect("Failed to get stdout");

        Ok(Self {
            process,
            stdin,
            stdout,
        })
    }

    /// Send a JSON-RPC request
    pub async fn send_request(&mut self, request: &serde_json::Value) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        let request_str = serde_json::to_string(request)?;
        let stdin = tokio::process::ChildStdin::from_std(self.stdin.try_clone()?)?;
        let mut stdin = stdin;
        stdin.write_all(request_str.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;

        Ok(())
    }

    /// Read a JSON-RPC response
    pub async fn read_response(&mut self) -> Result<serde_json::Value> {
        use tokio::io::{AsyncBufReadExt, BufReader};

        let stdout = tokio::process::ChildStdout::from_std(self.stdout.try_clone()?)?;
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        Ok(serde_json::from_str(&line)?)
    }

    /// Stop the server
    pub fn stop(mut self) -> Result<()> {
        self.process.kill()?;
        self.process.wait()?;
        Ok(())
    }
}

/// Performance measurement helper
pub struct PerformanceTimer {
    name: String,
    start: Instant,
}

impl PerformanceTimer {
    /// Start a new timer
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
        }
    }

    /// Stop the timer and print results
    pub fn stop(self) -> Duration {
        let duration = self.start.elapsed();
        println!("⏱️  {}: {:?}", self.name, duration);
        duration
    }
}

/// Test assertion helpers
pub mod assertions {
    use super::*;

    /// Assert that a JSON value contains a specific key
    pub fn assert_json_has_key(json: &serde_json::Value, key: &str) {
        assert!(
            json.get(key).is_some(),
            "Expected JSON to have key '{}' but got: {}",
            key,
            json
        );
    }

    /// Assert that a JSON array has a specific length
    pub fn assert_json_array_len(json: &serde_json::Value, expected_len: usize) {
        let arr = json.as_array().expect("Expected JSON array");
        assert_eq!(
            arr.len(),
            expected_len,
            "Expected array length {} but got {}",
            expected_len,
            arr.len()
        );
    }

    /// Assert that a command succeeded
    pub fn assert_command_success(output: &CommandOutput) {
        assert!(
            output.success(),
            "Command failed with exit code {:?}. stderr: {}",
            output.exit_code(),
            output.stderr
        );
    }

    /// Assert that a command failed
    pub fn assert_command_failed(output: &CommandOutput) {
        assert!(
            !output.success(),
            "Expected command to fail but it succeeded. stdout: {}",
            output.stdout
        );
    }
}

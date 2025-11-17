//! # CLI Test Utilities
//!
//! Shared utilities for testing the memory-cli crate.
//!
//! Provides:
//! - CLI command execution helpers
//! - Output validation utilities
//! - Security test harnesses

use assert_cmd::Command;
use std::path::PathBuf;
use tempfile::TempDir;

/// CLI test harness for executing commands programmatically
pub struct CliHarness {
    temp_dir: TempDir,
    config_path: PathBuf,
}

impl CliHarness {
    /// Create a new CLI test harness with temporary directory and config
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("test-config.toml");

        // Create a basic test config
        let config_content = r#"
[database]
turso_url = "file:test.db"
turso_token = "test-token"
redb_path = "test.redb"

[storage]
max_episodes_cache = 100
cache_ttl_seconds = 3600
pool_size = 5

[cli]
default_format = "json"
progress_bars = false
batch_size = 10
"#;

        std::fs::write(&config_path, config_content).expect("Failed to write test config");

        Self {
            temp_dir,
            config_path,
        }
    }

    /// Execute a CLI command with the given arguments
    pub fn execute<I, S>(&self, args: I) -> Command
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        // Use assert_cmd's cargo_bin to find the binary correctly
        #[allow(deprecated)]
        let mut cmd = Command::cargo_bin("memory-cli").expect("Failed to find memory-cli binary");
        cmd.arg("--config").arg(&self.config_path);
        cmd.args(args);
        cmd
    }

    /// Execute a CLI command and capture output as string
    pub fn execute_and_capture<I, S>(&self, args: I) -> std::result::Result<String, Box<dyn std::error::Error>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        let output = self.execute(args).output()?;
        Ok(String::from_utf8(output.stdout)?)
    }

    /// Get the temporary directory path
    pub fn temp_dir(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    /// Get the config file path
    pub fn config_path(&self) -> &std::path::Path {
        &self.config_path
    }
}

impl Default for CliHarness {
    fn default() -> Self {
        Self::new()
    }
}

/// Output validation utilities
pub mod validators {
    /// Validate JSON output format
    pub fn validate_json_output(output: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
        serde_json::from_str::<serde_json::Value>(output)?;
        Ok(())
    }

    /// Validate YAML output format
    pub fn validate_yaml_output(output: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
        serde_yaml::from_str::<serde_yaml::Value>(output)?;
        Ok(())
    }

    /// Validate human-readable output contains expected content
    pub fn validate_human_output_contains(output: &str, expected: &str) -> bool {
        output.contains(expected)
    }
}

/// Security test utilities
pub mod security {
    /// Test input sanitization
    pub fn test_input_sanitization(input: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
        // Test for common injection patterns
        let dangerous_patterns = [
            ";",
            "&&",
            "||",
            "`",
            "$(",
            "${",
            "<",
            ">",
            "|",
            "../",
            "..\\",
        ];

        for pattern in &dangerous_patterns {
            if input.contains(pattern) {
                return Err(format!("Potentially dangerous input detected: {}", pattern).into());
            }
        }

        Ok(())
    }

    /// Generate malicious test inputs
    pub fn generate_malicious_inputs() -> Vec<String> {
        vec![
            "test; rm -rf /".to_string(),
            "test && echo 'hacked'".to_string(),
            "test || true".to_string(),
            "test `whoami`".to_string(),
            "test $(pwd)".to_string(),
            "test ${USER}".to_string(),
            "test < /etc/passwd".to_string(),
            "test > /dev/null".to_string(),
            "test | cat".to_string(),
            "../../../etc/passwd".to_string(),
            "..\\..\\..\\windows\\system32".to_string(),
        ]
    }

    /// Test path traversal protection
    pub fn test_path_traversal_protection(path: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if path.contains("..") || path.starts_with("/") || path.starts_with("\\") || (path.len() >= 3 && path.chars().nth(1) == Some(':') && path.chars().nth(2) == Some('\\')) {
            return Err("Path traversal or absolute path detected".into());
        }
        Ok(())
    }
}

/// Performance testing utilities
pub mod performance {
    use super::*;
    use std::time::{Duration, Instant};

    /// Measure execution time of a CLI command
    pub fn measure_cli_execution<I, S>(
        harness: &CliHarness,
        args: I,
        iterations: usize,
    ) -> std::result::Result<Duration, Box<dyn std::error::Error>>
    where
        I: IntoIterator<Item = S> + Clone,
        S: AsRef<std::ffi::OsStr>,
    {
        let mut total_duration = Duration::new(0, 0);

        for _ in 0..iterations {
            let start = Instant::now();
            let _output = harness.execute(args.clone()).output()?;
            total_duration += start.elapsed();
        }

        Ok(total_duration / iterations as u32)
    }

    /// Validate performance is within acceptable bounds
    pub fn validate_performance(duration: Duration, max_duration: Duration) -> bool {
        duration <= max_duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_harness_creation() {
        let harness = CliHarness::new();
        assert!(harness.temp_dir().exists());
        assert!(harness.config_path().exists());
    }

    #[test]
    fn test_json_validation() {
        let json_output = r#"{"test": "value"}"#;
        assert!(validators::validate_json_output(json_output).is_ok());
    }

    #[test]
    fn test_yaml_validation() {
        let yaml_output = "test: value\n";
        assert!(validators::validate_yaml_output(yaml_output).is_ok());
    }

    #[test]
    fn test_input_sanitization() {
        assert!(security::test_input_sanitization("safe input").is_ok());
        assert!(security::test_input_sanitization("unsafe; input").is_err());
    }

    #[test]
    fn test_path_traversal_protection() {
        assert!(security::test_path_traversal_protection("safe/path").is_ok());
        assert!(security::test_path_traversal_protection("../unsafe").is_err());
    }
}
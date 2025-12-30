//! End-to-end tests for CLI embedding commands
//!
//! These tests verify the CLI embedding commands work correctly
//! with various embedding providers and configurations.

#[cfg(test)]
mod tests {
    use std::process::Command;
    use assert_cmd::prelude::*;
    use predicates::prelude::*;

    /// Helper to get the CLI binary path
    fn cli_binary() -> Command {
        Command::new(env!("CARGO_BIN_EXE_memory-cli"))
    }

    #[test]
    fn test_embedding_list_providers() {
        // Test that the list-providers command outputs provider information
        let mut cmd = cli_binary();
        cmd.arg("embedding")
           .arg("list-providers");

        cmd.assert()
            .success()
            .stdout(predicate::str::contains("Available Embedding Providers"))
            .stdout(predicate::str::contains("Local Provider"))
            .stdout(predicate::str::contains("OpenAI Provider"));
    }

    #[test]
    fn test_embedding_config_disabled() {
        // Test that config shows disabled state when embeddings are not enabled
        let mut cmd = cli_binary();
        cmd.arg("embedding")
           .arg("config");

        // Should show that embeddings are disabled
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("Status:"));
    }

    #[test]
    fn test_embedding_enable_disabled_commands() {
        // Test that embedding enable/disable commands work
        let mut cmd = cli_binary();
        cmd.arg("embedding")
           .arg("enable");

        // Should succeed (enable is session-based, no config change)
        cmd.assert().success();

        // Test disable
        let mut cmd = cli_binary();
        cmd.arg("embedding")
           .arg("disable");

        cmd.assert().success();
    }

    #[test]
    fn test_embedding_test_requires_config() {
        // Test that embedding test shows helpful message when not configured
        let mut cmd = cli_binary();
        cmd.arg("embedding")
           .arg("test");

        // Should succeed but show that embeddings are disabled
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("Embeddings are disabled"));
    }

    #[test]
    fn test_embedding_benchmark_requires_enabled() {
        // Test that benchmark fails gracefully when embeddings disabled
        let mut cmd = cli_binary();
        cmd.arg("embedding")
           .arg("benchmark");

        // Should fail with helpful error about embeddings being disabled
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("disabled"));
    }

    #[test]
    fn test_embedding_help() {
        // Test that embedding subcommand help works
        let mut cmd = cli_binary();
        cmd.arg("embedding")
           .arg("--help");

        cmd.assert()
            .success()
            .stdout(predicate::str::contains("Test"))
            .stdout(predicate::str::contains("Config"))
            .stdout(predicate::str::contains("ListProviders"))
            .stdout(predicate::str::contains("Benchmark"));
    }

    #[test]
    fn test_embedding_command_help() {
        // Test individual command help
        let commands = vec!["test", "config", "list-providers", "benchmark", "enable", "disable"];

        for cmd_name in commands {
            let mut cmd = cli_binary();
            cmd.arg("embedding")
               .arg(cmd_name)
               .arg("--help");

            cmd.assert().success();
        }
    }
}

/// Integration tests with mocked environment
#[cfg(test)]
mod integration_with_env {
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use assert_cmd::prelude::*;
    use predicates::prelude::*;

    /// Get the CLI binary path
    fn cli_binary() -> Command {
        Command::new(env!("CARGO_BIN_EXE_memory-cli"))
    }

    /// Create a test config file
    fn create_test_config(temp_dir: &PathBuf, embeddings_enabled: bool) -> PathBuf {
        let config_content = format!(r#"
[memory]
data_dir = "{}"

[embeddings]
enabled = {}
provider = "local"
model = "test-model"
dimension = 384
similarity_threshold = 0.7
"#,
            temp_dir.join("data").to_string_lossy(),
            embeddings_enabled
        );

        let config_path = temp_dir.join("test-config.toml");
        fs::write(&config_path, config_content).unwrap();
        config_path
    }

    #[test]
    fn test_embedding_config_with_disabled_embeddings() {
        // Test that config shows correct state when embeddings are disabled
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_test_config(&temp_dir.path().to_path_buf(), false);

        let mut cmd = cli_binary();
        cmd.arg("--config")
           .arg(&config_path)
           .arg("embedding")
           .arg("config");

        // Should show disabled status
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("Disabled"));
    }

    #[test]
    fn test_embedding_list_providers_with_config() {
        // Test that list-providers works with custom config
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_test_config(&temp_dir.path().to_path_buf(), false);

        let mut cmd = cli_binary();
        cmd.arg("--config")
           .arg(&config_path)
           .arg("embedding")
           .arg("list-providers");

        cmd.assert()
            .success()
            .stdout(predicate::str::contains("Local Provider"))
            .stdout(predicate::str::contains("OpenAI Provider"))
            .stdout(predicate::str::contains("Mistral Provider"));
    }
}

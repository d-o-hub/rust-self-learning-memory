//! Configuration file security and isolation tests.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_config_file_security() {
        let temp_dir = TempDir::new().unwrap();

        // Test that config files can't contain malicious content
        let malicious_configs = [
            r#"
[database]
turso_url = "file:test.db"
turso_token = "; rm -rf /"

[storage]
max_episodes_cache = 100
"#,
            r#"
[cli]
default_format = "human; echo 'injected'"
"#,
        ];

        for config_content in &malicious_configs {
            let config_path = temp_dir.path().join("malicious.toml");
            std::fs::write(&config_path, config_content).unwrap();

            #[allow(deprecated)]
            let mut cmd =
                Command::cargo_bin("do-memory-cli").expect("Failed to find memory-cli binary");
            cmd.arg("--config").arg(&config_path);
            cmd.arg("config");

            // Should fail validation or be safe
            cmd.assert().failure();
        }
    }

    #[test]
    fn test_config_isolation() {
        let temp_dir = TempDir::new().unwrap();

        // Use a safe redb path within temp directory
        let redb_path = temp_dir.path().join("test.redb");
        let redb_str = redb_path.display().to_string().replace('\\', "/");

        // Create a config that tries to access sensitive paths
        let sensitive_config = format!(
            r#"
[database]
turso_url = "file:/etc/passwd"
turso_token = "sensitive"
redb_path = "{}"

[storage]
max_episodes_cache = 100
cache_ttl_seconds = 3600
pool_size = 5

[cli]
default_format = "human"
progress_bars = false
batch_size = 10
"#,
            redb_str
        );

        let config_path = temp_dir.path().join("sensitive.toml");
        std::fs::write(&config_path, &sensitive_config).unwrap();

        #[allow(deprecated)]
        let mut cmd =
            Command::cargo_bin("do-memory-cli").expect("Failed to find memory-cli binary");
        cmd.arg("--config").arg(&config_path);
        cmd.args(["episode", "list"]);

        // Should fail safely when trying to access sensitive paths
        // The config itself is valid TOML, but connecting to the database should fail
        cmd.assert().failure().stderr(
            predicate::str::contains("Failed to connect to database")
                .or(predicate::str::contains("SQLite failure"))
                .or(predicate::str::contains("Storage error"))
                .or(predicate::str::contains(
                    "Turso database URL not configured",
                ))
                .or(predicate::str::contains("Failed to open database"))
                .or(predicate::str::contains(
                    "Turso storage feature not enabled",
                )),
        );
    }

    #[test]
    fn test_configuration_file_security() {
        let temp_dir = TempDir::new().unwrap();

        // Create safe redb path for all test configs
        let redb_path = temp_dir.path().join("test.redb");
        let redb_str = redb_path.display().to_string().replace('\\', "/");

        // Test that config files can't be used for attacks
        let large_token = "x".repeat(1024 * 1024);
        let large_config_base = format!(
            r#"
[database]
turso_url = "file:test.db"
turso_token = "{}"
redb_path = "{}"

[storage]
max_episodes_cache = 100
"#,
            large_token, redb_str
        );

        let large_config_complete = format!(
            "{}\n{}",
            large_config_base,
            r#"
cache_ttl_seconds = 3600
pool_size = 5

[cli]
default_format = "human"
progress_bars = false
batch_size = 10
"#
        );

        let command_injection_config = format!(
            r#"
[database]
turso_url = "file:test.db"
turso_token = "; rm -rf /"
redb_path = "{}"

[storage]
max_episodes_cache = 100
cache_ttl_seconds = 3600
pool_size = 5

[cli]
default_format = "human"
progress_bars = false
batch_size = 10
"#,
            redb_str
        );

        let path_traversal_config = format!(
            r#"
[database]
turso_url = "file:../../../etc/passwd"
redb_path = "{}"

[storage]
max_episodes_cache = 100
cache_ttl_seconds = 3600
pool_size = 5

[cli]
default_format = "human"
progress_bars = false
batch_size = 10
"#,
            redb_str
        );

        let malicious_configs = vec![
            // Config with command injection in values
            (command_injection_config.as_str(), "command_injection"),
            // Config with path traversal
            (path_traversal_config.as_str(), "path_traversal"),
            // Config with extremely large values (DoS)
            (large_config_complete.as_str(), "large_values"),
        ];

        for (config_content, attack_type) in malicious_configs {
            let config_path = temp_dir
                .path()
                .join(format!("malicious_{}.toml", attack_type));
            std::fs::write(&config_path, config_content).unwrap();

            #[allow(deprecated)]
            let mut cmd =
                Command::cargo_bin("do-memory-cli").expect("Failed to find memory-cli binary");
            cmd.arg("--config").arg(&config_path);
            cmd.args(["episode", "list"]);

            // Should handle malicious configs safely
            let result = cmd.assert();

            // Should not crash or execute dangerous operations
            match attack_type {
                "large_values" | "command_injection" => {
                    // Large configs and command injection might succeed (values are just strings)
                    // But episode list might fail if it tries to use them
                    let output = result.get_output();
                    if output.status.success() {
                        // If it succeeds, check it didn't execute the command
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        assert!(
                            !stdout.contains("root") && !stdout.contains("bin"),
                            "Command injection executed"
                        );
                    }
                }
                "path_traversal" => {
                    // Path traversal should fail when trying to access the file
                    result.failure().stderr(
                        predicate::str::contains("Failed to connect to database")
                            .or(predicate::str::contains("SQLite failure"))
                            .or(predicate::str::contains("Storage error"))
                            .or(predicate::str::contains("Failed to open database"))
                            .or(predicate::str::contains(
                                "Turso storage feature not enabled",
                            )),
                    );
                }
                _ => {
                    // Other attacks should be handled gracefully
                    let output = result.get_output();
                    assert!(
                        output.status.code().is_some(),
                        "{} config caused crash",
                        attack_type
                    );
                }
            }

            println!("Malicious config type '{}' handled safely", attack_type);
        }
    }
}

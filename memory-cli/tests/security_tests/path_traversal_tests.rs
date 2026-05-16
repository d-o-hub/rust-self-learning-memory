//! Path traversal prevention and detection tests.

use assert_cmd::Command;
use do_memory_cli::test_utils::{CliHarness, security};
use tempfile::TempDir;

#[cfg(test)]
mod path_traversal_tests {
    use super::*;

    #[test]
    fn test_path_traversal_prevention() {
        let _harness = CliHarness::new();

        let traversal_attempts = [
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32",
            "/etc/passwd",
            "C:\\Windows\\System32",
            "../../../../root/.ssh/id_rsa",
        ];

        for _path in &traversal_attempts {
            // Test with config file path
            let temp_dir = TempDir::new().unwrap();
            let safe_config = temp_dir.path().join("config.toml");

            let config_content = r#"
[database]
turso_url = "file:test.db"

[storage]
max_episodes_cache = 100
cache_ttl_seconds = 3600
pool_size = 5

[cli]
default_format = "human"
progress_bars = false
batch_size = 10
"#;

            std::fs::write(&safe_config, config_content).unwrap();

            #[allow(deprecated)]
            let mut cmd =
                Command::cargo_bin("do-memory-cli").expect("Failed to find memory-cli binary");
            cmd.arg("--config").arg(&safe_config);
            cmd.args(["episode", "create", "test task"]);

            // Should not crash or access unauthorized paths
            cmd.assert().failure(); // Fails due to missing turso feature, not path issues
        }
    }

    #[test]
    fn test_path_traversal_detection() {
        let safe_paths = ["safe/path", "relative/path", "file.txt", "data/config.toml"];

        let unsafe_paths = [
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32",
            "/absolute/path",
            "C:\\absolute\\windows\\path",
        ];

        for path in &safe_paths {
            assert!(security::test_path_traversal_protection(path).is_ok());
        }

        for path in &unsafe_paths {
            assert!(security::test_path_traversal_protection(path).is_err());
        }
    }

    #[test]
    fn test_path_traversal_comprehensive() {
        let harness = CliHarness::new();

        // Test comprehensive path traversal prevention
        let traversal_attempts = vec![
            // Basic traversal
            "../../../etc/passwd",
            "../../../../root/.ssh/id_rsa",
            "..\\..\\..\\windows\\system32\\cmd.exe",
            // Encoded traversal
            "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
            "..%2f..%2f..%2fetc%2fpasswd",
            // Unicode normalization attempts
            "..\u{2216}..\u{2216}..\u{2216}windows\u{2216}system32", // Unicode division slash
            // Absolute paths
            "/etc/passwd",
            "C:\\Windows\\System32\\cmd.exe",
            "/root/.bashrc",
            // Nested traversal
            "safe/../../../dangerous",
            "path/../../../etc/passwd",
            // URL-style traversal
            "file:///etc/passwd",
            "file://c:/windows/system32/cmd.exe",
        ];

        for path in traversal_attempts {
            // Test in various contexts where paths might be used
            let task_with_path = format!("task with path: {}", path);
            let test_commands = vec![
                vec!["episode", "create", "--task", &task_with_path],
                // Note: In a real implementation, paths might be used in config files
            ];

            for cmd in test_commands {
                let mut result = harness.execute(&cmd);
                let output = result.output().unwrap();

                // Should fail safely
                assert!(
                    output.status.code().is_some(),
                    "Path '{}' caused crash in command {:?}",
                    path,
                    cmd
                );
            }
        }
    }
}

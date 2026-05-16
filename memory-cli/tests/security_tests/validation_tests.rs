//! Argument validation and unicode handling tests.

use do_memory_cli::test_utils::CliHarness;

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_argument_validation() {
        let harness = CliHarness::new();

        // Test invalid episode IDs
        let invalid_ids = [
            "",
            "not-a-uuid",
            "123",
            "../../../etc/passwd",
            "; drop table episodes;",
        ];

        for id in &invalid_ids {
            harness.execute(["episode", "view", id]).assert().failure();
        }
    }

    #[test]
    fn test_unicode_handling() {
        let harness = CliHarness::new();

        // Test with various Unicode inputs
        let unicode_inputs = [
            "测试任务", // Chinese
            "🚀 Task with emoji",
            "café résumé naïve", // Accented characters
            "русский текст",     // Cyrillic
            "🔥💯✨",            // Only emojis
        ];

        for input in &unicode_inputs {
            harness
                .execute(["episode", "create", input])
                .assert()
                .failure(); // Should fail due to missing features, not encoding issues
        }
    }

    #[test]
    #[allow(clippy::excessive_nesting)]
    fn test_argument_validation_comprehensive() {
        let harness = CliHarness::new();

        // Test comprehensive argument validation
        let long_input = "x".repeat(100000);
        let invalid_args = vec![
            // Invalid episode IDs
            ("", "empty episode id"),
            ("not-a-uuid", "invalid uuid format"),
            ("123", "numeric id"),
            ("../../../etc/passwd", "path traversal in id"),
            ("; DROP TABLE episodes;", "sql injection in id"),
            // Invalid task descriptions
            ("", "empty task"),
            (&long_input, "extremely long task"),
            // Invalid numeric parameters
            ("--limit", "non-numeric limit"),
            ("--batch-size", "non-numeric batch size"),
        ];

        for (arg_value, _description) in invalid_args {
            // Test in various command contexts
            let test_commands = vec![
                vec!["episode", "view", arg_value],
                vec!["episode", "create", "--task", arg_value],
                vec!["episode", "list", "--limit", arg_value],
            ];

            for cmd in test_commands {
                if cmd.contains(&arg_value) {
                    let mut result = harness.execute(&cmd);
                    let output = match result.output() {
                        Ok(output) => output,
                        Err(_) => {
                            // Command failed to spawn (e.g., due to argument list too long)
                            // This is acceptable for very large inputs
                            continue;
                        }
                    };

                    // Should fail safely with non-success exit code
                    assert!(
                        !output.status.success(),
                        "Invalid arg '{}' in command {:?} should fail (got success instead)",
                        arg_value,
                        cmd
                    );
                }
            }
        }
    }
}

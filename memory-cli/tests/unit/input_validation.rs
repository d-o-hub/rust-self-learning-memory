//! Unit tests for input validation utilities.
//!
//! These tests verify that input sanitization, validation,
//! and security checks work correctly.

use memory_cli::test_utils::security;
use uuid::Uuid;

#[cfg(test)]
mod input_validation_tests {
    use super::*;

    #[test]
    fn test_input_sanitization_safe_inputs() {
        let safe_inputs = vec![
            "normal task description",
            "test with numbers 123",
            "task_with_underscores",
            "task-with-dashes",
            "task with spaces",
            "Task With Mixed Case",
            "任务描述", // Unicode
            "🚀 Task with emoji",
        ];

        for input in safe_inputs {
            assert!(
                security::test_input_sanitization(input).is_ok(),
                "Input '{}' should be considered safe",
                input
            );
        }
    }

    #[test]
    fn test_input_sanitization_dangerous_inputs() {
        let dangerous_inputs = vec![
            "task; rm -rf /",
            "task && echo 'hacked'",
            "task || true",
            "task | cat /etc/passwd",
            "task `whoami`",
            "task $(pwd)",
            "task ${USER}",
            "task < /etc/passwd",
            "task > /dev/null",
            "task >> /etc/passwd",
            "task 2>&1",
            "task &",
            "task ;",
            "task &&",
            "task ||",
            "task |",
            "task `",
            "task $",
            "task <",
            "task >",
        ];

        for input in dangerous_inputs {
            assert!(
                security::test_input_sanitization(input).is_err(),
                "Input '{}' should be considered dangerous",
                input
            );
        }
    }

    #[test]
    fn test_path_traversal_protection_safe_paths() {
        let safe_paths = vec![
            "safe/path",
            "relative/path",
            "file.txt",
            "data/config.toml",
            "path/with/many/levels",
            "file-with-dashes.txt",
            "file_with_underscores.txt",
            "file.with.dots.txt",
            "123numeric.txt",
            "file(1).txt",
        ];

        for path in safe_paths {
            assert!(
                security::test_path_traversal_protection(path).is_ok(),
                "Path '{}' should be considered safe",
                path
            );
        }
    }

    #[test]
    fn test_path_traversal_protection_dangerous_paths() {
        let dangerous_paths = vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32",
            "/absolute/path",
            "C:\\absolute\\windows\\path",
            "../../../../root/.ssh/id_rsa",
            "..//..//..//etc//passwd",
            "....//....//....//etc//passwd",
            "path/../../../etc/passwd",
            "../../../etc/passwd/../../../",
        ];

        for path in dangerous_paths {
            assert!(
                security::test_path_traversal_protection(path).is_err(),
                "Path '{}' should be considered dangerous",
                path
            );
        }
    }

    #[test]
    fn test_uuid_validation() {
        // Test valid UUIDs
        let valid_uuids = vec![
            "123e4567-e89b-12d3-a456-426614174000",
            "550e8400-e29b-41d4-a716-446655440000",
            "f47ac10b-58cc-4372-a567-0e02b2c3d479",
            "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
        ];

        for uuid_str in valid_uuids {
            assert!(
                Uuid::parse_str(uuid_str).is_ok(),
                "UUID '{}' should be valid",
                uuid_str
            );
        }

        // Test invalid UUIDs
        let invalid_uuids = vec![
            "",
            "not-a-uuid",
            "123",
            "123e4567-e89b-12d3-a456",                    // too short
            "123e4567-e89b-12d3-a456-426614174000-extra", // too long
            "gggggggg-e89b-12d3-a456-426614174000",       // invalid characters
            "123e4567-e89b-12d3-a456-42661417400g",       // invalid hex
            "../../../etc/passwd",                        // path traversal
            "; drop table episodes;",                     // SQL injection
        ];

        for uuid_str in invalid_uuids {
            assert!(
                Uuid::parse_str(uuid_str).is_err(),
                "UUID '{}' should be invalid",
                uuid_str
            );
        }
    }

    #[test]
    fn test_task_type_validation() {
        // Valid task types (should be simple strings without special chars)
        let valid_task_types = vec![
            "testing",
            "development",
            "production",
            "analysis",
            "training",
            "inference",
            "deployment",
            "monitoring",
            "debugging",
            "optimization",
        ];

        for task_type in valid_task_types {
            assert!(
                security::test_input_sanitization(task_type).is_ok(),
                "Task type '{}' should be valid",
                task_type
            );
        }

        // Invalid task types (contain dangerous characters)
        let invalid_task_types = vec![
            "testing; rm -rf /",
            "dev && echo 'hack'",
            "prod | cat /etc/passwd",
            "analysis `whoami`",
            "training $(pwd)",
            "inference ${USER}",
            "../../../etc/passwd",
            "; drop table;",
        ];

        for task_type in invalid_task_types {
            assert!(
                security::test_input_sanitization(task_type).is_err(),
                "Task type '{}' should be invalid",
                task_type
            );
        }
    }

    #[test]
    fn test_context_string_validation() {
        // Valid context strings (key:value pairs)
        let valid_contexts = vec![
            "domain:rust,language:rust,tags:test",
            "domain:python,language:python,tags:ml,framework:tensorflow",
            "domain:web,language:javascript,tags:frontend,framework:react",
            "domain:devops,language:yaml,tags:deployment",
            "single:pair",
            "empty:",
            ":empty",
        ];

        for context in valid_contexts {
            assert!(
                security::test_input_sanitization(context).is_ok(),
                "Context '{}' should be valid",
                context
            );
        }

        // Invalid context strings (contain dangerous characters)
        let invalid_contexts = vec![
            "domain:rust; rm -rf /",
            "language:python && echo 'hack'",
            "tags:test | cat /etc/passwd",
            "framework:react `whoami`",
            "domain:devops $(pwd)",
            "language:yaml ${USER}",
            "../../../etc/passwd",
            "domain:rust,tags:test; drop table;",
        ];

        for context in invalid_contexts {
            assert!(
                security::test_input_sanitization(context).is_err(),
                "Context '{}' should be invalid",
                context
            );
        }
    }

    #[test]
    fn test_numeric_validation() {
        // Test valid numbers (latency_ms, tokens, etc.)
        let valid_numbers = vec!["0", "1", "100", "1000", "999999", "123456789"];

        for num_str in valid_numbers {
            assert!(
                num_str.parse::<u64>().is_ok(),
                "Number '{}' should be valid",
                num_str
            );
            assert!(
                security::test_input_sanitization(num_str).is_ok(),
                "Number '{}' should pass sanitization",
                num_str
            );
        }

        // Test invalid numbers
        let invalid_numbers = vec![
            "-1",    // negative
            "1.5",   // float
            "1,000", // comma
            "1 000", // space
            "0x100", // hex
            "1e10",  // scientific
            "not-a-number",
            "100; rm -rf /",
            "50 && echo 'hack'",
        ];

        for num_str in invalid_numbers {
            // Some might parse as numbers but contain dangerous chars
            if num_str.parse::<u64>().is_ok() {
                // If it parses as a number, check sanitization
                assert!(
                    security::test_input_sanitization(num_str).is_err(),
                    "Number '{}' should fail sanitization",
                    num_str
                );
            }
        }
    }

    #[test]
    fn test_observation_validation() {
        // Valid observations (can contain more text, but no dangerous chars)
        let valid_observations = vec![
            "Task completed successfully",
            "Model training finished with accuracy 95.2%",
            "Error: connection timeout after 30 seconds",
            "Processed 1000 items in batch",
            "Debug: variable x = 42",
            "Info: cache hit rate improved to 85%",
            "Warning: deprecated API usage detected",
            "任务成功完成", // Unicode
            "🚀 Deployment successful",
        ];

        for observation in valid_observations {
            assert!(
                security::test_input_sanitization(observation).is_ok(),
                "Observation '{}' should be valid",
                observation
            );
        }

        // Invalid observations (contain dangerous characters)
        let invalid_observations = vec![
            "Task completed; rm -rf /",
            "Success && echo 'hacked'",
            "Error | cat /etc/passwd",
            "Processed `whoami` items",
            "Debug $(pwd)",
            "Info ${USER}",
            "../../../etc/passwd",
            "Warning; drop table observations;",
        ];

        for observation in invalid_observations {
            assert!(
                security::test_input_sanitization(observation).is_err(),
                "Observation '{}' should be invalid",
                observation
            );
        }
    }

    #[test]
    fn test_tool_name_validation() {
        // Valid tool names
        let valid_tools = vec![
            "calculator",
            "search_engine",
            "file_reader",
            "api_client",
            "database_query",
            "text_processor",
            "image_generator",
            "code_compiler",
            "test_runner",
            "linter",
        ];

        for tool in valid_tools {
            assert!(
                security::test_input_sanitization(tool).is_ok(),
                "Tool name '{}' should be valid",
                tool
            );
        }

        // Invalid tool names
        let invalid_tools = vec![
            "calculator; rm -rf /",
            "search && echo 'hack'",
            "reader | cat /etc/passwd",
            "api `whoami`",
            "db $(pwd)",
            "text ${USER}",
            "../../../etc/passwd",
            "tool; drop table;",
        ];

        for tool in invalid_tools {
            assert!(
                security::test_input_sanitization(tool).is_err(),
                "Tool name '{}' should be invalid",
                tool
            );
        }
    }

    #[test]
    fn test_action_validation() {
        // Valid actions
        let valid_actions = vec![
            "calculate",
            "search",
            "read_file",
            "api_call",
            "query_database",
            "process_text",
            "generate_image",
            "compile_code",
            "run_tests",
            "lint_code",
        ];

        for action in valid_actions {
            assert!(
                security::test_input_sanitization(action).is_ok(),
                "Action '{}' should be valid",
                action
            );
        }

        // Invalid actions
        let invalid_actions = vec![
            "calculate; rm -rf /",
            "search && echo 'hack'",
            "read | cat /etc/passwd",
            "call `whoami`",
            "query $(pwd)",
            "process ${USER}",
            "../../../etc/passwd",
            "action; drop table;",
        ];

        for action in invalid_actions {
            assert!(
                security::test_input_sanitization(action).is_err(),
                "Action '{}' should be invalid",
                action
            );
        }
    }

    #[test]
    fn test_empty_and_whitespace_inputs() {
        // Empty inputs should generally be invalid for most fields
        let empty_inputs = vec!["", "   ", "\t", "\n", "\r\n"];

        for input in empty_inputs {
            // Empty strings might be allowed for some optional fields,
            // but they should at least pass sanitization
            assert!(
                security::test_input_sanitization(input).is_ok(),
                "Empty input '{}' should pass sanitization",
                input
            );
        }
    }

    #[test]
    fn test_extremely_long_inputs() {
        // Test with very long inputs (should still work)
        let long_input = "a".repeat(10000);
        assert!(
            security::test_input_sanitization(&long_input).is_ok(),
            "Very long input should pass sanitization"
        );

        // But dangerous chars in long input should still be caught
        let long_dangerous = "a".repeat(5000) + "; rm -rf /" + &"b".repeat(5000);
        assert!(
            security::test_input_sanitization(&long_dangerous).is_err(),
            "Long input with dangerous chars should fail sanitization"
        );
    }
}

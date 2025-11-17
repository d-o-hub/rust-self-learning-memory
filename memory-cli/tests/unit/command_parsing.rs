//! Unit tests for CLI command parsing and argument validation.
//!
//! These tests verify that command-line arguments are parsed correctly,
//! validation works as expected, and error handling is proper.

use clap::Parser;
use memory_cli::{Cli, Commands};
use std::path::PathBuf;

#[cfg(test)]
mod command_parsing_tests {
    use super::*;

    #[test]
    fn test_cli_parsing_basic_help() {
        let args = vec!["memory-cli", "--help"];
        let cli = Cli::try_parse_from(args);
        // Help should cause early exit, but parsing should work
        assert!(cli.is_err()); // clap exits with error for --help
    }

    #[test]
    fn test_cli_parsing_version() {
        let args = vec!["memory-cli", "--version"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_err()); // clap exits with error for --version
    }

    #[test]
    fn test_cli_parsing_config_flag() {
        let args = vec!["memory-cli", "--config", "test.toml", "config"];
        let cli = Cli::parse_from(args);
        assert_eq!(cli.config, Some(PathBuf::from("test.toml")));
        assert_eq!(cli.format, memory_cli::output::OutputFormat::Human);
        assert!(!cli.verbose);
        assert!(!cli.dry_run);
        matches!(cli.command, Commands::Config);
    }

    #[test]
    fn test_cli_parsing_format_flags() {
        // Test JSON format
        let args = vec!["memory-cli", "--format", "json", "config"];
        let cli = Cli::parse_from(args);
        assert_eq!(cli.format, memory_cli::output::OutputFormat::Json);

        // Test YAML format
        let args = vec!["memory-cli", "--format", "yaml", "config"];
        let cli = Cli::parse_from(args);
        assert_eq!(cli.format, memory_cli::output::OutputFormat::Yaml);

        // Test human format (default)
        let args = vec!["memory-cli", "config"];
        let cli = Cli::parse_from(args);
        assert_eq!(cli.format, memory_cli::output::OutputFormat::Human);
    }

    #[test]
    fn test_cli_parsing_verbose_flag() {
        let args = vec!["memory-cli", "--verbose", "config"];
        let cli = Cli::parse_from(args);
        assert!(cli.verbose);
    }

    #[test]
    fn test_cli_parsing_dry_run_flag() {
        let args = vec!["memory-cli", "--dry-run", "config"];
        let cli = Cli::parse_from(args);
        assert!(cli.dry_run);
    }

    #[test]
    fn test_cli_parsing_invalid_format() {
        let args = vec!["memory-cli", "--format", "invalid", "config"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_err());
    }

    #[test]
    fn test_cli_parsing_episode_commands() {
        // Test episode create
        let args = vec!["memory-cli", "episode", "create", "test task"];
        let cli = Cli::parse_from(args);
        match cli.command {
            Commands::Episode { command } => {
                matches!(command, memory_cli::commands::EpisodeCommands::Create { .. });
            }
            _ => panic!("Expected episode command"),
        }

        // Test episode list
        let args = vec!["memory-cli", "episode", "list"];
        let cli = Cli::parse_from(args);
        match cli.command {
            Commands::Episode { command } => {
                matches!(command, memory_cli::commands::EpisodeCommands::List { .. });
            }
            _ => panic!("Expected episode command"),
        }

        // Test episode view with ID
        let args = vec!["memory-cli", "episode", "view", "123e4567-e89b-12d3-a456-426614174000"];
        let cli = Cli::parse_from(args);
        match cli.command {
            Commands::Episode { command } => {
                matches!(command, memory_cli::commands::EpisodeCommands::View { .. });
            }
            _ => panic!("Expected episode command"),
        }
    }

    #[test]
    fn test_cli_parsing_pattern_commands() {
        // Test pattern list
        let args = vec!["memory-cli", "pattern", "list"];
        let cli = Cli::parse_from(args);
        match cli.command {
            Commands::Pattern { command } => {
                matches!(command, memory_cli::commands::PatternCommands::List { .. });
            }
            _ => panic!("Expected pattern command"),
        }

        // Test pattern view
        let args = vec!["memory-cli", "pattern", "view", "123e4567-e89b-12d3-a456-426614174000"];
        let cli = Cli::parse_from(args);
        match cli.command {
            Commands::Pattern { command } => {
                matches!(command, memory_cli::commands::PatternCommands::View { .. });
            }
            _ => panic!("Expected pattern command"),
        }
    }

    #[test]
    fn test_cli_parsing_storage_commands() {
        // Test storage stats
        let args = vec!["memory-cli", "storage", "stats"];
        let cli = Cli::parse_from(args);
        match cli.command {
            Commands::Storage { command } => {
                matches!(command, memory_cli::commands::StorageCommands::Stats);
            }
            _ => panic!("Expected storage command"),
        }

        // Test storage sync
        let args = vec!["memory-cli", "storage", "sync"];
        let cli = Cli::parse_from(args);
        match cli.command {
            Commands::Storage { command } => {
                matches!(command, memory_cli::commands::StorageCommands::Sync { .. });
            }
            _ => panic!("Expected storage command"),
        }
    }

    #[test]
    fn test_cli_parsing_completion_command() {
        use clap_complete::Shell;

        let args = vec!["memory-cli", "completion", "bash"];
        let cli = Cli::parse_from(args);
        match cli.command {
            Commands::Completion { shell } => {
                assert_eq!(shell, Shell::Bash);
            }
            _ => panic!("Expected completion command"),
        }

        let args = vec!["memory-cli", "completion", "zsh"];
        let cli = Cli::parse_from(args);
        match cli.command {
            Commands::Completion { shell } => {
                assert_eq!(shell, Shell::Zsh);
            }
            _ => panic!("Expected completion command"),
        }
    }

    #[test]
    fn test_cli_parsing_config_command() {
        let args = vec!["memory-cli", "config"];
        let cli = Cli::parse_from(args);
        matches!(cli.command, Commands::Config);
    }

    #[test]
    fn test_cli_parsing_missing_subcommand() {
        let args = vec!["memory-cli"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_err());
    }

    #[test]
    fn test_cli_parsing_unknown_command() {
        let args = vec!["memory-cli", "unknown"];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_err());
    }

    #[test]
    fn test_cli_parsing_episode_create_with_context() {
        let args = vec!["memory-cli", "episode", "create", "test task", "--context", "domain:rust,language:rust,tags:test"];
        let cli = Cli::parse_from(args);
        match cli.command {
            Commands::Episode { command } => {
                if let memory_cli::commands::EpisodeCommands::Create { task, context } = command {
                    assert_eq!(task, "test task");
                    assert_eq!(context, Some("domain:rust,language:rust,tags:test".to_string()));
                } else {
                    panic!("Expected episode create command");
                }
            }
            _ => panic!("Expected episode command"),
        }
    }

    #[test]
    fn test_cli_parsing_episode_list_with_filters() {
        let args = vec!["memory-cli", "episode", "list", "--task-type", "testing", "--limit", "10"];
        let cli = Cli::parse_from(args);
        match cli.command {
            Commands::Episode { command } => {
                if let memory_cli::commands::EpisodeCommands::List { task_type, limit, status } = command {
                    assert_eq!(task_type, Some("testing".to_string()));
                    assert_eq!(limit, Some(10));
                    assert_eq!(status, None);
                } else {
                    panic!("Expected episode list command");
                }
            }
            _ => panic!("Expected episode command"),
        }
    }

    #[test]
    fn test_cli_parsing_episode_complete() {
        let args = vec!["memory-cli", "episode", "complete", "123e4567-e89b-12d3-a456-426614174000", "--outcome", "success"];
        let cli = Cli::parse_from(args);
        match cli.command {
            Commands::Episode { command } => {
                if let memory_cli::commands::EpisodeCommands::Complete { episode_id, outcome } = command {
                    assert_eq!(episode_id, "123e4567-e89b-12d3-a456-426614174000");
                    assert_eq!(outcome, Some("success".to_string()));
                } else {
                    panic!("Expected episode complete command");
                }
            }
            _ => panic!("Expected episode command"),
        }
    }

    #[test]
    fn test_cli_parsing_episode_log_step() {
        let args = vec![
            "memory-cli",
            "episode",
            "log-step",
            "123e4567-e89b-12d3-a456-426614174000",
            "--tool", "test_tool",
            "--action", "test_action",
            "--success",
            "--latency-ms", "100",
            "--tokens", "50",
            "--observation", "test observation"
        ];
        let cli = Cli::parse_from(args);
        match cli.command {
            Commands::Episode { command } => {
                if let memory_cli::commands::EpisodeCommands::LogStep {
                    episode_id,
                    tool,
                    action,
                    success,
                    latency_ms,
                    tokens,
                    observation,
                } = command {
                    assert_eq!(episode_id, "123e4567-e89b-12d3-a456-426614174000");
                    assert_eq!(tool, "test_tool");
                    assert_eq!(action, "test_action");
                    assert!(success);
                    assert_eq!(latency_ms, 100);
                    assert_eq!(tokens, 50);
                    assert_eq!(observation, "test observation");
                } else {
                    panic!("Expected episode log-step command");
                }
            }
            _ => panic!("Expected episode command"),
        }
    }
}
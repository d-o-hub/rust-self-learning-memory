#![expect(
    clippy::empty_line_after_doc_comments,
    reason = "blank line kept after doc comments"
)]
#![expect(dead_code, reason = "public API unused inside the crate")]
#![expect(unused_imports, reason = "re-exports kept for downstream users")]
#![expect(unused_variables, reason = "params kept for API symmetry")]
#![expect(unused_mut, reason = "mutation is conditional")]
#![expect(clippy::nonminimal_bool, reason = "boolean expression kept explicit")]
#![expect(clippy::needless_borrow, reason = "borrow kept for uniform call sites")]
#![expect(clippy::manual_clamp, reason = "explicit bounds clearer than clamp")]
#![expect(clippy::derivable_impls, reason = "hand-written impls document intent")]
#![expect(clippy::excessive_nesting, reason = "deeply nested control flow")]
#![expect(
    clippy::if_same_then_else,
    reason = "branches kept separate for clarity"
)]
#![expect(
    clippy::cognitive_complexity,
    reason = "long-standing complex functions"
)]
#![expect(clippy::doc_markdown, reason = "identifier backticks noisy in prose")]
#![cfg_attr(
    test,
    expect(clippy::panic, reason = "panic used for invariant violations")
)]
#![cfg_attr(
    not(test),
    expect(clippy::wildcard_imports, reason = "glob imports used for preludes")
)]
#![expect(
    clippy::cast_precision_loss,
    reason = "precision loss accepted in metric math"
)]
#![expect(
    clippy::cast_possible_truncation,
    reason = "integer narrowing bounded by checks"
)]
#![expect(
    clippy::cast_sign_loss,
    reason = "non-negative values cast to unsigned"
)]
#![expect(
    clippy::struct_excessive_bools,
    reason = "config struct expressed as flags"
)]
#![expect(clippy::match_same_arms, reason = "explicit arms aid maintenance")]
#![expect(
    clippy::redundant_closure_for_method_calls,
    reason = "explicit closures aid readability"
)]
#![expect(clippy::uninlined_format_args, reason = "format args kept explicit")]
#![expect(clippy::expect_used, reason = "infallible expect in known-good paths")]
#![expect(clippy::unwrap_used, reason = "infallible unwrap in known-good paths")]
#![expect(clippy::map_unwrap_or, reason = "explicit pattern clearer than map_or")]
#![expect(
    clippy::needless_pass_by_value,
    reason = "owned params kept for ergonomics"
)]
#![expect(clippy::unused_self, reason = "method kept for API symmetry")]
#![expect(clippy::unused_async, reason = "async kept for API symmetry")]
#![expect(clippy::assigning_clones, reason = "clone_from not always clearer")]
#![expect(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "extension comparison kept simple"
)]
#![expect(
    clippy::cast_lossless,
    reason = "explicit widening casts document intent"
)]
#![expect(
    clippy::fn_params_excessive_bools,
    reason = "boolean flags are part of the API"
)]
#![expect(
    clippy::format_push_string,
    reason = "push_str formatting kept explicit"
)]
#![expect(clippy::if_not_else, reason = "positive-first form reads better")]
#![expect(
    clippy::items_after_statements,
    reason = "helpers declared next to use"
)]
#![expect(clippy::manual_let_else, reason = "match kept for clarity")]
#![expect(
    clippy::manual_string_new,
    reason = "explicit String::new kept for clarity"
)]
#![expect(clippy::needless_continue, reason = "continue kept for explicitness")]
#![expect(clippy::ref_option, reason = "&Option kept for API consistency")]
#![cfg_attr(
    test,
    expect(clippy::single_char_pattern, reason = "single-char pattern kept")
)]
#![expect(clippy::single_match_else, reason = "explicit arms aid maintenance")]
#![expect(
    clippy::struct_field_names,
    reason = "shared field prefixes aid readability"
)]
#![expect(clippy::unnecessary_semicolon, reason = "semicolon kept for clarity")]
#![expect(missing_docs, reason = "public docs still incomplete")]

use clap::{CommandFactory, Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod config;
mod errors;
mod output;

#[cfg(test)]
mod test_utils;

use commands::*;
use config::{apply_db_path_override, initialize_storage, load_config_with_validation};
use output::OutputFormat;

#[derive(Parser)]
#[command(name = "do-memory-cli")]
#[command(about = "Command-line interface for Self-Learning Memory System")]
#[command(version, long_about = None)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Human)]
    format: OutputFormat,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Show what would be done without executing
    #[arg(long)]
    dry_run: bool,

    /// Storage mode to use (remote, local, memory)
    #[arg(long, env = "MEMORY_STORAGE_MODE")]
    storage_mode: Option<String>,

    /// Path to the local SQLite database file
    #[arg(long, env = "MEMORY_DB_PATH")]
    db_path: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Episode management commands
    #[command(alias = "ep")]
    Episode {
        #[command(subcommand)]
        command: EpisodeCommands,
    },
    /// Pattern analysis commands
    #[command(alias = "pat")]
    Pattern {
        #[command(subcommand)]
        command: PatternCommands,
    },
    /// Storage operations commands
    #[command(alias = "st")]
    Storage {
        #[command(subcommand)]
        command: StorageCommands,
    },
    /// Configuration validation and management
    #[command(alias = "cfg")]
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Health monitoring and diagnostics
    #[command(alias = "hp")]
    Health {
        #[command(subcommand)]
        command: HealthCommands,
    },
    /// Backup and restore operations
    #[command(alias = "bak")]
    Backup {
        #[command(subcommand)]
        command: BackupCommands,
    },
    /// Monitoring and metrics
    #[command(alias = "mon")]
    Monitor {
        #[command(subcommand)]
        command: MonitorCommands,
    },
    /// Log analysis and search
    #[command(alias = "log")]
    Logs {
        #[command(subcommand)]
        command: LogsCommands,
    },
    /// Evaluation and calibration commands
    #[command(alias = "ev")]
    Eval {
        #[command(subcommand)]
        command: EvalCommands,
    },
    /// Embedding provider management and testing
    #[command(alias = "emb")]
    Embedding {
        #[command(subcommand)]
        command: EmbeddingCommands,
    },
    /// Generate shell completion scripts
    #[command(alias = "comp")]
    Completion {
        /// Shell to generate completion for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    /// Tag management commands for episodes
    #[command(alias = "tg")]
    Tag {
        #[command(subcommand)]
        command: TagCommands,
    },
    /// Relationship management commands for episodes
    #[command(alias = "rel")]
    Relationship {
        #[command(subcommand)]
        command: crate::commands::relationships::StandaloneRelationshipCommands,
    },
    /// Playbook recommendation and management
    #[command(alias = "pb")]
    Playbook {
        #[command(subcommand)]
        command: PlaybookCommands,
    },
    /// Recommendation feedback tracking
    #[command(alias = "fb")]
    Feedback {
        #[command(subcommand)]
        command: FeedbackCommands,
    },
    /// External signal provider management
    #[command(alias = "sig", name = "external-signal")]
    ExternalSignal {
        #[command(subcommand)]
        command: crate::commands::ExternalSignalCommands,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Capture process start time for `health check`'s uptime metric (WG-159).
    crate::commands::health::init_uptime();

    let cli = Cli::parse();

    // Initialize tracing. Logs go to stderr so `--format json`/`yaml` output
    // on stdout stays machine-parseable for scripts; RUST_LOG (when set)
    // overrides the default level (debug with -v, info otherwise).
    let default_level = if cli.verbose { "debug" } else { "info" };
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(default_level));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();

    // Load configuration with validation
    let mut config = match &cli.config {
        Some(path) => load_config_with_validation(Some(path.as_ref()))?,
        None => load_config_with_validation(None)?,
    };

    // Apply CLI overrides
    if let Some(mode) = cli.storage_mode {
        config.database.storage_mode = Some(mode);
    }
    if let Some(path) = cli.db_path {
        // Issue #830: always override redb_path (Config::default() pre-fills
        // XDG). Sibling paths keep Turso SQLite and redb from sharing a file.
        apply_db_path_override(&mut config, &path);
    }

    // Accept `[storage].storage_mode` as a UX alias for `[database].storage_mode`
    // (issue #832). Prefer the database field when both are set.
    config.normalize_storage_mode();

    // Create memory system with storage backends
    let storage_result = initialize_storage(&config).await?;

    // Execute command
    match cli.command {
        Commands::Episode { command } => {
            handle_episode_command(
                command,
                &storage_result.memory,
                &config,
                cli.format,
                cli.dry_run,
            )
            .await
        }
        Commands::Pattern { command } => {
            handle_pattern_command(
                command,
                &storage_result.memory,
                &config,
                cli.format,
                cli.dry_run,
            )
            .await
        }
        Commands::Storage { command } => {
            handle_storage_command(
                command,
                &storage_result.memory,
                &config,
                cli.format,
                cli.dry_run,
            )
            .await
        }
        Commands::Config { command } => {
            handle_config_command(
                command,
                &storage_result.memory,
                &config,
                cli.format,
                cli.dry_run,
            )
            .await
        }
        Commands::Health { command } => {
            handle_health_command(
                command,
                &storage_result.memory,
                &config,
                cli.format,
                cli.dry_run,
            )
            .await
        }
        Commands::Backup { command } => {
            handle_backup_command(
                command,
                &storage_result.memory,
                &config,
                cli.format,
                cli.dry_run,
            )
            .await
        }
        Commands::Monitor { command } => {
            handle_monitor_command(
                command,
                &storage_result.memory,
                &config,
                cli.format,
                cli.dry_run,
            )
            .await
        }
        Commands::Logs { command } => {
            handle_logs_command(
                command,
                &storage_result.memory,
                &config,
                cli.format,
                cli.dry_run,
            )
            .await
        }
        Commands::Eval { command } => {
            handle_eval_command(
                command,
                &storage_result.memory,
                &config,
                cli.format,
                cli.dry_run,
            )
            .await
        }
        Commands::Embedding { command } => {
            handle_embedding_command(
                command,
                &storage_result.memory,
                &config,
                cli.format,
                cli.dry_run,
            )
            .await
        }
        Commands::Completion { shell } => {
            clap_complete::generate(
                shell,
                &mut Cli::command(),
                "do-memory-cli",
                &mut std::io::stdout(),
            );
            Ok(())
        }
        Commands::Tag { command } => {
            handle_tag_command(
                command,
                &storage_result.memory,
                &config,
                cli.format,
                cli.dry_run,
            )
            .await
        }
        Commands::Relationship { command } => {
            handle_relationship_command(
                command,
                &storage_result.memory,
                &config,
                cli.format,
                cli.dry_run,
            )
            .await
        }
        Commands::Playbook { command } => {
            handle_playbook_command(
                command,
                &storage_result.memory,
                &config,
                cli.format,
                cli.dry_run,
            )
            .await
        }
        Commands::Feedback { command } => {
            handle_feedback_command(
                command,
                &storage_result.memory,
                &config,
                cli.format,
                cli.dry_run,
            )
            .await
        }
        Commands::ExternalSignal { command } => {
            handle_external_signal_command(
                command,
                &storage_result.memory,
                &config,
                cli.format,
                cli.dry_run,
            )
            .await
        }
    }
}

#[cfg(test)]
mod cli_parsing_tests {
    use super::Cli;
    use clap::Parser;

    #[test]
    fn test_eval_stats_parses() {
        let cli = Cli::try_parse_from(["do-memory-cli", "eval", "stats", "web-development"])
            .expect("eval stats should parse");
        assert!(matches!(cli.command, crate::Commands::Eval { .. }));
    }

    #[test]
    fn test_eval_set_threshold_rejected() {
        // The set-threshold subcommand was removed because no override model
        // is persisted or consumed by reward calculation. Parsing must fail.
        let result = Cli::try_parse_from([
            "do-memory-cli",
            "eval",
            "set-threshold",
            "--domain",
            "web-development",
            "--duration",
            "300",
        ]);
        assert!(result.is_err());
    }
}

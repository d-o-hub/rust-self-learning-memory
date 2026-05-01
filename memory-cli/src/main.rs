#![allow(clippy::empty_line_after_doc_comments)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(clippy::ifs_same_cond)]
#![allow(clippy::nonminimal_bool)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::manual_clamp)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::excessive_nesting)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::panic)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::float_cmp)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unused_self)]
#![allow(clippy::unused_async)]
#![allow(clippy::assigning_clones)]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::fn_params_excessive_bools)]
#![allow(clippy::format_push_string)]
#![allow(clippy::if_not_else)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::manual_string_new)]
#![allow(clippy::needless_continue)]
#![allow(clippy::no_effect_underscore_binding)]
#![allow(clippy::redundant_else)]
#![allow(clippy::ref_option)]
#![allow(clippy::single_char_pattern)]
#![allow(clippy::single_match_else)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::type_complexity)]
#![allow(clippy::useless_conversion)]
#![allow(clippy::bool_to_int_with_if)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::unnecessary_semicolon)]
#![allow(missing_docs)]

use clap::{CommandFactory, Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod config;
mod errors;
mod output;

#[cfg(test)]
mod test_utils;

use commands::*;
use config::{initialize_storage, load_config_with_validation};
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
    let cli = Cli::parse();

    // Initialize tracing
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    // Load configuration with validation
    let config = match &cli.config {
        Some(path) => load_config_with_validation(Some(path.as_ref()))?,
        None => load_config_with_validation(None)?,
    };

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

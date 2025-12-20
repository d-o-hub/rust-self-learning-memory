use clap::{CommandFactory, Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod config;
mod errors;
mod output;

#[cfg(test)]
mod test_utils;

use commands::*;
use config::{initialize_storage, load_config};
use output::OutputFormat;

#[derive(Parser)]
#[command(name = "memory-cli")]
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
    /// Generate shell completion scripts
    #[command(alias = "comp")]
    Completion {
        /// Shell to generate completion for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
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

    // Load configuration
    let config = match &cli.config {
        Some(path) => load_config(Some(path.as_ref()))?,
        None => load_config(None)?,
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
        Commands::Completion { shell } => {
            clap_complete::generate(
                shell,
                &mut Cli::command(),
                "memory-cli",
                &mut std::io::stdout(),
            );
            Ok(())
        }
    }
}

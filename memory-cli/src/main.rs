use clap::{CommandFactory, Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod config;
mod output;

#[cfg(test)]
mod test_utils;

use commands::*;
use config::Config;
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
    Episode {
        #[command(subcommand)]
        command: EpisodeCommands,
    },
    /// Pattern analysis commands
    Pattern {
        #[command(subcommand)]
        command: PatternCommands,
    },
    /// Storage operations commands
    Storage {
        #[command(subcommand)]
        command: StorageCommands,
    },
    /// Configuration validation and management
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Health monitoring and diagnostics
    Health {
        #[command(subcommand)]
        command: HealthCommands,
    },
    /// Backup and restore operations
    Backup {
        #[command(subcommand)]
        command: BackupCommands,
    },
    /// Monitoring and metrics
    Monitor {
        #[command(subcommand)]
        command: MonitorCommands,
    },
    /// Log analysis and search
    Logs {
        #[command(subcommand)]
        command: LogsCommands,
    },
    /// Generate shell completion scripts
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
    let config = Config::load(cli.config.as_deref())?;

    // Create memory system with storage backends
    let memory = config.create_memory().await?;

    // Execute command
    match cli.command {
        Commands::Episode { command } => {
            handle_episode_command(command, &memory, &config, cli.format, cli.dry_run).await
        }
        Commands::Pattern { command } => {
            handle_pattern_command(command, &memory, &config, cli.format, cli.dry_run).await
        }
        Commands::Storage { command } => {
            handle_storage_command(command, &memory, &config, cli.format, cli.dry_run).await
        }
        Commands::Config { command } => {
            handle_config_command(command, &memory, &config, cli.format, cli.dry_run).await
        }
        Commands::Health { command } => {
            handle_health_command(command, &memory, &config, cli.format, cli.dry_run).await
        }
        Commands::Backup { command } => {
            handle_backup_command(command, &memory, &config, cli.format, cli.dry_run).await
        }
        Commands::Monitor { command } => {
            handle_monitor_command(command, &memory, &config, cli.format, cli.dry_run).await
        }
        Commands::Logs { command } => {
            handle_logs_command(command, &memory, &config, cli.format, cli.dry_run).await
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

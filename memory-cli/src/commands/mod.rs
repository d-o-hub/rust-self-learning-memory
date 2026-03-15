#![allow(ambiguous_glob_reexports)]

pub mod backup;
pub mod config;
pub mod embedding;
pub mod episode;
pub mod eval;
pub mod feedback;
pub mod health;
pub mod logs;
pub mod monitor;
pub mod pattern;
pub mod playbook;
pub mod relationships;
pub mod storage;
pub mod tag;

pub use backup::*;
pub use config::*;
pub use embedding::*;
pub use episode::*;
pub use eval::*;
pub use feedback::*;
pub use health::*;
pub use logs::*;
pub use monitor::*;
pub use pattern::*;
pub use playbook::*;
pub use relationships::*;
pub use storage::*;
pub use tag::*;

use crate::config::Config;
use crate::output::OutputFormat;

mod dispatch_episode;
mod dispatch_misc;
mod dispatch_pattern;

pub use dispatch_episode::{handle_episode_command, handle_relationships_command};
pub use dispatch_misc::{handle_embedding_command, handle_tag_command};
pub use dispatch_pattern::handle_pattern_command;

pub async fn handle_storage_command(
    command: StorageCommands,
    memory: &memory_core::SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    match command {
        StorageCommands::Stats => storage::storage_stats(memory, config, format).await,
        StorageCommands::Sync {
            force,
            dry_run: sync_dry_run,
        } => storage::sync_storage(memory, config, format, force, sync_dry_run || dry_run).await,
        StorageCommands::Vacuum {
            dry_run: vacuum_dry_run,
        } => storage::vacuum_storage(memory, config, format, vacuum_dry_run || dry_run).await,
        StorageCommands::Health => storage::storage_health(memory, config, format).await,
        StorageCommands::Connections => storage::connection_status(memory, config, format).await,
    }
}

pub async fn handle_config_command(
    command: ConfigCommands,
    memory: &memory_core::SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
    _dry_run: bool,
) -> anyhow::Result<()> {
    match command {
        ConfigCommands::Validate => config::validate_config(memory, config, format).await,
        ConfigCommands::Check => config::check_config(memory, config, format).await,
        ConfigCommands::Show => config::show_config(memory, config, format).await,
        ConfigCommands::Wizard => config::run_wizard().await,
    }
}

pub async fn handle_health_command(
    command: HealthCommands,
    memory: &memory_core::SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    match command {
        HealthCommands::Check => health::health_check(memory, config, format).await,
        HealthCommands::Status => health::health_status(memory, config, format).await,
        HealthCommands::Monitor { interval, duration } => {
            health::health_monitor(memory, config, format, interval, duration, dry_run).await
        }
    }
}

pub async fn handle_backup_command(
    command: BackupCommands,
    memory: &memory_core::SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    match command {
        BackupCommands::Create {
            path,
            format: backup_format,
            compress,
        } => {
            backup::create_backup(
                memory,
                config,
                format,
                path,
                backup_format,
                compress,
                dry_run,
            )
            .await
        }
        BackupCommands::List { path } => backup::list_backups(memory, config, format, path).await,
        BackupCommands::Restore {
            path,
            backup_id,
            force,
        } => backup::restore_backup(memory, config, format, path, backup_id, force, dry_run).await,
        BackupCommands::Verify { path, backup_id } => {
            backup::verify_backup(memory, config, format, path, backup_id).await
        }
    }
}

pub async fn handle_monitor_command(
    command: MonitorCommands,
    memory: &memory_core::SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
    _dry_run: bool,
) -> anyhow::Result<()> {
    match command {
        MonitorCommands::Status => monitor::monitor_status(memory, config, format).await,
        MonitorCommands::Metrics => monitor::monitor_metrics(memory, config, format).await,
        MonitorCommands::Export {
            format: export_format,
        } => monitor::export_metrics(memory, config, format, export_format).await,
    }
}

pub async fn handle_logs_command(
    command: LogsCommands,
    memory: &memory_core::SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    match command {
        LogsCommands::Analyze { since, filter } => {
            logs::analyze_logs(memory, config, format, since, filter).await
        }
        LogsCommands::Search {
            query,
            limit,
            since,
        } => logs::search_logs(memory, config, format, query, limit, since).await,
        LogsCommands::Export {
            path,
            format: export_format,
            since,
            filter,
        } => {
            logs::export_logs(
                memory,
                config,
                format,
                path,
                export_format,
                since,
                filter,
                dry_run,
            )
            .await
        }
        LogsCommands::Stats { since } => logs::logs_stats(memory, config, format, since).await,
    }
}

pub async fn handle_eval_command(
    command: EvalCommands,
    memory: &memory_core::SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
    _dry_run: bool,
) -> anyhow::Result<()> {
    match command {
        EvalCommands::Calibration {
            domain,
            all,
            min_episodes,
        } => eval::calibration(domain, all, min_episodes, memory, config, format).await,
        EvalCommands::Stats { domain } => eval::domain_stats(domain, memory, config, format).await,
        EvalCommands::SetThreshold {
            domain,
            duration,
            steps,
        } => eval::set_threshold(domain, duration, steps, memory, config, format).await,
    }
}

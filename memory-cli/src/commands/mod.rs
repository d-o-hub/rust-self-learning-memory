pub mod episode;
pub mod pattern;
pub mod storage;

pub use episode::*;
pub use pattern::*;
pub use storage::*;

use crate::config::Config;
use crate::output::OutputFormat;

pub async fn handle_episode_command(
    command: EpisodeCommands,
    memory: &memory_core::SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    match command {
        EpisodeCommands::Create { task, context } => {
            episode::create_episode(task, context, config, format, dry_run).await
        }
        EpisodeCommands::List {
            task_type,
            limit,
            status,
        } => episode::list_episodes(task_type, limit, status, config, format).await,
        EpisodeCommands::View { episode_id } => {
            episode::view_episode(episode_id, config, format).await
        }
        EpisodeCommands::Complete {
            episode_id,
            outcome,
        } => episode::complete_episode(episode_id, outcome, config, format, dry_run).await,
        EpisodeCommands::Search { query, limit } => {
            episode::search_episodes(query, limit, config, format).await
        }
        EpisodeCommands::LogStep {
            episode_id,
            tool,
            action,
            success,
            latency_ms,
            tokens,
            observation,
        } => {
            episode::log_step(
                episode_id,
                tool,
                action,
                success,
                latency_ms,
                tokens,
                observation,
                config,
                format,
                dry_run,
            )
            .await
        }
    }
}

pub async fn handle_pattern_command(
    command: PatternCommands,
    memory: &memory_core::SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    match command {
        PatternCommands::List {
            min_confidence,
            pattern_type,
            limit,
        } => {
            pattern::list_patterns(min_confidence, pattern_type, limit, memory, config, format)
                .await
        }
        PatternCommands::View { pattern_id } => {
            pattern::view_pattern(pattern_id, memory, config, format).await
        }
        PatternCommands::Analyze {
            pattern_id,
            episodes,
        } => pattern::analyze_pattern(pattern_id, episodes, memory, config, format).await,
        PatternCommands::Effectiveness { top, min_uses } => {
            pattern::pattern_effectiveness(top, min_uses, memory, config, format).await
        }
        PatternCommands::Decay {
            dry_run: decay_dry_run,
            force,
        } => pattern::decay_patterns(memory, config, format, decay_dry_run || dry_run, force).await,
    }
}

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

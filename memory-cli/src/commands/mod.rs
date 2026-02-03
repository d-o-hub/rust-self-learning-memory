#![allow(ambiguous_glob_reexports)]

pub mod backup;
pub mod config;
pub mod embedding;
pub mod episode;
pub mod eval;
pub mod health;
pub mod logs;
pub mod monitor;
pub mod pattern;
pub mod storage;
pub mod tag;

pub use backup::*;
pub use config::*;
pub use embedding::*;
pub use episode::*;
pub use eval::*;
pub use health::*;
pub use logs::*;
pub use monitor::*;
pub use pattern::*;
pub use storage::*;
pub use tag::*;

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
            create_episode(task, context, memory, config, format, dry_run).await
        }
        EpisodeCommands::List {
            task_type,
            limit,
            status,
            semantic_search,
            enable_embeddings,
            embedding_provider,
            embedding_model,
            since,
            until,
            sort,
            domain,
            tags,
            outcome,
            offset,
        } => {
            list_episodes(
                task_type,
                limit,
                status,
                semantic_search,
                enable_embeddings,
                embedding_provider,
                embedding_model,
                since,
                until,
                sort,
                domain,
                tags,
                outcome,
                offset,
                memory,
                config,
                format,
            )
            .await
        }
        EpisodeCommands::Filter {
            command: filter_cmd,
        } => handle_filter_command(filter_cmd, memory, config, format, dry_run).await,
        EpisodeCommands::View { episode_id } => {
            view_episode(episode_id, memory, config, format).await
        }
        EpisodeCommands::Complete {
            episode_id,
            outcome,
        } => complete_episode(episode_id, outcome, memory, config, format, dry_run).await,
        EpisodeCommands::Delete { episode_id } => {
            delete_episode(episode_id, memory, config, format, dry_run).await
        }
        EpisodeCommands::Update {
            episode_id,
            description,
            add_tag,
            remove_tag,
            set_tags,
            metadata,
        } => {
            update_episode(
                episode_id,
                description,
                add_tag,
                remove_tag,
                set_tags,
                metadata,
                memory,
                config,
                format,
                dry_run,
            )
            .await
        }
        EpisodeCommands::Search {
            query,
            limit,
            semantic,
            enable_embeddings,
            embedding_provider,
            embedding_model,
            fuzzy,
            fuzzy_threshold,
            regex,
            search_fields,
            sort,
        } => {
            search_episodes(
                query,
                limit,
                semantic,
                enable_embeddings,
                embedding_provider,
                embedding_model,
                fuzzy,
                fuzzy_threshold,
                regex,
                search_fields,
                sort,
                memory,
                config,
                format,
            )
            .await
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
            log_step(
                episode_id,
                tool,
                action,
                success,
                latency_ms,
                tokens,
                observation,
                memory,
                config,
                format,
                dry_run,
            )
            .await
        }
        EpisodeCommands::Bulk { episode_ids } => {
            bulk_get_episodes(episode_ids, memory, config, format).await
        }
        EpisodeCommands::Relationships(cmd) => {
            handle_relationships_command(cmd, memory, config, format, dry_run).await
        }
    }
}

pub async fn handle_relationships_command(
    command: crate::commands::episode::RelationshipCommands,
    memory: &memory_core::SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    use crate::commands::episode::*;

    match command {
        RelationshipCommands::AddRelationship {
            from_episode_id,
            to,
            r#type,
            reason,
            priority,
            created_by,
        } => {
            add_relationship(
                from_episode_id,
                to,
                r#type,
                reason,
                priority,
                created_by,
                memory,
                config,
                format,
                dry_run,
            )
            .await
        }
        RelationshipCommands::RemoveRelationship { relationship_id } => {
            remove_relationship(relationship_id, memory, config, format, dry_run).await
        }
        RelationshipCommands::ListRelationships {
            episode_id,
            direction,
            r#type,
            format: list_format,
        } => {
            list_relationships(
                episode_id,
                direction,
                r#type,
                list_format,
                memory,
                config,
                format,
                dry_run,
            )
            .await
        }
        RelationshipCommands::FindRelated {
            episode_id,
            r#type,
            limit,
            format: list_format,
        } => {
            find_related(
                episode_id,
                r#type,
                limit,
                list_format,
                memory,
                config,
                format,
                dry_run,
            )
            .await
        }
        RelationshipCommands::DependencyGraph {
            episode_id,
            depth,
            format: graph_format,
            output,
        } => {
            dependency_graph(
                episode_id,
                depth,
                graph_format,
                output,
                memory,
                config,
                format,
                dry_run,
            )
            .await
        }
        RelationshipCommands::ValidateCycles { episode_id, r#type } => {
            validate_cycles(episode_id, r#type, memory, config, format, dry_run).await
        }
        RelationshipCommands::TopologicalSort { episode_ids } => {
            topological_sort(episode_ids, memory, config, format, dry_run).await
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
        #[cfg(feature = "turso")]
        PatternCommands::Batch { command } => {
            // Get storage backend for batch operations
            // TODO: This is a workaround - batch commands should not need direct storage access
            use memory_storage_turso::TursoStorage;
            let storage = TursoStorage::from_config(&config.database)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create Turso storage: {}", e))?;

            pattern::execute_pattern_batch_command(command, storage).await
        }
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

pub async fn handle_embedding_command(
    command: EmbeddingCommands,
    _memory: &memory_core::SelfLearningMemory,
    config: &Config,
    _format: OutputFormat,
    _dry_run: bool,
) -> anyhow::Result<()> {
    match command {
        EmbeddingCommands::Test => embedding::test_embeddings(config).await,
        EmbeddingCommands::Config => embedding::show_config(config),
        EmbeddingCommands::ListProviders => embedding::list_providers(),
        EmbeddingCommands::Benchmark => embedding::benchmark_embeddings(config).await,
        EmbeddingCommands::Enable => embedding::enable_embeddings(),
        EmbeddingCommands::Disable => embedding::disable_embeddings(),
    }
}

pub async fn handle_tag_command(
    command: TagCommands,
    memory: &memory_core::SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    tag::handle_tag_command(command, memory, config, format, dry_run).await
}

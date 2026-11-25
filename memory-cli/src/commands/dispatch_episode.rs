use crate::commands::episode::{
    CleanupPolicyArgs, EpisodeCommands, bulk_get_episodes, checkpoint, cleanup_episodes_command,
    complete_episode, create_episode, delete_episode, fail_episode, handoff, log_step, resume,
    search_episodes, update_episode,
};
use crate::config::Config;
use crate::output::OutputFormat;

pub async fn handle_episode_command(
    command: EpisodeCommands,
    memory: &do_memory_core::SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    match command {
        EpisodeCommands::Create {
            task,
            context,
            domain,
        } => create_episode(task, context, domain, memory, config, format, dry_run).await,
        EpisodeCommands::List {
            task_type,
            limit,
            status,
            semantic_search,
            since,
            until,
            sort,
            domain,
            tags,
            outcome,
            offset,
        } => {
            crate::commands::episode::list_episodes(
                task_type,
                limit,
                status,
                semantic_search,
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
        EpisodeCommands::View { episode_id } => {
            crate::commands::episode::view_episode(episode_id, memory, config, format).await
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
            fuzzy,
            fuzzy_threshold,
            regex,
            search_fields,
            sort,
            domain,
            r#type,
        } => {
            search_episodes(
                query,
                limit,
                semantic,
                fuzzy,
                fuzzy_threshold,
                regex,
                search_fields,
                sort,
                domain,
                r#type,
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
        EpisodeCommands::Filter { command } => {
            crate::commands::episode::handle_filter_command(
                command, memory, config, format, dry_run,
            )
            .await
        }
        EpisodeCommands::Complete {
            episode_id,
            outcome,
            durable_timeout_secs,
        } => {
            complete_episode(
                episode_id,
                outcome,
                memory,
                config,
                format,
                dry_run,
                durable_timeout_secs,
            )
            .await
        }
        EpisodeCommands::Fail {
            episode_id,
            durable_timeout_secs,
        } => {
            fail_episode(
                episode_id,
                memory,
                config,
                format,
                dry_run,
                durable_timeout_secs,
            )
            .await
        }
        EpisodeCommands::Delete { episode_id } => {
            delete_episode(episode_id, memory, config, format, dry_run).await
        }
        EpisodeCommands::Cleanup {
            preset,
            criterion,
            max_age_days,
            min_reward,
            max_episodes,
            batch_size,
            count_only,
            confirm,
        } => {
            cleanup_episodes_command(
                CleanupPolicyArgs {
                    preset,
                    criterion,
                    max_age_days,
                    min_reward,
                    max_episodes,
                    batch_size,
                    dry_run,
                    confirm,
                },
                count_only,
                memory,
                config,
                format,
            )
            .await
        }
        EpisodeCommands::Checkpoint {
            episode_id,
            reason,
            note,
        } => checkpoint(episode_id, reason, note, memory, config, format, dry_run).await,
        EpisodeCommands::Handoff {
            checkpoint_id,
            full,
            max_bytes,
        } => {
            handoff(
                checkpoint_id,
                full,
                max_bytes,
                memory,
                config,
                format,
                dry_run,
            )
            .await
        }
        EpisodeCommands::Resume {
            checkpoint_id,
            full,
        } => resume(checkpoint_id, full, memory, config, format, dry_run).await,
        EpisodeCommands::Checkpoints { episode_id } => {
            crate::commands::episode::list_checkpoints(episode_id, memory, config, format).await
        }
        EpisodeCommands::Relationships(cmd) => {
            handle_relationships_command(cmd, memory, config, format, dry_run).await
        }
    }
}

pub async fn handle_relationships_command(
    command: crate::commands::episode::RelationshipCommands,
    memory: &do_memory_core::SelfLearningMemory,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::episode::{RetentionCriterionArg, RetentionPresetArg};
    use do_memory_core::{MemoryConfig, SelfLearningMemory, TaskContext, TaskOutcome, TaskType};

    #[tokio::test]
    async fn cleanup_is_preview_only_until_explicit_confirmation() {
        let memory = SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            ..Default::default()
        });
        let episode_id = memory
            .start_episode(
                "Failed episode".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;
        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Failure {
                    reason: "failed".to_string(),
                    error_details: None,
                },
            )
            .await
            .unwrap();

        let cleanup = |confirm| EpisodeCommands::Cleanup {
            preset: RetentionPresetArg::Default,
            criterion: Some(RetentionCriterionArg::FailedOnly),
            max_age_days: None,
            min_reward: None,
            max_episodes: None,
            batch_size: None,
            count_only: false,
            confirm,
        };

        handle_episode_command(
            cleanup(false),
            &memory,
            &Config::default(),
            OutputFormat::Json,
            false,
        )
        .await
        .unwrap();
        assert!(memory.get_episode(episode_id).await.is_ok());

        handle_episode_command(
            cleanup(true),
            &memory,
            &Config::default(),
            OutputFormat::Json,
            false,
        )
        .await
        .unwrap();
        assert!(memory.get_episode(episode_id).await.is_err());
    }
}

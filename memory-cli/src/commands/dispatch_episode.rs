use crate::commands::episode::{
    EpisodeCommands, bulk_get_episodes, complete_episode, create_episode, delete_episode, log_step,
    search_episodes, update_episode,
};
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
            crate::commands::episode::list_episodes(
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
            enable_embeddings,
            embedding_provider,
            embedding_model,
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
                enable_embeddings,
                embedding_provider,
                embedding_model,
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
        } => complete_episode(episode_id, outcome, memory, config, format, dry_run).await,
        EpisodeCommands::Delete { episode_id } => {
            delete_episode(episode_id, memory, config, format, dry_run).await
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

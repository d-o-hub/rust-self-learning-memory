use crate::commands::PatternCommands;
use crate::commands::pattern;
use crate::config::Config;
use crate::output::OutputFormat;

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
            domain,
            episodes,
        } => pattern::analyze_pattern(pattern_id, domain, episodes, memory, config, format).await,
        PatternCommands::Search {
            query,
            domain,
            limit,
            tags,
            min_relevance,
            filter_by_domain,
        } => {
            pattern::search_patterns(
                memory,
                query.as_deref().unwrap_or(""),
                &domain,
                tags,
                limit,
                min_relevance,
                filter_by_domain,
                format,
            )
            .await
        }
        PatternCommands::Recommend {
            task,
            domain,
            limit,
            tags,
        } => {
            pattern::recommend_patterns(
                memory,
                task.as_deref().unwrap_or(""),
                &domain,
                tags,
                limit,
                format,
            )
            .await
        }
        PatternCommands::Effectiveness { top, min_uses } => {
            pattern::pattern_effectiveness(top, min_uses, memory, config, format).await
        }
        PatternCommands::Decay {
            dry_run: decay_dry_run,
            force,
        } => pattern::decay_patterns(memory, config, format, decay_dry_run || dry_run, force).await,
        #[cfg(feature = "turso")]
        PatternCommands::Batch { command } => {
            use memory_storage_turso::TursoStorage;
            let turso_url = config
                .database
                .turso_url
                .as_deref()
                .unwrap_or("")
                .trim_start_matches("file:");
            let turso_token = config.database.turso_token.as_deref().unwrap_or("");
            let mut storage = TursoStorage::new(turso_url, turso_token)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create Turso storage: {}", e))?;

            pattern::execute_pattern_batch_command(command, &mut storage).await
        }
    }
}

use super::PlaybookCommands;
use super::commands;
use crate::config::Config;
use crate::output::OutputFormat;
use do_memory_core::SelfLearningMemory;

pub async fn handle_playbook_command(
    command: PlaybookCommands,
    memory: &SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
    _dry_run: bool,
) -> anyhow::Result<()> {
    match command {
        PlaybookCommands::Recommend {
            task,
            domain,
            task_type,
            max_steps,
            language,
            framework,
            tags,
        } => {
            let task_desc = task.unwrap_or_else(|| "General task".to_string());
            commands::recommend_playbook(
                &task_desc,
                &domain,
                &task_type,
                max_steps,
                language.as_deref(),
                framework.as_deref(),
                tags,
                memory,
                config,
                format,
            )
            .await
        }
        PlaybookCommands::Explain { pattern_id } => {
            commands::explain_pattern(&pattern_id, memory, config, format).await
        }
    }
}

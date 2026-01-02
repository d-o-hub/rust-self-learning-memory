//! List episodes command implementation

use super::types::{EpisodeCommands, EpisodeList, EpisodeSummary};
use crate::config::Config;
use crate::output::OutputFormat;
use memory_core::SelfLearningMemory;

#[allow(clippy::too_many_arguments)]
pub async fn list_episodes(
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] task_type: Option<String>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] limit: usize,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] status: Option<
        super::types::EpisodeStatus,
    >,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _semantic_search: Option<String>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _enable_embeddings: bool,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _embedding_provider: Option<
        String,
    >,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _embedding_model: Option<String>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] memory: &SelfLearningMemory,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _config: &Config,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] format: OutputFormat,
) -> anyhow::Result<()> {
    // Check if storage features are enabled
    #[cfg(not(feature = "turso"))]
    return Err(anyhow::anyhow!(
        "Turso storage feature not enabled. Use --features turso to enable."
    ));

    #[cfg(feature = "turso")]
    {
        // Validate task type if specified (human-friendly error messages)
        if let Some(task_type_str) = &task_type {
            match task_type_str.as_str() {
                "code_generation" | "debugging" | "testing" | "analysis" | "documentation"
                | "refactoring" | "other" => {}
                _ => {
                    return Err(anyhow::anyhow!(
                        "Invalid task type: '{}'.\n\
                     \nValid task types:\n\
                     • code_generation - Code generation tasks\n\
                     • debugging - Debugging and troubleshooting\n\
                     • testing - Test writing and execution\n\
                     • analysis - Code analysis and review\n\
                     • documentation - Documentation tasks\n\
                     • refactoring - Code refactoring\n\
                     • other - Other task types\n\
                     \nExample: memory-cli episode list --task-type debugging",
                        task_type_str
                    ))
                }
            };
        }

        // Use core's list_episodes API which implements lazy loading (memory → redb → Turso)
        let completed_only = match status {
            Some(super::types::EpisodeStatus::Completed) => Some(true),
            _ => None, // For InProgress or None, we filter client-side
        };

        // Get more than needed so we can apply client-side filters without losing items
        let mut episodes = memory
            .list_episodes(Some(limit * 2), None, completed_only)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list episodes: {}", e))?;

        // Apply client-side filtering for in-progress and task type
        episodes.retain(|episode| match status {
            Some(super::types::EpisodeStatus::InProgress) => !episode.is_complete(),
            Some(super::types::EpisodeStatus::Completed) => episode.is_complete(),
            None => true,
        });

        if let Some(ref tt) = task_type {
            let tt_lower = tt.to_lowercase();
            episodes.retain(|episode| {
                episode
                    .task_type
                    .to_string()
                    .to_lowercase()
                    .contains(&tt_lower)
            });
        }

        // Convert to summary format
        let mut episode_summaries: Vec<EpisodeSummary> = episodes
            .into_iter()
            .map(|episode| {
                let ep_status = if episode.is_complete() {
                    "completed"
                } else {
                    "in_progress"
                };
                let duration_ms = episode
                    .end_time
                    .map(|end| (end - episode.start_time).num_milliseconds() as u64);

                EpisodeSummary {
                    episode_id: episode.episode_id.to_string(),
                    task_description: episode.task_description,
                    status: ep_status.to_string(),
                    created_at: episode.start_time.to_rfc3339(),
                    duration_ms,
                    steps_count: episode.steps.len(),
                }
            })
            .collect();

        // Sort by created_at (newest first) and apply limit
        episode_summaries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let total_count = episode_summaries.len();
        episode_summaries.truncate(limit);

        let list = EpisodeList {
            episodes: episode_summaries,
            total_count,
        };

        format.print_output(&list)
    }
}

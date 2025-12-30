//! List episodes command

use crate::config::Config;
use crate::output::{Output, OutputFormat};
use memory_core::SelfLearningMemory;

#[derive(Debug, Clone, serde::Serialize)]
struct EpisodeSummary {
    episode_id: String,
    task_description: String,
    status: String,
    created_at: String,
    duration_ms: Option<u64>,
    steps_count: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
struct EpisodeList {
    episodes: Vec<EpisodeSummary>,
    total_count: usize,
}

impl Output for EpisodeList {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;
        writeln!(writer, "{}", "Episodes".green().bold())?;
        writeln!(writer, "Total: {} episodes\n", self.total_count)?;

        for (i, ep) in self.episodes.iter().enumerate() {
            writeln!(writer, "{}: [{}] {}", i + 1, ep.status, ep.task_description)?;
            writeln!(writer, "   ID: {} | Created: {}", ep.episode_id, ep.created_at)?;
            writeln!(
                writer,
                "   Steps: {} | Duration: {}ms",
                ep.steps_count,
                ep.duration_ms.map(|d| d.to_string()).unwrap_or_else(|| "N/A".to_string())
            )?;
            writeln!(writer)?;
        }
        Ok(())
    }
}

use super::EpisodeStatus;

pub async fn list_episodes(
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] task_type: Option<String>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] limit: usize,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] status: Option<EpisodeStatus>,
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
            Some(EpisodeStatus::Completed) => Some(true),
            _ => None, // For InProgress or None, we filter client-side
        };

        // Get more than needed so we can apply client-side filters without losing items
        let mut episodes = memory
            .list_episodes(Some(limit * 2), None, completed_only)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list episodes: {}", e))?;

        // Apply client-side filtering for in-progress and task type
        episodes.retain(|episode| match status {
            Some(EpisodeStatus::InProgress) => !episode.is_complete(),
            Some(EpisodeStatus::Completed) => episode.is_complete(),
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

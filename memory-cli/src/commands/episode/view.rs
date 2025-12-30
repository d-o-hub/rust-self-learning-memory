//! View episode command

use crate::config::Config;
#[cfg(feature = "turso")]
use crate::errors::{helpers, EnhancedError};
use crate::output::{Output, OutputFormat};
use memory_core::SelfLearningMemory;
use uuid::Uuid;

pub async fn view_episode(
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] episode_id: String,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] memory: &SelfLearningMemory,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _config: &Config,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] format: OutputFormat,
) -> anyhow::Result<()> {
    #[cfg(feature = "turso")]
    let episode_id_str = episode_id.clone();

    // Check if storage features are enabled
    #[cfg(not(feature = "turso"))]
    return Err(anyhow::anyhow!(
        "Turso storage feature not enabled. Use --features turso to enable."
    ));

    #[cfg(feature = "turso")]
    {
        // Parse episode ID
        let episode_uuid = Uuid::parse_str(&episode_id).context_with_help(
            &format!("Invalid episode ID format: {}", episode_id),
            helpers::INVALID_INPUT_HELP,
        )?;

        // Use the pre-initialized memory system
        // Get the episode
        let episode = memory.get_episode(episode_uuid).await.context_with_help(
            &format!("Episode not found: {}", episode_id_str),
            helpers::EPISODE_NOT_FOUND_HELP,
        )?;

        // Create a detailed view
        #[derive(Debug, serde::Serialize)]
        struct EpisodeDetail {
            episode_id: String,
            task_description: String,
            task_type: String,
            context: serde_json::Value,
            status: String,
            created_at: String,
            completed_at: Option<String>,
            duration_ms: Option<i64>,
            steps_count: usize,
            steps: Vec<serde_json::Value>,
            outcome: Option<serde_json::Value>,
            reward: Option<serde_json::Value>,
            reflection: Option<serde_json::Value>,
            patterns_count: usize,
            heuristics_count: usize,
        }

        impl Output for EpisodeDetail {
            fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
                use colored::*;

                writeln!(writer, "{}", "Episode Details".bold().underline())?;
                writeln!(writer, "ID: {}", self.episode_id.dimmed())?;
                writeln!(writer, "Task: {}", self.task_description)?;
                writeln!(writer, "Type: {}", self.task_type)?;
                writeln!(
                    writer,
                    "Status: {}",
                    if self.completed_at.is_some() {
                        "Completed".green()
                    } else {
                        "In Progress".yellow()
                    }
                )?;
                writeln!(writer, "Created: {}", self.created_at)?;

                if let Some(completed) = &self.completed_at {
                    writeln!(writer, "Completed: {}", completed)?;
                }

                if let Some(duration) = self.duration_ms {
                    writeln!(writer, "Duration: {}ms", duration)?;
                }

                writeln!(writer, "Steps: {}", self.steps_count)?;
                writeln!(writer, "Patterns: {}", self.patterns_count)?;
                writeln!(writer, "Heuristics: {}", self.heuristics_count)?;

                Ok(())
            }
        }

        let is_completed = episode.is_complete();
        let detail = EpisodeDetail {
            episode_id: episode.episode_id.to_string(),
            task_description: episode.task_description,
            task_type: episode.task_type.to_string(),
            context: serde_json::to_value(&episode.context)?,
            status: if is_completed {
                "completed"
            } else {
                "in_progress"
            }
            .to_string(),
            created_at: episode.start_time.to_rfc3339(),
            completed_at: episode.end_time.map(|t| t.to_rfc3339()),
            duration_ms: episode
                .end_time
                .map(|end| (end - episode.start_time).num_milliseconds()),
            steps_count: episode.steps.len(),
            steps: episode
                .steps
                .iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()?,
            outcome: episode
                .outcome
                .as_ref()
                .map(serde_json::to_value)
                .transpose()?,
            reward: episode
                .reward
                .as_ref()
                .map(serde_json::to_value)
                .transpose()?,
            reflection: episode
                .reflection
                .as_ref()
                .map(serde_json::to_value)
                .transpose()?,
            patterns_count: episode.patterns.len(),
            heuristics_count: episode.heuristics.len(),
        };

        format.print_output(&detail)
    }
}

//! View episode command implementation

use crate::config::Config;
#[cfg(feature = "turso")]
use crate::errors::{helpers, EnhancedError};
#[cfg(feature = "turso")]
use crate::output::Output;
use crate::output::OutputFormat;
use memory_core::SelfLearningMemory;

pub async fn view_episode(
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] episode_id: String,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] memory: &SelfLearningMemory,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _config: &Config,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] format: OutputFormat,
) -> anyhow::Result<()> {
    #[cfg(feature = "turso")]
    let episode_id_str = episode_id.clone();
    #[allow(unused_imports)]
    use uuid::Uuid;

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

                // Only show context if it's not default/empty
                if self.context != serde_json::json!({}) {
                    writeln!(writer, "\nContext:")?;
                    writeln!(writer, "{:#?}", self.context)?;
                }

                // Show outcome if available
                if let Some(ref outcome) = self.outcome {
                    writeln!(writer, "\nOutcome:")?;
                    writeln!(writer, "{:#?}", outcome)?;
                }

                // Show reflection if available
                if let Some(ref reflection) = self.reflection {
                    writeln!(writer, "\nReflection:")?;
                    writeln!(writer, "{}", reflection)?;
                }

                // Show reward if available
                if let Some(ref reward) = self.reward {
                    writeln!(writer, "\nReward:")?;
                    writeln!(writer, "{:#?}", reward)?;
                }

                Ok(())
            }
        }

        let outcome = match &episode.outcome {
            Some(outcome) => Some(serde_json::json!({
                "type": match outcome {
                    memory_core::TaskOutcome::Success { .. } => "success",
                    memory_core::TaskOutcome::PartialSuccess { .. } => "partial_success",
                    memory_core::TaskOutcome::Failure { .. } => "failure",
                },
            })),
            None => None,
        };

        let episode_detail = EpisodeDetail {
            episode_id: episode.episode_id.to_string(),
            task_description: episode.task_description.clone(),
            task_type: format!("{:?}", episode.task_type),
            context: serde_json::to_value(&episode.context).unwrap_or_default(),
            status: if episode.is_complete() {
                "completed".to_string()
            } else {
                "in_progress".to_string()
            },
            created_at: episode.start_time.to_rfc3339(),
            completed_at: episode.end_time.map(|t| t.to_rfc3339()),
            duration_ms: episode
                .end_time
                .map(|end| (end - episode.start_time).num_milliseconds()),
            steps_count: episode.steps.len(),
            steps: episode
                .steps
                .iter()
                .enumerate()
                .map(|(i, step)| {
                    let (success, observation) = match &step.result {
                        Some(memory_core::ExecutionResult::Success { output }) => {
                            (true, Some(output.clone()))
                        }
                        Some(memory_core::ExecutionResult::Error { message }) => {
                            (false, Some(message.clone()))
                        }
                        Some(memory_core::ExecutionResult::Timeout) => {
                            (false, Some("Timeout".to_string()))
                        }
                        None => (false, None),
                    };
                    serde_json::json!({
                        "step": i + 1,
                        "tool": step.tool,
                        "action": step.action,
                        "success": success,
                        "latency_ms": step.latency_ms,
                        "tokens": step.tokens_used,
                        "observation": observation,
                    })
                })
                .collect(),
            outcome,
            reward: episode.reward.map(|r| {
                serde_json::json!({
                    "total": r.total,
                    "base": r.base,
                    "efficiency": r.efficiency,
                    "complexity_bonus": r.complexity_bonus,
                    "quality_multiplier": r.quality_multiplier,
                    "learning_bonus": r.learning_bonus,
                })
            }),
            reflection: episode.reflection.map(|r| {
                serde_json::json!({
                    "successes": r.successes,
                    "improvements": r.improvements,
                    "insights": r.insights,
                    "generated_at": r.generated_at,
                })
            }),
            patterns_count: episode.patterns.len(),
            heuristics_count: episode.heuristics.len(),
        };

        format.print_output(&episode_detail)
    }
}

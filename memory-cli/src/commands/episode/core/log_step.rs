//! Log step command implementation

use crate::config::Config;
#[cfg(feature = "turso")]
use crate::errors::{helpers, EnhancedError};
use crate::output::OutputFormat;
use memory_core::SelfLearningMemory;

#[allow(clippy::too_many_arguments)]
pub async fn log_step(
    episode_id: String,
    tool: String,
    action: String,
    success: bool,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] latency_ms: Option<u64>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] tokens: Option<u32>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] observation: Option<String>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] memory: &SelfLearningMemory,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _config: &Config,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    if dry_run {
        println!(
            "Would log step for episode {}: tool={}, action={}, success={}",
            episode_id, tool, action, success
        );
        return Ok(());
    }

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
        // Get the current episode to determine step number
        let episode = memory.get_episode(episode_uuid).await.context_with_help(
            &format!("Failed to retrieve episode: {}", episode_id),
            helpers::EPISODE_NOT_FOUND_HELP,
        )?;

        let step_number = episode.steps.len() + 1;

        // Create execution step
        let mut step = ExecutionStep::new(step_number, tool.clone(), action.clone());

        // Set result based on success flag
        step.result = Some(if success {
            ExecutionResult::Success {
                output: observation.unwrap_or_else(|| "Step completed successfully".to_string()),
            }
        } else {
            ExecutionResult::Error {
                message: observation.unwrap_or_else(|| "Step failed".to_string()),
            }
        });

        // Set optional metadata
        if let Some(latency) = latency_ms {
            step.metadata
                .insert("latency_ms".to_string(), latency.to_string());
        }
        if let Some(token_count) = tokens {
            step.metadata
                .insert("tokens".to_string(), token_count.to_string());
        }

        // Log the step
        memory.log_step(episode_uuid, step).await;

        // Return success
        #[derive(Debug, serde::Serialize)]
        struct LogStepResult {
            episode_id: String,
            step_number: usize,
            tool: String,
            action: String,
            success: bool,
            status: String,
        }

        impl Output for LogStepResult {
            fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
                use colored::*;
                writeln!(writer, "{}", "Step Logged".green().bold())?;
                writeln!(writer, "Episode: {}", self.episode_id.dimmed())?;
                writeln!(writer, "Step: {}", self.step_number)?;
                writeln!(writer, "Tool: {}", self.tool)?;
                writeln!(writer, "Action: {}", self.action)?;
                writeln!(
                    writer,
                    "Success: {}",
                    if self.success {
                        "Yes".green()
                    } else {
                        "No".red()
                    }
                )?;
                Ok(())
            }
        }

        let result = LogStepResult {
            episode_id: episode_id.clone(),
            step_number,
            tool: tool.clone(),
            action: action.clone(),
            success,
            status: "logged".to_string(),
        };

        format.print_output(&result)
    }
}

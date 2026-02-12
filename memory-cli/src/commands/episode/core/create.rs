//! Create episode command implementation

use crate::config::Config;
#[cfg(feature = "turso")]
use crate::errors::{helpers, EnhancedError};
#[cfg(feature = "turso")]
use crate::output::Output;
use crate::output::OutputFormat;
use memory_core::SelfLearningMemory;
#[cfg(feature = "turso")]
use memory_core::{TaskContext, TaskType};
use std::path::PathBuf;

#[allow(clippy::too_many_arguments)]
pub async fn create_episode(
    task: String,
    context: Option<PathBuf>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] memory: &SelfLearningMemory,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _config: &Config,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    if dry_run {
        println!("Would create episode with task: {}", task);
        if let Some(context_path) = context {
            println!("Would load context from: {}", context_path.display());
        }
        return Ok(());
    }

    // Check if storage features are enabled
    #[cfg(not(feature = "turso"))]
    return Err(anyhow::anyhow!(
        "Turso storage feature not enabled.\n\
         \nTo enable Turso storage support:\n\
         • Install with: cargo install --path memory-cli --features turso\n\
         • Or build with: cargo build --features turso\n\
         • For full features: cargo install --path memory-cli --features full\n\
         \nAlternatively, configure a different storage backend in your config file."
    ));

    #[cfg(feature = "turso")]
    {
        // Load context from file if provided
        let context_data = if let Some(context_path) = context {
            let content = std::fs::read_to_string(&context_path).context_with_help(
                &format!("Failed to read context file: {}", context_path.display()),
                helpers::INVALID_INPUT_HELP,
            )?;

            // Try to parse as JSON first, then YAML
            if let Ok(ctx) = serde_json::from_str::<TaskContext>(&content) {
                ctx
            } else {
                serde_yaml::from_str(&content).context_with_help(
                    &format!("Failed to parse context file: {}", context_path.display()),
                    helpers::INVALID_INPUT_HELP,
                )?
            }
        } else {
            TaskContext::default()
        };

        // Use the pre-initialized memory system
        // Start the episode
        let episode_id = memory
            .start_episode(task.clone(), context_data, TaskType::CodeGeneration)
            .await;

        // Output the result
        #[derive(Debug, serde::Serialize)]
        struct CreateResult {
            episode_id: String,
            task: String,
            status: String,
        }

        impl Output for CreateResult {
            fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
                use colored::*;
                writeln!(writer, "{}", "Episode Created".green().bold())?;
                writeln!(writer, "ID: {}", self.episode_id.dimmed())?;
                writeln!(writer, "Task: {}", self.task)?;
                writeln!(writer, "Status: {}", self.status.green())?;
                Ok(())
            }
        }

        let result = CreateResult {
            episode_id: episode_id.to_string(),
            task: task.clone(),
            status: "created".to_string(),
        };

        format.print_output(&result)
    }
}

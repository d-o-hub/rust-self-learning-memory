//! Complete episode command implementation

use super::types::TaskOutcome;
use crate::config::Config;
#[cfg(feature = "turso")]
use crate::errors::{helpers, EnhancedError};
#[cfg(feature = "turso")]
use crate::output::Output;
use crate::output::OutputFormat;
use memory_core::SelfLearningMemory;

pub async fn complete_episode(
    episode_id: String,
    outcome: TaskOutcome,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] memory: &SelfLearningMemory,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _config: &Config,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    #[allow(unused_imports)]
    use memory_core::TaskOutcome as CoreTaskOutcome;
    #[allow(unused_imports)]
    use uuid::Uuid;

    if dry_run {
        println!(
            "Would complete episode {} with outcome: {:?}",
            episode_id, outcome
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
        // Map CLI outcome to core outcome
        let core_outcome = match outcome {
            TaskOutcome::Success => CoreTaskOutcome::Success {
                verdict: "Task completed successfully via CLI".to_string(),
                artifacts: vec![],
            },
            TaskOutcome::PartialSuccess => CoreTaskOutcome::PartialSuccess {
                verdict: "Task completed with partial success via CLI".to_string(),
                completed: vec!["Marked as partial success by user".to_string()],
                failed: vec![],
            },
            TaskOutcome::Failure => CoreTaskOutcome::Failure {
                reason: "Task failed via CLI".to_string(),
                error_details: Some("Marked as failed by user".to_string()),
            },
        };

        // Complete the episode
        memory
            .complete_episode(episode_uuid, core_outcome)
            .await
            .context_with_help(
                &format!("Failed to complete episode: {}", episode_id),
                helpers::EPISODE_NOT_FOUND_HELP,
            )?;

        // Return success
        #[derive(Debug, serde::Serialize)]
        struct CompleteResult {
            episode_id: String,
            status: String,
            outcome: String,
        }

        impl Output for CompleteResult {
            fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
                use colored::*;
                writeln!(writer, "{}", "Episode Completed".green().bold())?;
                writeln!(writer, "Episode: {}", self.episode_id.dimmed())?;
                writeln!(writer, "Status: {}", self.status.green())?;
                writeln!(writer, "Outcome: {}", self.outcome)?;
                Ok(())
            }
        }

        let result = CompleteResult {
            episode_id: episode_id.clone(),
            status: "completed".to_string(),
            outcome: format!("{:?}", outcome),
        };

        format.print_output(&result)
    }
}

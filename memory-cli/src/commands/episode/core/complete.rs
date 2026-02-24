//! Complete episode command implementation

use super::types::TaskOutcome;
use crate::config::Config;
#[cfg(feature = "turso")]
use crate::errors::{EnhancedError, helpers};
#[cfg(feature = "turso")]
use crate::output::Output;
use crate::output::OutputFormat;
use memory_core::SelfLearningMemory;

pub async fn complete_episode(
    episode_id: String,
    outcome: TaskOutcome,
    memory: &SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    use memory_core::TaskOutcome as CoreTaskOutcome;
    use uuid::Uuid;

    if dry_run {
        println!(
            "Would complete episode {} with outcome: {:?}",
            episode_id, outcome
        );
        return Ok(());
    }

    // Parse episode ID
    let episode_uuid = Uuid::parse_str(&episode_id)
        .map_err(|_| anyhow::anyhow!("Invalid episode ID format: {}", episode_id))?;

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
        .get_episode(episode_uuid)
        .await
        .map_err(|e| anyhow::anyhow!("Episode not found {}: {}", episode_id, e))?;

    memory
        .complete_episode(episode_uuid, core_outcome)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to complete episode {}: {}", episode_id, e))?;

    let outcome_str = format!("{:?}", outcome);

    #[cfg(feature = "turso")]
    {
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
            outcome: outcome_str.clone(),
        };

        format.print_output(&result)
    }

    #[cfg(not(feature = "turso"))]
    {
        match format {
            OutputFormat::Json => {
                let result = serde_json::json!({
                    "episode_id": episode_id,
                    "status": "completed",
                    "outcome": outcome_str,
                });
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            OutputFormat::Yaml => {
                let result = serde_json::json!({
                    "episode_id": episode_id,
                    "status": "completed",
                    "outcome": outcome_str,
                });
                println!("{}", serde_yaml::to_string(&result)?);
            }
            OutputFormat::Human => {
                println!("Episode completed: {}", episode_id);
                println!("Status: completed");
                println!("Outcome: {}", outcome_str);
            }
        }

        Ok(())
    }
}

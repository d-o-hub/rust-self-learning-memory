//! Episode deletion command

use crate::config::Config;
use crate::output::OutputFormat;
use memory_core::SelfLearningMemory;
use serde::Serialize;
use std::io::Write;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct DeletionResult {
    pub success: bool,
    pub episode_id: String,
    pub message: String,
}

impl crate::output::Output for DeletionResult {
    fn write_human<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        if self.success {
            writeln!(writer, "✓ {}", self.message)?;
        } else {
            writeln!(writer, "✗ {}", self.message)?;
        }
        Ok(())
    }
}

pub async fn delete_episode(
    episode_id: String,
    memory: &SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    let uuid = Uuid::parse_str(&episode_id)
        .map_err(|_| anyhow::anyhow!("Invalid episode ID format: {}", episode_id))?;

    if dry_run {
        println!("[DRY RUN] Would delete episode: {}", episode_id);
        return Ok(());
    }

    let result = delete_episode_by_id(memory, uuid).await?;
    format.print_output(&result)
}

async fn delete_episode_by_id(
    memory: &SelfLearningMemory,
    episode_id: Uuid,
) -> anyhow::Result<DeletionResult> {
    match memory.get_episode(episode_id).await {
        Ok(_) => {
            memory.delete_episode(episode_id).await?;

            Ok(DeletionResult {
                success: true,
                episode_id: episode_id.to_string(),
                message: format!("Successfully deleted episode: {}", episode_id),
            })
        }
        Err(_) => {
            anyhow::bail!("Episode not found: {}", episode_id);
        }
    }
}

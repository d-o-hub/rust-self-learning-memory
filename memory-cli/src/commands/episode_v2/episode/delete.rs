//! Episode deletion command

use memory_core::SelfLearningMemory;
use serde::Serialize;
use std::io::Write;
use uuid::Uuid;

/// Result of episode deletion
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

/// Delete an episode permanently
///
/// This command removes an episode from all storage backends (in-memory, cache, and durable storage).
/// **Warning**: This operation cannot be undone.
///
/// # Arguments
///
/// * `memory` - Reference to the memory system
/// * `episode_id` - UUID of the episode to delete
///
/// # Returns
///
/// `Ok(DeletionResult)` if deletion succeeds, or an error if the operation fails.
///
/// # Errors
///
/// Returns error if the episode doesn't exist or deletion fails.
pub async fn delete_episode(
    memory: &SelfLearningMemory,
    episode_id: Uuid,
) -> anyhow::Result<DeletionResult> {
    // Verify episode exists before deletion
    match memory.get_episode(episode_id).await {
        Ok(_) => {
            // Episode exists, proceed with deletion
            memory.delete_episode(episode_id).await?;

            Ok(DeletionResult {
                success: true,
                episode_id: episode_id.to_string(),
                message: format!("Successfully deleted episode: {}", episode_id),
            })
        }
        Err(_) => {
            // Episode not found
            anyhow::bail!("Episode not found: {}", episode_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory_core::{TaskContext, TaskType};

    #[tokio::test]
    async fn test_delete_existing_episode() {
        let memory = SelfLearningMemory::new();

        // Create an episode
        let episode_id = memory
            .start_episode(
                "Test episode".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        // Delete it
        let result = delete_episode(&memory, episode_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);

        // Verify it's gone
        assert!(memory.get_episode(episode_id).await.is_err());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_episode() {
        let memory = SelfLearningMemory::new();

        // Try to delete non-existent episode
        let fake_id = Uuid::new_v4();
        let result = delete_episode(&memory, fake_id).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_delete_episode_idempotent() {
        let memory = SelfLearningMemory::new();

        // Create and delete an episode
        let episode_id = memory
            .start_episode(
                "Test episode".to_string(),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        delete_episode(&memory, episode_id).await.unwrap();

        // Try to delete again - should fail
        let result = delete_episode(&memory, episode_id).await;
        assert!(result.is_err());
    }
}

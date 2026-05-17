//! Procedural memory retrieval implementation

use crate::types::{ProceduralMemory, TaskType};
use tracing::{debug, instrument};

use super::super::SelfLearningMemory;

impl SelfLearningMemory {
    /// Retrieve relevant procedural memories (learned skills/playbooks) for a task.
    ///
    /// # Arguments
    ///
    /// * `task_type` - Type of task being performed
    /// * `limit` - Maximum number of skills to return
    ///
    /// # Returns
    ///
    /// Vector of relevant procedural memories
    #[instrument(skip(self))]
    pub async fn retrieve_procedural_memories(
        &self,
        task_type: TaskType,
        limit: usize,
    ) -> Vec<ProceduralMemory> {
        debug!(task_type = ?task_type, limit = limit, "Retrieving procedural memories");

        // Try cache storage first
        if let Some(cache) = &self.cache_storage {
            match cache
                .query_procedural_memories(task_type, Some(limit))
                .await
            {
                Ok(memories) if !memories.is_empty() => return memories,
                Ok(_) => {}
                Err(e) => debug!("Failed to retrieve procedural memories from cache: {}", e),
            }
        }

        // Fall back to durable storage
        if let Some(turso) = &self.turso_storage {
            match turso
                .query_procedural_memories(task_type, Some(limit))
                .await
            {
                Ok(memories) => return memories,
                Err(e) => debug!("Failed to retrieve procedural memories from turso: {}", e),
            }
        }

        Vec::new()
    }
}

//! Collaboration methods for `SelfLearningMemory` (WG-126).

use super::SelfLearningMemory;
use crate::error::Result;
use crate::learning::distillation::TrajectoryRepresentation;
use crate::types::TaskType;
use uuid::Uuid;

impl SelfLearningMemory {
    /// Distill a trajectory for a given episode.
    pub async fn distill_trajectory(&self, episode_id: Uuid) -> Result<TrajectoryRepresentation> {
        let episode = self.get_episode(episode_id).await?;
        Ok(self.trajectory_distiller.distill(&episode))
    }

    /// Sync a collaborative prototype for a task type.
    pub async fn sync_collaborative_prototype(
        &self,
        task_type: TaskType,
        prototype: TrajectoryRepresentation,
    ) {
        let mut manager = self.collaboration_manager.write().await;
        manager.import_prototype(task_type, prototype);
    }

    /// Get a collaborative prototype for a task type.
    pub async fn get_collaborative_prototype(
        &self,
        task_type: TaskType,
    ) -> Option<TrajectoryRepresentation> {
        let manager = self.collaboration_manager.read().await;
        manager.prototypes.get(&task_type).cloned()
    }
}

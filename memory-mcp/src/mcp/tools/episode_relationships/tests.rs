//! Tests for episode relationship tools.

use crate::mcp::tools::episode_relationships::{
    AddEpisodeRelationshipInput, CheckRelationshipExistsInput, DependencyGraphInput,
    EpisodeRelationshipTools, FindRelatedEpisodesInput, GetEpisodeRelationshipsInput,
    GetTopologicalOrderInput, RemoveEpisodeRelationshipInput, ValidateNoCyclesInput,
};
use memory_core::SelfLearningMemory;
use memory_core::{TaskContext, TaskType};
use std::sync::Arc;

fn create_test_memory() -> Arc<SelfLearningMemory> {
    Arc::new(SelfLearningMemory::new())
}

async fn create_test_episode(memory: &SelfLearningMemory, description: &str) -> uuid::Uuid {
    memory
        .start_episode(
            description.to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await
}

mod cases;

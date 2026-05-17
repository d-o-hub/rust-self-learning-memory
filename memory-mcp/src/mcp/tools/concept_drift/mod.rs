use anyhow::Result;
use do_memory_core::SelfLearningMemory;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Input for concept drift analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptDriftInput {
    /// Parent episode ID (the concept identifier)
    pub parent_id: String,
}

/// Result of concept drift analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptDriftResult {
    pub parent_id: String,
    pub versions_analyzed: usize,
    pub changepoints: Vec<do_memory_core::patterns::Changepoint>,
    pub drift_detected: bool,
}

pub struct ConceptDriftTool {
    memory: Arc<SelfLearningMemory>,
}

impl ConceptDriftTool {
    pub fn new(memory: Arc<SelfLearningMemory>) -> Self {
        Self { memory }
    }

    pub async fn execute(&self, input: ConceptDriftInput) -> Result<ConceptDriftResult> {
        let parent_id = Uuid::parse_str(&input.parent_id)?;

        // Fetch all versions
        let versions = if let Some(turso) = self.memory.turso_storage.as_ref() {
            turso.get_episode_versions(parent_id).await?
        } else if let Some(cache) = self.memory.cache_storage.as_ref() {
            cache.get_episode_versions(parent_id).await?
        } else {
            Vec::new()
        };

        let mut analyzer = do_memory_core::patterns::drift::DriftAnalyzer::new();
        let changepoints = analyzer.analyze_drift(&versions)?;
        let drift_detected = !changepoints.is_empty();

        Ok(ConceptDriftResult {
            parent_id: input.parent_id,
            versions_analyzed: versions.len(),
            changepoints,
            drift_detected,
        })
    }
}

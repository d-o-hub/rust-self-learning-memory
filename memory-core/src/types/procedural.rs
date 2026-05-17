//! Procedural memory for learned heuristics-as-skills

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{Evidence, TaskType};

/// Procedural memory representing a learned skill or playbook
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProceduralMemory {
    /// Unique identifier for this procedural memory
    pub id: Uuid,
    /// Name of the skill or playbook
    pub name: String,
    /// Detailed description of the skill
    pub description: String,
    /// The type of task this skill applies to
    pub task_type: TaskType,
    /// The steps involved in this procedural skill
    pub steps: Vec<ProceduralStep>,
    /// Evidence and success metrics for this skill
    pub evidence: Evidence,
    /// Confidence level in this skill (0.0 to 1.0)
    pub confidence: f32,
    /// When this skill was first identified
    pub created_at: DateTime<Utc>,
    /// When this skill was last updated or refined
    pub updated_at: DateTime<Utc>,
    /// Version of this skill (increments on refinement)
    pub version: u32,
}

/// A single step within a procedural memory
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProceduralStep {
    /// Step order/sequence
    pub sequence: u32,
    /// Description of the action to take in this step
    pub action: String,
    /// Expected outcome or state change
    pub expected_outcome: String,
    /// Success rate of this specific step if tracked independently
    pub success_rate: f32,
}

impl ProceduralMemory {
    /// Create a new procedural memory
    pub fn new(
        name: String,
        description: String,
        task_type: TaskType,
        steps: Vec<ProceduralStep>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            task_type,
            steps,
            evidence: Evidence {
                episode_ids: Vec::new(),
                success_rate: 0.0,
                sample_size: 0,
            },
            confidence: 0.1, // Initial low confidence
            created_at: now,
            updated_at: now,
            version: 1,
        }
    }

    /// Update skill metrics with new execution evidence
    pub fn record_execution(&mut self, episode_id: Uuid, success: bool) {
        self.evidence.episode_ids.push(episode_id);
        self.evidence.sample_size += 1;

        // Recalculate global success rate
        let prev_success_count =
            self.evidence.success_rate * (self.evidence.sample_size - 1) as f32;
        let new_success_count = if success {
            prev_success_count + 1.0
        } else {
            prev_success_count
        };
        self.evidence.success_rate = new_success_count / self.evidence.sample_size as f32;

        // Adjust confidence based on sample size and success rate
        // Simple heuristic: log10 of sample size (capped at 1.0) weighted by success
        let sample_weight = (self.evidence.sample_size as f32).log10().min(1.0);
        self.confidence = self.evidence.success_rate * sample_weight;

        self.updated_at = Utc::now();
    }
}

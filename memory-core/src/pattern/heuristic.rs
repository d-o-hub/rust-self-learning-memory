//! Heuristic rules learned from patterns

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::Evidence;

/// Heuristic rule learned from patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Heuristic {
    /// Unique heuristic ID
    pub heuristic_id: Uuid,
    /// Condition to check (as natural language or code)
    pub condition: String,
    /// Recommended action
    pub action: String,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
    /// Evidence supporting this heuristic
    pub evidence: Evidence,
    /// When created
    pub created_at: DateTime<Utc>,
    /// Last updated
    pub updated_at: DateTime<Utc>,
}

impl Heuristic {
    /// Create a new heuristic
    pub fn new(condition: String, action: String, confidence: f32) -> Self {
        let now = Utc::now();
        Self {
            heuristic_id: Uuid::new_v4(),
            condition,
            action,
            confidence,
            evidence: Evidence {
                episode_ids: Vec::new(),
                success_rate: 0.0,
                sample_size: 0,
            },
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the heuristic with new evidence
    pub fn update_evidence(&mut self, episode_id: Uuid, success: bool) {
        self.evidence.episode_ids.push(episode_id);
        self.evidence.sample_size += 1;

        // Recalculate success rate
        let successes = if success {
            (self.evidence.success_rate * (self.evidence.sample_size - 1) as f32) + 1.0
        } else {
            self.evidence.success_rate * (self.evidence.sample_size - 1) as f32
        };

        self.evidence.success_rate = successes / self.evidence.sample_size as f32;
        self.updated_at = Utc::now();
    }
}

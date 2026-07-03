//! Procedural memory type definitions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::episode::PatternId;
use crate::memory::playbook::PlaybookStep;
use crate::patterns::PatternEffectiveness;
use crate::types::TaskContext;

/// Procedural memory represents learned heuristics-as-skills.
///
/// It bridges the gap between raw patterns and actionable workflows (playbooks).
/// Inspired by ParamAgent (2026) three-tier memory architecture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProceduralMemory {
    /// Unique identifier for this procedural memory
    pub id: Uuid,
    /// Name of the skill or heuristic
    pub name: String,
    /// Detailed description of what this skill achieves
    pub description: String,
    /// Context where this skill is most applicable
    pub context: TaskContext,
    /// Ordered steps to execute this skill
    pub steps: Vec<PlaybookStep>,
    /// Real-world effectiveness tracking
    pub effectiveness: PatternEffectiveness,
    /// Episodes that contributed to the learning of this skill
    pub source_episodes: Vec<Uuid>,
    /// Patterns that contributed to the learning of this skill
    pub source_patterns: Vec<PatternId>,
    /// When this procedural memory was created
    pub created_at: DateTime<Utc>,
    /// When this procedural memory was last updated or used
    pub updated_at: DateTime<Utc>,
}

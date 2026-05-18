//! Procedural memory management
//!
//! Procedural memory stores learned heuristics-as-skills, providing actionable
//! workflows synthesized from past experiences.

mod types;

pub use types::ProceduralMemory;

#[cfg(test)]
mod tests;

use crate::types::TaskContext;
use chrono::Utc;
use uuid::Uuid;

impl ProceduralMemory {
    /// Create a new procedural memory
    pub fn new(
        name: String,
        description: String,
        context: TaskContext,
        steps: Vec<crate::memory::playbook::PlaybookStep>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            context,
            steps,
            effectiveness: crate::pattern::PatternEffectiveness::default(),
            source_episodes: Vec::new(),
            source_patterns: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if this procedural memory is relevant to a given context
    #[must_use]
    pub fn is_relevant_to(&self, query_context: &TaskContext) -> bool {
        // Match on domain
        if self.context.domain == query_context.domain {
            return true;
        }

        // Match on language
        if self.context.language == query_context.language
            && self.context.language.is_some()
        {
            return true;
        }

        // Match on tags
        let common_tags: Vec<_> = self.context
            .tags
            .iter()
            .filter(|t| query_context.tags.contains(t))
            .collect();

        if !common_tags.is_empty() {
            return true;
        }

        false
    }

    /// Record that this procedural memory was retrieved in a query
    pub fn record_retrieval(&mut self) {
        self.effectiveness.record_retrieval();
        self.updated_at = Utc::now();
    }

    /// Record that this procedural memory was applied with an outcome
    pub fn record_application(&mut self, success: bool, reward_delta: f32) {
        self.effectiveness.record_application(success, reward_delta);
        self.updated_at = Utc::now();
    }

    /// Add a source episode ID
    pub fn add_source_episode(&mut self, episode_id: Uuid) {
        if !self.source_episodes.contains(&episode_id) {
            self.source_episodes.push(episode_id);
            self.updated_at = Utc::now();
        }
    }

    /// Add a source pattern ID
    pub fn add_source_pattern(&mut self, pattern_id: crate::episode::PatternId) {
        if !self.source_patterns.contains(&pattern_id) {
            self.source_patterns.push(pattern_id);
            self.updated_at = Utc::now();
        }
    }
}

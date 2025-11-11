//! # Reflection Generator
//!
//! Generates structured reflections from completed episodes by analyzing:
//! - Successful strategies and patterns
//! - Areas for improvement
//! - Key insights and learnings
//!
//! ## Example
//!
//! ```
//! use memory_core::reflection::ReflectionGenerator;
//! use memory_core::{Episode, TaskContext, TaskType, TaskOutcome, ExecutionStep};
//!
//! let context = TaskContext::default();
//! let mut episode = Episode::new("Test task".to_string(), context, TaskType::Testing);
//!
//! let mut step = ExecutionStep::new(1, "test_runner".to_string(), "Run tests".to_string());
//! episode.add_step(step);
//!
//! episode.complete(TaskOutcome::Success {
//!     verdict: "All tests passed".to_string(),
//!     artifacts: vec!["test_results.json".to_string()],
//! });
//!
//! let generator = ReflectionGenerator::new();
//! let reflection = generator.generate(&episode);
//!
//! assert!(!reflection.successes.is_empty() || !reflection.insights.is_empty());
//! ```

mod helpers;
mod improvement_analyzer;
mod insight_generator;
mod success_analyzer;

#[cfg(test)]
mod tests;

use crate::episode::Episode;
use crate::types::Reflection;
use chrono::Utc;
use tracing::{debug, instrument};

/// Maximum items in each reflection category
const MAX_REFLECTION_ITEMS: usize = 5;

/// Generator for episode reflections
#[derive(Clone)]
pub struct ReflectionGenerator {
    /// Maximum items per category
    max_items: usize,
}

impl Default for ReflectionGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ReflectionGenerator {
    /// Create a new reflection generator
    pub fn new() -> Self {
        Self {
            max_items: MAX_REFLECTION_ITEMS,
        }
    }

    /// Create a generator with custom max items
    pub fn with_max_items(max_items: usize) -> Self {
        Self { max_items }
    }

    /// Generate reflection from a completed episode
    #[instrument(skip(self, episode), fields(episode_id = %episode.episode_id))]
    pub fn generate(&self, episode: &Episode) -> Reflection {
        let mut successes = success_analyzer::identify_successes(episode, self.max_items);
        let mut improvements = improvement_analyzer::identify_improvements(episode, self.max_items);
        let mut insights = insight_generator::generate_insights(episode, self.max_items);

        // Add sophisticated analysis
        successes.extend(success_analyzer::analyze_success_patterns(episode));
        improvements.extend(improvement_analyzer::analyze_improvement_opportunities(
            episode,
        ));
        insights.extend(insight_generator::generate_contextual_insights(episode));

        // Limit to max items after combining all sources
        successes.truncate(self.max_items);
        improvements.truncate(self.max_items);
        insights.truncate(self.max_items);

        debug!(
            successes_count = successes.len(),
            improvements_count = improvements.len(),
            insights_count = insights.len(),
            "Generated reflection"
        );

        Reflection {
            successes,
            improvements,
            insights,
            generated_at: Utc::now(),
        }
    }
}

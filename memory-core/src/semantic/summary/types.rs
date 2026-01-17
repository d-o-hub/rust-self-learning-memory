//! Types for semantic summarization.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Condensed semantic summary of an episode.
///
/// Compresses a full episode into a concise, searchable summary with:
/// - Summary text (100-200 words)
/// - Key concepts for indexing
/// - Critical steps highlighting important actions
/// - Optional embedding vector for semantic search
///
/// # Fields
///
/// * `episode_id` - Original episode identifier
/// * `summary_text` - Concise 100-200 word summary
/// * `key_concepts` - Important concepts extracted (10-20 items)
/// * `key_steps` - Critical execution steps (3-5 items)
/// * `summary_embedding` - Optional embedding vector for semantic retrieval
/// * `created_at` - When this summary was created
///
/// # Examples
///
/// ```
/// use memory_core::semantic::EpisodeSummary;
/// use uuid::Uuid;
/// use chrono::Utc;
///
/// let summary = EpisodeSummary {
///     episode_id: Uuid::new_v4(),
///     summary_text: "Task: Implement user authentication. Steps: Analyzed requirements, \
///                    designed auth flow, implemented login/logout, added tests. \
///                    Outcome: Successfully implemented with 95% test coverage.".to_string(),
///     key_concepts: vec![
///         "authentication".to_string(),
///         "security".to_string(),
///         "rust".to_string(),
///     ],
///     key_steps: vec![
///         "Step 1: planner - Analyzed requirements".to_string(),
///         "Step 3: code_generator - Implemented login/logout".to_string(),
///         "Step 5: tester - Added comprehensive tests".to_string(),
///     ],
///     summary_embedding: None,
///     created_at: Utc::now(),
/// };
///
/// assert!(summary.summary_text.len() >= 100);
/// assert!(summary.summary_text.len() <= 500); // With some tolerance
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpisodeSummary {
    /// Original episode identifier
    pub episode_id: Uuid,
    /// Concise summary text (100-200 words)
    pub summary_text: String,
    /// Important concepts extracted
    pub key_concepts: Vec<String>,
    /// Critical steps (3-5)
    pub key_steps: Vec<String>,
    /// Optional embedding vector for semantic search
    pub summary_embedding: Option<Vec<f32>>,
    /// When summary was created
    pub created_at: DateTime<Utc>,
}

impl EpisodeSummary {
    /// Create a new empty summary for an episode.
    #[must_use]
    pub fn new(episode_id: Uuid) -> Self {
        Self {
            episode_id,
            summary_text: String::new(),
            key_concepts: Vec::new(),
            key_steps: Vec::new(),
            summary_embedding: None,
            created_at: Utc::now(),
        }
    }

    /// Get the word count of the summary.
    #[must_use]
    pub fn word_count(&self) -> usize {
        self.summary_text.split_whitespace().count()
    }
}

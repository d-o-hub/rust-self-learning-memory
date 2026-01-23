//! Core filter types for episode filtering
//!
//! This module defines the primary types used for filtering and querying episodes.

use crate::search::{SearchField, SearchMode};
use crate::types::TaskType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::builder::EpisodeFilterBuilder;

/// Outcome type for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutcomeType {
    /// Only successful outcomes
    Success,
    /// Only partial success outcomes
    PartialSuccess,
    /// Only failure outcomes
    Failure,
}

/// Comprehensive filter for querying episodes
///
/// Provides rich filtering capabilities across multiple dimensions.
/// All filter criteria are combined with AND logic (must match all specified criteria).
///
/// # Examples
///
/// ```
/// use memory_core::EpisodeFilter;
/// use memory_core::TaskType;
/// use chrono::Utc;
///
/// // Filter for successful code generation episodes in the last week
/// let filter = EpisodeFilter::builder()
///     .task_types(vec![TaskType::CodeGeneration])
///     .success_only(true)
///     .date_from(Utc::now() - chrono::Duration::days(7))
///     .build();
///
/// // Filter by tags
/// let tagged_filter = EpisodeFilter::builder()
///     .with_any_tags(vec!["async".to_string(), "networking".to_string()])
///     .build();
///
/// // Filter archived episodes
/// let archived_filter = EpisodeFilter::builder()
///     .archived_only(true)
///     .build();
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct EpisodeFilter {
    /// Filter by tags - matches episodes with ANY of these tags
    pub with_any_tags: Option<Vec<String>>,

    /// Filter by tags - matches episodes with ALL of these tags
    pub with_all_tags: Option<Vec<String>>,

    /// Filter by task types
    pub task_types: Option<Vec<TaskType>>,

    /// Filter by domains
    pub domains: Option<Vec<String>>,

    /// Episodes started after this date
    pub date_from: Option<DateTime<Utc>>,

    /// Episodes started before this date
    pub date_to: Option<DateTime<Utc>>,

    /// Only include completed episodes
    pub completed_only: Option<bool>,

    /// Only include archived episodes
    pub archived_only: Option<bool>,

    /// Exclude archived episodes
    pub exclude_archived: Option<bool>,

    /// Only include successful episodes
    pub success_only: Option<bool>,

    /// Minimum reward score
    pub min_reward: Option<f32>,

    /// Maximum reward score
    pub max_reward: Option<f32>,

    /// Search text in task description (case-insensitive)
    pub search_text: Option<String>,

    /// Filter by specific outcome type
    pub outcome_type: Option<OutcomeType>,

    /// Search mode (exact, fuzzy, regex)
    pub search_mode: Option<SearchMode>,

    /// Fields to search in (when using advanced search)
    pub search_fields: Option<Vec<SearchField>>,
}

impl EpisodeFilter {
    /// Create a new empty filter (matches all episodes)
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a builder for fluent filter construction
    #[must_use]
    pub fn builder() -> EpisodeFilterBuilder {
        EpisodeFilterBuilder::default()
    }
}

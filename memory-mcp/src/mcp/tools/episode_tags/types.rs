//! Episode tagging tool types and input/output structures.

use serde::{Deserialize, Serialize};

/// Input parameters for adding tags to an episode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddEpisodeTagsInput {
    /// Episode ID to add tags to
    pub episode_id: String,
    /// Tags to add
    pub tags: Vec<String>,
}

/// Output from adding tags to an episode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddEpisodeTagsOutput {
    /// Whether operation was successful
    pub success: bool,
    /// Episode ID that was modified
    pub episode_id: String,
    /// Number of tags added
    pub tags_added: usize,
    /// Current tags on the episode
    pub current_tags: Vec<String>,
    /// Message describing the result
    pub message: String,
}

/// Input parameters for removing tags from an episode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveEpisodeTagsInput {
    /// Episode ID to remove tags from
    pub episode_id: String,
    /// Tags to remove
    pub tags: Vec<String>,
}

/// Output from removing tags from an episode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveEpisodeTagsOutput {
    /// Whether operation was successful
    pub success: bool,
    /// Episode ID that was modified
    pub episode_id: String,
    /// Number of tags removed
    pub tags_removed: usize,
    /// Current tags on the episode
    pub current_tags: Vec<String>,
    /// Message describing the result
    pub message: String,
}

/// Input parameters for setting episode tags (replace all)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetEpisodeTagsInput {
    /// Episode ID to set tags on
    pub episode_id: String,
    /// New tags to set (replaces all existing)
    pub tags: Vec<String>,
}

/// Output from setting episode tags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetEpisodeTagsOutput {
    /// Whether operation was successful
    pub success: bool,
    /// Episode ID that was modified
    pub episode_id: String,
    /// Number of tags set
    pub tags_set: usize,
    /// Current tags on the episode
    pub current_tags: Vec<String>,
    /// Message describing the result
    pub message: String,
}

/// Input parameters for getting episode tags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEpisodeTagsInput {
    /// Episode ID to get tags for
    pub episode_id: String,
}

/// Output from getting episode tags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEpisodeTagsOutput {
    /// Whether operation was successful
    pub success: bool,
    /// Episode ID queried
    pub episode_id: String,
    /// Tags on the episode
    pub tags: Vec<String>,
    /// Message describing the result
    pub message: String,
}

/// Input parameters for searching episodes by tags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEpisodesByTagsInput {
    /// Tags to search for
    pub tags: Vec<String>,
    /// Whether to require all tags (AND) or any tag (OR)
    pub require_all: Option<bool>,
    /// Maximum number of results
    pub limit: Option<usize>,
}

/// Output from searching episodes by tags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEpisodesByTagsOutput {
    /// Whether operation was successful
    pub success: bool,
    /// Number of episodes found
    pub count: usize,
    /// Episode results
    pub episodes: Vec<EpisodeTagResult>,
    /// Search criteria used
    pub search_criteria: String,
    /// Message describing the result
    pub message: String,
}

/// A single episode result from tag search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeTagResult {
    /// Episode ID
    pub episode_id: String,
    /// Task description
    pub task_description: String,
    /// Task type
    pub task_type: String,
    /// Tags on this episode
    pub tags: Vec<String>,
    /// Episode start time (Unix timestamp)
    pub start_time: i64,
    /// Episode end time (Unix timestamp, if completed)
    pub end_time: Option<i64>,
    /// Outcome (if completed)
    pub outcome: Option<String>,
}

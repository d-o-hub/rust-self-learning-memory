//! Episode command payload types (enums and serializable DTOs).
//!
//! Split out of `types.rs` to keep that module within the 500 LOC limit
//! (AGENTS.md); re-exported from `types` so existing paths keep working.

use clap::{Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum EpisodeStatus {
    /// Episode is currently in progress
    InProgress,
    /// Episode has been completed
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum TaskOutcome {
    /// Task completed successfully
    Success,
    /// Task completed with partial success
    PartialSuccess,
    /// Task failed
    Failure,
}

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum EpisodeSortOrder {
    /// Sort by creation date, newest first
    Newest,
    /// Sort by creation date, oldest first
    Oldest,
    /// Sort by duration (longest first)
    Duration,
    /// Sort by relevance (semantic search only)
    Relevance,
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum, Default)]
pub enum SearchSortOrder {
    /// Sort by relevance score (best match first)
    #[default]
    Relevance,
    /// Sort by creation date, newest first
    Newest,
    /// Sort by creation date, oldest first
    Oldest,
    /// Sort by duration (longest first)
    Duration,
    /// Sort by success rate (most successful first)
    Success,
}

#[derive(Debug, Clone, PartialEq, Subcommand)]
pub enum FilterCommands {
    /// Save a new filter
    Save {
        /// Filter name
        #[arg(short, long)]
        name: String,

        /// Filter by task type
        #[arg(short, long)]
        task_type: Option<String>,

        /// Filter by status
        #[arg(short, long)]
        status: Option<EpisodeStatus>,

        /// Filter episodes created after this date (ISO 8601)
        #[arg(long)]
        since: Option<String>,

        /// Filter episodes created before this date (ISO 8601)
        #[arg(long)]
        until: Option<String>,

        /// Filter by domain
        #[arg(long)]
        domain: Option<String>,

        /// Filter by tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,

        /// Filter by task outcome
        #[arg(long)]
        outcome: Option<TaskOutcome>,

        /// Default limit for this filter
        #[arg(short, long)]
        limit: Option<usize>,
    },
    /// List all saved filters
    List,
    /// Apply a saved filter
    Apply {
        /// Filter name to apply
        #[arg(value_name = "FILTER_NAME")]
        filter_name: String,

        /// Override limit
        #[arg(short, long)]
        limit: Option<usize>,

        /// Override offset for pagination
        #[arg(long)]
        offset: Option<usize>,
    },
    /// Delete a saved filter
    Delete {
        /// Filter name to delete
        #[arg(value_name = "FILTER_NAME")]
        filter_name: String,
    },
    /// Show a saved filter's configuration
    Show {
        /// Filter name to show
        #[arg(value_name = "FILTER_NAME")]
        filter_name: String,
    },
}

#[derive(Debug, Serialize)]
pub struct EpisodeSummary {
    pub episode_id: String,
    pub task_description: String,
    pub status: String,
    pub created_at: String,
    pub duration_ms: Option<u64>,
    pub steps_count: usize,
}

#[derive(Debug, Serialize)]
pub struct EpisodeList {
    pub episodes: Vec<EpisodeSummary>,
    pub total_count: usize,
}

#[derive(Debug, Serialize)]
pub struct EpisodeListFiltered {
    pub episodes: Vec<EpisodeSummary>,
    pub total_count: usize,
    pub filtered_count: usize,
    pub applied_filters: AppliedFilters,
}

#[derive(Debug, Serialize)]
pub struct AppliedFilters {
    pub task_type: Option<String>,
    pub status: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub domain: Option<String>,
    pub tags: Option<String>,
    pub outcome: Option<String>,
    pub sort: String,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Serialize)]
pub struct EpisodeDetail {
    pub episode_id: String,
    pub task_description: String,
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub duration_ms: Option<u64>,
    pub outcome: Option<String>,
    pub steps: Vec<EpisodeStep>,
}

#[derive(Debug, Serialize)]
pub struct EpisodeStep {
    pub step_number: usize,
    pub tool: String,
    pub action: String,
    pub success: bool,
    pub latency_ms: Option<u64>,
    pub tokens: Option<u32>,
    pub observation: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct EpisodeSearchResult {
    pub episode_id: String,
    pub task_description: String,
    pub status: String,
    pub relevance_score: f32,
    pub matched_terms: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedFilter {
    pub name: String,
    pub task_type: Option<String>,
    pub status: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub domain: Option<String>,
    pub tags: Option<String>,
    pub outcome: Option<String>,
    pub limit: Option<usize>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct FilterList {
    pub filters: Vec<SavedFilter>,
    pub total_count: usize,
}

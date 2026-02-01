//! Episode command types and output structures

use crate::commands::episode_v2::relationships::RelationshipCommands;
use clap::{Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Subcommand)]
pub enum EpisodeCommands {
    /// Create a new episode
    Create {
        /// Task description
        #[arg(short, long)]
        task: String,

        /// Context file (JSON)
        #[arg(short, long, value_name = "FILE")]
        context: Option<std::path::PathBuf>,
    },

    /// List episodes
    List {
        /// Filter by task type
        #[arg(short, long)]
        task_type: Option<String>,

        /// Maximum number of episodes to return
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Filter by status
        #[arg(short, long)]
        status: Option<EpisodeStatus>,

        /// Enable semantic search using embeddings
        #[arg(long)]
        semantic_search: Option<String>,

        /// Enable embeddings for this operation
        #[arg(long)]
        enable_embeddings: bool,

        /// Override embedding provider (openai, local, cohere, ollama, custom)
        #[arg(long)]
        embedding_provider: Option<String>,

        /// Override embedding model
        #[arg(long)]
        embedding_model: Option<String>,

        /// Filter episodes created after this date (ISO 8601)
        #[arg(long)]
        since: Option<String>,

        /// Filter episodes created before this date (ISO 8601)
        #[arg(long)]
        until: Option<String>,

        /// Sort order (newest, oldest, duration, relevance)
        #[arg(long, default_value = "newest")]
        sort: EpisodeSortOrder,

        /// Filter by domain (e.g., 'web-api', 'data-processing')
        #[arg(long)]
        domain: Option<String>,

        /// Filter by tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,

        /// Filter by task outcome
        #[arg(long)]
        outcome: Option<TaskOutcome>,

        /// Skip count (for pagination)
        #[arg(long, default_value = "0")]
        offset: usize,
    },

    /// Manage saved episode filters
    Filter {
        #[command(subcommand)]
        command: FilterCommands,
    },

    /// View episode details
    View {
        /// Episode ID
        #[arg(value_name = "EPISODE_ID")]
        episode_id: String,
    },

    /// Complete an episode
    Complete {
        /// Episode ID
        #[arg(value_name = "EPISODE_ID")]
        episode_id: String,

        /// Task outcome
        #[arg(value_enum)]
        outcome: TaskOutcome,
    },

    /// Delete an episode
    Delete {
        /// Episode ID
        #[arg(value_name = "EPISODE_ID")]
        episode_id: String,
    },

    /// Search episodes
    Search {
        /// Search query
        #[arg(value_name = "QUERY")]
        query: String,

        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Enable semantic search using embeddings
        #[arg(long)]
        semantic: bool,

        /// Enable embeddings for this operation
        #[arg(long)]
        enable_embeddings: bool,

        /// Override embedding provider (openai, local, cohere, ollama, custom)
        #[arg(long)]
        embedding_provider: Option<String>,

        /// Override embedding model
        #[arg(long)]
        embedding_model: Option<String>,

        /// Enable fuzzy search (typo-tolerant)
        #[arg(long)]
        fuzzy: bool,

        /// Fuzzy search similarity threshold (0.0-1.0, default: 0.8)
        #[arg(long, default_value = "0.8")]
        fuzzy_threshold: f64,

        /// Enable regex pattern matching
        #[arg(long)]
        regex: bool,

        /// Fields to search in (description, steps, outcome, tags, domain, all)
        #[arg(long, value_delimiter = ',')]
        search_fields: Option<Vec<String>>,

        /// Sort order for results (relevance, newest, oldest, duration, success)
        #[arg(long, default_value = "relevance")]
        sort: SearchSortOrder,
    },

    /// Log an execution step
    LogStep {
        /// Episode ID
        #[arg(value_name = "EPISODE_ID")]
        episode_id: String,

        /// Tool name
        #[arg(short, long)]
        tool: String,

        /// Action description
        #[arg(short, long)]
        action: String,

        /// Whether the step was successful
        #[arg(long)]
        success: bool,

        /// Latency in milliseconds
        #[arg(long)]
        latency_ms: Option<u64>,

        /// Token count
        #[arg(long)]
        tokens: Option<u32>,

        /// Step observation
        #[arg(short, long)]
        observation: Option<String>,
    },

    /// Retrieve multiple episodes by IDs (comma-separated)
    Bulk {
        /// Comma-separated episode IDs
        #[arg(value_name = "EPISODE_IDS")]
        episode_ids: String,
    },

    /// Manage episode relationships
    #[command(subcommand)]
    Relationships(RelationshipCommands),
}

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum EpisodeStatus {
    /// Episode is currently in progress
    InProgress,
    /// Episode has been completed
    Completed,
}

#[derive(Debug, Clone, PartialEq, ValueEnum)]
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
#[allow(dead_code)]
pub struct EpisodeSummary {
    pub episode_id: String,
    pub task_description: String,
    pub status: String,
    pub created_at: String,
    pub duration_ms: Option<u64>,
    pub steps_count: usize,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct EpisodeList {
    pub episodes: Vec<EpisodeSummary>,
    pub total_count: usize,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct EpisodeListFiltered {
    pub episodes: Vec<EpisodeSummary>,
    pub total_count: usize,
    pub filtered_count: usize,
    pub applied_filters: AppliedFilters,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct EpisodeSearchResult {
    pub episode_id: String,
    pub task_description: String,
    pub status: String,
    pub relevance_score: f32,
    pub matched_terms: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct FilterList {
    pub filters: Vec<SavedFilter>,
    pub total_count: usize,
}

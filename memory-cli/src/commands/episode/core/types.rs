//! Episode command types and output structures

use super::super::relationships::RelationshipCommands;
use clap::Subcommand;

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

        /// Domain for the episode (e.g., 'web-api', 'data-processing')
        #[arg(long)]
        domain: Option<String>,
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

    /// Force-fail an abandoned in-progress episode (ADR-075)
    ///
    /// Completes the episode with `TaskOutcome::Failure` using the same
    /// verify-after-write durability rules as `episode complete`.
    Fail {
        /// Episode ID
        #[arg(value_name = "EPISODE_ID")]
        episode_id: String,
    },

    /// Delete an episode
    Delete {
        /// Episode ID
        #[arg(value_name = "EPISODE_ID")]
        episode_id: String,
    },

    /// Update an episode
    Update {
        /// Episode ID
        #[arg(value_name = "EPISODE_ID")]
        episode_id: String,

        /// New task description
        #[arg(short, long)]
        description: Option<String>,

        /// Tags to add (comma-separated)
        #[arg(long, value_delimiter = ',')]
        add_tag: Option<Vec<String>>,

        /// Tags to remove (comma-separated)
        #[arg(long, value_delimiter = ',')]
        remove_tag: Option<Vec<String>>,

        /// Set tags (replaces all existing, comma-separated)
        #[arg(long, value_delimiter = ',')]
        set_tags: Option<Vec<String>>,

        /// Metadata key=value pairs (can be specified multiple times)
        #[arg(long, value_name = "KEY=VALUE")]
        metadata: Option<Vec<String>>,
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

        /// Filter by domain (e.g., 'web-api', 'data-processing')
        #[arg(long)]
        domain: Option<String>,

        /// Filter by task type (code-generation, debugging, refactoring, testing, analysis, documentation, other)
        #[arg(long)]
        r#type: Option<String>,
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

    /// Create a checkpoint for an in-progress episode (ADR-044 Feature 3)
    Checkpoint {
        /// Episode ID to checkpoint
        #[arg(value_name = "EPISODE_ID")]
        episode_id: String,

        /// Reason for creating the checkpoint (e.g., 'Agent switch', 'Long-running task pause')
        #[arg(short, long)]
        reason: String,

        /// Optional note about the checkpoint
        #[arg(short, long)]
        note: Option<String>,
    },

    /// Get a handoff pack from a checkpoint (ADR-044 Feature 3)
    Handoff {
        /// Checkpoint ID to generate handoff pack from
        #[arg(value_name = "CHECKPOINT_ID")]
        checkpoint_id: String,

        /// Return the full unbounded pack (audit/debug) instead of the
        /// default byte-budgeted compact profile
        #[arg(long)]
        full: bool,

        /// Compact payload ceiling in bytes (default 8192, minimum 1024)
        #[arg(long, value_name = "BYTES")]
        max_bytes: Option<usize>,
    },

    /// Resume work from a checkpoint in a new episode (ADR-044 Feature 3)
    Resume {
        /// Checkpoint ID to resume from
        #[arg(value_name = "CHECKPOINT_ID")]
        checkpoint_id: String,

        /// Resume from the full pack instead of the default compact profile
        #[arg(long)]
        full: bool,
    },

    /// List checkpoints for an episode (ADR-044 Feature 3)
    Checkpoints {
        /// Episode ID to list checkpoints for
        #[arg(value_name = "EPISODE_ID")]
        episode_id: String,
    },

    /// Manage episode relationships
    #[command(subcommand)]
    Relationships(RelationshipCommands),
}

mod models;

pub use models::{
    AppliedFilters, EpisodeDetail, EpisodeList, EpisodeListFiltered, EpisodeSearchResult,
    EpisodeSortOrder, EpisodeStatus, EpisodeStep, EpisodeSummary, FilterCommands, FilterList,
    SavedFilter, SearchSortOrder, TaskOutcome,
};

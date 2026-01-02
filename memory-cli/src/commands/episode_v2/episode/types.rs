//! Episode command types and output structures

use clap::{Subcommand, ValueEnum};
use serde::Serialize;

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

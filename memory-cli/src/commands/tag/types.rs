//! Tag command types and data structures

use clap::Subcommand;
use serde::Serialize;

/// Tag management commands
#[derive(Subcommand)]
pub enum TagCommands {
    /// Add tags to an episode
    Add {
        /// Episode ID
        #[arg(value_name = "EPISODE_ID")]
        episode_id: String,

        /// Tags to add (one or more)
        #[arg(value_name = "TAG")]
        tags: Vec<String>,
    },

    /// Remove tags from an episode
    Remove {
        /// Episode ID
        #[arg(value_name = "EPISODE_ID")]
        episode_id: String,

        /// Tags to remove (one or more)
        #[arg(value_name = "TAG")]
        tags: Vec<String>,
    },

    /// Set/replace all tags on an episode
    Set {
        /// Episode ID
        #[arg(value_name = "EPISODE_ID")]
        episode_id: String,

        /// Tags to set (replaces all existing)
        #[arg(value_name = "TAG")]
        tags: Vec<String>,
    },

    /// List all tags with statistics (system-wide)
    List {
        /// Sort by: count (most used), name (alphabetical), recent (last used)
        #[arg(long, value_name = "SORT", default_value = "name")]
        sort_by: String,
    },

    /// Search episodes by tags
    Search {
        /// Tags to search for (one or more)
        #[arg(value_name = "TAG")]
        tags: Vec<String>,

        /// Use AND logic (all tags must match) instead of OR
        #[arg(long)]
        all: bool,

        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Show episode details with its tags
    Show {
        /// Episode ID
        #[arg(value_name = "EPISODE_ID")]
        episode_id: String,
    },
}

/// Result of adding tags to an episode
#[derive(Debug, Serialize)]
pub struct TagAddResult {
    pub episode_id: String,
    pub tags_added: usize,
    pub current_tags: Vec<String>,
    pub success: bool,
}

/// Result of removing tags from an episode
#[derive(Debug, Serialize)]
pub struct TagRemoveResult {
    pub episode_id: String,
    pub tags_removed: usize,
    pub current_tags: Vec<String>,
    pub success: bool,
}

/// Result of setting tags on an episode
#[derive(Debug, Serialize)]
pub struct TagSetResult {
    pub episode_id: String,
    pub tags_set: usize,
    pub current_tags: Vec<String>,
    pub success: bool,
}

/// Result of listing tags for an episode
#[derive(Debug, Serialize)]
pub struct TagListResult {
    pub episode_id: String,
    pub tags: Vec<String>,
    pub count: usize,
}

/// Tag statistics entry for system-wide tag list
#[derive(Debug, Serialize)]
pub struct TagStatEntry {
    pub tag: String,
    pub usage_count: usize,
    pub first_used: String,
    pub last_used: String,
}

/// Result of listing all tags with statistics (system-wide)
#[derive(Debug, Serialize)]
pub struct TagStatsResult {
    pub tags: Vec<TagStatEntry>,
    pub total_tags: usize,
    pub total_usage: usize,
    pub sort_by: String,
}

/// Episode information for tag search results
#[derive(Debug, Serialize)]
pub struct TagSearchEpisode {
    pub episode_id: String,
    pub task_description: String,
    pub task_type: String,
    pub tags: Vec<String>,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub outcome: Option<String>,
}

/// Result of searching episodes by tags
#[derive(Debug, Serialize)]
pub struct TagSearchResult {
    pub count: usize,
    pub episodes: Vec<TagSearchEpisode>,
    pub search_criteria: String,
}

/// Result of showing episode with tags
#[derive(Debug, Serialize)]
pub struct TagShowResult {
    pub episode_id: String,
    pub task_description: String,
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub duration_ms: Option<u64>,
    pub outcome: Option<String>,
    pub tags: Vec<String>,
    pub tags_count: usize,
}

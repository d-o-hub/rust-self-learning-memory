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

        /// Tag color (e.g., red, green, blue, yellow)
        #[arg(short, long, value_name = "COLOR")]
        color: Option<String>,
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

    /// List tags - optionally for a specific episode or all tags system-wide
    List {
        /// Episode ID (if provided, list tags for this episode only)
        #[arg(short, long, value_name = "EPISODE_ID")]
        episode: Option<String>,

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

        /// Enable partial matching (substring search)
        #[arg(short, long)]
        partial: bool,

        /// Enable case-sensitive matching
        #[arg(long)]
        case_sensitive: bool,

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

    /// Rename a tag across all episodes
    Rename {
        /// Current tag name
        #[arg(value_name = "OLD_TAG")]
        old_tag: String,

        /// New tag name
        #[arg(value_name = "NEW_TAG")]
        new_tag: String,

        /// Dry run - show what would be changed without making changes
        #[arg(long)]
        dry_run: bool,
    },

    /// Show detailed tag usage statistics
    Stats {
        /// Show top N tags (default: all)
        #[arg(short, long, value_name = "N")]
        top: Option<usize>,

        /// Sort by: count (most used), name (alphabetical), recent (last used)
        #[arg(short, long, value_name = "SORT", default_value = "count")]
        sort: String,
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

/// Result of renaming a tag
#[derive(Debug, Serialize)]
pub struct TagRenameResult {
    pub old_tag: String,
    pub new_tag: String,
    pub episodes_affected: usize,
    pub success: bool,
}

/// Detailed tag statistics entry for stats command
#[derive(Debug, Serialize)]
pub struct TagStatsDetailedEntry {
    pub tag: String,
    pub usage_count: usize,
    pub percentage: f64,
    pub first_used: String,
    pub last_used: String,
    pub average_per_episode: f64,
}

/// Result of detailed tag statistics
#[derive(Debug, Serialize)]
pub struct TagStatsDetailedResult {
    pub tags: Vec<TagStatsDetailedEntry>,
    pub total_tags: usize,
    pub total_usage: usize,
    pub total_episodes: usize,
    pub avg_tags_per_episode: f64,
    pub most_used_tag: Option<String>,
    pub least_used_tag: Option<String>,
    pub sort_by: String,
}

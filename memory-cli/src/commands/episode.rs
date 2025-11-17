use anyhow::Context;
use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;
use std::path::PathBuf;

use crate::config::Config;
use crate::output::{Output, OutputFormat};

#[derive(Subcommand)]
pub enum EpisodeCommands {
    /// Create a new episode
    Create {
        /// Task description
        #[arg(short, long)]
        task: String,

        /// Context file (JSON)
        #[arg(short, long, value_name = "FILE")]
        context: Option<PathBuf>,
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

#[derive(Debug, Clone, ValueEnum)]
pub enum EpisodeStatus {
    /// Episode is currently in progress
    InProgress,
    /// Episode has been completed
    Completed,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum TaskOutcome {
    /// Task completed successfully
    Success,
    /// Task completed with partial success
    PartialSuccess,
    /// Task failed
    Failure,
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

impl Output for EpisodeSummary {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        writeln!(writer, "Episode: {}", self.episode_id)?;
        writeln!(writer, "Task: {}", self.task_description)?;
        writeln!(writer, "Status: {}", self.status)?;
        writeln!(writer, "Created: {}", self.created_at)?;
        if let Some(duration) = self.duration_ms {
            writeln!(writer, "Duration: {}ms", duration)?;
        }
        writeln!(writer, "Steps: {}", self.steps_count)?;
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct EpisodeList {
    pub episodes: Vec<EpisodeSummary>,
    pub total_count: usize,
}

impl Output for EpisodeList {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{} episodes (showing {})", self.total_count, self.episodes.len())?;
        writeln!(writer, "{}", "─".repeat(80))?;

        for episode in &self.episodes {
            let status_color = match episode.status.as_str() {
                "completed" => Color::Green,
                "in_progress" => Color::Yellow,
                _ => Color::Red,
            };

            writeln!(writer, "{} {} {}",
                episode.episode_id[..8].to_string().dimmed(),
                episode.task_description.chars().take(50).collect::<String>(),
                episode.status.color(status_color).bold()
            )?;
        }

        Ok(())
    }
}

// Command implementations
pub async fn create_episode(
    task: String,
    context: Option<PathBuf>,
    _config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    if dry_run {
        println!("Would create episode with task: {}", task);
        if let Some(context_path) = context {
            println!("Would load context from: {}", context_path.display());
        }
        return Ok(());
    }

    // For now, just return a placeholder - actual implementation needs storage backends
    println!("Episode creation not yet implemented - requires storage backend configuration");
    Ok(())
}

pub async fn list_episodes(
    _task_type: Option<String>,
    _limit: usize,
    _status: Option<EpisodeStatus>,
    _config: &Config,
    _format: OutputFormat,
) -> anyhow::Result<()> {
    println!("Episode listing not yet implemented - requires storage backend configuration");
    Ok(())
}

pub async fn view_episode(
    episode_id: String,
    _config: &Config,
    _format: OutputFormat,
) -> anyhow::Result<()> {
    println!("Episode viewing not yet implemented for: {} - requires storage backend configuration", episode_id);
    Ok(())
}

pub async fn complete_episode(
    episode_id: String,
    outcome: TaskOutcome,
    _config: &Config,
    _format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    if dry_run {
        println!("Would complete episode {} with outcome: {:?}", episode_id, outcome);
        return Ok(());
    }

    println!("Episode completion not yet implemented - requires storage backend configuration");
    Ok(())
}

pub async fn search_episodes(
    query: String,
    _limit: usize,
    _config: &Config,
    _format: OutputFormat,
) -> anyhow::Result<()> {
    println!("Episode search not yet implemented for query: {} - requires storage backend configuration", query);
    Ok(())
}

pub async fn log_step(
    episode_id: String,
    tool: String,
    action: String,
    success: bool,
    _latency_ms: Option<u64>,
    _tokens: Option<u32>,
    _observation: Option<String>,
    _config: &Config,
    _format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    if dry_run {
        println!("Would log step for episode {}: tool={}, action={}, success={}",
                episode_id, tool, action, success);
        return Ok(());
    }

    println!("Step logging not yet implemented - requires storage backend configuration");
    Ok(())
}
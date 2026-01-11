//! Logs command types and output implementations.

use clap::Subcommand;
use serde::Serialize;
use std::collections::HashMap;

use crate::output::Output;

#[derive(Subcommand)]
pub enum LogsCommands {
    /// Analyze episode logs for patterns and insights
    Analyze {
        /// Time range to analyze (e.g., "1h", "24h", "7d")
        #[arg(short, long, default_value = "24h")]
        since: String,
        /// Filter by specific criteria
        #[arg(short, long)]
        filter: Option<String>,
    },
    /// Search logs with flexible queries
    Search {
        /// Search query
        query: String,
        /// Maximum number of results
        #[arg(short, long, default_value = "50")]
        limit: usize,
        /// Time range to search (e.g., "1h", "24h", "7d")
        #[arg(short, long, default_value = "24h")]
        since: String,
    },
    /// Export logs for external analysis
    Export {
        /// Output file path
        #[arg(short, long)]
        path: std::path::PathBuf,
        /// Export format (json, csv, txt)
        #[arg(short, long, default_value = "json")]
        format: ExportFormat,
        /// Time range to export (e.g., "1h", "24h", "7d")
        #[arg(short, long, default_value = "24h")]
        since: String,
        /// Filter criteria
        #[arg(long)]
        filter: Option<String>,
    },
    /// Show log statistics
    Stats {
        /// Time range to analyze (e.g., "1h", "24h", "7d")
        #[arg(short, long, default_value = "24h")]
        since: String,
    },
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum ExportFormat {
    Json,
    Csv,
    Txt,
}

#[derive(Debug, Serialize, Default)]
pub struct LogAnalysis {
    pub time_range: String,
    pub total_episodes: usize,
    pub total_steps: usize,
    pub average_steps_per_episode: f64,
    pub success_rate: f32,
    pub top_tools: Vec<ToolUsage>,
    pub error_patterns: Vec<ErrorPattern>,
    pub performance_trends: Vec<PerformanceTrend>,
}

#[derive(Debug, Serialize)]
pub struct ToolUsage {
    pub tool: String,
    pub usage_count: usize,
    pub success_rate: f32,
    pub average_latency_ms: f64,
}

#[derive(Debug, Serialize)]
pub struct ErrorPattern {
    pub error_type: String,
    pub occurrences: usize,
    pub affected_episodes: usize,
    pub common_messages: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PerformanceTrend {
    pub time_bucket: String,
    pub average_latency_ms: f64,
    pub success_rate: f32,
    pub episode_count: usize,
}

#[derive(Debug, Serialize)]
pub struct LogSearchResult {
    pub query: String,
    pub total_matches: usize,
    pub results: Vec<LogEntry>,
    pub time_range: String,
}

#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub episode_id: String,
    pub timestamp: String,
    pub level: String,
    pub tool: Option<String>,
    pub action: Option<String>,
    pub success: Option<bool>,
    pub latency_ms: Option<u64>,
    pub observation: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LogStats {
    pub time_range: String,
    pub total_episodes: usize,
    pub total_steps: usize,
    pub unique_tools: usize,
    pub success_rate: f32,
    pub average_latency_ms: f64,
    pub error_count: usize,
    pub episodes_by_hour: HashMap<String, usize>,
}

#[derive(Debug, Serialize)]
pub struct ExportResult {
    pub format: String,
    pub path: String,
    pub records_exported: usize,
    pub file_size_bytes: u64,
    pub duration_ms: u64,
}

// Output implementations
impl Output for LogAnalysis {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "Log Analysis Report".bold())?;
        writeln!(writer, "{}", "─".repeat(40))?;
        writeln!(writer, "Time Range: {}", self.time_range)?;
        writeln!(writer, "Total Episodes: {}", self.total_episodes)?;
        writeln!(writer, "Total Steps: {}", self.total_steps)?;
        writeln!(
            writer,
            "Avg Steps/Episode: {:.1}",
            self.average_steps_per_episode
        )?;
        writeln!(writer, "Success Rate: {:.1}%", self.success_rate * 100.0)?;

        if !self.top_tools.is_empty() {
            writeln!(writer, "\nTop Tools:")?;
            for tool in &self.top_tools {
                writeln!(
                    writer,
                    "  {}: {} uses ({:.1}% success, {:.1}ms avg)",
                    tool.tool.cyan(),
                    tool.usage_count,
                    tool.success_rate * 100.0,
                    tool.average_latency_ms
                )?;
            }
        }

        if !self.error_patterns.is_empty() {
            writeln!(writer, "\nError Patterns:")?;
            for error in &self.error_patterns {
                writeln!(
                    writer,
                    "  {}: {} occurrences in {} episodes",
                    error.error_type.red(),
                    error.occurrences,
                    error.affected_episodes
                )?;
            }
        }

        Ok(())
    }
}

impl Output for LogSearchResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "Log Search Results".bold())?;
        writeln!(writer, "{}", "─".repeat(40))?;
        writeln!(writer, "Query: {}", self.query.cyan())?;
        writeln!(writer, "Time Range: {}", self.time_range)?;
        writeln!(writer, "Total Matches: {}", self.total_matches)?;

        if self.results.is_empty() {
            writeln!(writer, "\nNo results found")?;
            return Ok(());
        }

        writeln!(writer, "\nResults:")?;
        for (i, result) in self.results.iter().enumerate() {
            writeln!(
                writer,
                "{}. Episode: {} at {}",
                i + 1,
                result.episode_id,
                result.timestamp
            )?;
            if let Some(tool) = &result.tool {
                write!(writer, "   Tool: {}", tool)?;
            }
            if let Some(action) = &result.action {
                write!(writer, ", Action: {}", action)?;
            }
            if let Some(success) = result.success {
                let status = if success { "✅" } else { "❌" };
                write!(writer, " {}", status)?;
            }
            if let Some(latency) = result.latency_ms {
                write!(writer, " ({}ms)", latency)?;
            }
            writeln!(writer)?;
            if let Some(obs) = &result.observation {
                if obs.len() > 100 {
                    writeln!(writer, "   Observation: {}...", &obs[..97])?;
                } else {
                    writeln!(writer, "   Observation: {}", obs)?;
                }
            }
        }

        Ok(())
    }
}

impl Output for LogStats {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "Log Statistics".bold())?;
        writeln!(writer, "{}", "─".repeat(40))?;
        writeln!(writer, "Time Range: {}", self.time_range)?;
        writeln!(writer, "Total Episodes: {}", self.total_episodes)?;
        writeln!(writer, "Total Steps: {}", self.total_steps)?;
        writeln!(writer, "Unique Tools: {}", self.unique_tools)?;
        writeln!(writer, "Success Rate: {:.1}%", self.success_rate * 100.0)?;
        writeln!(writer, "Average Latency: {:.1}ms", self.average_latency_ms)?;
        writeln!(writer, "Error Count: {}", self.error_count)?;

        if !self.episodes_by_hour.is_empty() {
            writeln!(writer, "\nEpisodes by Hour:")?;
            let mut sorted_hours: Vec<_> = self.episodes_by_hour.iter().collect();
            sorted_hours.sort_by_key(|(h, _)| *h);
            for (hour, count) in sorted_hours {
                writeln!(writer, "  {}: {} episodes", hour, count)?;
            }
        }

        Ok(())
    }
}

impl Output for ExportResult {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "Log Export Complete".bold().green())?;
        writeln!(writer, "{}", "─".repeat(40))?;
        writeln!(writer, "Format: {}", self.format)?;
        writeln!(writer, "Path: {}", self.path)?;
        writeln!(writer, "Records Exported: {}", self.records_exported)?;
        writeln!(
            writer,
            "File Size: {:.2} KB",
            self.file_size_bytes as f64 / 1024.0
        )?;
        writeln!(writer, "Duration: {}ms", self.duration_ms)?;

        Ok(())
    }
}

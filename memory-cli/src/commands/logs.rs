use clap::Subcommand;
use serde::Serialize;
use std::collections::HashMap;
use tokio::fs;


use crate::config::Config;
use crate::output::{Output, OutputFormat};

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

#[derive(Debug, Serialize)]
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

impl Output for LogAnalysis {
    fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
        use colored::*;

        writeln!(writer, "{}", "Log Analysis Report".bold())?;
        writeln!(writer, "{}", "─".repeat(40))?;
        writeln!(writer, "Time Range: {}", self.time_range)?;
        writeln!(writer, "Total Episodes: {}", self.total_episodes)?;
        writeln!(writer, "Total Steps: {}", self.total_steps)?;
        writeln!(writer, "Avg Steps/Episode: {:.1}", self.average_steps_per_episode)?;
        writeln!(writer, "Success Rate: {:.1}%", self.success_rate * 100.0)?;

        if !self.top_tools.is_empty() {
            writeln!(writer, "\nTop Tools:")?;
            for tool in &self.top_tools {
                writeln!(writer, "  {}: {} uses ({:.1}% success, {:.1}ms avg)",
                        tool.tool.cyan(),
                        tool.usage_count,
                        tool.success_rate * 100.0,
                        tool.average_latency_ms)?;
            }
        }

        if !self.error_patterns.is_empty() {
            writeln!(writer, "\nError Patterns:")?;
            for error in &self.error_patterns {
                writeln!(writer, "  {}: {} occurrences in {} episodes",
                        error.error_type.red(),
                        error.occurrences,
                        error.affected_episodes)?;
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
            writeln!(writer, "{}. Episode: {} at {}", i + 1, result.episode_id, result.timestamp)?;
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
        writeln!(writer, "File Size: {:.2} KB", self.file_size_bytes as f64 / 1024.0)?;
        writeln!(writer, "Duration: {}ms", self.duration_ms)?;

        Ok(())
    }
}

// Command implementations
pub async fn analyze_logs(
    memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    since: String,
    _filter: Option<String>,
) -> anyhow::Result<()> {
    // Get episodes within time range
    let episodes = if let Some(turso) = memory.turso_storage() {
        turso.query_episodes_since(chrono::Utc::now() - chrono::Duration::days(365)).await?
    } else if let Some(cache) = memory.cache_storage() {
        cache.query_episodes_since(chrono::Utc::now() - chrono::Duration::days(365)).await?
    } else {
        Vec::new()
    };
    let filtered_episodes = filter_episodes_by_time(&episodes, &since)?;

    let mut total_steps = 0;
    let mut successful_steps = 0;
    let mut tool_usage = HashMap::new();
    let mut error_patterns = HashMap::new();

    for episode in &filtered_episodes {
        total_steps += episode.steps.len();

        for step in &episode.steps {
            // Count tool usage
            let tool_stats = tool_usage.entry(step.tool.clone()).or_insert_with(|| ToolStats {
                count: 0,
                successes: 0,
                total_latency: 0,
            });
            tool_stats.count += 1;
            tool_stats.total_latency += step.latency_ms;

            let is_success = matches!(step.result, Some(memory_core::ExecutionResult::Success { .. }));
            if is_success {
                tool_stats.successes += 1;
                successful_steps += 1;
            }

            // Track errors
            if !is_success {
                let observation = match &step.result {
                    Some(memory_core::ExecutionResult::Error { message }) => message.clone(),
                    Some(memory_core::ExecutionResult::Timeout) => "Timeout".to_string(),
                    _ => "Unknown error".to_string(),
                };
                let error_key = extract_error_type(&observation);
                let error_stats = error_patterns.entry(error_key).or_insert_with(|| ErrorStats {
                    occurrences: 0,
                    episodes: std::collections::HashSet::new(),
                    messages: Vec::new(),
                });
                error_stats.occurrences += 1;
                error_stats.episodes.insert(episode.episode_id);
                if error_stats.messages.len() < 3 {
                    error_stats.messages.push(observation);
                }
            }
        }
    }

    let success_rate = if total_steps > 0 { successful_steps as f32 / total_steps as f32 } else { 0.0 };

    // Convert to output format
    let top_tools: Vec<ToolUsage> = tool_usage.into_iter()
        .map(|(tool, stats)| ToolUsage {
            tool,
            usage_count: stats.count,
            success_rate: if stats.count > 0 { stats.successes as f32 / stats.count as f32 } else { 0.0 },
            average_latency_ms: if stats.count > 0 { stats.total_latency as f64 / stats.count as f64 } else { 0.0 },
        })
        .collect();

    let error_patterns_vec: Vec<ErrorPattern> = error_patterns.into_iter()
        .map(|(error_type, stats)| ErrorPattern {
            error_type,
            occurrences: stats.occurrences,
            affected_episodes: stats.episodes.len(),
            common_messages: stats.messages,
        })
        .collect();

    let analysis = LogAnalysis {
        time_range: since,
        total_episodes: filtered_episodes.len(),
        total_steps,
        average_steps_per_episode: if filtered_episodes.is_empty() { 0.0 } else { total_steps as f64 / filtered_episodes.len() as f64 },
        success_rate,
        top_tools,
        error_patterns: error_patterns_vec,
        performance_trends: Vec::new(), // Would implement time-based analysis
    };

    format.print_output(&analysis)?;
    Ok(())
}

pub async fn search_logs(
    memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    query: String,
    limit: usize,
    since: String,
) -> anyhow::Result<()> {
    let episodes = if let Some(turso) = memory.turso_storage() {
        turso.query_episodes_since(chrono::Utc::now() - chrono::Duration::days(365)).await?
    } else if let Some(cache) = memory.cache_storage() {
        cache.query_episodes_since(chrono::Utc::now() - chrono::Duration::days(365)).await?
    } else {
        Vec::new()
    };
    let filtered_episodes = filter_episodes_by_time(&episodes, &since)?;

    let mut results = Vec::new();

    for episode in &filtered_episodes {
        for step in &episode.steps {
            let observation = match &step.result {
                Some(memory_core::ExecutionResult::Success { output }) => output.clone(),
                Some(memory_core::ExecutionResult::Error { message }) => message.clone(),
                Some(memory_core::ExecutionResult::Timeout) => "Timeout".to_string(),
                None => "".to_string(),
            };
            let matches = query.to_lowercase().split_whitespace().all(|q| {
                step.tool.to_lowercase().contains(q) ||
                step.action.to_lowercase().contains(q) ||
                observation.to_lowercase().contains(q)
            });

            if matches {
                let is_success = matches!(step.result, Some(memory_core::ExecutionResult::Success { .. }));
                let observation = match &step.result {
                    Some(memory_core::ExecutionResult::Success { output }) => output.clone(),
                    Some(memory_core::ExecutionResult::Error { message }) => message.clone(),
                    Some(memory_core::ExecutionResult::Timeout) => "Timeout".to_string(),
                    None => "No result".to_string(),
                };

                results.push(LogEntry {
                    episode_id: episode.episode_id.to_string(),
                    timestamp: episode.start_time.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                    level: if is_success { "INFO" } else { "ERROR" }.to_string(),
                    tool: Some(step.tool.clone()),
                    action: Some(step.action.clone()),
                    success: Some(is_success),
                    latency_ms: Some(step.latency_ms),
                    observation: Some(observation),
                });

                if results.len() >= limit {
                    break;
                }
            }
        }
        if results.len() >= limit {
            break;
        }
    }

    let search_result = LogSearchResult {
        query,
        total_matches: results.len(),
        results,
        time_range: since,
    };

    format.print_output(&search_result)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn export_logs(
    memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    path: std::path::PathBuf,
    export_format: ExportFormat,
    since: String,
    _filter: Option<String>,
    dry_run: bool,
) -> anyhow::Result<()> {
    if dry_run {
        println!("DRY RUN: Would export logs to {}", path.display());
        println!("Format: {:?}, Time range: {}", export_format, since);
        return Ok(());
    }

    let start_time = std::time::Instant::now();
    let episodes = if let Some(turso) = memory.turso_storage() {
        turso.query_episodes_since(chrono::Utc::now() - chrono::Duration::days(365)).await?
    } else if let Some(cache) = memory.cache_storage() {
        cache.query_episodes_since(chrono::Utc::now() - chrono::Duration::days(365)).await?
    } else {
        Vec::new()
    };
    let filtered_episodes = filter_episodes_by_time(&episodes, &since)?;

    let mut records_exported = 0;
    let content = match export_format {
        ExportFormat::Json => {
            let mut log_entries = Vec::new();
            for episode in &filtered_episodes {
                for step in &episode.steps {
                    let is_success = matches!(step.result, Some(memory_core::ExecutionResult::Success { .. }));
                    let observation = match &step.result {
                        Some(memory_core::ExecutionResult::Success { output }) => output.clone(),
                        Some(memory_core::ExecutionResult::Error { message }) => message.clone(),
                        Some(memory_core::ExecutionResult::Timeout) => "Timeout".to_string(),
                        None => "No result".to_string(),
                    };

                    log_entries.push(serde_json::json!({
                        "episode_id": episode.episode_id,
                        "timestamp": episode.start_time,
                        "tool": step.tool,
                        "action": step.action,
                        "success": is_success,
                        "latency_ms": step.latency_ms,
                        "observation": observation
                    }));
                    records_exported += 1;
                }
            }
            serde_json::to_string_pretty(&log_entries)?
        }
        ExportFormat::Csv => {
            let mut csv_content = "episode_id,timestamp,tool,action,success,latency_ms,observation\n".to_string();
            for episode in &filtered_episodes {
                for step in &episode.steps {
                    let is_success = matches!(step.result, Some(memory_core::ExecutionResult::Success { .. }));
                    let observation = match &step.result {
                        Some(memory_core::ExecutionResult::Success { output }) => output.clone(),
                        Some(memory_core::ExecutionResult::Error { message }) => message.clone(),
                        Some(memory_core::ExecutionResult::Timeout) => "Timeout".to_string(),
                        None => "No result".to_string(),
                    }.replace("\"", "\"\"");

                    csv_content.push_str(&format!("{},{},{},{},{},{},\"{}\"\n",
                        episode.episode_id,
                        episode.start_time.format("%Y-%m-%d %H:%M:%S UTC"),
                        step.tool,
                        step.action,
                        is_success,
                        step.latency_ms,
                        observation
                    ));
                    records_exported += 1;
                }
            }
            csv_content
        }
        ExportFormat::Txt => {
            let mut txt_content = String::new();
            for episode in &filtered_episodes {
                txt_content.push_str(&format!("Episode: {} ({})\n", episode.episode_id, episode.start_time));
                for step in &episode.steps {
                    let is_success = matches!(step.result, Some(memory_core::ExecutionResult::Success { .. }));
                    let observation = match &step.result {
                        Some(memory_core::ExecutionResult::Success { output }) => output.clone(),
                        Some(memory_core::ExecutionResult::Error { message }) => message.clone(),
                        Some(memory_core::ExecutionResult::Timeout) => "Timeout".to_string(),
                        None => "No result".to_string(),
                    };

                    txt_content.push_str(&format!("  [{}] {} -> {} ({}ms) {}\n",
                        if is_success { "OK" } else { "FAIL" },
                        step.tool,
                        step.action,
                        step.latency_ms,
                        observation
                    ));
                    records_exported += 1;
                }
                txt_content.push('\n');
            }
            txt_content
        }
    };

    fs::write(&path, &content).await?;
    let file_size = content.len() as u64;
    let duration_ms = start_time.elapsed().as_millis() as u64;

    let result = ExportResult {
        format: format!("{:?}", export_format).to_lowercase(),
        path: path.to_string_lossy().to_string(),
        records_exported,
        file_size_bytes: file_size,
        duration_ms,
    };

    format.print_output(&result)?;
    Ok(())
}

pub async fn logs_stats(
    memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    since: String,
) -> anyhow::Result<()> {
    let episodes = if let Some(turso) = memory.turso_storage() {
        turso.query_episodes_since(chrono::Utc::now() - chrono::Duration::days(365)).await?
    } else if let Some(cache) = memory.cache_storage() {
        cache.query_episodes_since(chrono::Utc::now() - chrono::Duration::days(365)).await?
    } else {
        Vec::new()
    };
    let filtered_episodes = filter_episodes_by_time(&episodes, &since)?;

    let mut total_steps = 0;
    let mut successful_steps = 0;
    let mut total_latency = 0u64;
    let mut error_count = 0;
    let mut tools = std::collections::HashSet::new();
    let mut episodes_by_hour = HashMap::new();

    for episode in &filtered_episodes {
        let hour = episode.start_time.format("%Y-%m-%d %H").to_string();
        *episodes_by_hour.entry(hour).or_insert(0) += 1;

        for step in &episode.steps {
            total_steps += 1;
            total_latency += step.latency_ms;
            tools.insert(&step.tool);

            let is_success = matches!(step.result, Some(memory_core::ExecutionResult::Success { .. }));
            if is_success {
                successful_steps += 1;
            } else {
                error_count += 1;
            }
        }
    }

    let success_rate = if total_steps > 0 { successful_steps as f32 / total_steps as f32 } else { 0.0 };
    let average_latency = if total_steps > 0 { total_latency as f64 / total_steps as f64 } else { 0.0 };

    let stats = LogStats {
        time_range: since,
        total_episodes: filtered_episodes.len(),
        total_steps,
        unique_tools: tools.len(),
        success_rate,
        average_latency_ms: average_latency,
        error_count,
        episodes_by_hour,
    };

    format.print_output(&stats)?;
    Ok(())
}

// Helper functions
fn filter_episodes_by_time(episodes: &[memory_core::Episode], since: &str) -> anyhow::Result<Vec<memory_core::Episode>> {
    let cutoff = parse_time_range(since)?;
    Ok(episodes.iter()
        .filter(|e| e.start_time > cutoff)
        .cloned()
        .collect())
}

fn parse_time_range(since: &str) -> anyhow::Result<chrono::DateTime<chrono::Utc>> {
    let now = chrono::Utc::now();
    match since {
        s if s.ends_with("h") => {
            let hours: i64 = s.trim_end_matches("h").parse()?;
            Ok(now - chrono::Duration::hours(hours))
        }
        s if s.ends_with("d") => {
            let days: i64 = s.trim_end_matches("d").parse()?;
            Ok(now - chrono::Duration::days(days))
        }
        s if s.ends_with("m") => {
            let minutes: i64 = s.trim_end_matches("m").parse()?;
            Ok(now - chrono::Duration::minutes(minutes))
        }
        _ => anyhow::bail!("Invalid time range format. Use '1h', '24h', '7d', etc."),
    }
}

fn extract_error_type(observation: &str) -> String {
    if observation.to_lowercase().contains("timeout") {
        "Timeout".to_string()
    } else if observation.to_lowercase().contains("connection") {
        "Connection Error".to_string()
    } else if observation.to_lowercase().contains("permission") {
        "Permission Denied".to_string()
    } else {
        "Unknown Error".to_string()
    }
}

#[derive(Debug)]
struct ToolStats {
    count: usize,
    successes: usize,
    total_latency: u64,
}

#[derive(Debug)]
struct ErrorStats {
    occurrences: usize,
    episodes: std::collections::HashSet<uuid::Uuid>,
    messages: Vec<String>,
}
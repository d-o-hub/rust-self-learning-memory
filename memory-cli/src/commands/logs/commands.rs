//! Logs command implementations.

use tokio::fs;

use crate::config::Config;
use crate::output::OutputFormat;

use super::types::*;

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
        turso
            .query_episodes_since(chrono::Utc::now() - chrono::Duration::days(365))
            .await?
    } else if let Some(cache) = memory.cache_storage() {
        cache
            .query_episodes_since(chrono::Utc::now() - chrono::Duration::days(365))
            .await?
    } else {
        return Err(anyhow::anyhow!("No storage backend available"));
    };

    let total_episodes = episodes.len();
    let total_steps: usize = episodes.iter().map(|e| e.steps.len()).sum();
    let successful_steps: usize = episodes
        .iter()
        .flat_map(|e| e.steps.iter())
        .filter(|s| s.is_success())
        .count();

    let analysis = LogAnalysis {
        time_range: since,
        total_episodes,
        total_steps,
        average_steps_per_episode: if total_episodes > 0 {
            total_steps as f64 / total_episodes as f64
        } else {
            0.0
        },
        success_rate: if total_steps > 0 {
            successful_steps as f32 / total_steps as f32
        } else {
            0.0
        },
        top_tools: Vec::new(),
        error_patterns: Vec::new(),
        performance_trends: Vec::new(),
    };

    format.print_output(&analysis)?;
    Ok(())
}

pub async fn search_logs(
    _memory: &memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    query: String,
    limit: usize,
    since: String,
) -> anyhow::Result<()> {
    let results = Vec::new();

    let search_result = LogSearchResult {
        query,
        total_matches: 0,
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
    _dry_run: bool,
) -> anyhow::Result<()> {
    let episodes = if let Some(turso) = memory.turso_storage() {
        turso
            .query_episodes_since(chrono::Utc::now() - chrono::Duration::days(365))
            .await?
    } else {
        Vec::new()
    };

    let content = match export_format {
        ExportFormat::Json => serde_json::to_string_pretty(&episodes)?,
        ExportFormat::Csv => {
            let mut csv = String::new();
            csv.push_str("episode_id,task_description,steps,outcome\n");
            for ep in &episodes {
                csv.push_str(&format!(
                    "{},{},{},{:?}\n",
                    ep.episode_id,
                    ep.task_description,
                    ep.steps.len(),
                    ep.outcome
                ));
            }
            csv
        }
        ExportFormat::Txt => {
            let mut txt = String::new();
            for ep in &episodes {
                txt.push_str(&format!("Episode: {}\n", ep.episode_id));
                txt.push_str(&format!("Task: {}\n", ep.task_description));
                txt.push_str(&format!("Steps: {}\n", ep.steps.len()));
                txt.push_str("---\n");
            }
            txt
        }
    };

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).await?;
        }
    }

    fs::write(&path, &content).await?;

    let result = ExportResult {
        format: format!("{:?}", export_format),
        path: path.to_string_lossy().to_string(),
        records_exported: episodes.len(),
        file_size_bytes: content.len() as u64,
        duration_ms: 0,
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
        turso
            .query_episodes_since(chrono::Utc::now() - chrono::Duration::days(365))
            .await?
    } else {
        Vec::new()
    };

    let total_episodes = episodes.len();
    let total_steps: usize = episodes.iter().map(|e| e.steps.len()).sum();
    let successful_steps: usize = episodes
        .iter()
        .flat_map(|e| e.steps.iter())
        .filter(|s| s.is_success())
        .count();

    let mut unique_tools = std::collections::HashSet::new();
    for ep in &episodes {
        for step in &ep.steps {
            unique_tools.insert(step.tool.clone());
        }
    }

    let stats = LogStats {
        time_range: since,
        total_episodes,
        total_steps,
        unique_tools: unique_tools.len(),
        success_rate: if total_steps > 0 {
            successful_steps as f32 / total_steps as f32
        } else {
            0.0
        },
        average_latency_ms: 0.0,
        error_count: total_steps.saturating_sub(successful_steps),
        episodes_by_hour: std::collections::HashMap::new(),
    };

    format.print_output(&stats)?;
    Ok(())
}

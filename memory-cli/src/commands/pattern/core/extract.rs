//! Pattern extract command implementation (ADR-076 §5)
//!
//! Re-runs pattern extraction for completed episodes. Uses the same extractor
//! pipeline as `complete_episode` and respects ADR-075 durability rules.

use crate::config::Config;
use crate::output::OutputFormat;
use do_memory_core::SelfLearningMemory;
use uuid::Uuid;

/// Result for a single episode re-extraction.
#[derive(Debug, serde::Serialize)]
pub struct EpisodeExtractResult {
    pub episode_id: String,
    pub patterns_extracted: usize,
    pub status: String,
}

/// Summary across one or more episodes.
#[derive(Debug, serde::Serialize)]
pub struct ExtractSummary {
    pub episodes_processed: usize,
    pub total_patterns_extracted: usize,
    pub results: Vec<EpisodeExtractResult>,
}

/// Re-run pattern extraction for a specific completed episode or all completed
/// episodes that currently have no patterns linked (when `--all` is used).
///
/// # Arguments
///
/// * `episode_id`   – Re-extract for this single episode UUID.
/// * `all`          – When `true`, re-extract for every completed episode with
///   no patterns yet. Mutually exclusive with `episode_id`.
/// * `memory`       – Shared memory instance.
/// * `_config`      – CLI configuration (reserved for future use).
/// * `format`       – Output format (human / json / yaml).
///
/// # Errors
///
/// Returns an error if:
/// - `episode_id` is provided but is not a valid UUID.
/// - `episode_id` is provided but the episode does not exist or is not complete.
/// - Neither `episode_id` nor `all` is set.
pub async fn extract_patterns(
    episode_id: Option<String>,
    all: bool,
    memory: &SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    match (episode_id, all) {
        (Some(id_str), false) => extract_single(id_str, memory, format).await,
        (None, true) => extract_all_without_patterns(memory, format).await,
        (Some(_), true) => anyhow::bail!("Provide either --episode-id <uuid> or --all, not both"),
        (None, false) => anyhow::bail!("Provide either --episode-id <uuid> or --all"),
    }
}

async fn extract_single(
    id_str: String,
    memory: &SelfLearningMemory,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let episode_uuid = Uuid::parse_str(&id_str)
        .map_err(|_| anyhow::anyhow!("Invalid episode ID format: {}", id_str))?;

    let count = memory
        .re_extract_patterns(episode_uuid)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to re-extract patterns for {}: {}", id_str, e))?;

    let summary = ExtractSummary {
        episodes_processed: 1,
        total_patterns_extracted: count,
        results: vec![EpisodeExtractResult {
            episode_id: id_str,
            patterns_extracted: count,
            status: "ok".to_string(),
        }],
    };

    print_summary(&summary, format)
}

async fn extract_all_without_patterns(
    memory: &SelfLearningMemory,
    format: OutputFormat,
) -> anyhow::Result<()> {
    // Retrieve all completed episodes that currently have no patterns linked.
    let episodes = memory
        .list_episodes(None, None, Some(true))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to list completed episodes: {}", e))?;

    let candidates: Vec<_> = episodes
        .into_iter()
        .filter(|ep| ep.patterns.is_empty())
        .collect();

    if candidates.is_empty() {
        let summary = ExtractSummary {
            episodes_processed: 0,
            total_patterns_extracted: 0,
            results: vec![],
        };
        return print_summary(&summary, format);
    }

    let mut results = Vec::with_capacity(candidates.len());
    let mut total = 0usize;

    for ep in &candidates {
        let episode_uuid = ep.episode_id;
        match memory.re_extract_patterns(episode_uuid).await {
            Ok(count) => {
                total += count;
                results.push(EpisodeExtractResult {
                    episode_id: episode_uuid.to_string(),
                    patterns_extracted: count,
                    status: "ok".to_string(),
                });
            }
            Err(e) => {
                results.push(EpisodeExtractResult {
                    episode_id: episode_uuid.to_string(),
                    patterns_extracted: 0,
                    status: format!("error: {e}"),
                });
            }
        }
    }

    let summary = ExtractSummary {
        episodes_processed: results.len(),
        total_patterns_extracted: total,
        results,
    };

    print_summary(&summary, format)
}

fn print_summary(summary: &ExtractSummary, format: OutputFormat) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(summary)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(summary)?);
        }
        OutputFormat::Human => {
            println!(
                "Pattern extraction complete: {} episode(s) processed, {} pattern(s) extracted",
                summary.episodes_processed, summary.total_patterns_extracted
            );
            for r in &summary.results {
                println!(
                    "  {} → {} pattern(s) [{}]",
                    r.episode_id, r.patterns_extracted, r.status
                );
            }
        }
    }
    Ok(())
}

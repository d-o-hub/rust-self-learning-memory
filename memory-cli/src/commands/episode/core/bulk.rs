//! Bulk episode operations command implementation
//!
//! This module provides commands for bulk episode retrieval by IDs.

use crate::config::Config;
#[cfg(feature = "turso")]
use crate::errors::{helpers, EnhancedError};
use crate::output::OutputFormat;
use memory_core::SelfLearningMemory;

/// Retrieve multiple episodes by their IDs in a single efficient operation
///
/// # Arguments
///
/// * `episode_ids` - Comma-separated list of episode UUIDs
/// * `memory` - Initialized memory system
/// * `_config` - CLI configuration
/// * `format` - Output format (human, json, yaml)
///
/// # Examples
///
/// ```bash
/// # Get multiple episodes
/// memory-cli episode bulk abc123...,def456...,ghi789...
///
/// # JSON output
/// memory-cli episode bulk abc123...,def456... --format json
/// ```
pub async fn bulk_get_episodes(
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] episode_ids: String,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] memory: &SelfLearningMemory,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _config: &Config,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] format: OutputFormat,
) -> anyhow::Result<()> {
    // Check if storage features are enabled
    #[cfg(not(feature = "turso"))]
    return Err(anyhow::anyhow!(
        "Turso storage feature not enabled. Use --features turso to enable."
    ));

    #[cfg(feature = "turso")]
    {
        // Parse comma-separated episode IDs
        let id_strings: Vec<&str> = episode_ids.split(',').map(|s| s.trim()).collect();

        if id_strings.is_empty() {
            return Err(anyhow::anyhow!(
                "No episode IDs provided.\n\
                \nUsage: memory-cli episode bulk <id1>,<id2>,<id3>...\n\
                \nExample: memory-cli episode bulk abc123...,def456..."
            ));
        }

        // Parse UUIDs
        let mut parsed_ids = Vec::new();
        let mut parse_errors = Vec::new();

        for (idx, id_str) in id_strings.iter().enumerate() {
            match Uuid::parse_str(id_str) {
                Ok(uuid) => parsed_ids.push(uuid),
                Err(_) => parse_errors.push(format!("  [{}] Invalid UUID: {}", idx + 1, id_str)),
            }
        }

        // Report any parse errors
        if !parse_errors.is_empty() {
            return Err(anyhow::anyhow!(
                "Failed to parse episode IDs:\n{}\n\
                \nExpected format: UUID v4 (e.g., 550e8400-e29b-41d4-a716-446655440000)",
                parse_errors.join("\n")
            ));
        }

        // Bulk retrieve episodes
        let episodes = memory
            .get_episodes_by_ids(&parsed_ids)
            .await
            .context_with_help("Failed to retrieve episodes", helpers::STORAGE_ERROR_HELP)?;

        // Report results
        let found_count = episodes.len();
        let requested_count = parsed_ids.len();

        if found_count == 0 {
            return Err(anyhow::anyhow!(
                "No episodes found for the provided IDs.\n\
                \nRequested {} episode(s), found 0.\n\
                \nTip: Use 'memory-cli episode list' to see available episodes.",
                requested_count
            ));
        }

        // Create bulk result output
        #[derive(Debug, serde::Serialize)]
        struct BulkEpisodeResult {
            requested_count: usize,
            found_count: usize,
            missing_count: usize,
            episodes: Vec<EpisodeSummary>,
        }

        #[derive(Debug, serde::Serialize)]
        struct EpisodeSummary {
            episode_id: String,
            task_description: String,
            task_type: String,
            status: String,
            created_at: String,
            completed_at: Option<String>,
            duration_ms: Option<i64>,
            steps_count: usize,
            patterns_count: usize,
            heuristics_count: usize,
        }

        impl Output for BulkEpisodeResult {
            fn write_human<W: std::io::Write>(&self, mut writer: W) -> anyhow::Result<()> {
                use colored::*;

                writeln!(
                    writer,
                    "{}",
                    "Bulk Episode Retrieval Results".bold().underline()
                )?;
                writeln!(writer, "Requested: {}", self.requested_count)?;
                writeln!(writer, "Found: {}", self.found_count.to_string().green())?;

                if self.missing_count > 0 {
                    writeln!(
                        writer,
                        "Missing: {}",
                        self.missing_count.to_string().yellow()
                    )?;
                }

                writeln!(writer)?;

                for (idx, episode) in self.episodes.iter().enumerate() {
                    writeln!(
                        writer,
                        "{}",
                        format!("Episode {} of {}", idx + 1, self.found_count).bold()
                    )?;
                    writeln!(writer, "  ID: {}", episode.episode_id.dimmed())?;
                    writeln!(writer, "  Task: {}", episode.task_description)?;
                    writeln!(writer, "  Type: {}", episode.task_type)?;
                    writeln!(
                        writer,
                        "  Status: {}",
                        if episode.completed_at.is_some() {
                            episode.status.green()
                        } else {
                            episode.status.yellow()
                        }
                    )?;
                    writeln!(writer, "  Created: {}", episode.created_at)?;

                    if let Some(ref completed) = episode.completed_at {
                        writeln!(writer, "  Completed: {}", completed)?;
                    }

                    if let Some(duration) = episode.duration_ms {
                        writeln!(writer, "  Duration: {}ms", duration)?;
                    }

                    writeln!(writer, "  Steps: {}", episode.steps_count)?;
                    writeln!(writer, "  Patterns: {}", episode.patterns_count)?;
                    writeln!(writer, "  Heuristics: {}", episode.heuristics_count)?;
                    writeln!(writer)?;
                }

                Ok(())
            }
        }

        let episode_summaries: Vec<EpisodeSummary> = episodes
            .iter()
            .map(|ep| EpisodeSummary {
                episode_id: ep.episode_id.to_string(),
                task_description: ep.task_description.clone(),
                task_type: format!("{:?}", ep.task_type),
                status: if ep.is_complete() {
                    "completed".to_string()
                } else {
                    "in_progress".to_string()
                },
                created_at: ep.start_time.to_rfc3339(),
                completed_at: ep.end_time.map(|t| t.to_rfc3339()),
                duration_ms: ep
                    .end_time
                    .map(|end| (end - ep.start_time).num_milliseconds()),
                steps_count: ep.steps.len(),
                patterns_count: ep.patterns.len(),
                heuristics_count: ep.heuristics.len(),
            })
            .collect();

        let result = BulkEpisodeResult {
            requested_count,
            found_count,
            missing_count: requested_count - found_count,
            episodes: episode_summaries,
        };

        format.print_output(&result)
    }
}

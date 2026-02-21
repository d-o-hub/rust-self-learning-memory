//! Search episodes command implementation

use super::types::{EpisodeList, SearchSortOrder};
use crate::config::Config;
use crate::output::OutputFormat;
use memory_core::EpisodeFilter;
use memory_core::search::{SearchField, SearchMode};
use memory_core::{Episode, SelfLearningMemory, TaskOutcome};

/// Calculate a success score for an episode (higher = more successful)
fn outcome_score(episode: &Episode) -> u8 {
    match &episode.outcome {
        Some(TaskOutcome::Success { .. }) => 3,
        Some(TaskOutcome::PartialSuccess { .. }) => 2,
        Some(TaskOutcome::Failure { .. }) => 1,
        None => 0, // In progress or no outcome
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn search_episodes(
    query: String,
    limit: usize,
    _semantic: bool,
    _enable_embeddings: bool,
    _embedding_provider: Option<String>,
    _embedding_model: Option<String>,
    fuzzy: bool,
    fuzzy_threshold: f64,
    regex: bool,
    search_fields: Option<Vec<String>>,
    sort: SearchSortOrder,
    memory: &SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    // Build filter with search mode
    let mut filter_builder = EpisodeFilter::builder().search_text(query.clone());

    // Configure search mode (priority: regex > fuzzy > exact)
    if regex {
        // Validate regex pattern before using it
        if let Err(e) = memory_core::search::validate_regex_pattern(&query) {
            return Err(anyhow::anyhow!("Invalid regex pattern: {}", e));
        }
        filter_builder = filter_builder.search_mode(SearchMode::Regex);
    } else if fuzzy {
        filter_builder = filter_builder.search_mode(SearchMode::Fuzzy {
            threshold: fuzzy_threshold,
        });
    } else {
        filter_builder = filter_builder.search_mode(SearchMode::Exact);
    }

    // Configure search fields
    if let Some(fields) = search_fields {
        let parsed_fields: Vec<SearchField> = fields
            .iter()
            .filter_map(|f| match f.to_lowercase().as_str() {
                "description" => Some(SearchField::Description),
                "steps" => Some(SearchField::Steps),
                "outcome" => Some(SearchField::Outcome),
                "tags" => Some(SearchField::Tags),
                "domain" => Some(SearchField::Domain),
                "all" => Some(SearchField::All),
                _ => {
                    eprintln!("Warning: Unknown field '{}', ignoring", f);
                    None
                }
            })
            .collect();

        if !parsed_fields.is_empty() {
            filter_builder = filter_builder.search_fields(parsed_fields);
        }
    }

    let filter = filter_builder.build();

    // Search using the filter
    let mut episodes = memory
        .list_episodes_filtered(filter, Some(limit), None)
        .await?;
    let total_count = episodes.len();

    // Apply sorting
    match sort {
        SearchSortOrder::Relevance => {
            // Default ordering from search - no additional sort needed
        }
        SearchSortOrder::Newest => {
            episodes.sort_by(|a, b| b.start_time.cmp(&a.start_time));
        }
        SearchSortOrder::Oldest => {
            episodes.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        }
        SearchSortOrder::Duration => {
            episodes.sort_by(|a, b| {
                let dur_a = a.end_time.map(|e| e - a.start_time);
                let dur_b = b.end_time.map(|e| e - b.start_time);
                dur_b.cmp(&dur_a) // Longest first
            });
        }
        SearchSortOrder::Success => {
            episodes.sort_by(|a, b| {
                let score_a = outcome_score(a);
                let score_b = outcome_score(b);
                score_b.cmp(&score_a) // Highest success first
            });
        }
    }

    // Convert to summary format
    let episode_summaries: Vec<super::types::EpisodeSummary> = episodes
        .into_iter()
        .map(|episode| {
            let status = if episode.is_complete() {
                "completed"
            } else {
                "in_progress"
            };
            let duration_ms = episode
                .end_time
                .map(|end| (end - episode.start_time).num_milliseconds() as u64);

            super::types::EpisodeSummary {
                episode_id: episode.episode_id.to_string(),
                task_description: episode.task_description,
                status: status.to_string(),
                created_at: episode.start_time.to_rfc3339(),
                duration_ms,
                steps_count: episode.steps.len(),
            }
        })
        .collect();

    let list = EpisodeList {
        episodes: episode_summaries,
        total_count, // For search, we don't know total count
    };

    #[cfg(feature = "turso")]
    {
        format.print_output(&list)
    }

    #[cfg(not(feature = "turso"))]
    {
        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&list)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&list)?);
            }
            OutputFormat::Human => {
                println!("Search results ({} episodes found)", list.episodes.len());
                println!("Total count: {}", list.total_count);
                println!();
                for episode in &list.episodes {
                    println!("ID: {}", episode.episode_id);
                    println!("Task: {}", episode.task_description);
                    println!("Status: {}", episode.status);
                    println!("Created: {}", episode.created_at);
                    if let Some(duration) = episode.duration_ms {
                        println!("Duration: {}ms", duration);
                    }
                    println!("Steps: {}", episode.steps_count);
                    println!();
                }
            }
        }

        Ok(())
    }
}

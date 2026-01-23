//! Search episodes command implementation

use super::types::{EpisodeSummary, SearchSortOrder};
use crate::config::Config;
use crate::output::OutputFormat;
use memory_core::search::{SearchField, SearchMode};
use memory_core::{Episode, EpisodeFilter, SelfLearningMemory, TaskOutcome};

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
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] query: String,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] limit: usize,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _semantic: bool,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _enable_embeddings: bool,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _embedding_provider: Option<
        String,
    >,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _embedding_model: Option<String>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] fuzzy: bool,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] fuzzy_threshold: f64,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] regex: bool,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] search_fields: Option<Vec<String>>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] sort: SearchSortOrder,
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
        let episode_summaries: Vec<EpisodeSummary> = episodes
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

                EpisodeSummary {
                    episode_id: episode.episode_id.to_string(),
                    task_description: episode.task_description,
                    status: status.to_string(),
                    created_at: episode.start_time.to_rfc3339(),
                    duration_ms,
                    steps_count: episode.steps.len(),
                }
            })
            .collect();

        let list = super::types::EpisodeList {
            episodes: episode_summaries,
            total_count, // For search, we don't know total count
        };

        format.print_output(&list)
    }
}

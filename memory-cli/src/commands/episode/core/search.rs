//! Search episodes command implementation

use super::types::{EpisodeList, SearchSortOrder};
use crate::config::Config;
use crate::output::OutputFormat;
use do_memory_core::EpisodeFilter;
use do_memory_core::search::{SearchField, SearchMode};
use do_memory_core::{Episode, SelfLearningMemory, TaskOutcome, TaskType};

/// Calculate a success score for an episode (higher = more successful)
fn outcome_score(episode: &Episode) -> u8 {
    match &episode.outcome {
        Some(TaskOutcome::Success { .. }) => 3,
        Some(TaskOutcome::PartialSuccess { .. }) => 2,
        Some(TaskOutcome::Failure { .. }) => 1,
        Some(TaskOutcome::Abstained { .. }) => 1, // Between failure and partial in core; clamped to u8 scale
        None => 0,                                // In progress or no outcome
    }
}

/// Maximum allowed limit for episode search operations
const MAX_SEARCH_LIMIT: usize = 1000;

/// Minimum allowed limit for episode search operations
const MIN_SEARCH_LIMIT: usize = 1;

#[expect(clippy::too_many_arguments)]
pub async fn search_episodes(
    query: String,
    mut limit: usize,
    semantic: bool,
    fuzzy: bool,
    fuzzy_threshold: f64,
    regex: bool,
    search_fields: Option<Vec<String>>,
    sort: SearchSortOrder,
    domain: Option<String>,
    task_type: Option<String>,
    memory: &SelfLearningMemory,
    config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    // Clamp limit to prevent resource exhaustion
    limit = limit.clamp(MIN_SEARCH_LIMIT, MAX_SEARCH_LIMIT);

    // Parse the task type filter once; both search paths apply it.
    let parsed_task_type = match task_type.as_deref() {
        None => None,
        Some(tt) => Some(match tt.to_lowercase().as_str() {
            "code-generation" | "code_generation" => TaskType::CodeGeneration,
            "debugging" => TaskType::Debugging,
            "refactoring" => TaskType::Refactoring,
            "testing" => TaskType::Testing,
            "analysis" => TaskType::Analysis,
            "documentation" => TaskType::Documentation,
            "other" => TaskType::Other,
            _ => {
                return Err(anyhow::anyhow!(
                    "Unknown task type '{}'. Valid types: code-generation, debugging, refactoring, testing, analysis, documentation, other",
                    tt
                ));
            }
        }),
    };

    let mut episodes = if semantic {
        crate::commands::embedding::activate_configured_semantic_service(memory, config).await?;
        let semantic_service = memory
            .live_semantic_service()
            .await
            .ok_or_else(|| anyhow::anyhow!("Semantic service unavailable after activation"))?;

        // Semantic search matches by meaning; lexical mode/field options are
        // rejected by clap, so only the metadata filters apply here.
        let results = semantic_service
            .find_similar_episodes(
                &query,
                &do_memory_core::types::TaskContext::default(),
                MAX_SEARCH_LIMIT,
            )
            .await?;
        let mut candidates: Vec<Episode> = results.into_iter().map(|result| result.item).collect();

        if let Some(domain_filter) = domain.as_deref() {
            candidates.retain(|episode| episode.context.domain == domain_filter);
        }
        if let Some(expected_type) = parsed_task_type {
            candidates.retain(|episode| episode.task_type == expected_type);
        }
        candidates.truncate(limit);
        candidates
    } else {
        // Build filter with search mode
        let mut filter_builder = EpisodeFilter::builder().search_text(query.clone());

        // Add domain filter if provided
        if let Some(d) = domain {
            filter_builder = filter_builder.domains(vec![d]);
        }

        // Add task type filter if provided
        if let Some(parsed_type) = parsed_task_type {
            filter_builder = filter_builder.task_types(vec![parsed_type]);
        }

        // Configure search mode (priority: regex > fuzzy > exact)
        if regex {
            // Validate regex pattern before using it
            if let Err(e) = do_memory_core::search::validate_regex_pattern(&query) {
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
        memory
            .list_episodes_filtered(filter, Some(limit), None)
            .await?
    };
    let total_count = episodes.len();

    // Apply sorting
    match sort {
        SearchSortOrder::Relevance => {
            // Default ordering from search - no additional sort needed
        }
        SearchSortOrder::Newest => {
            episodes.sort_by_key(|b| std::cmp::Reverse(b.start_time));
        }
        SearchSortOrder::Oldest => {
            episodes.sort_by_key(|a| a.start_time);
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

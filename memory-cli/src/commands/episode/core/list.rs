//! List episodes command implementation

use super::types::{
    AppliedFilters, EpisodeList, EpisodeListFiltered, EpisodeSortOrder, EpisodeSummary,
};
use crate::config::Config;
use crate::output::OutputFormat;
use chrono::Utc;
use memory_core::SelfLearningMemory;

#[allow(clippy::too_many_arguments)]
pub async fn list_episodes(
    task_type: Option<String>,
    limit: usize,
    status: Option<super::types::EpisodeStatus>,
    _semantic_search: Option<String>,
    _enable_embeddings: bool,
    _embedding_provider: Option<String>,
    _embedding_model: Option<String>,
    since: Option<String>,
    until: Option<String>,
    sort: EpisodeSortOrder,
    domain: Option<String>,
    tags: Option<String>,
    outcome: Option<super::types::TaskOutcome>,
    offset: usize,
    memory: &SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    if let Some(ref task_type_str) = task_type {
        match task_type_str.as_str() {
            "code_generation" | "debugging" | "testing" | "analysis" | "documentation"
            | "refactoring" | "other" => {}
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid task type: '{}'.\n\
                 \nValid task types:\n\
                 • code_generation - Code generation tasks\n\
                 • debugging - Debugging and troubleshooting\n\
                 • testing - Test writing and execution\n\
                 • analysis - Code analysis and review\n\
                 • documentation - Documentation tasks\n\
                 • refactoring - Code refactoring\n\
                 • other - Other task types\n\
                 \nExample: memory-cli episode list --task-type debugging",
                    task_type_str
                ));
            }
        };
    }

    let completed_only = match status {
        Some(super::types::EpisodeStatus::Completed) => Some(true),
        _ => None,
    };

    let mut episodes = memory
        .list_episodes(Some(limit * 2 + offset), None, completed_only)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to list episodes: {}", e))?;

    episodes.retain(|episode| match status {
        Some(super::types::EpisodeStatus::InProgress) => !episode.is_complete(),
        Some(super::types::EpisodeStatus::Completed) => episode.is_complete(),
        None => true,
    });

    if let Some(ref tt) = task_type {
        let tt_lower = tt.to_lowercase();
        episodes.retain(|episode| {
            episode
                .task_type
                .to_string()
                .to_lowercase()
                .contains(&tt_lower)
        });
    }

    let since_date = since.clone().and_then(|s| {
        chrono::DateTime::parse_from_rfc3339(&s)
            .ok()
            .map(|d| d.with_timezone(&Utc))
    });
    let until_date = until.clone().and_then(|u| {
        chrono::DateTime::parse_from_rfc3339(&u)
            .ok()
            .map(|d| d.with_timezone(&Utc))
    });

    episodes.retain(|episode| {
        let episode_start = episode.start_time.with_timezone(&Utc);
        let after_since = since_date.map_or(true, |d| episode_start >= d);
        let before_until = until_date.map_or(true, |d| episode_start <= d);
        after_since && before_until
    });

    if let Some(ref dom) = domain {
        let dom_lower = dom.to_lowercase();
        episodes.retain(move |episode| episode.context.domain.to_lowercase().contains(&dom_lower));
    }

    if let Some(ref tag_str) = tags {
        let tag_list: Vec<String> = tag_str.split(',').map(|s| s.trim().to_string()).collect();
        episodes.retain(|episode| {
            let episode_tags: &Vec<String> = &episode.context.tags;
            tag_list.iter().any(|t| {
                episode_tags
                    .iter()
                    .any(|et| et.to_lowercase() == t.to_lowercase())
            })
        });
    }

    if let Some(ref out) = outcome {
        episodes.retain(|episode| {
            episode.outcome.as_ref().is_some_and(|o| {
                let o_str = format!("{:?}", o).to_lowercase();
                let out_str = format!("{:?}", out).to_lowercase();
                o_str.contains(&out_str)
            })
        });
    }

    let mut episode_summaries: Vec<EpisodeSummary> = episodes
        .into_iter()
        .map(|episode| {
            let ep_status = if episode.is_complete() {
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
                status: ep_status.to_string(),
                created_at: episode.start_time.to_rfc3339(),
                duration_ms,
                steps_count: episode.steps.len(),
            }
        })
        .collect();

    match sort {
        EpisodeSortOrder::Newest => {
            episode_summaries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        }
        EpisodeSortOrder::Oldest => {
            episode_summaries.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        }
        EpisodeSortOrder::Duration => {
            episode_summaries.sort_by(|a, b| {
                let a_dur = a.duration_ms.unwrap_or(0);
                let b_dur = b.duration_ms.unwrap_or(0);
                b_dur.cmp(&a_dur)
            });
        }
        EpisodeSortOrder::Relevance => {}
    }

    let total_count = episode_summaries.len();

    episode_summaries = episode_summaries
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    let filtered_count = episode_summaries.len();
    let applied_filters = AppliedFilters {
        task_type,
        status: status.map(|s| format!("{:?}", s)),
        since,
        until,
        domain,
        tags,
        outcome: outcome.map(|o| format!("{:?}", o)),
        sort: format!("{:?}", sort),
        offset,
        limit,
    };

    let filtered_list = EpisodeListFiltered {
        episodes: episode_summaries,
        total_count,
        filtered_count,
        applied_filters,
    };

    #[cfg(feature = "turso")]
    {
        format.print_output(&filtered_list)
    }

    #[cfg(not(feature = "turso"))]
    {
        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&filtered_list)?);
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&filtered_list)?);
            }
            OutputFormat::Human => {
                println!("Episodes ({} shown)", filtered_list.filtered_count);
                println!("Total count: {}", filtered_list.total_count);
                println!();
                for episode in &filtered_list.episodes {
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

#[allow(clippy::too_many_arguments)]
pub async fn list_episodes_basic(
    task_type: Option<String>,
    limit: usize,
    status: Option<super::types::EpisodeStatus>,
    _semantic_search: Option<String>,
    _enable_embeddings: bool,
    _embedding_provider: Option<String>,
    _embedding_model: Option<String>,
    memory: &SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
) -> anyhow::Result<()> {
    if let Some(ref task_type_str) = task_type {
        match task_type_str.as_str() {
            "code_generation" | "debugging" | "testing" | "analysis" | "documentation"
            | "refactoring" | "other" => {}
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid task type: '{}'.\n\
                 \nValid task types:\n\
                 • code_generation - Code generation tasks\n\
                 • debugging - Debugging and troubleshooting\n\
                 • testing - Test writing and execution\n\
                 • analysis - Code analysis and review\n\
                 • documentation - Documentation tasks\n\
                 • refactoring - Code refactoring\n\
                 • other - Other task types\n\
                 \nExample: memory-cli episode list --task-type debugging",
                    task_type_str
                ));
            }
        };
    }

    let completed_only = match status {
        Some(super::types::EpisodeStatus::Completed) => Some(true),
        _ => None,
    };

    let mut episodes = memory
        .list_episodes(Some(limit * 2), None, completed_only)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to list episodes: {}", e))?;

    episodes.retain(|episode| match status {
        Some(super::types::EpisodeStatus::InProgress) => !episode.is_complete(),
        Some(super::types::EpisodeStatus::Completed) => episode.is_complete(),
        None => true,
    });

    if let Some(ref tt) = task_type {
        let tt_lower = tt.to_lowercase();
        episodes.retain(|episode| {
            episode
                .task_type
                .to_string()
                .to_lowercase()
                .contains(&tt_lower)
        });
    }

    let mut episode_summaries: Vec<EpisodeSummary> = episodes
        .into_iter()
        .map(|episode| {
            let ep_status = if episode.is_complete() {
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
                status: ep_status.to_string(),
                created_at: episode.start_time.to_rfc3339(),
                duration_ms,
                steps_count: episode.steps.len(),
            }
        })
        .collect();

    episode_summaries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    let total_count = episode_summaries.len();
    episode_summaries.truncate(limit);

    let list = EpisodeList {
        episodes: episode_summaries,
        total_count,
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
                println!("Episodes ({} shown)", list.episodes.len());
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

//! List episodes command implementation

use super::types::EpisodeSortOrder;
use crate::config::Config;
use crate::output::OutputFormat;
use memory_core::SelfLearningMemory;

#[allow(clippy::too_many_arguments)]
pub async fn list_episodes(
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] task_type: Option<String>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] limit: usize,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] status: Option<
        super::types::EpisodeStatus,
    >,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _semantic_search: Option<String>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _enable_embeddings: bool,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _embedding_provider: Option<
        String,
    >,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _embedding_model: Option<String>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] since: Option<String>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] until: Option<String>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] sort: EpisodeSortOrder,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] domain: Option<String>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] tags: Option<String>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] outcome: Option<
        super::types::TaskOutcome,
    >,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] offset: usize,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] memory: &SelfLearningMemory,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _config: &Config,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] format: OutputFormat,
) -> anyhow::Result<()> {
    #[cfg(not(feature = "turso"))]
    return Err(anyhow::anyhow!(
        "Turso storage feature not enabled. Use --features turso to enable."
    ));

    #[cfg(feature = "turso")]
    {
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
                    ))
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
            let domain_clone = domain.clone();
            episodes
                .retain(move |episode| episode.context.domain.to_lowercase().contains(&dom_lower));
            drop(domain_clone);
        }

        if let Some(ref tag_str) = tags {
            let tag_list: Vec<String> = tag_str.split(',').map(|s| s.trim().to_string()).collect();
            let tags_clone = tags.clone();
            episodes.retain(|episode| {
                let episode_tags: &Vec<String> = &episode.context.tags;
                tag_list.iter().any(|t| {
                    episode_tags
                        .iter()
                        .any(|et| et.to_lowercase() == t.to_lowercase())
                })
            });
            drop(tags_clone);
        }

        if let Some(ref out) = outcome {
            let outcome_clone = outcome.clone();
            episodes.retain(|episode| {
                episode.outcome.as_ref().is_some_and(|o| {
                    let o_str = format!("{:?}", o).to_lowercase();
                    let out_str = format!("{:?}", out).to_lowercase();
                    o_str.contains(&out_str)
                })
            });
            drop(outcome_clone);
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

        let filtered_count = episode_summaries.len();
        let filtered_list = EpisodeListFiltered {
            episodes: episode_summaries,
            total_count,
            filtered_count,
            applied_filters,
        };

        format.print_output(&filtered_list)
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn list_episodes_basic(
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] task_type: Option<String>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] limit: usize,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] status: Option<
        super::types::EpisodeStatus,
    >,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _semantic_search: Option<String>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _enable_embeddings: bool,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _embedding_provider: Option<
        String,
    >,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _embedding_model: Option<String>,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] memory: &SelfLearningMemory,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] _config: &Config,
    #[cfg_attr(not(feature = "turso"), allow(unused_variables))] format: OutputFormat,
) -> anyhow::Result<()> {
    #[cfg(not(feature = "turso"))]
    return Err(anyhow::anyhow!(
        "Turso storage feature not enabled. Use --features turso to enable."
    ));

    #[cfg(feature = "turso")]
    {
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
                    ))
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

        format.print_output(&list)
    }
}

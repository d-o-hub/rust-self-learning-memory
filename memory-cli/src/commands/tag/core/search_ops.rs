//! Tag search command implementation
//!
//! Split out of `core.rs` to keep individual source files under the 500 LOC
//! quality gate (WG-185).

use super::MAX_TAGS_PER_OPERATION;
use crate::commands::tag::types::{TagSearchEpisode, TagSearchResult};
use crate::output::OutputFormat;
use do_memory_core::SelfLearningMemory;

/// Maximum allowed limit for tag search operations
const MAX_TAG_SEARCH_LIMIT: usize = 1000;
/// Minimum allowed limit for tag search operations
const MIN_TAG_SEARCH_LIMIT: usize = 1;

/// Search episodes by tags
#[expect(clippy::too_many_arguments)]
pub async fn search_by_tags(
    mut tags: Vec<String>,
    require_all: bool,
    partial: bool,
    case_sensitive: bool,
    mut limit: usize,
    memory: &SelfLearningMemory,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    if tags.is_empty() {
        return Err(anyhow::anyhow!("At least one tag must be specified"));
    }

    // Clamp limit and tags to prevent resource exhaustion
    limit = limit.clamp(MIN_TAG_SEARCH_LIMIT, MAX_TAG_SEARCH_LIMIT);
    tags.truncate(MAX_TAGS_PER_OPERATION);

    if dry_run {
        let logic = if require_all { "ALL" } else { "ANY" };
        let partial_msg = if partial { " (partial matching)" } else { "" };
        let case_msg = if case_sensitive {
            " (case-sensitive)"
        } else {
            " (case-insensitive)"
        };
        println!(
            "Would search for episodes with {} of these tags{}{}: {} (limit: {})",
            logic,
            partial_msg,
            case_msg,
            tags.join(", "),
            limit
        );
        return Ok(());
    }

    // Get all episodes and filter by tags
    let all_episodes = memory.get_all_episodes().await?;

    let mut matching_episodes = Vec::new();

    for episode in all_episodes {
        let episode_tags = &episode.tags;

        // Determine matching function based on partial and case_sensitive flags
        let tag_matches = |search_tag: &str, episode_tag: &str| -> bool {
            if partial {
                // Partial matching (substring search)
                if case_sensitive {
                    episode_tag.contains(search_tag)
                } else {
                    episode_tag
                        .to_lowercase()
                        .contains(&search_tag.to_lowercase())
                }
            } else {
                // Exact matching
                if case_sensitive {
                    episode_tag == search_tag
                } else {
                    episode_tag.eq_ignore_ascii_case(search_tag)
                }
            }
        };

        let matches = if require_all {
            // Check if episode has ALL requested tags
            tags.iter()
                .all(|search_tag| episode_tags.iter().any(|et| tag_matches(search_tag, et)))
        } else {
            // Check if episode has ANY requested tag
            tags.iter()
                .any(|search_tag| episode_tags.iter().any(|et| tag_matches(search_tag, et)))
        };

        if matches {
            matching_episodes.push(TagSearchEpisode {
                episode_id: episode.episode_id.to_string(),
                task_description: episode.task_description.clone(),
                task_type: format!("{:?}", episode.task_type),
                tags: episode.tags.clone(),
                start_time: episode.start_time.timestamp(),
                end_time: episode.end_time.map(|t| t.timestamp()),
                outcome: episode.outcome.map(|o| format!("{:?}", o)),
            });

            if matching_episodes.len() >= limit {
                break;
            }
        }
    }

    let mut search_criteria = if require_all {
        format!("All of: [{}]", tags.join(", "))
    } else {
        format!("Any of: [{}]", tags.join(", "))
    };
    if partial {
        search_criteria.push_str(" (partial)");
    }
    if case_sensitive {
        search_criteria.push_str(" (case-sensitive)");
    }

    let result = TagSearchResult {
        count: matching_episodes.len(),
        episodes: matching_episodes,
        search_criteria,
    };

    format.print_output(&result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use do_memory_core::SelfLearningMemory;

    #[test]
    fn test_min_tag_search_limit_is_one() {
        assert_eq!(MIN_TAG_SEARCH_LIMIT, 1);
    }

    #[test]
    fn test_max_tag_search_limit_is_one_thousand() {
        assert_eq!(MAX_TAG_SEARCH_LIMIT, 1000);
    }

    #[tokio::test]
    async fn test_search_by_tags_empty_tags_returns_error() {
        let memory = SelfLearningMemory::new();
        let result = search_by_tags(
            vec![],
            false,
            false,
            false,
            10,
            &memory,
            OutputFormat::Human,
            false,
        )
        .await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("At least one tag must be specified"));
    }

    #[tokio::test]
    async fn test_search_by_tags_dry_run() {
        let memory = SelfLearningMemory::new();
        let result = search_by_tags(
            vec!["rust".to_string()],
            true,
            true,
            true,
            5,
            &memory,
            OutputFormat::Human,
            true,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_by_tags_limit_clamped_below() {
        let memory = SelfLearningMemory::new();
        let result = search_by_tags(
            vec!["rust".to_string()],
            false,
            false,
            false,
            0,
            &memory,
            OutputFormat::Human,
            true,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_by_tags_limit_clamped_above() {
        let memory = SelfLearningMemory::new();
        let result = search_by_tags(
            vec!["rust".to_string()],
            false,
            false,
            false,
            9999,
            &memory,
            OutputFormat::Human,
            true,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_by_tags_exact_match() {
        let memory = SelfLearningMemory::new();
        let result = search_by_tags(
            vec!["rust".to_string()],
            false,
            false,
            false,
            10,
            &memory,
            OutputFormat::Human,
            true,
        )
        .await;
        assert!(result.is_ok());
    }
}

//! Tag command implementations

use super::types::{
    TagAddResult, TagCommands, TagListResult, TagRemoveResult, TagRenameResult, TagSearchEpisode,
    TagSearchResult, TagSetResult, TagShowResult, TagStatEntry, TagStatsDetailedEntry,
    TagStatsDetailedResult, TagStatsResult,
};
use crate::config::Config;
use crate::output::OutputFormat;
use memory_core::SelfLearningMemory;
use uuid::Uuid;

mod stats_ops;
use stats_ops::{rename_tag, show_tag_stats};

/// Handle tag subcommands
pub async fn handle_tag_command(
    command: TagCommands,
    memory: &SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    match command {
        TagCommands::Add {
            episode_id,
            tags,
            color,
        } => add_tags(episode_id, tags, color, memory, format, dry_run).await,
        TagCommands::Remove { episode_id, tags } => {
            remove_tags(episode_id, tags, memory, format, dry_run).await
        }
        TagCommands::Set { episode_id, tags } => {
            set_tags(episode_id, tags, memory, format, dry_run).await
        }
        TagCommands::List { episode, sort_by } => {
            if let Some(episode_id) = episode {
                list_episode_tags(episode_id, memory, format, dry_run).await
            } else {
                list_all_tags(sort_by, memory, format, dry_run).await
            }
        }
        TagCommands::Search {
            tags,
            all,
            partial,
            case_sensitive,
            limit,
        } => {
            search_by_tags(
                tags,
                all,
                partial,
                case_sensitive,
                limit,
                memory,
                format,
                dry_run,
            )
            .await
        }
        TagCommands::Show { episode_id } => {
            show_episode_with_tags(episode_id, memory, format, dry_run).await
        }
        TagCommands::Rename {
            old_tag,
            new_tag,
            dry_run: cmd_dry_run,
        } => rename_tag(old_tag, new_tag, memory, format, dry_run || cmd_dry_run).await,
        TagCommands::Stats { top, sort } => {
            show_tag_stats(top, sort, memory, format, dry_run).await
        }
    }
}

/// Add tags to an episode
pub async fn add_tags(
    episode_id: String,
    tags: Vec<String>,
    color: Option<String>,
    memory: &SelfLearningMemory,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    if tags.is_empty() {
        return Err(anyhow::anyhow!("At least one tag must be specified"));
    }

    // Validate color if provided
    let color = color.map(|c| c.to_lowercase());
    if let Some(ref c) = color {
        let valid_colors = [
            "red", "green", "blue", "yellow", "orange", "purple", "pink", "cyan", "gray",
        ];
        if !valid_colors.contains(&c.as_str()) {
            return Err(anyhow::anyhow!(
                "Invalid color '{}'. Valid colors are: {}",
                c,
                valid_colors.join(", ")
            ));
        }
    }

    if dry_run {
        let color_msg = color
            .map(|c| format!(" with color '{}'", c))
            .unwrap_or_default();
        println!(
            "Would add {} tag(s) to episode {}{}: {}",
            tags.len(),
            episode_id,
            color_msg,
            tags.join(", ")
        );
        return Ok(());
    }

    let uuid = parse_episode_id(&episode_id)?;

    // Get current tags before adding
    let tags_before = memory.get_episode_tags(uuid).await?;
    let before_count = tags_before.len();

    // Add tags (note: color is currently informational only - stored in metadata would require storage changes)
    memory.add_episode_tags(uuid, tags).await?;

    // Get updated tags
    let current_tags = memory.get_episode_tags(uuid).await?;
    let tags_added = current_tags.len() - before_count;

    let result = TagAddResult {
        episode_id,
        tags_added,
        current_tags,
        success: tags_added > 0,
    };

    format.print_output(&result)
}

/// Remove tags from an episode
pub async fn remove_tags(
    episode_id: String,
    tags: Vec<String>,
    memory: &SelfLearningMemory,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    if tags.is_empty() {
        return Err(anyhow::anyhow!("At least one tag must be specified"));
    }

    if dry_run {
        println!(
            "Would remove {} tag(s) from episode {}: {}",
            tags.len(),
            episode_id,
            tags.join(", ")
        );
        return Ok(());
    }

    let uuid = parse_episode_id(&episode_id)?;

    // Get current tags before removing
    let tags_before = memory.get_episode_tags(uuid).await?;
    let before_count = tags_before.len();

    // Remove tags
    memory.remove_episode_tags(uuid, tags).await?;

    // Get updated tags
    let current_tags = memory.get_episode_tags(uuid).await?;
    let tags_removed = before_count - current_tags.len();

    let result = TagRemoveResult {
        episode_id,
        tags_removed,
        current_tags,
        success: tags_removed > 0,
    };

    format.print_output(&result)
}

/// List tags for a specific episode
pub async fn list_episode_tags(
    episode_id: String,
    memory: &SelfLearningMemory,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    if dry_run {
        println!("Would list tags for episode {}", episode_id);
        return Ok(());
    }

    let uuid = parse_episode_id(&episode_id)?;

    // Get tags for episode
    let tags = memory.get_episode_tags(uuid).await?;

    let result = TagListResult {
        episode_id,
        tags: tags.clone(),
        count: tags.len(),
    };

    format.print_output(&result)
}

/// Set/replace all tags on an episode
pub async fn set_tags(
    episode_id: String,
    tags: Vec<String>,
    memory: &SelfLearningMemory,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    if dry_run {
        println!(
            "Would set {} tag(s) on episode {}: {}",
            tags.len(),
            episode_id,
            tags.join(", ")
        );
        return Ok(());
    }

    let uuid = parse_episode_id(&episode_id)?;

    // Set tags
    memory.set_episode_tags(uuid, tags).await?;

    // Get updated tags
    let current_tags = memory.get_episode_tags(uuid).await?;

    let result = TagSetResult {
        episode_id,
        tags_set: current_tags.len(),
        current_tags,
        success: true,
    };

    format.print_output(&result)
}

/// List all tags with statistics (system-wide)
pub async fn list_all_tags(
    sort_by: String,
    memory: &SelfLearningMemory,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    if dry_run {
        println!(
            "Would list all tags with statistics (sorted by: {})",
            sort_by
        );
        return Ok(());
    }

    // Validate sort_by parameter
    let sort_by = sort_by.to_lowercase();
    if !["count", "name", "recent"].contains(&sort_by.as_str()) {
        return Err(anyhow::anyhow!(
            "Invalid sort-by value: '{}'. Must be one of: count, name, recent",
            sort_by
        ));
    }

    // Get tag statistics from memory
    let stats = memory.get_tag_statistics().await?;

    // Convert to tag entries
    let mut tag_entries: Vec<TagStatEntry> = stats
        .into_iter()
        .map(|(tag, stat)| TagStatEntry {
            tag,
            usage_count: stat.usage_count,
            first_used: stat.first_used.format("%Y-%m-%d %H:%M").to_string(),
            last_used: stat.last_used.format("%Y-%m-%d %H:%M").to_string(),
        })
        .collect();

    // Sort according to the sort_by parameter
    match sort_by.as_str() {
        "count" => {
            // Sort by usage count (descending), then by name
            tag_entries.sort_by(|a, b| {
                b.usage_count
                    .cmp(&a.usage_count)
                    .then_with(|| a.tag.cmp(&b.tag))
            });
        }
        "recent" => {
            // Sort by last used (descending), then by name
            tag_entries.sort_by(|a, b| {
                b.last_used
                    .cmp(&a.last_used)
                    .then_with(|| a.tag.cmp(&b.tag))
            });
        }
        _ => {
            // Default: sort by name (alphabetical)
            tag_entries.sort_by(|a, b| a.tag.cmp(&b.tag));
        }
    }

    let total_usage: usize = tag_entries.iter().map(|e| e.usage_count).sum();

    let result = TagStatsResult {
        total_tags: tag_entries.len(),
        total_usage,
        sort_by,
        tags: tag_entries,
    };

    format.print_output(&result)
}

/// Search episodes by tags
#[allow(clippy::too_many_arguments)]
pub async fn search_by_tags(
    tags: Vec<String>,
    require_all: bool,
    partial: bool,
    case_sensitive: bool,
    limit: usize,
    memory: &SelfLearningMemory,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    if tags.is_empty() {
        return Err(anyhow::anyhow!("At least one tag must be specified"));
    }

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

/// Show episode details with its tags
pub async fn show_episode_with_tags(
    episode_id: String,
    memory: &SelfLearningMemory,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    if dry_run {
        println!("Would show episode {} with tags", episode_id);
        return Ok(());
    }

    let uuid = parse_episode_id(&episode_id)?;

    // Get episode
    let episode = memory.get_episode(uuid).await?;

    // Get tags
    let tags = memory.get_episode_tags(uuid).await?;

    // Extract values before creating result
    let task_description = episode.task_description.clone();
    let duration_ms = episode.duration().map(|d| d.num_milliseconds() as u64);

    let result = TagShowResult {
        episode_id: episode.episode_id.to_string(),
        task_description,
        status: if episode.end_time.is_some() {
            "completed".to_string()
        } else {
            "in_progress".to_string()
        },
        created_at: episode.start_time.to_rfc3339(),
        completed_at: episode.end_time.map(|t| t.to_rfc3339()),
        duration_ms,
        outcome: episode.outcome.map(|o| format!("{:?}", o)),
        tags_count: tags.len(),
        tags,
    };

    format.print_output(&result)
}

/// Parse episode ID string into UUID
fn parse_episode_id(episode_id: &str) -> anyhow::Result<Uuid> {
    Uuid::parse_str(episode_id)
        .map_err(|e| anyhow::anyhow!("Invalid episode ID '{}': {}", episode_id, e))
}

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

/// Rename a tag across all episodes
pub async fn rename_tag(
    old_tag: String,
    new_tag: String,
    memory: &SelfLearningMemory,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    if old_tag.is_empty() || new_tag.is_empty() {
        return Err(anyhow::anyhow!(
            "Both old and new tag names must be specified"
        ));
    }

    if old_tag.to_lowercase() == new_tag.to_lowercase() {
        return Err(anyhow::anyhow!(
            "Old tag '{}' and new tag '{}' are the same (case-insensitive)",
            old_tag,
            new_tag
        ));
    }

    // Get all episodes
    let all_episodes = memory.get_all_episodes().await?;

    // Find episodes that have the old tag
    let mut affected_episodes = Vec::new();
    for episode in &all_episodes {
        if episode
            .tags
            .iter()
            .any(|t| t.eq_ignore_ascii_case(&old_tag))
        {
            affected_episodes.push(episode);
        }
    }

    if dry_run {
        println!(
            "Would rename tag '{}' to '{}' across {} episode(s)",
            old_tag,
            new_tag,
            affected_episodes.len()
        );
        for ep in &affected_episodes {
            println!("  - {}", ep.episode_id);
        }
        return Ok(());
    }

    if affected_episodes.is_empty() {
        let result = TagRenameResult {
            old_tag,
            new_tag,
            episodes_affected: 0,
            success: false,
        };
        return format.print_output(&result);
    }

    // Perform the rename on each affected episode
    let mut count = 0;
    for episode in affected_episodes {
        let mut new_tags: Vec<String> = episode
            .tags
            .iter()
            .filter(|t| !t.eq_ignore_ascii_case(&old_tag))
            .cloned()
            .collect();

        // Check if new tag already exists on this episode
        let has_new_tag = new_tags.iter().any(|t| t.eq_ignore_ascii_case(&new_tag));
        if !has_new_tag {
            new_tags.push(new_tag.clone());
        }

        memory
            .set_episode_tags(episode.episode_id, new_tags)
            .await?;
        count += 1;
    }

    let result = TagRenameResult {
        old_tag,
        new_tag,
        episodes_affected: count,
        success: count > 0,
    };

    format.print_output(&result)
}

/// Show detailed tag statistics
pub async fn show_tag_stats(
    top: Option<usize>,
    sort: String,
    memory: &SelfLearningMemory,
    format: OutputFormat,
    dry_run: bool,
) -> anyhow::Result<()> {
    if dry_run {
        println!("Would show tag statistics (top: {:?}, sort: {})", top, sort);
        return Ok(());
    }

    // Validate sort parameter
    let sort = sort.to_lowercase();
    if !["count", "name", "recent"].contains(&sort.as_str()) {
        return Err(anyhow::anyhow!(
            "Invalid sort value: '{}'. Must be one of: count, name, recent",
            sort
        ));
    }

    // Get all episodes for statistics
    let all_episodes = memory.get_all_episodes().await?;
    let total_episodes = all_episodes.len();

    // Calculate tag statistics
    let mut stats_map: std::collections::HashMap<String, (usize, i64, i64)> =
        std::collections::HashMap::new();

    for episode in &all_episodes {
        for tag in &episode.tags {
            let entry = stats_map.entry(tag.clone()).or_insert((
                0,
                episode.start_time.timestamp(),
                episode.start_time.timestamp(),
            ));
            entry.0 += 1; // count
            entry.1 = entry.1.min(episode.start_time.timestamp()); // first_used
            entry.2 = entry.2.max(episode.start_time.timestamp()); // last_used
        }
    }

    // Calculate derived statistics
    let total_tags = stats_map.len();
    let total_usage: usize = stats_map.values().map(|(c, _, _)| c).sum();
    let avg_tags_per_episode = if total_episodes > 0 {
        total_usage as f64 / total_episodes as f64
    } else {
        0.0
    };

    // Find most and least used tags
    let mut sorted_by_count: Vec<_> = stats_map.iter().collect();
    sorted_by_count.sort_by(|a, b| b.1.0.cmp(&a.1.0));
    let most_used_tag = sorted_by_count.first().map(|(t, _)| (*t).clone());
    let least_used_tag = sorted_by_count.last().map(|(t, _)| (*t).clone());

    // Build entries
    let mut entries: Vec<TagStatsDetailedEntry> = stats_map
        .into_iter()
        .map(
            |(tag, (count, first_used, last_used))| TagStatsDetailedEntry {
                tag,
                usage_count: count,
                percentage: if total_usage > 0 {
                    (count as f64 / total_usage as f64) * 100.0
                } else {
                    0.0
                },
                first_used: chrono::DateTime::from_timestamp(first_used, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_default(),
                last_used: chrono::DateTime::from_timestamp(last_used, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_default(),
                average_per_episode: if total_episodes > 0 {
                    count as f64 / total_episodes as f64
                } else {
                    0.0
                },
            },
        )
        .collect();

    // Sort according to parameter
    match sort.as_str() {
        "count" => {
            entries.sort_by(|a, b| {
                b.usage_count
                    .cmp(&a.usage_count)
                    .then_with(|| a.tag.cmp(&b.tag))
            });
        }
        "recent" => {
            entries.sort_by(|a, b| {
                b.last_used
                    .cmp(&a.last_used)
                    .then_with(|| a.tag.cmp(&b.tag))
            });
        }
        _ => {
            entries.sort_by(|a, b| a.tag.cmp(&b.tag));
        }
    }

    // Apply top limit if specified
    if let Some(top_n) = top {
        entries.truncate(top_n);
    }

    let result = TagStatsDetailedResult {
        tags: entries,
        total_tags,
        total_usage,
        total_episodes,
        avg_tags_per_episode,
        most_used_tag,
        least_used_tag,
        sort_by: sort,
    };

    format.print_output(&result)
}

/// Parse episode ID string into UUID
fn parse_episode_id(episode_id: &str) -> anyhow::Result<Uuid> {
    Uuid::parse_str(episode_id)
        .map_err(|e| anyhow::anyhow!("Invalid episode ID '{}': {}", episode_id, e))
}

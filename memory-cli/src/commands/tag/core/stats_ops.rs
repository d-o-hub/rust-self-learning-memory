use crate::output::OutputFormat;
use memory_core::SelfLearningMemory;

use super::{TagRenameResult, TagStatsDetailedEntry, TagStatsDetailedResult};

pub(super) async fn rename_tag(
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

    let all_episodes = memory.get_all_episodes().await?;

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

    let mut count = 0;
    for episode in affected_episodes {
        let mut new_tags: Vec<String> = episode
            .tags
            .iter()
            .filter(|t| !t.eq_ignore_ascii_case(&old_tag))
            .cloned()
            .collect();

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

pub(super) async fn show_tag_stats(
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

    let sort = sort.to_lowercase();
    if !["count", "name", "recent"].contains(&sort.as_str()) {
        return Err(anyhow::anyhow!(
            "Invalid sort value: '{}'. Must be one of: count, name, recent",
            sort
        ));
    }

    let all_episodes = memory.get_all_episodes().await?;
    let total_episodes = all_episodes.len();

    let mut stats_map: std::collections::HashMap<String, (usize, i64, i64)> =
        std::collections::HashMap::new();

    for episode in &all_episodes {
        for tag in &episode.tags {
            let entry = stats_map.entry(tag.clone()).or_insert((
                0,
                episode.start_time.timestamp(),
                episode.start_time.timestamp(),
            ));
            entry.0 += 1;
            entry.1 = entry.1.min(episode.start_time.timestamp());
            entry.2 = entry.2.max(episode.start_time.timestamp());
        }
    }

    let total_tags = stats_map.len();
    let total_usage: usize = stats_map.values().map(|(c, _, _)| c).sum();
    let avg_tags_per_episode = if total_episodes > 0 {
        total_usage as f64 / total_episodes as f64
    } else {
        0.0
    };

    let mut sorted_by_count: Vec<_> = stats_map.iter().collect();
    sorted_by_count.sort_by(|a, b| b.1.0.cmp(&a.1.0));
    let most_used_tag = sorted_by_count.first().map(|(t, _)| (*t).clone());
    let least_used_tag = sorted_by_count.last().map(|(t, _)| (*t).clone());

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

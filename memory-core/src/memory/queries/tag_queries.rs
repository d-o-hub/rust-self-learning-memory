use crate::Result;
use crate::episode::Episode;
use crate::memory::SelfLearningMemory;
use std::collections::HashMap;
use tracing::{debug, info};

/// Tag statistics
#[derive(Debug, Clone)]
pub struct TagStats {
    pub tag: String,
    pub usage_count: usize,
    pub first_used: chrono::DateTime<chrono::Utc>,
    pub last_used: chrono::DateTime<chrono::Utc>,
}

/// List episodes that have the specified tags
///
/// # Arguments
///
/// * `memory` - Memory instance
/// * `tags` - Tags to search for
/// * `match_all` - If true, episodes must have ALL tags; if false, ANY tag
/// * `limit` - Maximum number of episodes to return
pub async fn list_episodes_by_tags(
    memory: &SelfLearningMemory,
    tags: Vec<String>,
    match_all: bool,
    limit: Option<usize>,
) -> Result<Vec<Episode>> {
    debug!(
        "Querying episodes by tags (match_all={}): {:?}",
        match_all, tags
    );

    if tags.is_empty() {
        return Ok(Vec::new());
    }

    // Get all episodes and filter by tags
    let all_episodes = super::get_all_episodes(
        &memory.episodes_fallback,
        memory.cache_storage.as_ref(),
        memory.turso_storage.as_ref(),
    )
    .await?;

    let mut matching: Vec<Episode> = all_episodes
        .into_iter()
        .filter(|ep| {
            if match_all {
                // Episode must have ALL specified tags
                tags.iter().all(|tag| ep.has_tag(tag))
            } else {
                // Episode must have ANY specified tag
                tags.iter().any(|tag| ep.has_tag(tag))
            }
        })
        .collect();

    // Sort by creation time (newest first)
    matching.sort_by(|a, b| b.start_time.cmp(&a.start_time));

    // Apply limit
    if let Some(limit) = limit {
        matching.truncate(limit);
    }

    info!("Found {} episodes matching tags", matching.len());
    Ok(matching)
}

/// Get all unique tags across all episodes
pub async fn get_all_tags(memory: &SelfLearningMemory) -> Result<Vec<String>> {
    debug!("Fetching all tags");

    let all_episodes = super::get_all_episodes(
        &memory.episodes_fallback,
        memory.cache_storage.as_ref(),
        memory.turso_storage.as_ref(),
    )
    .await?;

    let mut tags = std::collections::HashSet::new();
    for episode in all_episodes {
        for tag in &episode.tags {
            tags.insert(tag.clone());
        }
    }

    let mut tags: Vec<String> = tags.into_iter().collect();
    tags.sort();

    info!("Retrieved {} unique tags", tags.len());
    Ok(tags)
}

/// Get tag statistics
pub async fn get_tag_statistics(memory: &SelfLearningMemory) -> Result<HashMap<String, TagStats>> {
    debug!("Fetching tag statistics");

    let all_episodes = super::get_all_episodes(
        &memory.episodes_fallback,
        memory.cache_storage.as_ref(),
        memory.turso_storage.as_ref(),
    )
    .await?;

    let mut stats: HashMap<String, TagStats> = HashMap::new();

    for episode in all_episodes {
        for tag in &episode.tags {
            stats
                .entry(tag.clone())
                .and_modify(|s| {
                    s.usage_count += 1;
                    if episode.start_time < s.first_used {
                        s.first_used = episode.start_time;
                    }
                    if episode.start_time > s.last_used {
                        s.last_used = episode.start_time;
                    }
                })
                .or_insert(TagStats {
                    tag: tag.clone(),
                    usage_count: 1,
                    first_used: episode.start_time,
                    last_used: episode.start_time,
                });
        }
    }

    info!("Calculated statistics for {} tags", stats.len());
    Ok(stats)
}

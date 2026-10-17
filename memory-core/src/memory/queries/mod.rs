//! Episode and pattern retrieval queries for `SelfLearningMemory`.
//!
//! This module provides methods for lazy loading episodes and patterns
//! from storage backends, with proper fallback handling.

// Type alias for complex nested type to avoid parser issues with >>>>
type EpisodeMap = tokio::sync::RwLock<HashMap<Uuid, Arc<Episode>>>;

use crate::Result;
use crate::episode::Episode;
use crate::patterns::Pattern;
use chrono::{TimeZone, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

mod batch;
mod tag_queries;

pub use batch::get_episodes_by_ids;
pub use tag_queries::{TagStats, get_all_tags, get_tag_statistics, list_episodes_by_tags};

/// Default limit for `get_all_episodes` (used for backfilling)
const GET_ALL_EPISODES_LIMIT: Option<usize> = Some(1000);

/// Merge a candidate episode into the in-progress candidate map using revision-aware selection.
fn merge_candidate_episode(
    map: &mut HashMap<Uuid, (Arc<Episode>, crate::sync::EpisodeOrigin)>,
    episode: Episode,
    origin: crate::sync::EpisodeOrigin,
) {
    use crate::sync::select_winning_episode;

    let incoming = Arc::new(episode);
    let id = incoming.episode_id;
    match map.entry(id) {
        std::collections::hash_map::Entry::Occupied(mut entry) => {
            let (existing, existing_origin) = entry.get();
            let winner = select_winning_episode(existing, *existing_origin, &incoming, origin);
            let winner_origin = if Arc::ptr_eq(&winner, &incoming) {
                origin
            } else {
                *existing_origin
            };
            entry.insert((winner, winner_origin));
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
            entry.insert((incoming, origin));
        }
    }
}

/// Get all episodes with proper lazy loading from storage backends.
///
/// This method implements the lazy loading pattern: memory → redb → Turso.
/// It first checks the in-memory cache, then falls back to cache storage
/// (redb), and finally to durable storage (Turso) if needed.
///
/// Used primarily for backfilling embeddings and comprehensive episode retrieval.
/// Uses a higher limit (1000) since this is used for batch operations.
pub async fn get_all_episodes(
    episodes_fallback: &tokio::sync::RwLock<HashMap<Uuid, Arc<Episode>>>,
    cache_storage: Option<&Arc<dyn crate::StorageBackend>>,
    turso_storage: Option<&Arc<dyn crate::StorageBackend>>,
) -> Result<Vec<Episode>> {
    use crate::sync::{EpisodeOrigin, select_winning_episode};

    // 1) Start with in-memory episodes - collect Arcs directly with origin tracking
    let mut all_episodes: HashMap<Uuid, (Arc<Episode>, EpisodeOrigin)> = {
        let episodes = episodes_fallback.read().await;
        episodes
            .iter()
            .map(|(id, ep)| (*id, (Arc::clone(ep), EpisodeOrigin::InMemory)))
            .collect()
    };

    // 2) Try to fetch from cache storage (redb) if we might be missing episodes
    if let Some(cache) = cache_storage {
        let since = Utc
            .timestamp_millis_opt(0)
            .single()
            .unwrap_or_else(Utc::now);
        if let Ok(cache_episodes) = cache
            .query_episodes_since(since, GET_ALL_EPISODES_LIMIT)
            .await
        {
            debug!(
                cache_count = cache_episodes.len(),
                "Fetched episodes from cache storage"
            );
            for episode in cache_episodes {
                merge_candidate_episode(&mut all_episodes, episode, EpisodeOrigin::Cache);
            }
        }
    }

    // 3) Try to fetch from durable storage (Turso) for completeness
    if let Some(turso) = turso_storage {
        let since = Utc
            .timestamp_millis_opt(0)
            .single()
            .unwrap_or_else(Utc::now);
        if let Ok(turso_episodes) = turso
            .query_episodes_since(since, GET_ALL_EPISODES_LIMIT)
            .await
        {
            debug!(
                turso_count = turso_episodes.len(),
                "Fetched episodes from durable storage"
            );
            for episode in turso_episodes {
                merge_candidate_episode(&mut all_episodes, episode, EpisodeOrigin::Durable);
            }
        }
    }

    // 4) Update in-memory cache with winning episodes, ensuring newer in-memory values are preserved
    {
        let mut episodes_cache = episodes_fallback.write().await;
        for (id, (winner, winner_origin)) in &mut all_episodes {
            if let Some(existing_in_mem) = episodes_cache.get(id) {
                let final_winner = select_winning_episode(
                    existing_in_mem,
                    EpisodeOrigin::InMemory,
                    winner,
                    *winner_origin,
                );
                if !Arc::ptr_eq(&final_winner, existing_in_mem) {
                    episodes_cache.insert(*id, Arc::clone(&final_winner));
                }
                *winner = final_winner;
            } else {
                episodes_cache.insert(*id, Arc::clone(winner));
            }
        }
    }

    let total_count = all_episodes.len();
    info!(
        total_episodes = total_count,
        "Retrieved all episodes from all storage backends"
    );

    // Convert Arc<Episode> to Episode for public API (dereference and clone each)
    Ok(all_episodes
        .into_values()
        .map(|(arc_ep, _)| (*arc_ep).clone())
        .collect())
}

/// Get all patterns with proper lazy loading from storage backends.
///
/// This method implements the lazy loading pattern: memory → redb → Turso.
/// It first checks the in-memory cache, then falls back to cache storage
/// (redb), and finally to durable storage (Turso) if needed. This is what
/// makes `pattern list` / `pattern search` work across process boundaries,
/// since the in-memory `patterns_fallback` is empty in a fresh CLI process.
///
/// Used primarily for backfilling embeddings and comprehensive pattern retrieval.
pub async fn get_all_patterns(
    patterns_fallback: &tokio::sync::RwLock<HashMap<crate::episode::PatternId, Pattern>>,
    cache_storage: Option<&Arc<dyn crate::StorageBackend>>,
    turso_storage: Option<&Arc<dyn crate::StorageBackend>>,
) -> Result<Vec<Pattern>> {
    use crate::episode::PatternId;

    let mut all_patterns: HashMap<PatternId, Pattern> = HashMap::new();

    // 1) In-memory cache
    {
        let patterns = patterns_fallback.read().await;
        for (id, pattern) in patterns.iter() {
            all_patterns.insert(*id, pattern.clone());
        }
    }

    // 2) Cache storage (redb)
    if let Some(cache) = cache_storage {
        match cache.get_all_patterns().await {
            Ok(patterns) => {
                debug!(
                    cache_count = patterns.len(),
                    "Fetched patterns from cache storage"
                );
                for pattern in patterns {
                    all_patterns.entry(pattern.id()).or_insert(pattern);
                }
            }
            Err(e) => {
                // Prefer warn so empty list after backend failure is diagnosable
                // (issue #831 symptom looks identical to "no patterns").
                tracing::warn!("Failed to fetch patterns from cache storage: {}", e);
            }
        }
    }

    // 3) Durable storage (Turso)
    if let Some(turso) = turso_storage {
        match turso.get_all_patterns().await {
            Ok(patterns) => {
                debug!(
                    turso_count = patterns.len(),
                    "Fetched patterns from durable storage"
                );
                for pattern in patterns {
                    all_patterns.entry(pattern.id()).or_insert(pattern);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to fetch patterns from durable storage: {}", e);
            }
        }
    }

    // 4) Update in-memory cache with any newly discovered patterns
    {
        let mut patterns_cache = patterns_fallback.write().await;
        for (id, pattern) in &all_patterns {
            if !patterns_cache.contains_key(id) {
                patterns_cache.insert(*id, pattern.clone());
            }
        }
    }

    let total_count = all_patterns.len();
    info!(
        total_patterns = total_count,
        "Retrieved all patterns from all storage backends"
    );

    Ok(all_patterns.into_values().collect())
}

/// List episodes with optional filtering, using proper lazy loading.
///
/// This is the preferred method for CLI commands and user interfaces
/// that need to list episodes with optional filters.
///
/// # Deprecated Parameters
///
/// The `completed_only` parameter is deprecated. Use `filter.completed_only` instead.
pub async fn list_episodes(
    episodes_fallback: &tokio::sync::RwLock<HashMap<Uuid, Arc<Episode>>>,
    cache_storage: Option<&Arc<dyn crate::StorageBackend>>,
    turso_storage: Option<&Arc<dyn crate::StorageBackend>>,
    limit: Option<usize>,
    offset: Option<usize>,
    completed_only: Option<bool>,
) -> Result<Vec<Episode>> {
    // Get all episodes with lazy loading
    let mut all_episodes =
        get_all_episodes(episodes_fallback, cache_storage, turso_storage).await?;

    // Apply filters (backward compatibility)
    if let Some(true) = completed_only {
        all_episodes.retain(|e| e.is_complete());
    }

    // Sort by start time (newest first) for consistent ordering
    all_episodes.sort_by_key(|b| std::cmp::Reverse(b.start_time));

    // Apply pagination
    let offset = offset.unwrap_or(0);
    if offset > 0 {
        all_episodes.drain(0..offset.min(all_episodes.len()));
    }

    if let Some(limit) = limit {
        all_episodes.truncate(limit);
    }

    info!(
        total_returned = all_episodes.len(),
        "Listed episodes with filters"
    );

    // Episodes are already Vec<Episode> from get_all_episodes
    Ok(all_episodes)
}

/// List episodes with advanced filtering support.
///
/// This is the new preferred method for querying episodes with rich filtering
/// capabilities including tags, date ranges, task types, and more.
///
/// # Arguments
///
/// * `episodes_fallback` - In-memory episode cache
/// * `cache_storage` - Optional cache storage backend (redb)
/// * `turso_storage` - Optional durable storage backend (Turso)
/// * `filter` - Episode filter criteria
/// * `limit` - Maximum number of episodes to return
/// * `offset` - Number of episodes to skip (for pagination)
///
/// # Examples
///
/// ```
/// use do_memory_core::{SelfLearningMemory, EpisodeFilter, TaskType};
///
/// # async fn example() -> anyhow::Result<()> {
/// let memory = SelfLearningMemory::new();
///
/// // Get recent successful episodes
/// let filter = EpisodeFilter::builder()
///     .success_only(true)
///     .exclude_archived(true)
///     .build();
///
/// let episodes = memory.list_episodes_filtered(filter, Some(10), None).await?;
/// # Ok(())
/// # }
/// ```
pub async fn list_episodes_filtered(
    episodes_fallback: &EpisodeMap,
    cache_storage: Option<&Arc<dyn crate::StorageBackend>>,
    turso_storage: Option<&Arc<dyn crate::StorageBackend>>,
    filter: super::filters::EpisodeFilter,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<Episode>> {
    // Get all episodes with lazy loading (now returns Vec<Episode>)
    let all_episodes = get_all_episodes(episodes_fallback, cache_storage, turso_storage).await?;

    // Apply filter - use apply() for Vec<Episode>
    let mut filtered = filter.apply(all_episodes);

    // Sort by start time (newest first) for consistent ordering
    filtered.sort_by_key(|b| std::cmp::Reverse(b.start_time));

    // Apply pagination
    let offset = offset.unwrap_or(0);
    if offset > 0 {
        filtered.drain(0..offset.min(filtered.len()));
    }

    if let Some(limit) = limit {
        filtered.truncate(limit);
    }

    info!(
        total_returned = filtered.len(),
        "Listed episodes with advanced filters"
    );

    Ok(filtered)
}

/// Get patterns extracted from a specific episode
#[expect(clippy::unused_async)]
pub async fn get_episode_patterns(
    _episode_id: Uuid,
    _patterns_fallback: &tokio::sync::RwLock<HashMap<crate::episode::PatternId, Pattern>>,
) -> Result<Vec<Pattern>> {
    // For now, return empty vector
    // In a real implementation, this would query the storage backend
    // to find patterns that were extracted from the given episode
    Ok(vec![])
}

/// Check if Turso storage is configured
pub fn has_turso_storage(turso_storage: &Option<Arc<dyn crate::StorageBackend>>) -> bool {
    turso_storage.is_some()
}

/// Check if cache storage is configured
pub fn has_cache_storage(cache_storage: &Option<Arc<dyn crate::StorageBackend>>) -> bool {
    cache_storage.is_some()
}

/// Get a reference to the Turso storage backend (if configured)
pub fn turso_storage(
    turso_storage: &Option<Arc<dyn crate::StorageBackend>>,
) -> Option<&Arc<dyn crate::StorageBackend>> {
    turso_storage.as_ref()
}

/// Get a reference to the cache storage backend (if configured)
pub fn cache_storage(
    cache_storage: &Option<Arc<dyn crate::StorageBackend>>,
) -> Option<&Arc<dyn crate::StorageBackend>> {
    cache_storage.as_ref()
}

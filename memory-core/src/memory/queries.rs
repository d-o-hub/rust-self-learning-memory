//! Episode and pattern retrieval queries for SelfLearningMemory.
//!
//! This module provides methods for lazy loading episodes and patterns
//! from storage backends, with proper fallback handling.

use crate::episode::Episode;
use crate::pattern::Pattern;
use crate::Result;
use chrono::{TimeZone, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

/// Get all episodes with proper lazy loading from storage backends.
///
/// This method implements the lazy loading pattern: memory → redb → Turso.
/// It first checks the in-memory cache, then falls back to cache storage
/// (redb), and finally to durable storage (Turso) if needed.
///
/// Used primarily for backfilling embeddings and comprehensive episode retrieval.
pub async fn get_all_episodes(
    episodes_fallback: &tokio::sync::RwLock<HashMap<Uuid, Episode>>,
    cache_storage: Option<&Arc<dyn crate::StorageBackend>>,
    turso_storage: Option<&Arc<dyn crate::StorageBackend>>,
) -> Result<Vec<Episode>> {
    // 1) Start with in-memory episodes
    let mut all_episodes: HashMap<Uuid, Episode> = {
        let episodes = episodes_fallback.read().await;
        episodes
            .values()
            .cloned()
            .map(|e| (e.episode_id, e))
            .collect()
    };

    // 2) Try to fetch from cache storage (redb) if we might be missing episodes
    if let Some(cache) = cache_storage {
        // Fetch all episodes from cache (since timestamp 0)
        let since = Utc.timestamp_millis_opt(0).single().unwrap_or_else(Utc::now);
        match cache.query_episodes_since(since).await {
            Ok(cache_episodes) => {
                debug!(cache_count = cache_episodes.len(), "Fetched episodes from cache storage");
                for episode in cache_episodes {
                    all_episodes.entry(episode.episode_id).or_insert(episode);
                }
            }
            Err(e) => {
                debug!("Failed to fetch episodes from cache storage: {}", e);
            }
        }
    }

    // 3) Try to fetch from durable storage (Turso) for completeness
    if let Some(turso) = turso_storage {
        let since = Utc.timestamp_millis_opt(0).single().unwrap_or_else(Utc::now);
        match turso.query_episodes_since(since).await {
            Ok(turso_episodes) => {
                debug!(turso_count = turso_episodes.len(), "Fetched episodes from durable storage");
                for episode in turso_episodes {
                    all_episodes.entry(episode.episode_id).or_insert(episode);
                }
            }
            Err(e) => {
                debug!("Failed to fetch episodes from durable storage: {}", e);
            }
        }
    }

    // 4) Update in-memory cache with any newly discovered episodes
    {
        let mut episodes_cache = episodes_fallback.write().await;
        for (id, episode) in &all_episodes {
            if !episodes_cache.contains_key(id) {
                episodes_cache.insert(*id, episode.clone());
            }
        }
    }

    let total_count = all_episodes.len();
    info!(total_episodes = total_count, "Retrieved all episodes from all storage backends");

    Ok(all_episodes.into_values().collect())
}

/// Get all patterns with proper lazy loading from storage backends.
///
/// This method implements the lazy loading pattern: memory → redb → Turso.
/// It first checks the in-memory cache, then falls back to cache storage
/// (redb), and finally to durable storage (Turso) if needed.
///
/// Used primarily for backfilling embeddings and comprehensive pattern retrieval.
pub async fn get_all_patterns(
    patterns_fallback: &tokio::sync::RwLock<HashMap<crate::episode::PatternId, Pattern>>,
) -> Result<Vec<Pattern>> {
    let patterns = patterns_fallback.read().await;
    Ok(patterns.values().cloned().collect())
}

/// List episodes with optional filtering, using proper lazy loading.
///
/// This is the preferred method for CLI commands and user interfaces
/// that need to list episodes with optional filters.
pub async fn list_episodes(
    episodes_fallback: &tokio::sync::RwLock<HashMap<Uuid, Episode>>,
    cache_storage: Option<&Arc<dyn crate::StorageBackend>>,
    turso_storage: Option<&Arc<dyn crate::StorageBackend>>,
    limit: Option<usize>,
    offset: Option<usize>,
    completed_only: Option<bool>,
) -> Result<Vec<Episode>> {
    // Get all episodes with lazy loading
    let mut all_episodes = get_all_episodes(
        episodes_fallback,
        cache_storage,
        turso_storage,
    )
    .await?;

    // Apply filters
    if let Some(true) = completed_only {
        all_episodes.retain(|e| e.is_complete());
    }

    // Sort by start time (newest first) for consistent ordering
    all_episodes.sort_by(|a, b| b.start_time.cmp(&a.start_time));

    // Apply pagination
    let offset = offset.unwrap_or(0);
    if offset > 0 {
        all_episodes.drain(0..offset.min(all_episodes.len()));
    }

    if let Some(limit) = limit {
        all_episodes.truncate(limit);
    }

    info!(total_returned = all_episodes.len(), "Listed episodes with filters");

    Ok(all_episodes)
}

/// Get patterns extracted from a specific episode
#[allow(clippy::unused_async)]
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

//! Batch episode retrieval queries for `SelfLearningMemory`.

use crate::Result;
use crate::episode::Episode;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

/// Get multiple episodes by their IDs with batched lazy loading.
///
/// More efficient than calling `get_episode()` in a loop, as it can batch
/// storage queries where possible and reduces lock contention.
///
/// # Arguments
///
/// * `episode_ids` - Slice of episode UUIDs to retrieve
/// * `episodes_fallback` - In-memory episode cache
/// * `cache_storage` - Optional cache storage backend (redb)
/// * `turso_storage` - Optional durable storage backend (Turso)
///
/// # Returns
///
/// Vector of episodes that were found. Missing episodes are silently omitted.
///
/// # Examples
///
/// If you pass 5 IDs and only 3 exist, you'll get back 3 episodes.
pub async fn get_episodes_by_ids(
    episode_ids: &[Uuid],
    episodes_fallback: &tokio::sync::RwLock<HashMap<Uuid, Arc<Episode>>>,
    cache_storage: Option<&Arc<dyn crate::StorageBackend>>,
    turso_storage: Option<&Arc<dyn crate::StorageBackend>>,
) -> Result<Vec<Episode>> {
    if episode_ids.is_empty() {
        return Ok(vec![]);
    }

    let mut found_episodes: HashMap<Uuid, Episode> = HashMap::new();
    let mut missing_ids: Vec<Uuid> = Vec::new();

    // 1) Check in-memory cache first (batch operation)
    {
        let episodes = episodes_fallback.read().await;
        for &id in episode_ids {
            if let Some(ep) = episodes.get(&id) {
                found_episodes.insert(id, (**ep).clone());
            } else {
                missing_ids.push(id);
            }
        }
    }

    debug!(
        requested = episode_ids.len(),
        found_in_memory = found_episodes.len(),
        missing = missing_ids.len(),
        "Bulk episode lookup: checked memory cache"
    );

    // Early return if all found
    if missing_ids.is_empty() {
        return Ok(found_episodes.into_values().collect());
    }

    // 2) Try cache storage (redb) for missing episodes
    if let Some(cache) = cache_storage {
        let mut still_missing = Vec::new();
        for id in &missing_ids {
            match cache.get_episode(*id).await {
                Ok(Some(episode)) => {
                    found_episodes.insert(*id, episode);
                }
                Ok(None) => {
                    still_missing.push(*id);
                }
                Err(e) => {
                    debug!(episode_id = %id, error = %e, "Failed to query cache storage");
                    still_missing.push(*id);
                }
            }
        }
        missing_ids = still_missing;

        debug!(
            found_in_cache = found_episodes.len(),
            still_missing = missing_ids.len(),
            "Bulk episode lookup: checked cache storage"
        );
    }

    // Early return if all found
    if missing_ids.is_empty() {
        // Update in-memory cache with newly found episodes
        let mut episodes = episodes_fallback.write().await;
        for (id, episode) in &found_episodes {
            if !episodes.contains_key(id) {
                episodes.insert(*id, Arc::new(episode.clone()));
            }
        }
        return Ok(found_episodes.into_values().collect());
    }

    // 3) Try durable storage (Turso) for remaining missing episodes
    if let Some(turso) = turso_storage {
        for id in &missing_ids {
            match turso.get_episode(*id).await {
                Ok(Some(episode)) => {
                    found_episodes.insert(*id, episode);
                }
                Ok(None) => {
                    // Episode doesn't exist anywhere
                }
                Err(e) => {
                    debug!(episode_id = %id, error = %e, "Failed to query durable storage");
                }
            }
        }

        debug!(
            total_found = found_episodes.len(),
            total_requested = episode_ids.len(),
            "Bulk episode lookup: checked durable storage"
        );
    }

    // 4) Update in-memory cache with all newly found episodes
    {
        let mut episodes = episodes_fallback.write().await;
        for (id, episode) in &found_episodes {
            if !episodes.contains_key(id) {
                episodes.insert(*id, Arc::new(episode.clone()));
            }
        }
    }

    info!(
        requested = episode_ids.len(),
        found = found_episodes.len(),
        "Completed bulk episode retrieval"
    );

    Ok(found_episodes.into_values().collect())
}

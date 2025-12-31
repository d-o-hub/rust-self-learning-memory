//! # Storage Synchronization
//!
//! Simple write-through coordination between Turso (durable) and redb (cache) storage layers.
//!
//! The synchronizer uses a write-through pattern:
//! - All writes go to Turso (durable storage) first - this must succeed
//! - Cache writes to redb are best effort - failures are logged but don't cause errors
//! - Turso is always the source of truth
//!
//! ## Example
//!
//! ```ignore
//! use memory_core::sync::StorageSynchronizer;
//!
//! let sync = StorageSynchronizer::new(turso_storage, redb_storage);
//! sync.store_episode(episode).await?; // Writes to Turso, then caches to redb
//! ```

use crate::{Episode, Error, Heuristic, Pattern, Result};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

/// Storage synchronizer with write-through pattern
///
/// Coordinates writes between durable storage (Turso) and cache (redb) using
/// a simple write-through approach where Turso is source of truth.
pub struct StorageSynchronizer<T, R> {
    /// Durable storage (typically Turso - source of truth)
    pub turso: Arc<T>,
    /// Cache storage (typically redb - best effort)
    pub redb: Arc<R>,
}

impl<T, R> StorageSynchronizer<T, R> {
    /// Create a new storage synchronizer
    #[must_use]
    pub fn new(turso: Arc<T>, redb: Arc<R>) -> Self {
        Self { turso, redb }
    }
}

// Implementations using the StorageBackend trait

impl<T, R> StorageSynchronizer<T, R>
where
    T: crate::storage::StorageBackend + 'static,
    R: crate::storage::StorageBackend + 'static,
{
    /// Store an episode using write-through pattern
    ///
    /// Writes to Turso (durable storage) first, then attempts to cache in redb.
    /// If cache write fails, a warning is logged but the operation still succeeds.
    ///
    /// # Arguments
    ///
    /// * `episode` - Episode to store
    ///
    /// # Errors
    ///
    /// Returns error only if Turso write fails
    pub async fn store_episode(&self, episode: &Episode) -> Result<()> {
        // Write to durable storage first (must succeed)
        self.turso.store_episode(episode).await?;

        // Cache write is best effort
        if let Err(e) = self.redb.store_episode(episode).await {
            warn!(
                "Cache write failed for episode {}: {}",
                episode.episode_id, e
            );
        }

        info!("Stored episode {} (write-through)", episode.episode_id);
        Ok(())
    }

    /// Store a pattern using write-through pattern
    ///
    /// Writes to Turso first, then attempts to cache in redb.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Pattern to store
    ///
    /// # Errors
    ///
    /// Returns error only if Turso write fails
    pub async fn store_pattern(&self, pattern: &Pattern) -> Result<()> {
        // Write to durable storage first (must succeed)
        self.turso.store_pattern(pattern).await?;

        // Cache write is best effort
        let pattern_id = pattern.id();
        if let Err(e) = self.redb.store_pattern(pattern).await {
            warn!("Cache write failed for pattern {}: {}", pattern_id, e);
        }

        info!("Stored pattern {} (write-through)", pattern_id);
        Ok(())
    }

    /// Store a heuristic using write-through pattern
    ///
    /// Writes to Turso first, then attempts to cache in redb.
    ///
    /// # Arguments
    ///
    /// * `heuristic` - Heuristic to store
    ///
    /// # Errors
    ///
    /// Returns error only if Turso write fails
    pub async fn store_heuristic(&self, heuristic: &Heuristic) -> Result<()> {
        // Write to durable storage first (must succeed)
        self.turso.store_heuristic(heuristic).await?;

        // Cache write is best effort
        if let Err(e) = self.redb.store_heuristic(heuristic).await {
            warn!(
                "Cache write failed for heuristic {}: {}",
                heuristic.heuristic_id, e
            );
        }

        info!(
            "Stored heuristic {} (write-through)",
            heuristic.heuristic_id
        );
        Ok(())
    }

    /// Sync a single episode from Turso to redb cache
    ///
    /// Fetches the episode from Turso and stores it in redb cache.
    /// This is useful for manual cache warming or recovery scenarios.
    ///
    /// # Arguments
    ///
    /// * `episode_id` - UUID of the episode to sync
    ///
    /// # Errors
    ///
    /// Returns error if episode not found in Turso
    pub async fn sync_episode_to_cache(&self, episode_id: Uuid) -> Result<()> {
        info!("Syncing episode {} to cache", episode_id);

        // Fetch from Turso (source of truth)
        let episode = self
            .turso
            .get_episode(episode_id)
            .await?
            .ok_or_else(|| Error::Storage(format!("Episode {episode_id} not found in Turso")))?;

        // Store in redb cache (best effort)
        if let Err(e) = self.redb.store_episode(&episode).await {
            warn!(
                "Cache write failed for episode {}: {}",
                episode.episode_id, e
            );
        }

        info!("Successfully synced episode {} to cache", episode_id);
        Ok(())
    }
}

/// Helper trait to get pattern ID from any pattern variant
trait PatternIdGetter {
    fn id(&self) -> crate::episode::PatternId;
}

impl PatternIdGetter for Pattern {
    fn id(&self) -> crate::episode::PatternId {
        match self {
            Pattern::ToolSequence { id, .. } => *id,
            Pattern::DecisionPoint { id, .. } => *id,
            Pattern::ErrorRecovery { id, .. } => *id,
            Pattern::ContextPattern { id, .. } => *id,
        }
    }
}

/// Conflict resolution strategy
///
/// Note: With write-through pattern, conflicts are much less likely since
/// all writes go through Turso first. This is primarily kept for manual
/// cache recovery scenarios.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConflictResolution {
    /// Use Turso (durable) as source of truth
    #[default]
    TursoWins,
    /// Use redb (cache) value
    RedbWins,
    /// Use most recently updated
    MostRecent,
}

/// Resolve conflict between two episodes
#[must_use]
pub fn resolve_episode_conflict(
    turso_episode: &Episode,
    redb_episode: &Episode,
    strategy: ConflictResolution,
) -> Episode {
    match strategy {
        ConflictResolution::TursoWins => turso_episode.clone(),
        ConflictResolution::RedbWins => redb_episode.clone(),
        ConflictResolution::MostRecent => {
            // Compare based on last modification (use end_time or start_time)
            let turso_time = turso_episode
                .end_time
                .unwrap_or(turso_episode.start_time);
            let redb_time = redb_episode.end_time.unwrap_or(redb_episode.start_time);

            if turso_time >= redb_time {
                turso_episode.clone()
            } else {
                redb_episode.clone()
            }
        }
    }
}

/// Resolve conflict between two patterns
#[must_use]
pub fn resolve_pattern_conflict(
    turso_pattern: &Pattern,
    redb_pattern: &Pattern,
    strategy: ConflictResolution,
) -> Pattern {
    match strategy {
        ConflictResolution::TursoWins => turso_pattern.clone(),
        ConflictResolution::RedbWins => redb_pattern.clone(),
        ConflictResolution::MostRecent => {
            // Compare based on success rate
            if turso_pattern.success_rate() >= redb_pattern.success_rate() {
                turso_pattern.clone()
            } else {
                redb_pattern.clone()
            }
        }
    }
}

/// Resolve conflict between two heuristics
#[must_use]
pub fn resolve_heuristic_conflict(
    turso_heuristic: &Heuristic,
    redb_heuristic: &Heuristic,
    strategy: ConflictResolution,
) -> Heuristic {
    match strategy {
        ConflictResolution::TursoWins => turso_heuristic.clone(),
        ConflictResolution::RedbWins => redb_heuristic.clone(),
        ConflictResolution::MostRecent => {
            if turso_heuristic.updated_at >= redb_heuristic.updated_at {
                turso_heuristic.clone()
            } else {
                redb_heuristic.clone()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TaskContext, TaskType};

    #[test]
    fn test_conflict_resolution_turso_wins() {
        let context = TaskContext::default();
        let episode1 = Episode::new("Task 1".to_string(), context.clone(), TaskType::Testing);
        let mut episode2 = Episode::new("Task 2".to_string(), context, TaskType::Testing);
        episode2.episode_id = episode1.episode_id; // Same ID, different content

        let resolved =
            resolve_episode_conflict(&episode1, &episode2, ConflictResolution::TursoWins);
        assert_eq!(resolved.task_description, "Task 1");
    }

    #[test]
    fn test_conflict_resolution_redb_wins() {
        let context = TaskContext::default();
        let episode1 = Episode::new("Task 1".to_string(), context.clone(), TaskType::Testing);
        let mut episode2 = Episode::new("Task 2".to_string(), context, TaskType::Testing);
        episode2.episode_id = episode1.episode_id;

        let resolved = resolve_episode_conflict(&episode1, &episode2, ConflictResolution::RedbWins);
        assert_eq!(resolved.task_description, "Task 2");
    }

    #[test]
    fn test_conflict_resolution_most_recent() {
        let context = TaskContext::default();
        let episode1 = Episode::new("Older task".to_string(), context.clone(), TaskType::Testing);
        let mut episode2 = Episode::new("Newer task".to_string(), context, TaskType::Testing);
        episode2.episode_id = episode1.episode_id;
        episode2.end_time = Some(chrono::Utc::now());

        let resolved = resolve_episode_conflict(&episode1, &episode2, ConflictResolution::MostRecent);
        assert_eq!(resolved.task_description, "Newer task");
    }
}

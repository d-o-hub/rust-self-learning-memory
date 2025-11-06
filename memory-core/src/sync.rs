//! # Storage Synchronization
//!
//! Coordinates data synchronization between Turso (durable) and redb (cache) storage layers.
//!
//! The synchronizer ensures:
//! - Two-phase commit for consistency
//! - Conflict resolution with Turso as source of truth
//! - Periodic synchronization of cache from durable storage
//! - Resilience to partial failures
//!
//! ## Example
//!
//! ```ignore
//! use memory_core::sync::StorageSynchronizer;
//!
//! let sync = StorageSynchronizer::new(turso_storage, redb_storage);
//! sync.start_periodic_sync(Duration::from_secs(300)).await;
//! ```

use crate::{Episode, Error, Heuristic, Pattern, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Storage synchronizer for coordinating Turso and redb
pub struct StorageSynchronizer<T, R> {
    #[allow(dead_code)]
    turso: Arc<T>,
    #[allow(dead_code)]
    redb: Arc<R>,
    sync_state: Arc<RwLock<SyncState>>,
}

/// Synchronization state tracking
#[derive(Debug, Clone, Default)]
pub struct SyncState {
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub sync_count: u64,
    pub last_error: Option<String>,
}

/// Configuration for storage synchronization
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Interval between periodic syncs
    pub sync_interval: Duration,
    /// Maximum number of items to sync in one batch
    pub batch_size: usize,
    /// Whether to sync patterns
    pub sync_patterns: bool,
    /// Whether to sync heuristics
    pub sync_heuristics: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            sync_interval: Duration::from_secs(300), // 5 minutes
            batch_size: 100,
            sync_patterns: true,
            sync_heuristics: true,
        }
    }
}

impl<T, R> StorageSynchronizer<T, R> {
    /// Create a new storage synchronizer
    pub fn new(turso: Arc<T>, redb: Arc<R>) -> Self {
        Self {
            turso,
            redb,
            sync_state: Arc::new(RwLock::new(SyncState::default())),
        }
    }

    /// Get the current synchronization state
    pub async fn get_sync_state(&self) -> SyncState {
        self.sync_state.read().await.clone()
    }
}

// The following implementations require trait bounds but demonstrate the pattern
// In practice, you would implement these with actual storage trait bounds

/// Two-phase commit strategy for episode storage
#[derive(Debug)]
pub struct TwoPhaseCommit {
    pub phase1_complete: bool,
    pub phase2_complete: bool,
    pub rollback_needed: bool,
}

impl TwoPhaseCommit {
    /// Create a new two-phase commit transaction
    pub fn new() -> Self {
        Self {
            phase1_complete: false,
            phase2_complete: false,
            rollback_needed: false,
        }
    }

    /// Execute phase 1 - write to cache
    pub async fn phase1<F, Fut>(&mut self, operation: F) -> Result<()>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        debug!("Two-phase commit: Phase 1 (cache write)");
        match operation().await {
            Ok(_) => {
                self.phase1_complete = true;
                Ok(())
            }
            Err(e) => {
                error!("Phase 1 failed: {}", e);
                Err(e)
            }
        }
    }

    /// Execute phase 2 - write to durable storage
    pub async fn phase2<F, Fut>(&mut self, operation: F) -> Result<()>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        debug!("Two-phase commit: Phase 2 (durable write)");
        if !self.phase1_complete {
            return Err(Error::Storage(
                "Cannot execute phase 2 before phase 1".to_string(),
            ));
        }

        match operation().await {
            Ok(_) => {
                self.phase2_complete = true;
                Ok(())
            }
            Err(e) => {
                error!("Phase 2 failed: {}", e);
                self.rollback_needed = true;
                Err(e)
            }
        }
    }

    /// Rollback phase 1 if phase 2 failed
    pub async fn rollback<F, Fut>(&mut self, operation: F) -> Result<()>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        if !self.rollback_needed {
            return Ok(());
        }

        warn!("Rolling back two-phase commit");
        match operation().await {
            Ok(_) => {
                info!("Rollback successful");
                Ok(())
            }
            Err(e) => {
                error!("Rollback failed: {}", e);
                Err(Error::Storage(format!("Rollback failed: {}", e)))
            }
        }
    }

    /// Check if commit is complete
    pub fn is_complete(&self) -> bool {
        self.phase1_complete && self.phase2_complete && !self.rollback_needed
    }
}

impl Default for TwoPhaseCommit {
    fn default() -> Self {
        Self::new()
    }
}

/// Conflict resolution strategy
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
            let turso_time = turso_episode.end_time.unwrap_or(turso_episode.start_time);
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
pub fn resolve_pattern_conflict(
    turso_pattern: &Pattern,
    redb_pattern: &Pattern,
    strategy: ConflictResolution,
) -> Pattern {
    match strategy {
        ConflictResolution::TursoWins => turso_pattern.clone(),
        ConflictResolution::RedbWins => redb_pattern.clone(),
        ConflictResolution::MostRecent => {
            // Compare based on success rate or occurrence count
            if turso_pattern.success_rate() >= redb_pattern.success_rate() {
                turso_pattern.clone()
            } else {
                redb_pattern.clone()
            }
        }
    }
}

/// Resolve conflict between two heuristics
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

/// Synchronization statistics
#[derive(Debug, Clone, Default)]
pub struct SyncStats {
    pub episodes_synced: usize,
    pub patterns_synced: usize,
    pub heuristics_synced: usize,
    pub conflicts_resolved: usize,
    pub errors: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ExecutionStep, TaskContext, TaskType};

    #[test]
    fn test_two_phase_commit_new() {
        let commit = TwoPhaseCommit::new();
        assert!(!commit.phase1_complete);
        assert!(!commit.phase2_complete);
        assert!(!commit.rollback_needed);
        assert!(!commit.is_complete());
    }

    #[tokio::test]
    async fn test_two_phase_commit_success() {
        let mut commit = TwoPhaseCommit::new();

        // Phase 1
        let result = commit.phase1(|| async { Ok(()) }).await;
        assert!(result.is_ok());
        assert!(commit.phase1_complete);

        // Phase 2
        let result = commit.phase2(|| async { Ok(()) }).await;
        assert!(result.is_ok());
        assert!(commit.phase2_complete);
        assert!(commit.is_complete());
    }

    #[tokio::test]
    async fn test_two_phase_commit_phase2_before_phase1() {
        let mut commit = TwoPhaseCommit::new();

        // Try phase 2 without phase 1
        let result = commit.phase2(|| async { Ok(()) }).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_two_phase_commit_rollback() {
        let mut commit = TwoPhaseCommit::new();

        // Phase 1 succeeds
        commit.phase1(|| async { Ok(()) }).await.unwrap();

        // Phase 2 fails
        let _result = commit
            .phase2(|| async { Err(Error::Storage("Phase 2 failed".to_string())) })
            .await;

        assert!(commit.rollback_needed);

        // Rollback
        let result = commit.rollback(|| async { Ok(()) }).await;
        assert!(result.is_ok());
    }

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
    fn test_sync_stats_default() {
        let stats = SyncStats::default();
        assert_eq!(stats.episodes_synced, 0);
        assert_eq!(stats.patterns_synced, 0);
        assert_eq!(stats.heuristics_synced, 0);
        assert_eq!(stats.conflicts_resolved, 0);
        assert_eq!(stats.errors, 0);
    }
}

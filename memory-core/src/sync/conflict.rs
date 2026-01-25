//! Conflict resolution for storage synchronization

use crate::{Episode, Heuristic, Pattern};
use std::sync::Arc;

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
#[must_use]
pub fn resolve_episode_conflict(
    turso_episode: &Arc<Episode>,
    redb_episode: &Arc<Episode>,
    strategy: ConflictResolution,
) -> Arc<Episode> {
    match strategy {
        ConflictResolution::TursoWins => Arc::clone(turso_episode),
        ConflictResolution::RedbWins => Arc::clone(redb_episode),
        ConflictResolution::MostRecent => {
            // Compare based on last modification (use end_time or start_time)
            let turso_time = turso_episode.end_time.unwrap_or(turso_episode.start_time);
            let redb_time = redb_episode.end_time.unwrap_or(redb_episode.start_time);

            if turso_time >= redb_time {
                Arc::clone(turso_episode)
            } else {
                Arc::clone(redb_episode)
            }
        }
    }
}

/// Resolve conflict between two patterns
#[must_use]
pub fn resolve_pattern_conflict(
    turso_pattern: &Arc<Pattern>,
    redb_pattern: &Arc<Pattern>,
    strategy: ConflictResolution,
) -> Arc<Pattern> {
    match strategy {
        ConflictResolution::TursoWins => Arc::clone(turso_pattern),
        ConflictResolution::RedbWins => Arc::clone(redb_pattern),
        ConflictResolution::MostRecent => {
            // Compare based on success rate or occurrence count
            if turso_pattern.success_rate() >= redb_pattern.success_rate() {
                Arc::clone(turso_pattern)
            } else {
                Arc::clone(redb_pattern)
            }
        }
    }
}

/// Resolve conflict between two heuristics
#[must_use]
pub fn resolve_heuristic_conflict(
    turso_heuristic: &Arc<Heuristic>,
    redb_heuristic: &Arc<Heuristic>,
    strategy: ConflictResolution,
) -> Arc<Heuristic> {
    match strategy {
        ConflictResolution::TursoWins => Arc::clone(turso_heuristic),
        ConflictResolution::RedbWins => Arc::clone(redb_heuristic),
        ConflictResolution::MostRecent => {
            if turso_heuristic.updated_at >= redb_heuristic.updated_at {
                Arc::clone(turso_heuristic)
            } else {
                Arc::clone(redb_heuristic)
            }
        }
    }
}

//! Conflict resolution for storage synchronization

use crate::{Episode, Heuristic, Pattern};

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

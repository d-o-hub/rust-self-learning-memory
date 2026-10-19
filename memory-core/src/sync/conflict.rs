//! Conflict resolution for storage synchronization

use crate::{Episode, Heuristic, Pattern};
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// Origin/source layer of an episode payload
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EpisodeOrigin {
    /// In-memory fallback map
    InMemory,
    /// Fast cache storage (e.g., redb)
    Cache,
    /// Durable storage (e.g., Turso)
    Durable,
}

/// Extract the modification timestamp and whether explicit revision metadata was present.
///
/// Looks for `"modified_at"` in `episode.metadata` (parsed as RFC3339 or epoch millis).
/// If missing or unparseable, falls back to `episode.end_time.unwrap_or(episode.start_time)`.
#[must_use]
pub fn episode_revision_info(episode: &Episode) -> (DateTime<Utc>, bool) {
    if let Some(val) = episode.metadata.get("modified_at") {
        if let Ok(dt) = DateTime::parse_from_rfc3339(val) {
            return (dt.with_timezone(&Utc), true);
        }
        if let Ok(ms) = val.parse::<i64>() {
            if let Some(dt) = DateTime::from_timestamp_millis(ms) {
                return (dt, true);
            }
        }
    }
    (episode.end_time.unwrap_or(episode.start_time), false)
}

/// Select the winning episode version when merging records from different sources.
///
/// Revision-aware comparison rules:
/// 1. Compare modification timestamps (`modified_at`). The strictly greater timestamp wins.
/// 2. If timestamps are equal:
///    a. Explicit revision metadata wins over missing/legacy revision metadata.
///    b. Durable storage (Turso) wins over cache (redb) or in-memory storage.
///    c. If origins are equal, existing record is preserved.
#[must_use]
pub fn select_winning_episode(
    existing: &Arc<Episode>,
    existing_origin: EpisodeOrigin,
    incoming: &Arc<Episode>,
    incoming_origin: EpisodeOrigin,
) -> Arc<Episode> {
    let (existing_time, existing_explicit) = episode_revision_info(existing);
    let (incoming_time, incoming_explicit) = episode_revision_info(incoming);

    match incoming_time.cmp(&existing_time) {
        std::cmp::Ordering::Greater => Arc::clone(incoming),
        std::cmp::Ordering::Less => Arc::clone(existing),
        std::cmp::Ordering::Equal => match (incoming_explicit, existing_explicit) {
            (true, false) => Arc::clone(incoming),
            (false, true) => Arc::clone(existing),
            _ => {
                if incoming_origin >= existing_origin {
                    Arc::clone(incoming)
                } else {
                    Arc::clone(existing)
                }
            }
        },
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TaskContext, TaskType};

    #[test]
    fn test_stale_cache_replaced_by_newer_durable() {
        let context = TaskContext::default();
        let t1 = Utc::now();
        let t2 = t1 + chrono::Duration::seconds(10);

        let mut old_cache =
            Episode::new("Old cache".to_string(), context.clone(), TaskType::Testing);
        old_cache
            .metadata
            .insert("modified_at".to_string(), t1.to_rfc3339());
        let old_cache_arc = Arc::new(old_cache);

        let mut new_durable = Episode::new("New durable".to_string(), context, TaskType::Testing);
        new_durable.episode_id = old_cache_arc.episode_id;
        new_durable
            .metadata
            .insert("modified_at".to_string(), t2.to_rfc3339());
        let new_durable_arc = Arc::new(new_durable);

        let winner = select_winning_episode(
            &old_cache_arc,
            EpisodeOrigin::Cache,
            &new_durable_arc,
            EpisodeOrigin::Durable,
        );

        assert_eq!(winner.task_description, "New durable");
    }

    #[test]
    fn test_newer_cache_not_overwritten_by_older_durable() {
        let context = TaskContext::default();
        let t1 = Utc::now();
        let t2 = t1 + chrono::Duration::seconds(10);

        let mut new_cache =
            Episode::new("New cache".to_string(), context.clone(), TaskType::Testing);
        new_cache
            .metadata
            .insert("modified_at".to_string(), t2.to_rfc3339());
        let new_cache_arc = Arc::new(new_cache);

        let mut old_durable = Episode::new("Old durable".to_string(), context, TaskType::Testing);
        old_durable.episode_id = new_cache_arc.episode_id;
        old_durable
            .metadata
            .insert("modified_at".to_string(), t1.to_rfc3339());
        let old_durable_arc = Arc::new(old_durable);

        let winner = select_winning_episode(
            &new_cache_arc,
            EpisodeOrigin::Cache,
            &old_durable_arc,
            EpisodeOrigin::Durable,
        );

        assert_eq!(winner.task_description, "New cache");
    }

    #[test]
    fn test_equal_timestamps_durable_wins() {
        let context = TaskContext::default();
        let t1 = Utc::now();

        let mut cache_ep = Episode::new(
            "Cache payload".to_string(),
            context.clone(),
            TaskType::Testing,
        );
        cache_ep.start_time = t1;
        let cache_arc = Arc::new(cache_ep);

        let mut durable_ep =
            Episode::new("Durable payload".to_string(), context, TaskType::Testing);
        durable_ep.episode_id = cache_arc.episode_id;
        durable_ep.start_time = t1;
        let durable_arc = Arc::new(durable_ep);

        let winner = select_winning_episode(
            &cache_arc,
            EpisodeOrigin::Cache,
            &durable_arc,
            EpisodeOrigin::Durable,
        );

        assert_eq!(winner.task_description, "Durable payload");
    }

    #[test]
    fn test_explicit_revision_wins_over_missing_at_same_timestamp() {
        let context = TaskContext::default();
        let t1 = Utc::now();

        let mut legacy_ep = Episode::new(
            "Legacy payload".to_string(),
            context.clone(),
            TaskType::Testing,
        );
        legacy_ep.start_time = t1;
        let legacy_arc = Arc::new(legacy_ep);

        let mut explicit_ep =
            Episode::new("Explicit payload".to_string(), context, TaskType::Testing);
        explicit_ep.episode_id = legacy_arc.episode_id;
        explicit_ep.start_time = t1;
        explicit_ep
            .metadata
            .insert("modified_at".to_string(), t1.to_rfc3339());
        let explicit_arc = Arc::new(explicit_ep);

        let winner = select_winning_episode(
            &legacy_arc,
            EpisodeOrigin::InMemory,
            &explicit_arc,
            EpisodeOrigin::Cache,
        );

        assert_eq!(winner.task_description, "Explicit payload");
    }
}

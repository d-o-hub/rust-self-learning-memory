//! Recency reconciliation for the recommendation episode index.
//!
//! `recommendation_episode_index` maps an episode ID to one of the recommendation sessions
//! recorded for it, so the value carries no ordering key. The winner therefore has to be
//! derived from the session rows rather than from write order: [`session_rank`] is the
//! ordering and [`RedbStorage::repair_recommendation_index`] is the reconciliation pass that
//! rebuilds the index from it.

use crate::with_db_timeout;
use crate::{RECOMMENDATION_EPISODE_INDEX_TABLE, RECOMMENDATION_SESSIONS_TABLE, RedbStorage};
use chrono::{DateTime, Utc};
use do_memory_core::memory::attribution::RecommendationSession;
use do_memory_core::{Error, Result};
use redb::ReadableTable;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Ordering key of a session: newest first, session ID as the deterministic tie-break.
///
/// Tuple `Ord` is lexicographic, so two sessions recorded in the same instant are still
/// ordered — the larger session ID wins regardless of which row reached storage first.
pub(crate) fn session_rank(session: &RecommendationSession) -> (DateTime<Utc>, Uuid) {
    (session.timestamp, session.session_id)
}

/// Newest session per episode, keyed by the episode ID string.
fn collect_winners(
    write_txn: &redb::WriteTransaction,
) -> Result<HashMap<String, (DateTime<Utc>, Uuid)>> {
    let sessions = write_txn
        .open_table(RECOMMENDATION_SESSIONS_TABLE)
        .map_err(|e| Error::Storage(format!("Failed to open recommendation sessions: {e}")))?;

    let mut winners: HashMap<String, (DateTime<Utc>, Uuid)> = HashMap::new();
    for result in sessions
        .iter()
        .map_err(|e| Error::Storage(format!("Failed to read recommendation sessions: {e}")))?
    {
        let (_, value) =
            result.map_err(|e| Error::Storage(format!("Failed to read session row: {e}")))?;
        let Ok(session) = postcard::from_bytes::<RecommendationSession>(value.value()) else {
            // An undecodable row cannot win an index entry. The session table is left
            // untouched: this pass only reconciles the index.
            continue;
        };
        let episode_key = session.episode_id.to_string();
        let rank = session_rank(&session);
        let replace = match winners.get(&episode_key) {
            Some(current) => *current < rank,
            None => true,
        };
        if replace {
            winners.insert(episode_key, rank);
        }
    }

    Ok(winners)
}

/// Rewrite every index entry that disagrees with `winners`, dropping entries whose session
/// row is gone and adding entries for episodes that have sessions but no row.
///
/// The existing rows are snapshotted before any mutation, so no iterator is live while the
/// table is written.
fn rebuild_episode_index(
    write_txn: &mut redb::WriteTransaction,
    winners: &HashMap<String, (DateTime<Utc>, Uuid)>,
) -> Result<usize> {
    let existing: HashMap<String, String> = {
        let index = write_txn
            .open_table(RECOMMENDATION_EPISODE_INDEX_TABLE)
            .map_err(|e| Error::Storage(format!("Failed to open episode index: {e}")))?;
        let mut rows = HashMap::new();
        for result in index
            .iter()
            .map_err(|e| Error::Storage(format!("Failed to read episode index: {e}")))?
        {
            let (key, value) =
                result.map_err(|e| Error::Storage(format!("Failed read index row: {e}")))?;
            rows.insert(key.value().to_string(), value.value().to_string());
        }
        rows
    };

    let mut changes: Vec<(String, Option<String>)> = Vec::new();
    for (episode, (_, session_id)) in winners {
        if existing.get(episode) != Some(&session_id.to_string()) {
            changes.push((episode.clone(), Some(session_id.to_string())));
        }
    }
    for episode in existing.keys() {
        if !winners.contains_key(episode) {
            changes.push((episode.clone(), None));
        }
    }

    if changes.is_empty() {
        return Ok(0);
    }

    let mut index = write_txn
        .open_table(RECOMMENDATION_EPISODE_INDEX_TABLE)
        .map_err(|e| Error::Storage(format!("Failed to open episode index: {e}")))?;
    for (episode, replacement) in &changes {
        match replacement {
            Some(session_key) => {
                let _ = index
                    .insert(episode.as_str(), session_key.as_str())
                    .map_err(|e| {
                        Error::Storage(format!("Failed to reindex episode {episode}: {e}"))
                    })?;
            }
            None => {
                let _ = index.remove(episode.as_str()).map_err(|e| {
                    Error::Storage(format!("Failed to drop stale index row {episode}: {e}"))
                })?;
            }
        }
    }

    Ok(changes.len())
}

impl RedbStorage {
    /// Rebuild the recommendation episode index from the session rows it points at.
    ///
    /// Each episode ends up indexed by the session with the greatest [`session_rank`], which
    /// corrects what the pre-#1066 write path left behind: last write won, so an older session
    /// stored after a newer one could own the entry indefinitely. Index entries whose session
    /// row is gone are dropped and episodes with sessions but no entry are added. Session and
    /// feedback rows are never modified or removed.
    ///
    /// Runs in a single write transaction and returns the number of index entries changed.
    /// Every public constructor calls it while initializing tables, so reopening a cache
    /// heals itself; [`RedbStorage::open_unchecked`] deliberately does not.
    pub async fn repair_recommendation_index(&self) -> Result<usize> {
        let db = Arc::clone(&self.db);

        with_db_timeout(move || {
            let mut write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {e}")))?;

            let winners = collect_winners(&write_txn)?;
            let repaired = rebuild_episode_index(&mut write_txn, &winners)?;

            if repaired > 0 {
                write_txn
                    .commit()
                    .map_err(|e| Error::Storage(format!("Failed to commit index repair: {e}")))?;
            } else {
                // Nothing changed: drop the transaction instead of rewriting the file.
                write_txn
                    .abort()
                    .map_err(|e| Error::Storage(format!("Failed to release index lock: {e}")))?;
            }

            Ok(repaired)
        })
        .await
    }
}

#[cfg(test)]
#[path = "recommendation_index_tests.rs"]
mod recommendation_index_tests;

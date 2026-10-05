//! Crate-internal tests for the recommendation episode index (issue #1066).
//!
//! The write path and the reconciliation pass are tested here rather than in `tests/` because
//! a legacy database has to be *fabricated*: an index row that disagrees with the session rows
//! can no longer be produced through the public API. `RedbStorage::db` is private to the crate
//! root, so this module can open the raw table.

use crate::recommendation_index::session_rank;
use crate::{RECOMMENDATION_EPISODE_INDEX_TABLE, RECOMMENDATION_SESSIONS_TABLE, RedbStorage};
use chrono::{DateTime, Utc};
use do_memory_core::memory::attribution::RecommendationSession;
use std::sync::Arc;
use uuid::Uuid;

fn session_for(episode: Uuid, id: Uuid, at: DateTime<Utc>) -> RecommendationSession {
    RecommendationSession {
        session_id: id,
        episode_id: episode,
        timestamp: at,
        recommended_pattern_ids: vec![format!("pattern-{}", id.simple())],
        recommended_playbook_ids: vec![],
    }
}

/// Two UUIDs ordered so a test can name the tie-break winner without guessing.
fn ordered_ids() -> (Uuid, Uuid) {
    let (first, second) = (Uuid::new_v4(), Uuid::new_v4());
    if first < second {
        (first, second)
    } else {
        (second, first)
    }
}

/// Point an episode's index row at `session` without consulting the session rows.
async fn force_index_entry(storage: &RedbStorage, episode: Uuid, session: Uuid) {
    let db = Arc::clone(&storage.db);
    let episode_key = episode.to_string();
    let session_key = session.to_string();
    let write_txn = db.begin_write().unwrap();
    {
        let mut index = write_txn
            .open_table(RECOMMENDATION_EPISODE_INDEX_TABLE)
            .unwrap();
        index
            .insert(episode_key.as_str(), session_key.as_str())
            .unwrap();
    }
    write_txn.commit().unwrap();
}

/// Write an index row and a matching session row for an episode, bypassing the store path.
async fn force_undecodable_session(storage: &RedbStorage, session: Uuid) {
    let db = Arc::clone(&storage.db);
    let session_key = session.to_string();
    let write_txn = db.begin_write().unwrap();
    {
        let mut table = write_txn.open_table(RECOMMENDATION_SESSIONS_TABLE).unwrap();
        table
            .insert(session_key.as_str(), [0xFFu8, 0xFF, 0xFF].as_slice())
            .unwrap();
    }
    write_txn.commit().unwrap();
}

async fn storage_in(dir: &tempfile::TempDir, name: &str) -> RedbStorage {
    RedbStorage::new(&dir.path().join(name))
        .await
        .expect("open redb")
}

/// The ordering key itself: timestamp first, session ID only as the tie-break.
#[test]
fn session_rank_orders_by_time_then_session_id() {
    let episode = Uuid::new_v4();
    let at = Utc::now();
    let (lower, higher) = ordered_ids();

    let earlier = session_rank(&session_for(
        episode,
        higher,
        at - chrono::Duration::seconds(1),
    ));
    let later = session_rank(&session_for(episode, lower, at));
    assert!(
        later > earlier,
        "a later timestamp must win even against the larger session ID"
    );

    let first = session_rank(&session_for(episode, lower, at));
    let second = session_rank(&session_for(episode, higher, at));
    assert!(
        second > first,
        "equal timestamps must fall through to the session ID"
    );
}

/// Acceptance 1: an older session stored after a newer one must not take the index.
#[tokio::test]
async fn older_session_stored_later_keeps_the_newer_winner() {
    let dir = tempfile::TempDir::new().unwrap();
    let storage = storage_in(&dir, "write-order.redb").await;

    let episode = Uuid::new_v4();
    let now = Utc::now();
    let newer = Uuid::new_v4();
    let older = Uuid::new_v4();

    storage
        .store_recommendation_session(&session_for(episode, newer, now))
        .await
        .unwrap();
    storage
        .store_recommendation_session(&session_for(
            episode,
            older,
            now - chrono::Duration::hours(1),
        ))
        .await
        .unwrap();

    let indexed = storage
        .get_recommendation_session_for_episode(episode)
        .await
        .unwrap()
        .expect("the episode must stay indexed");
    assert_eq!(
        indexed.session_id, newer,
        "write order must not decide the indexed session"
    );
    assert_eq!(
        storage.list_recommendation_sessions().await.unwrap().len(),
        2,
        "the losing session row must still be stored"
    );
}

/// Acceptance 2: equal timestamps resolve by session ID, in either insertion order.
#[tokio::test]
async fn equal_timestamps_resolve_by_session_id() {
    let dir = tempfile::TempDir::new().unwrap();
    let at = Utc::now();

    for newest_first in [true, false] {
        let storage = storage_in(&dir, "tie-break.redb").await;
        let episode = Uuid::new_v4();
        let (lower, higher) = ordered_ids();
        let (first, second) = if newest_first {
            (higher, lower)
        } else {
            (lower, higher)
        };

        storage
            .store_recommendation_session(&session_for(episode, first, at))
            .await
            .unwrap();
        storage
            .store_recommendation_session(&session_for(episode, second, at))
            .await
            .unwrap();

        let indexed = storage
            .get_recommendation_session_for_episode(episode)
            .await
            .unwrap()
            .expect("the episode must be indexed");
        assert_eq!(
            indexed.session_id, higher,
            "the larger session ID must win equal timestamps (stored {first} first)"
        );
    }
}

/// Acceptance 3: a database whose index was written by the last-write-wins path is repaired
/// on reopen, and no session row is lost.
#[tokio::test]
async fn reopening_repairs_a_write_order_winner() {
    let dir = tempfile::TempDir::new().unwrap();
    let storage = storage_in(&dir, "legacy.redb").await;

    let episode = Uuid::new_v4();
    let now = Utc::now();
    let newer = Uuid::new_v4();
    let older = Uuid::new_v4();
    storage
        .store_recommendation_session(&session_for(episode, newer, now))
        .await
        .unwrap();
    storage
        .store_recommendation_session(&session_for(
            episode,
            older,
            now - chrono::Duration::hours(2),
        ))
        .await
        .unwrap();

    // Simulate the legacy outcome: the index points at the older session.
    force_index_entry(&storage, episode, older).await;
    assert_eq!(
        storage
            .get_recommendation_session_for_episode(episode)
            .await
            .unwrap()
            .map(|s| s.session_id),
        Some(older),
        "the fabricated legacy row must be readable before the repair"
    );

    let repaired = storage.repair_recommendation_index().await.unwrap();
    assert_eq!(repaired, 1, "one index row must change");
    assert_eq!(
        storage
            .get_recommendation_session_for_episode(episode)
            .await
            .unwrap()
            .map(|s| s.session_id),
        Some(newer),
        "the newest session must own the entry after repair"
    );
    assert_eq!(
        storage.list_recommendation_sessions().await.unwrap().len(),
        2,
        "repair must not drop session rows"
    );

    // A fresh handle over the same file must see the repaired state without changing it again.
    drop(storage);
    let reopened = storage_in(&dir, "legacy.redb").await;
    assert_eq!(
        reopened
            .get_recommendation_session_for_episode(episode)
            .await
            .unwrap()
            .map(|s| s.session_id),
        Some(newer),
        "the repair must be durable"
    );
    assert_eq!(
        reopened.repair_recommendation_index().await.unwrap(),
        0,
        "repair must be idempotent"
    );
}

/// An index row whose session is gone is dropped, and an episode whose index row is missing
/// is re-added.
#[tokio::test]
async fn repair_drops_orphans_and_backfills_missing_rows() {
    let dir = tempfile::TempDir::new().unwrap();
    let storage = storage_in(&dir, "orphan.redb").await;

    let with_session = Uuid::new_v4();
    let orphan_episode = Uuid::new_v4();
    storage
        .store_recommendation_session(&session_for(with_session, Uuid::new_v4(), Utc::now()))
        .await
        .unwrap();
    // An index row for an episode with no session rows at all.
    force_index_entry(&storage, orphan_episode, Uuid::new_v4()).await;

    // Remove the index row of the episode that does have a session.
    let db = Arc::clone(&storage.db);
    let key = with_session.to_string();
    let write_txn = db.begin_write().unwrap();
    {
        let mut index = write_txn
            .open_table(RECOMMENDATION_EPISODE_INDEX_TABLE)
            .unwrap();
        index.remove(key.as_str()).unwrap();
    }
    write_txn.commit().unwrap();
    assert!(
        storage
            .get_recommendation_session_for_episode(with_session)
            .await
            .unwrap()
            .is_none(),
        "the backfilled episode must start unindexed"
    );

    let repaired = storage.repair_recommendation_index().await.unwrap();
    assert_eq!(repaired, 2, "one orphan dropped and one row backfilled");
    assert!(
        storage
            .get_recommendation_session_for_episode(with_session)
            .await
            .unwrap()
            .is_some(),
        "the missing entry must be rebuilt from the session rows"
    );
    assert!(
        storage
            .get_recommendation_session_for_episode(orphan_episode)
            .await
            .unwrap()
            .is_none(),
        "an entry with no session row must be removed"
    );
}

/// A session row that cannot be decoded cannot win an entry, and does not fail the pass.
#[tokio::test]
async fn repair_skips_undecodable_session_rows() {
    let dir = tempfile::TempDir::new().unwrap();
    let storage = storage_in(&dir, "corrupt.redb").await;

    let episode = Uuid::new_v4();
    let good = Uuid::new_v4();
    storage
        .store_recommendation_session(&session_for(episode, good, Utc::now()))
        .await
        .unwrap();
    let corrupt = Uuid::new_v4();
    force_undecodable_session(&storage, corrupt).await;

    let repaired = storage.repair_recommendation_index().await.unwrap();
    assert_eq!(repaired, 0, "the readable winner already owns the entry");
    assert_eq!(
        storage
            .get_recommendation_session_for_episode(episode)
            .await
            .unwrap()
            .map(|s| s.session_id),
        Some(good),
        "a corrupt row must not take an episode's index entry"
    );
    assert!(
        storage.list_recommendation_sessions().await.is_err(),
        "the repair leaves the corrupt row in place, so the listing surface still reports it"
    );
}

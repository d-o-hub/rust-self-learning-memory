//! Episode modification watermark bookkeeping.
//!
//! `episode_revisions` is a side table keyed by episode id that records the
//! millisecond timestamp of the last write to that episode. It backs the
//! `(modified_at, episode_id)` keyset scan used by incremental sync without
//! touching the postcard-serialized `Episode` layout (postcard is positional,
//! so adding a field would break decoding of existing rows).

use do_memory_core::{Error, Result};

/// Insert or refresh the modification watermark for `episode_id`.
///
/// The stored value is `max(now, previous_max + 1)`, so every committed write
/// receives a value strictly greater than every earlier one. A concurrent write
/// therefore can never be ordered before an existing cursor, which is what makes
/// the synchronizer's `(modified_at, episode_id)` watermark safe to advance.
///
/// Must be called on every episode write; callers that already hold an open
/// transaction should reuse its connection so the episode row and its
/// watermark commit together.
pub(crate) async fn record_episode_revision(
    conn: &libsql::Connection,
    episode_id: &str,
    modified_at_ms: i64,
) -> Result<()> {
    const SQL: &str = r#"
        INSERT INTO episode_revisions (episode_id, modified_at_ms)
        VALUES (
            ?,
            MAX(?, COALESCE((SELECT MAX(modified_at_ms) FROM episode_revisions), 0) + 1)
        )
        ON CONFLICT(episode_id) DO UPDATE SET modified_at_ms = excluded.modified_at_ms
    "#;

    conn.execute(SQL, libsql::params![episode_id, modified_at_ms])
        .await
        .map_err(|e| Error::Storage(format!("Failed to record episode revision: {}", e)))?;
    Ok(())
}

/// Remove the modification watermark for a deleted episode.
pub(crate) async fn forget_episode_revision(
    conn: &libsql::Connection,
    episode_id: &str,
) -> Result<()> {
    const SQL: &str = "DELETE FROM episode_revisions WHERE episode_id = ?";

    conn.execute(SQL, libsql::params![episode_id])
        .await
        .map_err(|e| Error::Storage(format!("Failed to remove episode revision: {}", e)))?;
    Ok(())
}

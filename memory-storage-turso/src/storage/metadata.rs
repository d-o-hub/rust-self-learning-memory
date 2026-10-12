//! Key/value metadata helpers backed by the `metadata` table.
//!
//! Used for durable synchronizer bookkeeping (the incremental-sync watermark)
//! and for one-shot migration flags.

use do_memory_core::{Error, Result};

/// Metadata key under which the incremental-sync watermark (milliseconds since
/// the Unix epoch) is persisted.
pub(crate) const SYNC_WATERMARK_KEY: &str = "sync_watermark_ms";

/// Metadata key marking that pre-existing episodes have been backfilled into
/// `episode_revisions`.
pub(crate) const EPISODE_REVISIONS_BACKFILL_KEY: &str = "episode_revisions_backfilled";

/// Read a metadata value by key.
pub(crate) async fn get_metadata(conn: &libsql::Connection, key: &str) -> Result<Option<String>> {
    let mut rows = conn
        .query(
            "SELECT value FROM metadata WHERE key = ?",
            libsql::params![key],
        )
        .await
        .map_err(|e| Error::Storage(format!("Failed to read metadata {key}: {e}")))?;

    match rows
        .next()
        .await
        .map_err(|e| Error::Storage(format!("Failed to fetch metadata {key}: {e}")))?
    {
        Some(row) => {
            let value: String = row
                .get(0)
                .map_err(|e| Error::Storage(format!("Failed to decode metadata {key}: {e}")))?;
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

/// Insert or update a metadata value.
pub(crate) async fn store_metadata(
    conn: &libsql::Connection,
    key: &str,
    value: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO metadata (key, value, updated_at) VALUES (?, ?, strftime('%s','now')) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        libsql::params![key, value],
    )
    .await
    .map_err(|e| Error::Storage(format!("Failed to write metadata {key}: {e}")))?;
    Ok(())
}

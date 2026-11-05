//! Single-pattern deletion for Turso.
//!
//! Split out of `pattern_core.rs` to keep that file under the 500-LOC ceiling.

use crate::TursoStorage;
use do_memory_core::{Error, Result, episode::PatternId};
use tracing::{debug, error, info};

impl TursoStorage {
    /// Delete a single pattern and its derived records.
    ///
    /// Removes the pattern row (the `patterns_ad` trigger clears `patterns_fts`),
    /// the pattern embedding, and every episode-pattern relationship that
    /// references the pattern. Deleting a pattern that does not exist is a
    /// no-op, so callers can retry safely.
    pub async fn delete_pattern(&self, id: PatternId) -> Result<()> {
        debug!("Deleting pattern: {}", id);
        self.with_connection(async |conn| {
            conn.execute("BEGIN TRANSACTION", ()).await.map_err(|e| {
                Error::Storage(format!(
                    "Failed to begin pattern deletion transaction: {}",
                    e
                ))
            })?;

            let id_str = id.to_string();

            #[cfg(not(feature = "turso_multi_dimension"))]
            let statements: [&str; 3] = [
                "DELETE FROM episode_pattern_relationships WHERE pattern_id = ?",
                "DELETE FROM embeddings WHERE item_type = 'pattern' AND item_id = ?",
                "DELETE FROM patterns WHERE pattern_id = ?",
            ];

            #[cfg(feature = "turso_multi_dimension")]
            let statements: [&str; 8] = [
                "DELETE FROM episode_pattern_relationships WHERE pattern_id = ?",
                "DELETE FROM embeddings_384 WHERE item_type = 'pattern' AND item_id = ?",
                "DELETE FROM embeddings_1024 WHERE item_type = 'pattern' AND item_id = ?",
                "DELETE FROM embeddings_1536 WHERE item_type = 'pattern' AND item_id = ?",
                "DELETE FROM embeddings_3072 WHERE item_type = 'pattern' AND item_id = ?",
                "DELETE FROM embeddings_other WHERE item_type = 'pattern' AND item_id = ?",
                "DELETE FROM embeddings WHERE item_type = 'pattern' AND item_id = ?",
                "DELETE FROM patterns WHERE pattern_id = ?",
            ];

            for sql in statements {
                if let Err(e) = conn.execute(sql, libsql::params![id_str.clone()]).await {
                    if let Err(rollback_err) = conn.execute("ROLLBACK", ()).await {
                        error!("Failed to rollback pattern deletion: {}", rollback_err);
                    }
                    return Err(Error::Storage(format!(
                        "Failed to delete pattern {}: {}",
                        id, e
                    )));
                }
            }

            conn.execute("COMMIT", ()).await.map_err(|e| {
                Error::Storage(format!(
                    "Failed to commit pattern deletion transaction: {}",
                    e
                ))
            })?;

            info!("Successfully deleted pattern: {}", id);
            Ok(())
        })
        .await
    }
}

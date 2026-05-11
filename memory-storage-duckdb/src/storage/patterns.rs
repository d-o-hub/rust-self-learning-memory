use crate::DuckDbStorage;
use do_memory_core::types::learning::Pattern;
use do_memory_core::{Error, Result};
use duckdb::params;
use std::sync::Arc;

impl DuckDbStorage {
    pub(crate) async fn store_pattern_internal(&self, pattern: &Pattern) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let pattern = pattern.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let data_json = serde_json::to_string(&pattern)
                .map_err(|e| Error::Storage(format!("Failed to serialize pattern: {e}")))?;

            conn.execute(
                "INSERT OR REPLACE INTO patterns (
                    pattern_id, pattern_type, pattern_data, success_rate,
                    context_domain, context_language, occurrence_count, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
                params![
                    pattern.id,
                    "generic", // Placeholder for now
                    data_json,
                    pattern.success_rate,
                    pattern.metadata.get("domain").and_then(|v| v.as_str()),
                    pattern.metadata.get("language").and_then(|v| v.as_str()),
                    1, // Default occurrence count
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store pattern: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn load_patterns_internal(&self) -> Result<Vec<Pattern>> {
        let conn_arc = Arc::clone(&self.conn);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare("SELECT CAST(pattern_data AS VARCHAR) FROM patterns")
                .map_err(|e| Error::Storage(e.to_string()))?;
            let mut rows = stmt.query([]).map_err(|e| Error::Storage(e.to_string()))?;

            let mut patterns = Vec::new();
            while let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let data_json: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let pattern: Pattern = serde_json::from_str(&data_json).map_err(|e| {
                    // Sanitize error: do not include data_json in the error message
                    Error::Storage(format!("Failed to deserialize pattern: {e}"))
                })?;
                patterns.push(pattern);
            }
            Ok::<Vec<Pattern>, Error>(patterns)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }
}

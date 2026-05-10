use crate::DuckDbStorage;
use do_memory_core::{Error, Result};
use duckdb::params;
use std::sync::Arc;
use uuid::Uuid;

impl DuckDbStorage {
    pub(crate) async fn store_pattern_internal(
        &self,
        pattern: &do_memory_core::Pattern,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let pattern = pattern.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let data_json = serde_json::to_string(&pattern)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;

            let pattern_type = match pattern {
                do_memory_core::Pattern::ToolSequence { .. } => "tool_sequence",
                do_memory_core::Pattern::DecisionPoint { .. } => "decision_point",
                do_memory_core::Pattern::ErrorRecovery { .. } => "error_recovery",
                do_memory_core::Pattern::ContextPattern { .. } => "context_pattern",
            };

            conn.execute(
                "INSERT INTO patterns (
                    pattern_id, pattern_type, pattern_data, success_rate,
                    context_domain, context_language, occurrence_count
                ) VALUES (?, ?, ?, ?, ?, ?, ?)",
                params![
                    pattern.id().to_string(),
                    pattern_type,
                    data_json,
                    pattern.success_rate(),
                    pattern
                        .context()
                        .map(|c| c.domain.clone())
                        .unwrap_or_default(),
                    pattern.context().and_then(|c| c.language.clone()),
                    i64::try_from(pattern.sample_size()).unwrap_or(0),
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store pattern: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    /// Retrieves a pattern by its ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub(crate) async fn get_pattern_internal(
        &self,
        id: Uuid,
    ) -> Result<Option<do_memory_core::Pattern>> {
        let conn_arc = Arc::clone(&self.conn);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare("SELECT CAST(pattern_data AS VARCHAR) FROM patterns WHERE pattern_id = ?")
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![id.to_string()])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let data_json: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let pattern = serde_json::from_str(&data_json).map_err(|e| {
                    Error::Storage(format!("pattern parse: {e} | json='{data_json}'"))
                })?;
                Ok::<Option<do_memory_core::Pattern>, Error>(Some(pattern))
            } else {
                Ok::<Option<do_memory_core::Pattern>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }
}

use crate::DuckDbStorage;
use chrono::{DateTime, Utc};
use do_memory_core::{Error, Result};
use duckdb::params;
use std::sync::Arc;
use uuid::Uuid;

impl DuckDbStorage {
    pub(crate) async fn store_heuristic_internal(
        &self,
        heuristic: &do_memory_core::Heuristic,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let heuristic = heuristic.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let evidence_json = serde_json::to_string(&heuristic.evidence)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;

            conn.execute(
                "INSERT INTO heuristics (
                    heuristic_id, condition_text, action_text, confidence, evidence, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?)",
                params![
                    heuristic.heuristic_id.to_string(),
                    heuristic.condition,
                    heuristic.action,
                    heuristic.confidence,
                    evidence_json,
                    heuristic.created_at.to_rfc3339(),
                    heuristic.updated_at.to_rfc3339(),
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store heuristic: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    /// Retrieves a heuristic by its ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub(crate) async fn get_heuristic_internal(
        &self,
        id: Uuid,
    ) -> Result<Option<do_memory_core::Heuristic>> {
        let conn_arc = Arc::clone(&self.conn);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare("SELECT heuristic_id, condition_text, action_text, confidence, CAST(evidence AS VARCHAR), strftime(CAST(created_at AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ'), strftime(CAST(updated_at AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ') FROM heuristics WHERE heuristic_id = ?")
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![id.to_string()])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let id_str: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let condition: String = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
                let action: String = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
                let confidence: f32 = row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
                let evidence_json: String = row.get(4).map_err(|e| Error::Storage(e.to_string()))?;
                let created_at_str: String = row.get(5).map_err(|e| Error::Storage(e.to_string()))?;
                let updated_at_str: String = row.get(6).map_err(|e| Error::Storage(e.to_string()))?;

                let heuristic = do_memory_core::Heuristic {
                    heuristic_id: Uuid::parse_str(&id_str)
                        .map_err(|e| Error::Storage(e.to_string()))?,
                    condition,
                    action,
                    confidence,
                    evidence: serde_json::from_str(&evidence_json).map_err(|e| {
                        Error::Storage(format!("heuristic evidence parse: {e} | json='{evidence_json}'"))
                    })?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .or_else(|_| DateTime::parse_from_str(&created_at_str, "%Y-%m-%dT%H:%M:%S.%fZ"))
                        .map_err(|e| Error::Storage(format!("heuristic created_at parse: {e} | val='{created_at_str}'")))?
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                        .or_else(|_| DateTime::parse_from_str(&updated_at_str, "%Y-%m-%dT%H:%M:%S.%fZ"))
                        .map_err(|e| Error::Storage(format!("heuristic updated_at parse: {e} | val='{updated_at_str}'")))?
                        .with_timezone(&Utc),
                };
                Ok::<Option<do_memory_core::Heuristic>, Error>(Some(heuristic))
            } else {
                Ok::<Option<do_memory_core::Heuristic>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    // ========== Analytical Queries ==========
}

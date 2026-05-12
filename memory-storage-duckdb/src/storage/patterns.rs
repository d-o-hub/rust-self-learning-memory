use crate::DuckDbStorage;
use do_memory_core::Pattern;
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

            let pattern_type = match &pattern {
                Pattern::ToolSequence { .. } => "tool_sequence",
                Pattern::DecisionPoint { .. } => "decision_point",
                Pattern::ErrorRecovery { .. } => "error_recovery",
                Pattern::ContextPattern { .. } => "context_pattern",
            };

            let domain = pattern.context().map(|c| c.domain.as_str());
            let language = pattern.context().and_then(|c| c.language.as_deref());

            let sample_size = i64::try_from(pattern.sample_size()).map_err(|e| {
                Error::Storage(format!(
                    "Failed to convert sample size for pattern {}: {e}",
                    pattern.id()
                ))
            })?;

            conn.execute(
                "INSERT OR REPLACE INTO patterns (
                    pattern_id, pattern_type, pattern_data, success_rate,
                    context_domain, context_language, occurrence_count, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
                params![
                    pattern.id().to_string(),
                    pattern_type,
                    data_json,
                    f64::from(pattern.success_rate()),
                    domain,
                    language,
                    sample_size,
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
                    let safe_summary = if data_json.len() > 50 {
                        format!("{}...", &data_json[..50])
                    } else {
                        data_json.clone()
                    };
                    Error::Storage(format!(
                        "Failed to deserialize pattern (len={}): {e}. Data snippet: {}",
                        data_json.len(),
                        safe_summary
                    ))
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

pub(crate) async fn get_pattern_internal(&self, id: Uuid) -> Result<Option<Pattern>> {
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
            let pattern: Pattern = serde_json::from_str(&data_json)
                .map_err(|e| Error::Storage(format!("Failed to deserialize pattern: {e}")))?;
            Ok::<Option<Pattern>, Error>(Some(pattern))
        } else {
            Ok::<Option<Pattern>, Error>(None)
        }
    })
    .await
    .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
    Ok(res)
}

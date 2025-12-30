//! Heuristic CRUD operations for Turso storage

use crate::TursoStorage;
use libsql::Row;
use memory_core::{Error, Heuristic, Result};
use tracing::{debug, info};
use uuid::Uuid;

impl TursoStorage {
    /// Store a heuristic
    pub async fn store_heuristic(&self, heuristic: &Heuristic) -> Result<()> {
        debug!("Storing heuristic: {}", heuristic.heuristic_id);
        let conn = self.get_connection().await?;

        let sql = r#"
            INSERT OR REPLACE INTO heuristics (
                heuristic_id, condition_text, action_text, confidence, evidence, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#;

        let evidence_json =
            serde_json::to_string(&heuristic.evidence).map_err(Error::Serialization)?;

        conn.execute(
            sql,
            libsql::params![
                heuristic.heuristic_id.to_string(),
                heuristic.condition.clone(),
                heuristic.action.clone(),
                heuristic.confidence,
                evidence_json,
                heuristic.created_at.timestamp(),
                heuristic.updated_at.timestamp(),
            ],
        )
        .await
        .map_err(|e| Error::Storage(format!("Failed to store heuristic: {}", e)))?;

        info!("Successfully stored heuristic: {}", heuristic.heuristic_id);
        Ok(())
    }

    /// Retrieve a heuristic by ID
    pub async fn get_heuristic(&self, id: Uuid) -> Result<Option<Heuristic>> {
        debug!("Retrieving heuristic: {}", id);
        let conn = self.get_connection().await?;

        let sql = r#"
            SELECT heuristic_id, condition_text, action_text, confidence, evidence, created_at, updated_at
            FROM heuristics WHERE heuristic_id = ?
        "#;

        let mut rows = conn
            .query(sql, libsql::params![id.to_string()])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query heuristic: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch heuristic row: {}", e)))?
        {
            let heuristic = row_to_heuristic(&row)?;
            Ok(Some(heuristic))
        } else {
            Ok(None)
        }
    }

    /// Get all heuristics
    pub async fn get_heuristics(&self) -> Result<Vec<Heuristic>> {
        debug!("Retrieving all heuristics");
        let conn = self.get_connection().await?;

        let sql = r#"
            SELECT heuristic_id, condition_text, action_text, confidence, evidence, created_at, updated_at
            FROM heuristics ORDER BY confidence DESC
        "#;

        let mut rows = conn
            .query(sql, ())
            .await
            .map_err(|e| Error::Storage(format!("Failed to query heuristics: {}", e)))?;

        let mut heuristics = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch heuristic row: {}", e)))?
        {
            heuristics.push(row_to_heuristic(&row)?);
        }

        info!("Found {} heuristics", heuristics.len());
        Ok(heuristics)
    }
}

/// Convert a database row to a Heuristic
fn row_to_heuristic(row: &Row) -> Result<Heuristic> {
    let id: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
    let heuristic_id = Uuid::parse_str(&id).map_err(|e| Error::Storage(e.to_string()))?;
    let condition: String = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
    let action: String = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
    let confidence: f64 = row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
    let evidence_json: String = row.get(4).map_err(|e| Error::Storage(e.to_string()))?;
    let created_at_timestamp: i64 = row.get(5).map_err(|e| Error::Storage(e.to_string()))?;
    let updated_at_timestamp: i64 = row.get(6).map_err(|e| Error::Storage(e.to_string()))?;

    let evidence: memory_core::types::Evidence = serde_json::from_str(&evidence_json)
        .map_err(|e| Error::Storage(format!("Failed to parse evidence: {}", e)))?;

    Ok(Heuristic {
        heuristic_id,
        condition,
        action,
        confidence: confidence as f32,
        evidence,
        created_at: chrono::DateTime::from_timestamp(created_at_timestamp, 0).unwrap_or_default(),
        updated_at: chrono::DateTime::from_timestamp(updated_at_timestamp, 0).unwrap_or_default(),
    })
}

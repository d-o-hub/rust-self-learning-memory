//! Procedural memory storage operations for Turso

use do_memory_core::{ProceduralMemory, ProceduralStep, Result, TaskType};
use libsql::{Row, params};
use serde_json;
use uuid::Uuid;

use crate::TursoStorage;

impl TursoStorage {
    /// Store a procedural memory
    pub async fn store_procedural_memory(&self, procedural: &ProceduralMemory) -> Result<()> {
        let conn = self.get_connection().await?;

        let id = procedural.id.to_string();
        let name = procedural.name.clone();
        let description = procedural.description.clone();
        let task_type = procedural.task_type.to_string();
        let steps = serde_json::to_string(&procedural.steps).map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to serialize steps: {}", e))
        })?;
        let evidence = serde_json::to_string(&procedural.evidence).map_err(|e| {
            do_memory_core::Error::Storage(format!("Failed to serialize evidence: {}", e))
        })?;
        let confidence = procedural.confidence as f64;
        let version = procedural.version as i64;
        let created_at = procedural.created_at.timestamp();
        let updated_at = procedural.updated_at.timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO procedural_memories (
                id, name, description, task_type, steps, evidence, confidence, version, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                id, name, description, task_type, steps, evidence, confidence, version, created_at, updated_at
            ],
        ).await.map_err(|e| do_memory_core::Error::Storage(format!("Failed to store procedural memory: {}", e)))?;

        Ok(())
    }

    /// Retrieve a procedural memory by ID
    pub async fn get_procedural_memory(&self, id: Uuid) -> Result<Option<ProceduralMemory>> {
        let conn = self.get_connection().await?;
        let mut rows = conn
            .query(
                "SELECT id, name, description, task_type, steps, evidence, confidence, version, created_at, updated_at 
                 FROM procedural_memories WHERE id = ?",
                params![id.to_string()],
            )
            .await
            .map_err(|e| do_memory_core::Error::Storage(format!("Failed to query procedural memory: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| do_memory_core::Error::Storage(format!("Failed to fetch row: {}", e)))?
        {
            Ok(Some(self.row_to_procedural_memory(&row)?))
        } else {
            Ok(None)
        }
    }

    /// Query procedural memories by task type
    pub async fn query_procedural_memories(
        &self,
        task_type: TaskType,
        limit: Option<usize>,
    ) -> Result<Vec<ProceduralMemory>> {
        let conn = self.get_connection().await?;
        let limit = do_memory_core::storage::apply_query_limit(limit);

        let mut rows = conn
            .query(
                "SELECT id, name, description, task_type, steps, evidence, confidence, version, created_at, updated_at 
                 FROM procedural_memories WHERE task_type = ? LIMIT ?",
                params![task_type.to_string(), limit as i64],
            )
            .await
            .map_err(|e| do_memory_core::Error::Storage(format!("Failed to query procedural memories: {}", e)))?;

        let mut results = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| do_memory_core::Error::Storage(format!("Failed to fetch row: {}", e)))?
        {
            results.push(self.row_to_procedural_memory(&row)?);
        }

        Ok(results)
    }

    /// Convert a database row to a ProceduralMemory struct
    fn row_to_procedural_memory(&self, row: &Row) -> Result<ProceduralMemory> {
        let id_str: String = row
            .get(0)
            .map_err(|e| do_memory_core::Error::Storage(format!("Column 0 (id) error: {}", e)))?;
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| do_memory_core::Error::Storage(format!("UUID parse error: {}", e)))?;

        let name: String = row
            .get(1)
            .map_err(|e| do_memory_core::Error::Storage(format!("Column 1 (name) error: {}", e)))?;
        let description: String = row.get(2).map_err(|e| {
            do_memory_core::Error::Storage(format!("Column 2 (description) error: {}", e))
        })?;

        let task_type_str: String = row.get(3).map_err(|e| {
            do_memory_core::Error::Storage(format!("Column 3 (task_type) error: {}", e))
        })?;
        let task_type: TaskType = task_type_str
            .parse()
            .map_err(|e| do_memory_core::Error::Storage(format!("TaskType parse error: {}", e)))?;

        let steps_json: String = row.get(4).map_err(|e| {
            do_memory_core::Error::Storage(format!("Column 4 (steps) error: {}", e))
        })?;
        let steps: Vec<ProceduralStep> = serde_json::from_str(&steps_json).map_err(|e| {
            do_memory_core::Error::Storage(format!("Steps deserialize error: {}", e))
        })?;

        let evidence_json: String = row.get(5).map_err(|e| {
            do_memory_core::Error::Storage(format!("Column 5 (evidence) error: {}", e))
        })?;
        let evidence = serde_json::from_str(&evidence_json).map_err(|e| {
            do_memory_core::Error::Storage(format!("Evidence deserialize error: {}", e))
        })?;

        let confidence: f64 = row.get(6).map_err(|e| {
            do_memory_core::Error::Storage(format!("Column 6 (confidence) error: {}", e))
        })?;
        let version: i64 = row.get(7).map_err(|e| {
            do_memory_core::Error::Storage(format!("Column 7 (version) error: {}", e))
        })?;

        let created_at_ts: i64 = row.get(8).map_err(|e| {
            do_memory_core::Error::Storage(format!("Column 8 (created_at) error: {}", e))
        })?;
        let updated_at_ts: i64 = row.get(9).map_err(|e| {
            do_memory_core::Error::Storage(format!("Column 9 (updated_at) error: {}", e))
        })?;

        Ok(ProceduralMemory {
            id,
            name,
            description,
            task_type,
            steps,
            evidence,
            confidence: confidence as f32,
            version: version as u32,
            created_at: chrono::DateTime::from_timestamp(created_at_ts, 0).unwrap_or_default(),
            updated_at: chrono::DateTime::from_timestamp(updated_at_ts, 0).unwrap_or_default(),
        })
    }
}

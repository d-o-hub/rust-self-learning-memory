//! Procedural memory CRUD operations for Turso storage

use crate::TursoStorage;
use do_memory_core::procedural::ProceduralMemory;
use do_memory_core::{Error, Result};
use libsql::Row;
use tracing::{debug, info};
use uuid::Uuid;

impl TursoStorage {
    /// Store a procedural memory
    pub async fn store_procedural_memory(&self, procedural: &ProceduralMemory) -> Result<()> {
        debug!("Storing procedural memory: {}", procedural.id);
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        const SQL: &str = r#"
            INSERT OR REPLACE INTO procedural_memory (
                procedural_id, name, description, context, steps,
                effectiveness, source_episodes, source_patterns,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        let context_json = serde_json::to_string(&procedural.context).map_err(Error::Serialization)?;
        let steps_json = serde_json::to_string(&procedural.steps).map_err(Error::Serialization)?;
        let effectiveness_json =
            serde_json::to_string(&procedural.effectiveness).map_err(Error::Serialization)?;
        let source_episodes_json =
            serde_json::to_string(&procedural.source_episodes).map_err(Error::Serialization)?;
        let source_patterns_json =
            serde_json::to_string(&procedural.source_patterns).map_err(Error::Serialization)?;

        // Use prepared statement cache
        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, SQL)
            .await
            .map_err(|e| Error::Storage(format!("Failed to prepare statement: {}", e)))?;

        stmt.execute(libsql::params![
            procedural.id.to_string(),
            procedural.name.clone(),
            procedural.description.clone(),
            context_json,
            steps_json,
            effectiveness_json,
            source_episodes_json,
            source_patterns_json,
            procedural.created_at.timestamp(),
            procedural.updated_at.timestamp(),
        ])
        .await
        .map_err(|e| Error::Storage(format!("Failed to store procedural memory: {}", e)))?;

        info!("Successfully stored procedural memory: {}", procedural.id);
        Ok(())
    }

    /// Retrieve a procedural memory by ID
    pub async fn get_procedural_memory(&self, id: Uuid) -> Result<Option<ProceduralMemory>> {
        debug!("Retrieving procedural memory: {}", id);
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        const SQL: &str = r#"
            SELECT procedural_id, name, description, context, steps,
                   effectiveness, source_episodes, source_patterns,
                   created_at, updated_at
            FROM procedural_memory WHERE procedural_id = ?
        "#;

        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, SQL)
            .await
            .map_err(|e| Error::Storage(format!("Failed to prepare statement: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params![id.to_string()])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query procedural memory: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch procedural memory row: {}", e)))?
        {
            Ok(Some(row_to_procedural(&row)?))
        } else {
            Ok(None)
        }
    }

    /// Delete a procedural memory by ID
    pub async fn delete_procedural_memory(&self, id: Uuid) -> Result<()> {
        debug!("Deleting procedural memory: {}", id);
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        const SQL: &str = "DELETE FROM procedural_memory WHERE procedural_id = ?";

        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, SQL)
            .await
            .map_err(|e| Error::Storage(format!("Failed to prepare statement: {}", e)))?;

        stmt.execute(libsql::params![id.to_string()])
            .await
            .map_err(|e| Error::Storage(format!("Failed to delete procedural memory: {}", e)))?;

        info!("Successfully deleted procedural memory: {}", id);
        Ok(())
    }

    /// Query procedural memories
    pub async fn query_procedural_memories(
        &self,
        limit: Option<usize>,
    ) -> Result<Vec<ProceduralMemory>> {
        debug!("Querying procedural memories");
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        let limit = do_memory_core::apply_query_limit(limit);

        const SQL: &str = r#"
            SELECT procedural_id, name, description, context, steps,
                   effectiveness, source_episodes, source_patterns,
                   created_at, updated_at
            FROM procedural_memory
            ORDER BY updated_at DESC
            LIMIT ?
        "#;

        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, SQL)
            .await
            .map_err(|e| Error::Storage(format!("Failed to prepare statement: {}", e)))?;

        let mut rows = stmt
            .query(libsql::params![limit as i64])
            .await
            .map_err(|e| Error::Storage(format!("Failed to query procedural memories: {}", e)))?;

        let mut results = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to fetch procedural memory row: {}", e)))?
        {
            results.push(row_to_procedural(&row)?);
        }

        Ok(results)
    }
}

/// Convert a database row to a ProceduralMemory
fn row_to_procedural(row: &Row) -> Result<ProceduralMemory> {
    let id_str: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
    let id = Uuid::parse_str(&id_str).map_err(|e| Error::Storage(e.to_string()))?;
    let name: String = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
    let description: String = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
    let context_json: String = row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
    let steps_json: String = row.get(4).map_err(|e| Error::Storage(e.to_string()))?;
    let effectiveness_json: String = row.get(5).map_err(|e| Error::Storage(e.to_string()))?;
    let source_episodes_json: String = row.get(6).map_err(|e| Error::Storage(e.to_string()))?;
    let source_patterns_json: String = row.get(7).map_err(|e| Error::Storage(e.to_string()))?;
    let created_at: i64 = row.get(8).map_err(|e| Error::Storage(e.to_string()))?;
    let updated_at: i64 = row.get(9).map_err(|e| Error::Storage(e.to_string()))?;

    let context = serde_json::from_str(&context_json).map_err(Error::Serialization)?;
    let steps = serde_json::from_str(&steps_json).map_err(Error::Serialization)?;
    let effectiveness = serde_json::from_str(&effectiveness_json).map_err(Error::Serialization)?;
    let source_episodes = serde_json::from_str(&source_episodes_json).map_err(Error::Serialization)?;
    let source_patterns = serde_json::from_str(&source_patterns_json).map_err(Error::Serialization)?;

    Ok(ProceduralMemory {
        id,
        name,
        description,
        context,
        steps,
        effectiveness,
        source_episodes,
        source_patterns,
        created_at: chrono::DateTime::from_timestamp(created_at, 0).unwrap_or_default(),
        updated_at: chrono::DateTime::from_timestamp(updated_at, 0).unwrap_or_default(),
    })
}

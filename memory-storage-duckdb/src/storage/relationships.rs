use crate::DuckDbStorage;
use chrono::{DateTime, Utc};
use do_memory_core::{Error, Result};
use duckdb::params;
use std::sync::Arc;
use uuid::Uuid;

impl DuckDbStorage {
    pub(crate) async fn store_relationship_internal(
        &self,
        rel: &do_memory_core::episode::EpisodeRelationship,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let rel = rel.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let metadata_json = serde_json::to_string(&rel.metadata)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;

            conn.execute(
                "INSERT OR REPLACE INTO episode_relationships (
                    relationship_id, from_episode_id, to_episode_id, relationship_type,
                    reason, created_by, priority, metadata, created_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    rel.id.to_string(),
                    rel.from_episode_id.to_string(),
                    rel.to_episode_id.to_string(),
                    rel.relationship_type.as_str(),
                    rel.metadata.reason,
                    rel.metadata.created_by,
                    rel.metadata.priority.map(i32::from),
                    metadata_json,
                    rel.created_at.to_rfc3339(),
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store relationship: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn remove_relationship_internal(&self, id: Uuid) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            conn.execute(
                "DELETE FROM episode_relationships WHERE relationship_id = ?",
                params![id.to_string()],
            )
            .map_err(|e| Error::Storage(format!("Failed to remove relationship: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn get_relationships_internal(
        &self,
        episode_id: Uuid,
        direction: do_memory_core::episode::Direction,
    ) -> Result<Vec<do_memory_core::episode::EpisodeRelationship>> {
        let conn_arc = Arc::clone(&self.conn);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let query = match direction {
                do_memory_core::episode::Direction::Outgoing => {
                    "SELECT relationship_id, from_episode_id, to_episode_id, relationship_type,
                     CAST(metadata AS VARCHAR), strftime(CAST(created_at AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ')
                     FROM episode_relationships WHERE from_episode_id = ?"
                }
                do_memory_core::episode::Direction::Incoming => {
                    "SELECT relationship_id, from_episode_id, to_episode_id, relationship_type,
                     CAST(metadata AS VARCHAR), strftime(CAST(created_at AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ')
                     FROM episode_relationships WHERE to_episode_id = ?"
                }
                do_memory_core::episode::Direction::Both => {
                    "SELECT relationship_id, from_episode_id, to_episode_id, relationship_type,
                     CAST(metadata AS VARCHAR), strftime(CAST(created_at AS TIMESTAMP), '%Y-%m-%dT%H:%M:%S.%fZ')
                     FROM episode_relationships WHERE from_episode_id = ? OR to_episode_id = ?"
                }
            };

            let mut stmt = conn
                .prepare(query)
                .map_err(|e| Error::Storage(e.to_string()))?;

            let id_str = episode_id.to_string();
            let mut rows = if direction == do_memory_core::episode::Direction::Both {
                stmt.query(params![id_str, id_str])
            } else {
                stmt.query(params![id_str])
            }
            .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rels = Vec::new();
            while let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let id: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let from_id: String = row.get(1).map_err(|e| Error::Storage(e.to_string()))?;
                let to_id: String = row.get(2).map_err(|e| Error::Storage(e.to_string()))?;
                let rel_type_str: String = row.get(3).map_err(|e| Error::Storage(e.to_string()))?;
                let metadata_json: String = row.get(4).map_err(|e| Error::Storage(e.to_string()))?;
                let created_at_str: String = row.get(5).map_err(|e| Error::Storage(e.to_string()))?;

                rels.push(do_memory_core::episode::EpisodeRelationship {
                    id: Uuid::parse_str(&id).map_err(|e| Error::Storage(e.to_string()))?,
                    from_episode_id: Uuid::parse_str(&from_id)
                        .map_err(|e| Error::Storage(e.to_string()))?,
                    to_episode_id: Uuid::parse_str(&to_id)
                        .map_err(|e| Error::Storage(e.to_string()))?,
                    relationship_type: do_memory_core::episode::RelationshipType::parse(&rel_type_str)
                        .map_err(Error::Storage)?,
                    metadata: serde_json::from_str(&metadata_json)
                        .map_err(|e| Error::Storage(e.to_string()))?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .or_else(|_| DateTime::parse_from_str(&created_at_str, "%Y-%m-%dT%H:%M:%S.%fZ"))
                        .map_err(|e| Error::Storage(e.to_string()))?
                        .with_timezone(&Utc),
                });
            }
            Ok::<Vec<do_memory_core::episode::EpisodeRelationship>, Error>(rels)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    pub(crate) async fn relationship_exists_internal(
        &self,
        from_episode_id: Uuid,
        to_episode_id: Uuid,
        rel_type: do_memory_core::episode::RelationshipType,
    ) -> Result<bool> {
        let conn_arc = Arc::clone(&self.conn);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare(
                    "SELECT 1 FROM episode_relationships
                     WHERE from_episode_id = ? AND to_episode_id = ? AND relationship_type = ?",
                )
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![
                    from_episode_id.to_string(),
                    to_episode_id.to_string(),
                    rel_type.as_str()
                ])
                .map_err(|e| Error::Storage(e.to_string()))?;

            Ok::<bool, Error>(
                rows.next()
                    .map_err(|e| Error::Storage(e.to_string()))?
                    .is_some(),
            )
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    // ========== Recommendation Attribution Methods ==========
}

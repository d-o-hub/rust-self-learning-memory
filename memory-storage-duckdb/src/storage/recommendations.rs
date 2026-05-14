use crate::DuckDbStorage;
use do_memory_core::{Error, Result};
use duckdb::params;
use std::sync::Arc;
use uuid::Uuid;

impl DuckDbStorage {
    pub(crate) async fn store_recommendation_session_internal(
        &self,
        session: &do_memory_core::memory::attribution::RecommendationSession,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let session = session.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let payload_json = serde_json::to_string(&session)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;

            conn.execute(
                "INSERT OR REPLACE INTO recommendation_sessions (
                    session_id, episode_id, timestamp, payload
                ) VALUES (?, ?, ?, ?)",
                params![
                    session.session_id.to_string(),
                    session.episode_id.to_string(),
                    session.timestamp.to_rfc3339(),
                    payload_json,
                ],
            )
            .map_err(|e| Error::Storage(format!("Failed to store recommendation session: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn get_recommendation_session_internal(
        &self,
        session_id: Uuid,
    ) -> Result<Option<do_memory_core::memory::attribution::RecommendationSession>> {
        let conn_arc = Arc::clone(&self.conn);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare("SELECT CAST(payload AS VARCHAR) FROM recommendation_sessions WHERE session_id = ?")
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![session_id.to_string()])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let payload_json: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let session = serde_json::from_str(&payload_json)
                    .map_err(|e| Error::Storage(e.to_string()))?;
                Ok::<Option<do_memory_core::memory::attribution::RecommendationSession>, Error>(Some(session))
            } else {
                Ok::<Option<do_memory_core::memory::attribution::RecommendationSession>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    pub(crate) async fn get_recommendation_session_for_episode_internal(
        &self,
        episode_id: Uuid,
    ) -> Result<Option<do_memory_core::memory::attribution::RecommendationSession>> {
        let conn_arc = Arc::clone(&self.conn);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare("SELECT CAST(payload AS VARCHAR) FROM recommendation_sessions WHERE episode_id = ? ORDER BY timestamp DESC LIMIT 1")
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![episode_id.to_string()])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let payload_json: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let session = serde_json::from_str(&payload_json)
                    .map_err(|e| Error::Storage(e.to_string()))?;
                Ok::<Option<do_memory_core::memory::attribution::RecommendationSession>, Error>(Some(session))
            } else {
                Ok::<Option<do_memory_core::memory::attribution::RecommendationSession>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    pub(crate) async fn store_recommendation_feedback_internal(
        &self,
        feedback: &do_memory_core::memory::attribution::RecommendationFeedback,
    ) -> Result<()> {
        let conn_arc = Arc::clone(&self.conn);
        let feedback = feedback.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let payload_json = serde_json::to_string(&feedback)
                .map_err(|e| Error::Storage(format!("Serialization error: {e}")))?;

            conn.execute(
                "INSERT OR REPLACE INTO recommendation_feedback (session_id, payload) VALUES (?, ?)",
                params![feedback.session_id.to_string(), payload_json],
            )
            .map_err(|e| Error::Storage(format!("Failed to store recommendation feedback: {e}")))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(())
    }

    pub(crate) async fn get_recommendation_feedback_internal(
        &self,
        session_id: Uuid,
    ) -> Result<Option<do_memory_core::memory::attribution::RecommendationFeedback>> {
        let conn_arc = Arc::clone(&self.conn);
        let res = tokio::task::spawn_blocking(move || {
            let conn = conn_arc.lock();
            let mut stmt = conn
                .prepare("SELECT CAST(payload AS VARCHAR) FROM recommendation_feedback WHERE session_id = ?")
                .map_err(|e| Error::Storage(e.to_string()))?;

            let mut rows = stmt
                .query(params![session_id.to_string()])
                .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| Error::Storage(e.to_string()))? {
                let payload_json: String = row.get(0).map_err(|e| Error::Storage(e.to_string()))?;
                let feedback = serde_json::from_str(&payload_json)
                    .map_err(|e| Error::Storage(e.to_string()))?;
                Ok::<Option<do_memory_core::memory::attribution::RecommendationFeedback>, Error>(Some(feedback))
            } else {
                Ok::<Option<do_memory_core::memory::attribution::RecommendationFeedback>, Error>(None)
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {e}")))??;
        Ok(res)
    }

    // ========== Monitoring Storage Methods ==========
}

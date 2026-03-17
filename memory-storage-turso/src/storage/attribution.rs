//! Recommendation Attribution CRUD operations for Turso storage

use crate::TursoStorage;
use memory_core::memory::attribution::{RecommendationFeedback, RecommendationSession};
use memory_core::{Error, Result};
use tracing::debug;
use uuid::Uuid;

impl TursoStorage {
    /// Store a recommendation session
    pub async fn store_recommendation_session(
        &self,
        session: &RecommendationSession,
    ) -> Result<()> {
        debug!("Storing recommendation session: {}", session.session_id);
        let (conn, conn_id) = self.get_connection_with_id().await?;

        const SQL: &str = r#"
            INSERT OR REPLACE INTO recommendation_sessions (
                session_id, episode_id, timestamp, recommended_pattern_ids, recommended_playbook_ids
            ) VALUES (?, ?, ?, ?, ?)
        "#;

        let patterns_json = serde_json::to_string(&session.recommended_pattern_ids)
            .map_err(Error::Serialization)?;
        let playbooks_json = serde_json::to_string(&session.recommended_playbook_ids)
            .map_err(Error::Serialization)?;

        let stmt = self.prepare_cached(conn_id, &conn, SQL).await?;
        stmt.execute(libsql::params![
            session.session_id.to_string(),
            session.episode_id.to_string(),
            session.timestamp.timestamp(),
            patterns_json,
            playbooks_json,
        ])
        .await
        .map_err(|e| Error::Storage(format!("Failed to store session: {}", e)))?;

        self.clear_prepared_cache(conn_id);
        Ok(())
    }

    /// Store recommendation feedback
    pub async fn store_recommendation_feedback(
        &self,
        feedback: &RecommendationFeedback,
    ) -> Result<()> {
        debug!(
            "Storing recommendation feedback for session: {}",
            feedback.session_id
        );
        let (conn, conn_id) = self.get_connection_with_id().await?;

        const SQL: &str = r#"
            INSERT OR REPLACE INTO recommendation_feedback (
                session_id, applied_pattern_ids, consulted_episode_ids, outcome, agent_rating, created_at
            ) VALUES (?, ?, ?, ?, ?, ?)
        "#;

        let applied_json =
            serde_json::to_string(&feedback.applied_pattern_ids).map_err(Error::Serialization)?;
        let consulted_json =
            serde_json::to_string(&feedback.consulted_episode_ids).map_err(Error::Serialization)?;
        let outcome_json =
            serde_json::to_string(&feedback.outcome).map_err(Error::Serialization)?;

        let stmt = self.prepare_cached(conn_id, &conn, SQL).await?;
        stmt.execute(libsql::params![
            feedback.session_id.to_string(),
            applied_json,
            consulted_json,
            outcome_json,
            feedback.agent_rating,
            chrono::Utc::now().timestamp(),
        ])
        .await
        .map_err(|e| Error::Storage(format!("Failed to store feedback: {}", e)))?;

        self.clear_prepared_cache(conn_id);
        Ok(())
    }

    /// Retrieve a recommendation session
    pub async fn get_recommendation_session(&self, session_id: Uuid) -> Result<Option<RecommendationSession>> {
        debug!("Retrieving recommendation session: {}", session_id);
        let (conn, conn_id) = self.get_connection_with_id().await?;

        const SQL: &str = r#"
            SELECT session_id, episode_id, timestamp, recommended_pattern_ids, recommended_playbook_ids
            FROM recommendation_sessions WHERE session_id = ?
        "#;

        let stmt = self.prepare_cached(conn_id, &conn, SQL).await?;
        let mut rows = stmt.query(libsql::params![session_id.to_string()]).await.map_err(|e: libsql::Error| Error::Storage(e.to_string()))?;

        if let Some(row) = rows.next().await.map_err(|e: libsql::Error| Error::Storage(e.to_string()))? {
            let session_id_str: String = row.get(0).map_err(|e: libsql::Error| Error::Storage(e.to_string()))?;
            let episode_id_str: String = row.get(1).map_err(|e: libsql::Error| Error::Storage(e.to_string()))?;
            let timestamp: i64 = row.get(2).map_err(|e: libsql::Error| Error::Storage(e.to_string()))?;
            let patterns_json: String = row.get(3).map_err(|e: libsql::Error| Error::Storage(e.to_string()))?;
            let playbooks_json: String = row.get(4).map_err(|e: libsql::Error| Error::Storage(e.to_string()))?;

            let session = RecommendationSession {
                session_id: Uuid::parse_str(&session_id_str).map_err(|e| Error::Storage(e.to_string()))?,
                episode_id: Uuid::parse_str(&episode_id_str).map_err(|e| Error::Storage(e.to_string()))?,
                timestamp: chrono::DateTime::from_timestamp(timestamp, 0).unwrap_or_default(),
                recommended_pattern_ids: serde_json::from_str(&patterns_json).map_err(Error::Serialization)?,
                recommended_playbook_ids: serde_json::from_str(&playbooks_json).map_err(Error::Serialization)?,
            };
            self.clear_prepared_cache(conn_id);
            Ok(Some(session))
        } else {
            self.clear_prepared_cache(conn_id);
            Ok(None)
        }
    }

    /// Retrieve recommendation feedback
    pub async fn get_recommendation_feedback(&self, session_id: Uuid) -> Result<Option<RecommendationFeedback>> {
        debug!("Retrieving recommendation feedback for session: {}", session_id);
        let (conn, conn_id) = self.get_connection_with_id().await?;

        const SQL: &str = r#"
            SELECT session_id, applied_pattern_ids, consulted_episode_ids, outcome, agent_rating
            FROM recommendation_feedback WHERE session_id = ?
        "#;

        let stmt = self.prepare_cached(conn_id, &conn, SQL).await?;
        let mut rows = stmt.query(libsql::params![session_id.to_string()]).await.map_err(|e: libsql::Error| Error::Storage(e.to_string()))?;

        if let Some(row) = rows.next().await.map_err(|e: libsql::Error| Error::Storage(e.to_string()))? {
            let session_id_str: String = row.get(0).map_err(|e: libsql::Error| Error::Storage(e.to_string()))?;
            let applied_json: String = row.get(1).map_err(|e: libsql::Error| Error::Storage(e.to_string()))?;
            let consulted_json: String = row.get(2).map_err(|e: libsql::Error| Error::Storage(e.to_string()))?;
            let outcome_json: String = row.get(3).map_err(|e: libsql::Error| Error::Storage(e.to_string()))?;
            let agent_rating: Option<f64> = row.get(4).ok();

            let feedback = RecommendationFeedback {
                session_id: Uuid::parse_str(&session_id_str).map_err(|e| Error::Storage(e.to_string()))?,
                applied_pattern_ids: serde_json::from_str(&applied_json).map_err(Error::Serialization)?,
                consulted_episode_ids: serde_json::from_str(&consulted_json).map_err(Error::Serialization)?,
                outcome: serde_json::from_str(&outcome_json).map_err(Error::Serialization)?,
                agent_rating: agent_rating.map(|r| r as f32),
            };
            self.clear_prepared_cache(conn_id);
            Ok(Some(feedback))
        } else {
            self.clear_prepared_cache(conn_id);
            Ok(None)
        }
    }
}

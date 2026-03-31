//! Recommendation attribution storage operations (ADR-044)

use crate::TursoStorage;
use do_memory_core::memory::attribution::{
    RecommendationFeedback, RecommendationSession, RecommendationStats,
};
use do_memory_core::{Error, Result};
use libsql::params;
use tracing::debug;
use uuid::Uuid;

impl TursoStorage {
    pub async fn store_recommendation_session(
        &self,
        session: &RecommendationSession,
    ) -> Result<()> {
        const SQL: &str = r#"
            INSERT INTO recommendation_sessions (session_id, episode_id, timestamp, payload)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(session_id) DO UPDATE SET
                episode_id = excluded.episode_id,
                timestamp = excluded.timestamp,
                payload = excluded.payload
        "#;

        let payload = serde_json::to_string(session)
            .map_err(|e| {
                Error::Storage(format!("Failed to serialize recommendation session: {}", e))
            })?
            .into_boxed_str();
        let (conn, _conn_id) = self.get_connection_with_id().await?;
        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, SQL)
            .await
            .map_err(|e| Error::Storage(format!("Failed to prepare session insert: {}", e)))?;

        stmt.execute(params![
            session.session_id.to_string(),
            session.episode_id.to_string(),
            session.timestamp.timestamp(),
            payload,
        ])
        .await
        .map_err(|e| Error::Storage(format!("Failed to persist recommendation session: {}", e)))?;

        debug!(session_id = %session.session_id, "Stored recommendation session");
        Ok(())
    }

    pub async fn get_recommendation_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<RecommendationSession>> {
        let (conn, _conn_id) = self.get_connection_with_id().await?;
        let mut rows = conn
            .query(
                "SELECT payload FROM recommendation_sessions WHERE session_id = ?",
                params![session_id.to_string()],
            )
            .await
            .map_err(|e| {
                Error::Storage(format!("Failed to query recommendation session: {}", e))
            })?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to read session row: {}", e)))?
        {
            let payload: String = row
                .get(0)
                .map_err(|e| Error::Storage(format!("Failed to read session payload: {}", e)))?;
            let session = serde_json::from_str(&payload).map_err(|e| {
                Error::Storage(format!(
                    "Failed to deserialize recommendation session: {}",
                    e
                ))
            })?;
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    pub async fn get_recommendation_session_for_episode(
        &self,
        episode_id: Uuid,
    ) -> Result<Option<RecommendationSession>> {
        let (conn, _conn_id) = self.get_connection_with_id().await?;
        let mut rows = conn
            .query(
                "SELECT payload FROM recommendation_sessions WHERE episode_id = ? ORDER BY timestamp DESC LIMIT 1",
                params![episode_id.to_string()],
            )
            .await
            .map_err(|e| {
                Error::Storage(format!("Failed to query recommendation session: {}", e))
            })?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to read session row: {}", e)))?
        {
            let payload: String = row
                .get(0)
                .map_err(|e| Error::Storage(format!("Failed to read session payload: {}", e)))?;
            let session = serde_json::from_str(&payload).map_err(|e| {
                Error::Storage(format!(
                    "Failed to deserialize recommendation session: {}",
                    e
                ))
            })?;
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    pub async fn store_recommendation_feedback(
        &self,
        feedback: &RecommendationFeedback,
    ) -> Result<()> {
        const SQL: &str = r#"
            INSERT INTO recommendation_feedback (session_id, payload)
            VALUES (?1, ?2)
            ON CONFLICT(session_id) DO UPDATE SET payload = excluded.payload
        "#;

        let payload = serde_json::to_string(feedback)
            .map_err(|e| {
                Error::Storage(format!(
                    "Failed to serialize recommendation feedback: {}",
                    e
                ))
            })?
            .into_boxed_str();
        let (conn, _conn_id) = self.get_connection_with_id().await?;
        let stmt = self
            .prepared_cache
            .get_or_prepare(&conn, SQL)
            .await
            .map_err(|e| Error::Storage(format!("Failed to prepare feedback insert: {}", e)))?;

        stmt.execute(params![feedback.session_id.to_string(), payload])
            .await
            .map_err(|e| {
                Error::Storage(format!("Failed to persist recommendation feedback: {}", e))
            })?;

        debug!(session_id = %feedback.session_id, "Stored recommendation feedback");
        Ok(())
    }

    pub async fn get_recommendation_feedback(
        &self,
        session_id: Uuid,
    ) -> Result<Option<RecommendationFeedback>> {
        let (conn, _conn_id) = self.get_connection_with_id().await?;
        let mut rows = conn
            .query(
                "SELECT payload FROM recommendation_feedback WHERE session_id = ?",
                params![session_id.to_string()],
            )
            .await
            .map_err(|e| {
                Error::Storage(format!("Failed to query recommendation feedback: {}", e))
            })?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to read feedback row: {}", e)))?
        {
            let payload: String = row
                .get(0)
                .map_err(|e| Error::Storage(format!("Failed to read feedback payload: {}", e)))?;
            let feedback = serde_json::from_str(&payload).map_err(|e| {
                Error::Storage(format!(
                    "Failed to deserialize recommendation feedback: {}",
                    e
                ))
            })?;
            Ok(Some(feedback))
        } else {
            Ok(None)
        }
    }

    pub async fn get_recommendation_stats(&self) -> Result<RecommendationStats> {
        let (conn, _conn_id) = self.get_connection_with_id().await?;

        // Gather sessions
        let mut rows = conn
            .query("SELECT payload FROM recommendation_sessions", params![])
            .await
            .map_err(|e| {
                Error::Storage(format!("Failed to query recommendation sessions: {}", e))
            })?;

        let mut stats = RecommendationStats::default();
        let mut total_recommended = 0usize;

        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to read session row: {}", e)))?
        {
            let payload: String = row
                .get(0)
                .map_err(|e| Error::Storage(format!("Failed to read session payload: {}", e)))?;
            let session: RecommendationSession = serde_json::from_str(&payload).map_err(|e| {
                Error::Storage(format!(
                    "Failed to deserialize recommendation session: {}",
                    e
                ))
            })?;

            stats.total_sessions += 1;
            total_recommended += session.recommended_pattern_ids.len();
        }

        // Gather feedback
        let mut feedback_rows = conn
            .query("SELECT payload FROM recommendation_feedback", params![])
            .await
            .map_err(|e| {
                Error::Storage(format!("Failed to query recommendation feedback: {}", e))
            })?;

        let mut total_applied = 0usize;
        let mut successful_applications = 0usize;
        let mut total_ratings = 0f32;
        let mut rating_count = 0usize;

        while let Some(row) = feedback_rows
            .next()
            .await
            .map_err(|e| Error::Storage(format!("Failed to read feedback row: {}", e)))?
        {
            let payload: String = row
                .get(0)
                .map_err(|e| Error::Storage(format!("Failed to read feedback payload: {}", e)))?;
            let feedback: RecommendationFeedback = serde_json::from_str(&payload).map_err(|e| {
                Error::Storage(format!(
                    "Failed to deserialize recommendation feedback: {}",
                    e
                ))
            })?;

            stats.total_feedback += 1;
            total_applied += feedback.applied_pattern_ids.len();

            if matches!(
                feedback.outcome,
                do_memory_core::types::TaskOutcome::Success { .. }
                    | do_memory_core::types::TaskOutcome::PartialSuccess { .. }
            ) {
                successful_applications += feedback.applied_pattern_ids.len();
            }

            if let Some(rating) = feedback.agent_rating {
                total_ratings += rating;
                rating_count += 1;
            }
        }

        stats.patterns_applied = total_applied;
        stats.patterns_ignored = total_recommended.saturating_sub(total_applied);
        stats.successful_applications = successful_applications;

        stats.adoption_rate = if total_recommended > 0 {
            total_applied as f32 / total_recommended as f32
        } else {
            0.0
        };

        stats.success_after_adoption_rate = if total_applied > 0 {
            successful_applications as f32 / total_applied as f32
        } else {
            0.0
        };

        stats.avg_agent_rating = if rating_count > 0 {
            Some(total_ratings / rating_count as f32)
        } else {
            None
        };

        Ok(stats)
    }
}

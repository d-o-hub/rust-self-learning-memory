//! Recommendation attribution storage for redb cache

use crate::{
    with_db_timeout, RedbStorage, RECOMMENDATION_EPISODE_INDEX_TABLE,
    RECOMMENDATION_FEEDBACK_TABLE, RECOMMENDATION_SESSIONS_TABLE,
};
use memory_core::memory::attribution::{
    RecommendationFeedback, RecommendationSession, RecommendationStats,
};
use memory_core::{Error, Result};
use redb::{ReadableDatabase, ReadableTable, TableDefinition};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use uuid::Uuid;

impl RedbStorage {
    pub async fn store_recommendation_session(
        &self,
        session: &RecommendationSession,
    ) -> Result<()> {
        let db = Arc::clone(&self.db);
        let session_bytes = postcard::to_allocvec(session).map_err(|e| {
            Error::Storage(format!("Failed to serialize recommendation session: {}", e))
        })?;
        let session_key = session.session_id.to_string();
        let episode_key = session.episode_id.to_string();

        with_db_timeout(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                let mut session_table = write_txn
                    .open_table(RECOMMENDATION_SESSIONS_TABLE)
                    .map_err(|e| {
                        Error::Storage(format!(
                            "Failed to open recommendation sessions table: {}",
                            e
                        ))
                    })?;
                session_table
                    .insert(session_key.as_str(), session_bytes.as_slice())
                    .map_err(|e| {
                        Error::Storage(format!("Failed to insert recommendation session: {}", e))
                    })?;
            }

            {
                let mut episode_index = write_txn
                    .open_table(RECOMMENDATION_EPISODE_INDEX_TABLE)
                    .map_err(|e| {
                        Error::Storage(format!(
                            "Failed to open recommendation episode index: {}",
                            e
                        ))
                    })?;
                episode_index
                    .insert(episode_key.as_str(), session_key.as_str())
                    .map_err(|e| {
                        Error::Storage(format!(
                            "Failed to update recommendation episode index: {}",
                            e
                        ))
                    })?;
            }

            write_txn.commit().map_err(|e| {
                Error::Storage(format!(
                    "Failed to commit recommendation session transaction: {}",
                    e
                ))
            })?;
            Ok(())
        })
        .await
    }

    pub async fn get_recommendation_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<RecommendationSession>> {
        self.read_postcard_entry(RECOMMENDATION_SESSIONS_TABLE, session_id.to_string())
            .await
    }

    pub async fn get_recommendation_session_for_episode(
        &self,
        episode_id: Uuid,
    ) -> Result<Option<RecommendationSession>> {
        let db = Arc::clone(&self.db);
        let episode_key = episode_id.to_string();

        let session_id = with_db_timeout(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;
            let table = read_txn
                .open_table(RECOMMENDATION_EPISODE_INDEX_TABLE)
                .map_err(|e| {
                    Error::Storage(format!(
                        "Failed to open recommendation episode index: {}",
                        e
                    ))
                })?;
            let entry = table
                .get(episode_key.as_str())
                .map_err(|e| Error::Storage(format!("Failed to read episode index: {}", e)))?;
            let value = entry.map(|v| v.value().to_string());
            Ok(value)
        })
        .await?;

        if let Some(session_id) = session_id {
            self.read_postcard_entry(RECOMMENDATION_SESSIONS_TABLE, session_id)
                .await
        } else {
            Ok(None)
        }
    }

    pub async fn store_recommendation_feedback(
        &self,
        feedback: &RecommendationFeedback,
    ) -> Result<()> {
        let db = Arc::clone(&self.db);
        let feedback_bytes = postcard::to_allocvec(feedback).map_err(|e| {
            Error::Storage(format!(
                "Failed to serialize recommendation feedback: {}",
                e
            ))
        })?;
        let session_key = feedback.session_id.to_string();

        with_db_timeout(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;
            {
                let mut table = write_txn
                    .open_table(RECOMMENDATION_FEEDBACK_TABLE)
                    .map_err(|e| {
                        Error::Storage(format!(
                            "Failed to open recommendation feedback table: {}",
                            e
                        ))
                    })?;
                table
                    .insert(session_key.as_str(), feedback_bytes.as_slice())
                    .map_err(|e| {
                        Error::Storage(format!("Failed to insert recommendation feedback: {}", e))
                    })?;
            }
            write_txn.commit().map_err(|e| {
                Error::Storage(format!(
                    "Failed to commit recommendation feedback transaction: {}",
                    e
                ))
            })?;
            Ok(())
        })
        .await
    }

    pub async fn get_recommendation_feedback(
        &self,
        session_id: Uuid,
    ) -> Result<Option<RecommendationFeedback>> {
        self.read_postcard_entry(RECOMMENDATION_FEEDBACK_TABLE, session_id.to_string())
            .await
    }

    pub async fn get_recommendation_stats(&self) -> Result<RecommendationStats> {
        let sessions = self
            .collect_postcard_entries::<RecommendationSession>(RECOMMENDATION_SESSIONS_TABLE)
            .await?;
        let feedback = self
            .collect_postcard_entries::<RecommendationFeedback>(RECOMMENDATION_FEEDBACK_TABLE)
            .await?;

        let mut stats = RecommendationStats {
            total_sessions: sessions.len(),
            total_feedback: feedback.len(),
            ..RecommendationStats::default()
        };

        let total_recommended: usize = sessions
            .iter()
            .map(|session| session.recommended_pattern_ids.len())
            .sum();
        let mut total_applied = 0usize;
        let mut successful_applications = 0usize;
        let mut total_ratings = 0f32;
        let mut rating_count = 0usize;

        for fb in &feedback {
            total_applied += fb.applied_pattern_ids.len();
            if matches!(
                fb.outcome,
                memory_core::types::TaskOutcome::Success { .. }
                    | memory_core::types::TaskOutcome::PartialSuccess { .. }
            ) {
                successful_applications += fb.applied_pattern_ids.len();
            }
            if let Some(rating) = fb.agent_rating {
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

    async fn read_postcard_entry<T>(
        &self,
        table: TableDefinition<'static, &'static str, &'static [u8]>,
        key: String,
    ) -> Result<Option<T>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let db = Arc::clone(&self.db);

        with_db_timeout(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;
            let table = read_txn
                .open_table(table)
                .map_err(|e| Error::Storage(format!("Failed to open table: {}", e)))?;
            let guard = table
                .get(key.as_str())
                .map_err(|e| Error::Storage(format!("Failed to read value: {}", e)))?;
            if let Some(value) = guard {
                let entry = postcard::from_bytes(value.value())
                    .map_err(|e| Error::Storage(format!("Failed to deserialize entry: {}", e)))?;
                Ok(Some(entry))
            } else {
                Ok(None)
            }
        })
        .await
    }

    async fn collect_postcard_entries<T>(
        &self,
        table: TableDefinition<'static, &'static str, &'static [u8]>,
    ) -> Result<Vec<T>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let db = Arc::clone(&self.db);

        with_db_timeout(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;
            let table = read_txn
                .open_table(table)
                .map_err(|e| Error::Storage(format!("Failed to open table: {}", e)))?;

            let mut entries = Vec::new();
            let iter = table
                .iter()
                .map_err(|e| Error::Storage(format!("Failed to iterate table entries: {}", e)))?;
            for result in iter {
                let (_key, value) = result
                    .map_err(|e| Error::Storage(format!("Failed to read table entry: {}", e)))?;
                let entry = postcard::from_bytes(value.value())
                    .map_err(|e| Error::Storage(format!("Failed to deserialize entry: {}", e)))?;
                entries.push(entry);
            }

            Ok(entries)
        })
        .await
    }
}

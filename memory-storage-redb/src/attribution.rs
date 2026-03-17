//! Recommendation Attribution CRUD operations for redb storage

use crate::{RedbStorage, with_db_timeout};
use memory_core::memory::attribution::{RecommendationFeedback, RecommendationSession};
use memory_core::{Error, Result};
use redb::{ReadableDatabase, TableDefinition};
use uuid::Uuid;

pub(crate) const SESSIONS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("recommendation_sessions");
pub(crate) const FEEDBACK_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("recommendation_feedback");

impl RedbStorage {
    /// Store a recommendation session
    pub async fn store_recommendation_session(
        &self,
        session: &RecommendationSession,
    ) -> Result<()> {
        let db = self.db.clone();
        let key = session.session_id.to_string();
        let value = serde_json::to_vec(session).map_err(Error::Serialization)?;

        with_db_timeout(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(e.to_string()))?;
            {
                let mut table = write_txn
                    .open_table(SESSIONS_TABLE)
                    .map_err(|e| Error::Storage(e.to_string()))?;
                table
                    .insert(key.as_str(), value.as_slice())
                    .map_err(|e| Error::Storage(e.to_string()))?;
            }
            write_txn
                .commit()
                .map_err(|e| Error::Storage(e.to_string()))?;
            Ok(())
        })
        .await
    }

    /// Store recommendation feedback
    pub async fn store_recommendation_feedback(
        &self,
        feedback: &RecommendationFeedback,
    ) -> Result<()> {
        let db = self.db.clone();
        let key = feedback.session_id.to_string();
        let value = serde_json::to_vec(feedback).map_err(Error::Serialization)?;

        with_db_timeout(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(e.to_string()))?;
            {
                let mut table = write_txn
                    .open_table(FEEDBACK_TABLE)
                    .map_err(|e| Error::Storage(e.to_string()))?;
                table
                    .insert(key.as_str(), value.as_slice())
                    .map_err(|e| Error::Storage(e.to_string()))?;
            }
            write_txn
                .commit()
                .map_err(|e| Error::Storage(e.to_string()))?;
            Ok(())
        })
        .await
    }

    /// Retrieve a recommendation session
    pub async fn get_recommendation_session(&self, session_id: Uuid) -> Result<Option<RecommendationSession>> {
        let db = self.db.clone();
        let key = session_id.to_string();

        with_db_timeout(move || {
            let read_txn = db.begin_read().map_err(|e| Error::Storage(e.to_string()))?;
            let table = read_txn.open_table(SESSIONS_TABLE).map_err(|e| Error::Storage(e.to_string()))?;
            match table.get(key.as_str()).map_err(|e| Error::Storage(e.to_string()))? {
                Some(bytes_guard) => {
                    let session: RecommendationSession = serde_json::from_slice(bytes_guard.value()).map_err(Error::Serialization)?;
                    Ok(Some(session))
                }
                None => Ok(None),
            }
        }).await
    }

    /// Retrieve recommendation feedback
    pub async fn get_recommendation_feedback(&self, session_id: Uuid) -> Result<Option<RecommendationFeedback>> {
        let db = self.db.clone();
        let key = session_id.to_string();

        with_db_timeout(move || {
            let read_txn = db.begin_read().map_err(|e| Error::Storage(e.to_string()))?;
            let table = read_txn.open_table(FEEDBACK_TABLE).map_err(|e| Error::Storage(e.to_string()))?;
            match table.get(key.as_str()).map_err(|e| Error::Storage(e.to_string()))? {
                Some(bytes_guard) => {
                    let feedback: RecommendationFeedback = serde_json::from_slice(bytes_guard.value()).map_err(Error::Serialization)?;
                    Ok(Some(feedback))
                }
                None => Ok(None),
            }
        }).await
    }
}

#[cfg(test)]
mod redb_attribution_tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_redb_attribution_persistence() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.redb");
        let storage = RedbStorage::new(&db_path).await.unwrap();

        let session = RecommendationSession {
            session_id: Uuid::new_v4(),
            episode_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            recommended_pattern_ids: vec!["p1".to_string()],
            recommended_playbook_ids: vec![],
        };

        storage.store_recommendation_session(&session).await.unwrap();

        let feedback = RecommendationFeedback {
            session_id: session.session_id,
            applied_pattern_ids: vec!["p1".to_string()],
            consulted_episode_ids: vec![],
            outcome: memory_core::TaskOutcome::Success {
                verdict: "ok".to_string(),
                artifacts: vec![],
            },
            agent_rating: None,
        };

        storage.store_recommendation_feedback(&feedback).await.unwrap();

        let retrieved_session = storage.get_recommendation_session(session.session_id).await.unwrap().unwrap();
        assert_eq!(retrieved_session.session_id, session.session_id);

        let retrieved_feedback = storage.get_recommendation_feedback(session.session_id).await.unwrap().unwrap();
        assert_eq!(retrieved_feedback.session_id, session.session_id);
    }
}

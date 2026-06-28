//! StorageBackend trait implementation for CachedTursoStorage
//!
//! Delegates all operations to the underlying cached/storage methods.

use super::wrapper::CachedTursoStorage;
use async_trait::async_trait;
use do_memory_core::memory::attribution::{
    RecommendationFeedback, RecommendationSession, RecommendationStats,
};
use do_memory_core::{
    Episode, Error, Heuristic, Pattern, Result, StorageBackend, episode::PatternId,
};
use uuid::Uuid;

#[async_trait]
impl StorageBackend for CachedTursoStorage {
    async fn store_episode(&self, episode: &Episode) -> Result<()> {
        self.store_episode_cached(episode)
            .await
            .map_err(|e| Error::Storage(format!("Cache store error: {}", e)))
    }

    async fn get_episode(&self, id: Uuid) -> Result<Option<Episode>> {
        self.get_episode_cached(id)
            .await
            .map_err(|e| Error::Storage(format!("Cache get error: {}", e)))
    }

    async fn delete_episode(&self, id: Uuid) -> Result<()> {
        self.delete_episode_cached(id)
            .await
            .map_err(|e| Error::Storage(format!("Cache delete error: {}", e)))
    }

    async fn store_pattern(&self, pattern: &Pattern) -> Result<()> {
        self.store_pattern_cached(pattern)
            .await
            .map_err(|e| Error::Storage(format!("Cache store error: {}", e)))
    }

    async fn get_pattern(&self, id: PatternId) -> Result<Option<Pattern>> {
        self.get_pattern_cached(id)
            .await
            .map_err(|e| Error::Storage(format!("Cache get error: {}", e)))
    }

    async fn store_heuristic(&self, heuristic: &Heuristic) -> Result<()> {
        self.store_heuristic_cached(heuristic)
            .await
            .map_err(|e| Error::Storage(format!("Cache store error: {}", e)))
    }

    async fn get_heuristic(&self, id: Uuid) -> Result<Option<Heuristic>> {
        self.get_heuristic_cached(id)
            .await
            .map_err(|e| Error::Storage(format!("Cache get error: {}", e)))
    }

    async fn query_episodes_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
        limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        self.storage
            .query_episodes_since(since, limit)
            .await
            .map_err(|e| Error::Storage(format!("Query error: {}", e)))
    }

    async fn query_episodes_by_metadata(
        &self,
        key: &str,
        value: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        self.storage
            .query_episodes_by_metadata(key, value, limit)
            .await
            .map_err(|e| Error::Storage(format!("Query error: {}", e)))
    }

    async fn store_embedding(&self, id: &str, embedding: Vec<f32>) -> Result<()> {
        self.storage
            .store_embedding(id, embedding)
            .await
            .map_err(|e| Error::Storage(format!("Store embedding error: {}", e)))
    }

    async fn get_embedding(&self, id: &str) -> Result<Option<Vec<f32>>> {
        self.storage
            .get_embedding(id)
            .await
            .map_err(|e| Error::Storage(format!("Get embedding error: {}", e)))
    }

    async fn delete_embedding(&self, id: &str) -> Result<bool> {
        self.storage
            .delete_embedding(id)
            .await
            .map_err(|e| Error::Storage(format!("Delete embedding error: {}", e)))
    }

    async fn store_embeddings_batch(&self, embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        self.storage
            .store_embeddings_batch(embeddings)
            .await
            .map_err(|e| Error::Storage(format!("Batch store embeddings error: {}", e)))
    }

    async fn get_embeddings_batch(&self, ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        self.storage
            .get_embeddings_batch(ids)
            .await
            .map_err(|e| Error::Storage(format!("Batch get embeddings error: {}", e)))
    }

    async fn store_recommendation_session(&self, session: &RecommendationSession) -> Result<()> {
        self.storage
            .store_recommendation_session(session)
            .await
            .map_err(|e| Error::Storage(format!("Store recommendation session error: {}", e)))
    }

    async fn get_recommendation_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<RecommendationSession>> {
        self.storage
            .get_recommendation_session(session_id)
            .await
            .map_err(|e| Error::Storage(format!("Get recommendation session error: {}", e)))
    }

    async fn get_recommendation_session_for_episode(
        &self,
        episode_id: Uuid,
    ) -> Result<Option<RecommendationSession>> {
        self.storage
            .get_recommendation_session_for_episode(episode_id)
            .await
            .map_err(|e| {
                Error::Storage(format!("Get recommendation session (episode) error: {}", e))
            })
    }

    async fn store_recommendation_feedback(&self, feedback: &RecommendationFeedback) -> Result<()> {
        self.storage
            .store_recommendation_feedback(feedback)
            .await
            .map_err(|e| Error::Storage(format!("Store recommendation feedback error: {}", e)))
    }

    async fn get_recommendation_feedback(
        &self,
        session_id: Uuid,
    ) -> Result<Option<RecommendationFeedback>> {
        self.storage
            .get_recommendation_feedback(session_id)
            .await
            .map_err(|e| Error::Storage(format!("Get recommendation feedback error: {}", e)))
    }

    async fn get_recommendation_stats(&self) -> Result<RecommendationStats> {
        self.storage
            .get_recommendation_stats()
            .await
            .map_err(|e| Error::Storage(format!("Get recommendation stats error: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TursoStorage;
    use crate::cache::config::CacheConfig;
    use crate::cache::wrapper::CachedTursoStorage;
    use do_memory_core::{Evidence, TaskContext, TaskOutcome, TaskType};
    use libsql::Builder;
    use tempfile::TempDir;

    async fn setup() -> (CachedTursoStorage, TempDir) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test_backend.db");
        let db = Builder::new_local(&db_path).build().await.unwrap();
        let storage = TursoStorage::from_database(db).unwrap();
        storage.initialize_schema().await.unwrap();
        let cached = CachedTursoStorage::new(storage, CacheConfig::default());
        (cached, dir)
    }

    fn test_episode() -> Episode {
        Episode {
            episode_id: Uuid::new_v4(),
            task_type: TaskType::CodeGeneration,
            task_description: "backend test".to_string(),
            context: TaskContext {
                domain: "test".to_string(),
                ..Default::default()
            },
            steps: vec![],
            outcome: None,
            reward: None,
            reflection: None,
            patterns: vec![],
            heuristics: vec![],
            applied_patterns: vec![],
            salient_features: None,
            start_time: chrono::Utc::now(),
            end_time: None,
            metadata: std::collections::HashMap::new(),
            tags: vec![],
            checkpoints: vec![],
        }
    }

    #[tokio::test]
    async fn test_storage_backend_episode_lifecycle() {
        let (cached, _dir) = setup().await;
        let ep = test_episode();

        StorageBackend::store_episode(&cached, &ep).await.unwrap();
        let got = StorageBackend::get_episode(&cached, ep.episode_id)
            .await
            .unwrap();
        assert_eq!(got.unwrap().episode_id, ep.episode_id);

        StorageBackend::delete_episode(&cached, ep.episode_id)
            .await
            .unwrap();
        let gone = StorageBackend::get_episode(&cached, ep.episode_id)
            .await
            .unwrap();
        assert!(gone.is_none());
    }

    #[tokio::test]
    async fn test_storage_backend_pattern_and_heuristic() {
        let (cached, _dir) = setup().await;

        let pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["t1".into()],
            context: TaskContext {
                domain: "test".into(),
                ..Default::default()
            },
            success_rate: 0.5,
            avg_latency: chrono::Duration::milliseconds(10),
            occurrence_count: 1,
            effectiveness: Default::default(),
        };
        StorageBackend::store_pattern(&cached, &pattern)
            .await
            .unwrap();
        let got = StorageBackend::get_pattern(&cached, pattern.id())
            .await
            .unwrap();
        assert!(got.is_some());

        let h = Heuristic {
            heuristic_id: Uuid::new_v4(),
            condition: "c".to_string(),
            action: "a".to_string(),
            confidence: 0.9,
            evidence: Evidence {
                episode_ids: vec![],
                success_rate: 0.9,
                sample_size: 1,
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        StorageBackend::store_heuristic(&cached, &h).await.unwrap();
        let got_h = StorageBackend::get_heuristic(&cached, h.heuristic_id)
            .await
            .unwrap();
        assert!(got_h.is_some());
    }

    #[tokio::test]
    async fn test_storage_backend_queries() {
        let (cached, _dir) = setup().await;
        let ep = test_episode();
        StorageBackend::store_episode(&cached, &ep).await.unwrap();

        let since = chrono::Utc::now() - chrono::Duration::hours(1);
        let results = StorageBackend::query_episodes_since(&cached, since, Some(10))
            .await
            .unwrap();
        assert!(!results.is_empty());

        let _ =
            StorageBackend::query_episodes_by_metadata(&cached, "domain", "test", Some(10)).await;
    }

    #[tokio::test]
    async fn test_storage_backend_embeddings() {
        let (cached, _dir) = setup().await;

        let embedding: Vec<f32> = (0..384).map(|i| i as f32 / 384.0).collect();
        StorageBackend::store_embedding(&cached, "e1", embedding)
            .await
            .unwrap();
        let got = StorageBackend::get_embedding(&cached, "e1").await.unwrap();
        assert!(got.is_some());

        let deleted = StorageBackend::delete_embedding(&cached, "e1")
            .await
            .unwrap();
        assert!(deleted);

        let emb1: Vec<f32> = (0..384).map(|i| i as f32 / 384.0).collect();
        let emb2: Vec<f32> = (0..384).map(|i| (384 - i) as f32 / 384.0).collect();
        StorageBackend::store_embeddings_batch(
            &cached,
            vec![("b1".into(), emb1), ("b2".into(), emb2)],
        )
        .await
        .unwrap();

        let batch = StorageBackend::get_embeddings_batch(&cached, &["b1".into(), "b2".into()])
            .await
            .unwrap();
        assert_eq!(batch.len(), 2);
    }

    #[tokio::test]
    async fn test_storage_backend_recommendations() {
        let (cached, _dir) = setup().await;

        let session = RecommendationSession {
            session_id: Uuid::new_v4(),
            episode_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            recommended_pattern_ids: vec!["p1".into()],
            recommended_playbook_ids: vec![],
        };

        StorageBackend::store_recommendation_session(&cached, &session)
            .await
            .unwrap();
        let got = StorageBackend::get_recommendation_session(&cached, session.session_id)
            .await
            .unwrap();
        assert!(got.is_some());

        let by_ep =
            StorageBackend::get_recommendation_session_for_episode(&cached, session.episode_id)
                .await
                .unwrap();
        assert!(by_ep.is_some());

        let feedback = RecommendationFeedback {
            session_id: session.session_id,
            applied_pattern_ids: vec!["p1".into()],
            consulted_episode_ids: vec![],
            outcome: TaskOutcome::Success {
                verdict: "done".into(),
                artifacts: vec![],
            },
            agent_rating: Some(0.9),
        };
        StorageBackend::store_recommendation_feedback(&cached, &feedback)
            .await
            .unwrap();
        let got_fb = StorageBackend::get_recommendation_feedback(&cached, session.session_id)
            .await
            .unwrap();
        assert!(got_fb.is_some());

        let stats = StorageBackend::get_recommendation_stats(&cached)
            .await
            .unwrap();
        assert!(stats.total_sessions >= 1);
    }
}

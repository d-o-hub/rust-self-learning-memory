use super::super::{MonitoringStorageBackend, SimpleMonitoringStorage};
use crate::monitoring::types::{AgentType, ExecutionRecord};
use crate::storage::StorageBackend;
use crate::{Episode, Result};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

struct MockStorage {
    episodes: std::sync::Mutex<Vec<Episode>>,
}

#[async_trait]
impl StorageBackend for MockStorage {
    async fn store_episode(&self, episode: &Episode) -> Result<()> {
        self.episodes.lock().unwrap().push(episode.clone());
        Ok(())
    }
    async fn get_episode(&self, _id: Uuid) -> Result<Option<Episode>> {
        Ok(None)
    }
    async fn delete_episode(&self, _id: Uuid) -> Result<()> {
        Ok(())
    }
    async fn store_pattern(&self, _pattern: &crate::Pattern) -> Result<()> {
        Ok(())
    }
    async fn get_pattern(&self, _id: crate::episode::PatternId) -> Result<Option<crate::Pattern>> {
        Ok(None)
    }
    async fn store_heuristic(&self, _heuristic: &crate::Heuristic) -> Result<()> {
        Ok(())
    }
    async fn get_heuristic(&self, _id: Uuid) -> Result<Option<crate::Heuristic>> {
        Ok(None)
    }
    async fn query_episodes_since(
        &self,
        _since: chrono::DateTime<Utc>,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        Ok(vec![])
    }
    async fn query_episodes_by_metadata(
        &self,
        key: &str,
        value: &str,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        let episodes = self.episodes.lock().unwrap();
        Ok(episodes
            .iter()
            .filter(|e| e.metadata.get(key).map(|v| v == value).unwrap_or(false))
            .cloned()
            .collect())
    }
    async fn store_embedding(&self, _id: &str, _embedding: Vec<f32>) -> Result<()> {
        Ok(())
    }
    async fn get_embedding(&self, _id: &str) -> Result<Option<Vec<f32>>> {
        Ok(None)
    }
    async fn delete_embedding(&self, _id: &str) -> Result<bool> {
        Ok(true)
    }
    async fn store_embeddings_batch(&self, _embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        Ok(())
    }
    async fn get_embeddings_batch(&self, _ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        Ok(vec![])
    }
    async fn store_relationship(&self, _: &crate::episode::EpisodeRelationship) -> Result<()> {
        Ok(())
    }
    async fn remove_relationship(&self, _: Uuid) -> Result<()> {
        Ok(())
    }
    async fn get_relationships(
        &self,
        _: Uuid,
        _: crate::episode::Direction,
    ) -> Result<Vec<crate::episode::EpisodeRelationship>> {
        Ok(vec![])
    }
    async fn relationship_exists(
        &self,
        _: Uuid,
        _: Uuid,
        _: crate::episode::RelationshipType,
    ) -> Result<bool> {
        Ok(false)
    }
    async fn store_recommendation_session(
        &self,
        _: &crate::memory::attribution::RecommendationSession,
    ) -> Result<()> {
        Ok(())
    }
    async fn get_recommendation_session(
        &self,
        _: Uuid,
    ) -> Result<Option<crate::memory::attribution::RecommendationSession>> {
        Ok(None)
    }
    async fn get_recommendation_session_for_episode(
        &self,
        _: Uuid,
    ) -> Result<Option<crate::memory::attribution::RecommendationSession>> {
        Ok(None)
    }
    async fn store_recommendation_feedback(
        &self,
        _: &crate::memory::attribution::RecommendationFeedback,
    ) -> Result<()> {
        Ok(())
    }
    async fn get_recommendation_feedback(
        &self,
        _: Uuid,
    ) -> Result<Option<crate::memory::attribution::RecommendationFeedback>> {
        Ok(None)
    }
    async fn get_recommendation_stats(
        &self,
    ) -> Result<crate::memory::attribution::RecommendationStats> {
        Ok(crate::memory::attribution::RecommendationStats::default())
    }
    async fn cleanup_episodes(
        &self,
        _: &crate::episode::EpisodeRetentionPolicy,
    ) -> Result<crate::episode::CleanupResult> {
        Ok(crate::episode::CleanupResult::new())
    }
    async fn count_cleanup_candidates(
        &self,
        _: &crate::episode::EpisodeRetentionPolicy,
    ) -> Result<usize> {
        Ok(0)
    }
}

#[tokio::test]
async fn test_load_execution_records_sorting() -> Result<()> {
    let storage = Arc::new(MockStorage {
        episodes: std::sync::Mutex::new(vec![]),
    });
    let monitoring = SimpleMonitoringStorage::new(storage);

    let now = Utc::now();
    for i in 0..5 {
        let mut record = ExecutionRecord::new(
            format!("agent-{i}"),
            AgentType::Other,
            true,
            std::time::Duration::from_millis(100),
            None,
            None,
        );
        record.started_at = now + chrono::Duration::minutes(i64::from(i));
        monitoring.store_execution_record(&record).await?;
    }

    let records: Vec<ExecutionRecord> = monitoring.load_execution_records(None, 10).await?;
    assert_eq!(records.len(), 5);
    for i in 0..4 {
        assert!(records[i].started_at >= records[i + 1].started_at);
    }
    Ok(())
}

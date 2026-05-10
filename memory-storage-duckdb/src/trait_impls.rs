use async_trait::async_trait;
use chrono::{DateTime, Utc};
use do_memory_core::{Result, StorageBackend};
use std::sync::Arc;
use uuid::Uuid;

#[async_trait]
impl StorageBackend for crate::DuckDbStorage {
    async fn store_episode(&self, episode: &do_memory_core::Episode) -> Result<()> {
        self.store_episode_internal(episode).await
    }

    async fn get_episode(&self, id: Uuid) -> Result<Option<do_memory_core::Episode>> {
        self.get_episode_internal(id).await
    }

    async fn delete_episode(&self, id: Uuid) -> Result<()> {
        self.delete_episode_internal(id).await
    }

    async fn store_pattern(&self, pattern: &do_memory_core::Pattern) -> Result<()> {
        self.store_pattern_internal(pattern).await
    }

    async fn get_pattern(&self, id: Uuid) -> Result<Option<do_memory_core::Pattern>> {
        self.get_pattern_internal(id).await
    }

    async fn store_heuristic(&self, heuristic: &do_memory_core::Heuristic) -> Result<()> {
        self.store_heuristic_internal(heuristic).await
    }

    async fn get_heuristic(&self, id: Uuid) -> Result<Option<do_memory_core::Heuristic>> {
        self.get_heuristic_internal(id).await
    }

    async fn query_episodes_since(
        &self,
        since: DateTime<Utc>,
        limit: Option<usize>,
    ) -> Result<Vec<do_memory_core::Episode>> {
        self.query_episodes_since_internal(since, limit).await
    }

    async fn query_episodes_by_metadata(
        &self,
        key: &str,
        value: &str,
        limit: Option<usize>,
    ) -> Result<Vec<do_memory_core::Episode>> {
        self.query_episodes_by_metadata_internal(key, value, limit).await
    }

    async fn store_embedding(&self, id: &str, embedding: Vec<f32>) -> Result<()> {
        self.store_embedding_internal(id, embedding).await
    }

    async fn get_embedding(&self, id: &str) -> Result<Option<Vec<f32>>> {
        self.get_embedding_internal(id).await
    }

    async fn delete_embedding(&self, id: &str) -> Result<bool> {
        self.delete_embedding_internal(id).await
    }

    async fn store_embeddings_batch(&self, embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        self.store_embeddings_batch_internal(embeddings).await
    }

    async fn get_embeddings_batch(&self, ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        self.get_embeddings_batch_internal(ids).await
    }

    // ========== Relationship Storage Methods ==========

    async fn store_relationship(
        &self,
        relationship: &do_memory_core::episode::EpisodeRelationship,
    ) -> Result<()> {
        self.store_relationship_internal(relationship).await
    }

    async fn remove_relationship(&self, relationship_id: Uuid) -> Result<()> {
        self.remove_relationship_internal(relationship_id).await
    }

    async fn get_relationships(
        &self,
        episode_id: Uuid,
        direction: do_memory_core::episode::Direction,
    ) -> Result<Vec<do_memory_core::episode::EpisodeRelationship>> {
        self.get_relationships_internal(episode_id, direction).await
    }

    async fn relationship_exists(
        &self,
        from_episode_id: Uuid,
        to_episode_id: Uuid,
        relationship_type: do_memory_core::episode::RelationshipType,
    ) -> Result<bool> {
        self.relationship_exists_internal(from_episode_id, to_episode_id, relationship_type)
            .await
    }

    // ========== Recommendation Attribution ==========

    async fn store_recommendation_session(
        &self,
        session: &do_memory_core::memory::attribution::RecommendationSession,
    ) -> Result<()> {
        self.store_recommendation_session_internal(session).await
    }

    async fn get_recommendation_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<do_memory_core::memory::attribution::RecommendationSession>> {
        self.get_recommendation_session_internal(session_id).await
    }

    async fn get_recommendation_session_for_episode(
        &self,
        episode_id: Uuid,
    ) -> Result<Option<do_memory_core::memory::attribution::RecommendationSession>> {
        self.get_recommendation_session_for_episode_internal(episode_id).await
    }

    async fn store_recommendation_feedback(
        &self,
        feedback: &do_memory_core::memory::attribution::RecommendationFeedback,
    ) -> Result<()> {
        self.store_recommendation_feedback_internal(feedback).await
    }

    async fn get_recommendation_feedback(
        &self,
        session_id: Uuid,
    ) -> Result<Option<do_memory_core::memory::attribution::RecommendationFeedback>> {
        self.get_recommendation_feedback_internal(session_id).await
    }

    fn set_event_emitter(&self, emitter: Arc<dyn do_memory_core::types::event::EventEmitter>) {
        let mut lock = self.event_emitter.write();
        *lock = Some(emitter);
    }
}

#[async_trait]
impl do_memory_core::monitoring::storage::MonitoringStorageBackend for crate::DuckDbStorage {
    async fn store_execution_record(
        &self,
        record: &do_memory_core::monitoring::types::ExecutionRecord,
    ) -> Result<()> {
        self.store_execution_record_internal(record).await
    }

    async fn store_agent_metrics(
        &self,
        metrics: &do_memory_core::monitoring::types::AgentMetrics,
    ) -> Result<()> {
        self.store_agent_metrics_internal(metrics).await
    }

    async fn store_task_metrics(
        &self,
        metrics: &do_memory_core::monitoring::types::TaskMetrics,
    ) -> Result<()> {
        self.store_task_metrics_internal(metrics).await
    }

    async fn load_agent_metrics(
        &self,
        agent_name: &str,
    ) -> Result<Option<do_memory_core::monitoring::types::AgentMetrics>> {
        self.load_agent_metrics_internal(agent_name).await
    }

    async fn load_execution_records(
        &self,
        agent_name: Option<&str>,
        limit: usize,
    ) -> Result<Vec<do_memory_core::monitoring::types::ExecutionRecord>> {
        self.load_execution_records_internal(agent_name, limit).await
    }

    async fn load_task_metrics(
        &self,
        task_type: &str,
    ) -> Result<Option<do_memory_core::monitoring::types::TaskMetrics>> {
        self.load_task_metrics_internal(task_type).await
    }
}

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
        _since: DateTime<Utc>,
        _limit: Option<usize>,
    ) -> Result<Vec<do_memory_core::Episode>> {
        // Implementation for analytical queries would go here
        Ok(Vec::new())
    }

    async fn query_episodes_by_metadata(
        &self,
        _key: &str,
        _value: &str,
        _limit: Option<usize>,
    ) -> Result<Vec<do_memory_core::Episode>> {
        Ok(Vec::new())
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
        Ok(Vec::new())
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
        _record: &do_memory_core::monitoring::types::ExecutionRecord,
    ) -> Result<()> {
        Ok(())
    }

    async fn store_agent_metrics(
        &self,
        _metrics: &do_memory_core::monitoring::types::AgentMetrics,
    ) -> Result<()> {
        Ok(())
    }

    async fn store_task_metrics(
        &self,
        _metrics: &do_memory_core::monitoring::types::TaskMetrics,
    ) -> Result<()> {
        Ok(())
    }

    async fn load_agent_metrics(
        &self,
        _agent_name: &str,
    ) -> Result<Option<do_memory_core::monitoring::types::AgentMetrics>> {
        Ok(None)
    }

    async fn load_execution_records(
        &self,
        _agent_name: Option<&str>,
        _limit: usize,
    ) -> Result<Vec<do_memory_core::monitoring::types::ExecutionRecord>> {
        Ok(Vec::new())
    }

    async fn load_task_metrics(
        &self,
        _task_type: &str,
    ) -> Result<Option<do_memory_core::monitoring::types::TaskMetrics>> {
        Ok(None)
    }
}

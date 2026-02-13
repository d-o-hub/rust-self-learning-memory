//! Trait implementations for TursoStorage.
//!
//! This module contains the StorageBackend and MonitoringStorageBackend
//! trait implementations to keep lib.rs under 500 LOC.

use async_trait::async_trait;
use memory_core::{Error, Result, StorageBackend};

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStatistics {
    pub episode_count: usize,
    pub pattern_count: usize,
    pub heuristic_count: usize,
}

/// Implement the unified StorageBackend trait for TursoStorage
#[async_trait]
impl StorageBackend for super::TursoStorage {
    async fn store_episode(&self, episode: &memory_core::Episode) -> Result<()> {
        super::TursoStorage::store_episode(self, episode).await
    }

    async fn get_episode(&self, id: uuid::Uuid) -> Result<Option<memory_core::Episode>> {
        super::TursoStorage::get_episode(self, id).await
    }

    async fn delete_episode(&self, id: uuid::Uuid) -> Result<()> {
        super::TursoStorage::delete_episode(self, id).await
    }

    async fn store_pattern(&self, pattern: &memory_core::Pattern) -> Result<()> {
        super::TursoStorage::store_pattern(self, pattern).await
    }

    async fn get_pattern(
        &self,
        id: memory_core::episode::PatternId,
    ) -> Result<Option<memory_core::Pattern>> {
        super::TursoStorage::get_pattern(self, id).await
    }

    async fn store_heuristic(&self, heuristic: &memory_core::Heuristic) -> Result<()> {
        super::TursoStorage::store_heuristic(self, heuristic).await
    }

    async fn get_heuristic(&self, id: uuid::Uuid) -> Result<Option<memory_core::Heuristic>> {
        super::TursoStorage::get_heuristic(self, id).await
    }

    async fn query_episodes_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
        limit: Option<usize>,
    ) -> Result<Vec<memory_core::Episode>> {
        super::TursoStorage::query_episodes_since(self, since, limit).await
    }

    async fn query_episodes_by_metadata(
        &self,
        key: &str,
        value: &str,
        limit: Option<usize>,
    ) -> Result<Vec<memory_core::Episode>> {
        super::TursoStorage::query_episodes_by_metadata(self, key, value, limit).await
    }

    async fn store_embedding(&self, id: &str, embedding: Vec<f32>) -> Result<()> {
        super::TursoStorage::store_embedding_backend(self, id, embedding).await
    }

    async fn get_embedding(&self, id: &str) -> Result<Option<Vec<f32>>> {
        super::TursoStorage::get_embedding_backend(self, id).await
    }

    async fn delete_embedding(&self, id: &str) -> Result<bool> {
        super::TursoStorage::delete_embedding_backend(self, id).await
    }

    async fn store_embeddings_batch(&self, embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        super::TursoStorage::store_embeddings_batch_backend(self, embeddings).await
    }

    async fn get_embeddings_batch(&self, ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        super::TursoStorage::get_embeddings_batch_backend(self, ids).await
    }

    // ========== Relationship Storage Methods ==========

    async fn store_relationship(
        &self,
        relationship: &memory_core::episode::EpisodeRelationship,
    ) -> Result<()> {
        super::TursoStorage::store_relationship(self, relationship).await
    }

    async fn remove_relationship(&self, relationship_id: uuid::Uuid) -> Result<()> {
        super::TursoStorage::remove_relationship(self, relationship_id).await
    }

    async fn get_relationships(
        &self,
        episode_id: uuid::Uuid,
        direction: memory_core::episode::Direction,
    ) -> Result<Vec<memory_core::episode::EpisodeRelationship>> {
        super::TursoStorage::get_relationships(self, episode_id, direction).await
    }

    async fn relationship_exists(
        &self,
        from_episode_id: uuid::Uuid,
        to_episode_id: uuid::Uuid,
        relationship_type: memory_core::episode::RelationshipType,
    ) -> Result<bool> {
        super::TursoStorage::relationship_exists(
            self,
            from_episode_id,
            to_episode_id,
            relationship_type,
        )
        .await
    }
}

/// Implement the MonitoringStorageBackend trait for TursoStorage
#[async_trait]
impl memory_core::monitoring::storage::MonitoringStorageBackend for super::TursoStorage {
    async fn store_execution_record(
        &self,
        record: &memory_core::monitoring::types::ExecutionRecord,
    ) -> Result<()> {
        super::TursoStorage::store_execution_record(self, record)
            .await
            .map_err(|e| Error::Storage(format!("Storage error: {}", e)))
    }

    async fn store_agent_metrics(
        &self,
        metrics: &memory_core::monitoring::types::AgentMetrics,
    ) -> Result<()> {
        super::TursoStorage::store_agent_metrics(self, metrics)
            .await
            .map_err(|e| Error::Storage(format!("Storage error: {}", e)))
    }

    async fn store_task_metrics(
        &self,
        metrics: &memory_core::monitoring::types::TaskMetrics,
    ) -> Result<()> {
        super::TursoStorage::store_task_metrics(self, metrics)
            .await
            .map_err(|e| Error::Storage(format!("Storage error: {}", e)))
    }

    async fn load_agent_metrics(
        &self,
        agent_name: &str,
    ) -> Result<Option<memory_core::monitoring::types::AgentMetrics>> {
        super::TursoStorage::load_agent_metrics(self, agent_name)
            .await
            .map_err(|e| Error::Storage(format!("Storage error: {}", e)))
    }

    async fn load_execution_records(
        &self,
        agent_name: Option<&str>,
        limit: usize,
    ) -> Result<Vec<memory_core::monitoring::types::ExecutionRecord>> {
        super::TursoStorage::load_execution_records(self, agent_name, limit)
            .await
            .map_err(|e| Error::Storage(format!("Storage error: {}", e)))
    }

    async fn load_task_metrics(
        &self,
        task_type: &str,
    ) -> Result<Option<memory_core::monitoring::types::TaskMetrics>> {
        super::TursoStorage::load_task_metrics(self, task_type)
            .await
            .map_err(|e| Error::Storage(format!("Storage error: {}", e)))
    }
}

//! Tests for the checkpoint projection query (`query.rs`, HY-03).

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use super::CheckpointMeta;
use crate::error::Error;
use crate::memory::SelfLearningMemory;
use crate::storage::{StorageBackend, StorageBackendCapabilities};
use crate::types::{MemoryConfig, TaskContext, TaskType};
use crate::{Episode, Heuristic, Pattern, PatternId, Result};

/// In-memory backend serving a preloaded checkpoint projection and/or whole
/// episodes, with call counters so the tests can prove which tier answered.
///
/// The `Mutex` guards are only held for short synchronous map reads; no lock is
/// ever held across an `.await`.
struct ProjectionBackend {
    supports_projection: bool,
    projection: Mutex<HashMap<Uuid, Vec<CheckpointMeta>>>,
    episodes: Mutex<HashMap<Uuid, Episode>>,
    projection_calls: AtomicUsize,
    episode_reads: AtomicUsize,
}

impl ProjectionBackend {
    fn new(supports_projection: bool) -> Self {
        Self {
            supports_projection,
            projection: Mutex::new(HashMap::new()),
            episodes: Mutex::new(HashMap::new()),
            projection_calls: AtomicUsize::new(0),
            episode_reads: AtomicUsize::new(0),
        }
    }

    fn assert_projection_called(&self, expected: usize) {
        assert_eq!(self.projection_calls.load(Ordering::SeqCst), expected);
    }

    fn assert_episode_reads(&self, expected: usize) {
        assert_eq!(self.episode_reads.load(Ordering::SeqCst), expected);
    }
}

#[async_trait]
impl StorageBackendCapabilities for ProjectionBackend {
    fn supports_checkpoints_query(&self) -> bool {
        self.supports_projection
    }
}

#[async_trait]
impl StorageBackend for ProjectionBackend {
    async fn store_episode(&self, _episode: &Episode) -> Result<()> {
        Ok(())
    }
    async fn get_episode(&self, id: Uuid) -> Result<Option<Episode>> {
        self.episode_reads.fetch_add(1, Ordering::SeqCst);
        Ok(self.episodes.lock().unwrap().get(&id).cloned())
    }
    async fn delete_episode(&self, _id: Uuid) -> Result<()> {
        Ok(())
    }
    async fn store_pattern(&self, _pattern: &Pattern) -> Result<()> {
        Ok(())
    }
    async fn get_pattern(&self, _id: PatternId) -> Result<Option<Pattern>> {
        Ok(None)
    }
    async fn store_heuristic(&self, _heuristic: &Heuristic) -> Result<()> {
        Ok(())
    }
    async fn get_heuristic(&self, _id: Uuid) -> Result<Option<Heuristic>> {
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
        _key: &str,
        _value: &str,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        Ok(vec![])
    }
    async fn store_embedding(&self, _id: &str, _embedding: Vec<f32>) -> Result<()> {
        Ok(())
    }
    async fn get_embedding(&self, _id: &str) -> Result<Option<Vec<f32>>> {
        Ok(None)
    }
    async fn delete_embedding(&self, _id: &str) -> Result<bool> {
        Ok(false)
    }
    async fn store_embeddings_batch(&self, _embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        Ok(())
    }
    async fn get_embeddings_batch(&self, _ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        Ok(vec![])
    }
    async fn get_checkpoints(&self, episode_id: Uuid) -> Result<Option<Vec<CheckpointMeta>>> {
        self.projection_calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.projection.lock().unwrap().get(&episode_id).cloned())
    }
}

fn memory_with(backend: ProjectionBackend) -> (SelfLearningMemory, Arc<ProjectionBackend>) {
    let concrete = Arc::new(backend);
    let durable: Arc<dyn StorageBackend> = concrete.clone();
    let cache: Arc<dyn StorageBackend> = concrete.clone();
    let memory = SelfLearningMemory::with_storage(MemoryConfig::default(), durable, cache);
    (memory, concrete)
}

fn episode_with_checkpoint(episode_id: Uuid, label: &str) -> Episode {
    let mut episode = Episode::new(
        "test".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );
    episode.episode_id = episode_id;
    episode.checkpoints.push(CheckpointMeta::new(
        episode_id,
        label.to_string(),
        1,
        None,
        None,
    ));
    episode
}

#[tokio::test]
async fn get_checkpoints_projects_from_fallback_without_storage() {
    // Arrange: fallback-only memory with one real checkpoint.
    let memory = SelfLearningMemory::new();
    let episode_id = memory
        .start_episode(
            "projection test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;
    let created =
        crate::memory::checkpoint::checkpoint_episode(&memory, episode_id, "pause".to_string())
            .await
            .expect("checkpoint creation");

    // Act
    let projected = memory.get_checkpoints(episode_id).await;

    // Assert
    let projected = projected.expect("fallback projection");
    assert_eq!(projected, vec![created]);
}

#[tokio::test]
async fn get_checkpoints_uses_durable_projection_when_supported() {
    // Arrange: backend advertises the projection and preloads one checkpoint.
    let episode_id = Uuid::new_v4();
    let meta = CheckpointMeta::new(episode_id, "durable".to_string(), 2, None, None);
    let backend = ProjectionBackend::new(true);
    backend
        .projection
        .lock()
        .unwrap()
        .insert(episode_id, vec![meta.clone()]);
    let (memory, backend) = memory_with(backend);

    // Act
    let projected = memory.get_checkpoints(episode_id).await;

    // Assert
    assert_eq!(projected.expect("durable projection"), vec![meta]);
    backend.assert_projection_called(1);
    backend.assert_episode_reads(0);
}

#[tokio::test]
async fn get_checkpoints_falls_back_to_full_read_when_capability_off() {
    // Arrange: backend holds the episode but does not advertise the projection.
    let episode_id = Uuid::new_v4();
    let backend = ProjectionBackend::new(false);
    backend
        .episodes
        .lock()
        .unwrap()
        .insert(episode_id, episode_with_checkpoint(episode_id, "stored"));
    let (memory, backend) = memory_with(backend);

    // Act
    let projected = memory.get_checkpoints(episode_id).await;

    // Assert
    assert_eq!(
        projected.expect("full-read fallback").len(),
        1,
        "full-episode read must still serve the list"
    );
    backend.assert_projection_called(0);
    backend.assert_episode_reads(1);
}

#[tokio::test]
async fn get_checkpoints_falls_through_when_projection_misses() {
    // Arrange: projection answers `None` while the backend holds the episode
    // (the hybrid cache-tier parity case).
    let episode_id = Uuid::new_v4();
    let backend = ProjectionBackend::new(true);
    backend
        .episodes
        .lock()
        .unwrap()
        .insert(episode_id, episode_with_checkpoint(episode_id, "stored"));
    let (memory, backend) = memory_with(backend);

    // Act
    let projected = memory.get_checkpoints(episode_id).await;

    // Assert
    assert_eq!(projected.expect("projection miss fallthrough").len(), 1);
    backend.assert_projection_called(1);
    backend.assert_episode_reads(1);
}

#[tokio::test]
async fn get_checkpoints_returns_not_found_when_missing_everywhere() {
    // Arrange: nothing holds the episode.
    let (memory, backend) = memory_with(ProjectionBackend::new(true));

    // Act
    let projected = memory.get_checkpoints(Uuid::new_v4()).await;

    // Assert
    match projected {
        Err(Error::NotFound(_)) => {}
        other => panic!("expected Error::NotFound, got {other:?}"),
    }
    backend.assert_projection_called(1);
    // The full read consults the same mock as cache and durable tier (2 reads),
    // both miss, and `get_episode` surfaces `Error::NotFound`.
    backend.assert_episode_reads(2);
}

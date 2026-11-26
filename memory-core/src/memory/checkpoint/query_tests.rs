//! Tests for the checkpoint projection query (`query.rs`, HY-03).

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use uuid::Uuid;

use super::CheckpointMeta;
use crate::Episode;
use crate::error::Error;
use crate::memory::SelfLearningMemory;
use crate::storage::TestStubBackend as StubBackend;
use crate::storage::{StorageBackend, StorageBackendCapabilities};
use crate::types::{MemoryConfig, TaskContext, TaskType};

/// Focused backend for the projection: the crate-shared [`StubBackend`]
/// satisfies the required no-op contract, and this wrapper adds only the
/// behavior under test — the projection map, an episode map, and call
/// counters.
///
/// The `Mutex` guards are only held for short synchronous map reads; no lock is
/// ever held across an `.await`.
struct ProjectionBackend {
    noop: StubBackend,
    supports_projection: bool,
    projection: Mutex<HashMap<Uuid, Vec<CheckpointMeta>>>,
    episodes: Mutex<HashMap<Uuid, Episode>>,
    projection_calls: AtomicUsize,
    episode_reads: AtomicUsize,
}

impl ProjectionBackend {
    fn new(supports_projection: bool) -> Self {
        Self {
            noop: StubBackend,
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
    async fn store_episode(&self, episode: &Episode) -> crate::Result<()> {
        self.noop.store_episode(episode).await
    }
    async fn get_episode(&self, id: Uuid) -> crate::Result<Option<Episode>> {
        self.episode_reads.fetch_add(1, Ordering::SeqCst);
        Ok(self.episodes.lock().unwrap().get(&id).cloned())
    }
    async fn delete_episode(&self, id: Uuid) -> crate::Result<()> {
        self.noop.delete_episode(id).await
    }
    async fn store_pattern(&self, pattern: &crate::Pattern) -> crate::Result<()> {
        self.noop.store_pattern(pattern).await
    }
    async fn get_pattern(&self, id: crate::PatternId) -> crate::Result<Option<crate::Pattern>> {
        self.noop.get_pattern(id).await
    }
    async fn store_heuristic(&self, heuristic: &crate::Heuristic) -> crate::Result<()> {
        self.noop.store_heuristic(heuristic).await
    }
    async fn get_heuristic(&self, id: Uuid) -> crate::Result<Option<crate::Heuristic>> {
        self.noop.get_heuristic(id).await
    }
    async fn query_episodes_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
        limit: Option<usize>,
    ) -> crate::Result<Vec<Episode>> {
        self.noop.query_episodes_since(since, limit).await
    }
    async fn query_episodes_by_metadata(
        &self,
        key: &str,
        value: &str,
        limit: Option<usize>,
    ) -> crate::Result<Vec<Episode>> {
        self.noop
            .query_episodes_by_metadata(key, value, limit)
            .await
    }
    async fn store_embedding(&self, id: &str, embedding: Vec<f32>) -> crate::Result<()> {
        self.noop.store_embedding(id, embedding).await
    }
    async fn get_embedding(&self, id: &str) -> crate::Result<Option<Vec<f32>>> {
        self.noop.get_embedding(id).await
    }
    async fn delete_embedding(&self, id: &str) -> crate::Result<bool> {
        self.noop.delete_embedding(id).await
    }
    async fn store_embeddings_batch(
        &self,
        embeddings: Vec<(String, Vec<f32>)>,
    ) -> crate::Result<()> {
        self.noop.store_embeddings_batch(embeddings).await
    }
    async fn get_embeddings_batch(&self, ids: &[String]) -> crate::Result<Vec<Option<Vec<f32>>>> {
        self.noop.get_embeddings_batch(ids).await
    }
    async fn get_checkpoints(
        &self,
        episode_id: Uuid,
    ) -> crate::Result<Option<Vec<CheckpointMeta>>> {
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

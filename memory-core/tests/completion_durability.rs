//! ADR-075: `complete_episode` must hard-fail when a configured backend cannot store.

#![allow(clippy::unwrap_used, clippy::panic)] // test-only mock backends

use async_trait::async_trait;
use chrono::Utc;
use do_memory_core::episode::PatternId;
use do_memory_core::storage::StorageBackend;
use do_memory_core::{
    Episode, Error, Heuristic, MemoryConfig, Pattern, Result, SelfLearningMemory, TaskContext,
    TaskOutcome, TaskType,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use uuid::Uuid;

/// Backend that always fails `store_episode`.
struct FailingStoreBackend {
    label: &'static str,
    store_calls: AtomicUsize,
}

impl FailingStoreBackend {
    fn new(label: &'static str) -> Self {
        Self {
            label,
            store_calls: AtomicUsize::new(0),
        }
    }
}

/// Backend that always succeeds `store_episode`.
struct OkStoreBackend {
    store_calls: AtomicUsize,
    last_complete: AtomicBool,
}

impl OkStoreBackend {
    fn new() -> Self {
        Self {
            store_calls: AtomicUsize::new(0),
            last_complete: AtomicBool::new(false),
        }
    }
}

#[async_trait]
impl StorageBackend for FailingStoreBackend {
    async fn store_episode(&self, _episode: &Episode) -> Result<()> {
        self.store_calls.fetch_add(1, Ordering::SeqCst);
        Err(Error::Storage(format!("{} store_episode failed", self.label)))
    }
    async fn get_episode(&self, _id: Uuid) -> Result<Option<Episode>> {
        Ok(None)
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
        Ok(true)
    }
    async fn store_embeddings_batch(&self, _embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        Ok(())
    }
    async fn get_embeddings_batch(&self, _ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        Ok(vec![])
    }
}

#[async_trait]
impl StorageBackend for OkStoreBackend {
    async fn store_episode(&self, episode: &Episode) -> Result<()> {
        self.store_calls.fetch_add(1, Ordering::SeqCst);
        self.last_complete
            .store(episode.is_complete(), Ordering::SeqCst);
        Ok(())
    }
    async fn get_episode(&self, _id: Uuid) -> Result<Option<Episode>> {
        Ok(None)
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
        Ok(true)
    }
    async fn store_embeddings_batch(&self, _embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        Ok(())
    }
    async fn get_embeddings_batch(&self, _ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        Ok(vec![])
    }
}

fn test_config() -> MemoryConfig {
    MemoryConfig {
        quality_threshold: 0.0, // allow empty-step episodes (CLI operator path)
        pattern_extraction_threshold: 1.0,
        enable_summarization: false,
        enable_embeddings: false,
        batch_config: None,
        ..MemoryConfig::default()
    }
}

fn failure_outcome() -> TaskOutcome {
    TaskOutcome::Failure {
        reason: "operator fail".into(),
        error_details: None,
    }
}

#[tokio::test]
async fn complete_episode_errs_when_cache_store_fails() {
    let cache = Arc::new(FailingStoreBackend::new("cache"));
    let turso = Arc::new(OkStoreBackend::new());

    let memory = SelfLearningMemory::with_storage(
        test_config(),
        Arc::clone(&turso) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );

    let episode_id = memory
        .start_episode("stuck bot".into(), TaskContext::default(), TaskType::Testing)
        .await;

    // Isolate complete_episode store attempts from start_episode writes.
    cache.store_calls.store(0, Ordering::SeqCst);
    turso.store_calls.store(0, Ordering::SeqCst);

    let result = memory
        .complete_episode(episode_id, failure_outcome())
        .await;

    assert!(result.is_err(), "must not return Ok after cache store failure");
    let err = result.unwrap_err();
    match err {
        Error::Storage(msg) => {
            assert!(
                msg.contains("cache"),
                "error should name cache backend: {msg}"
            );
        }
        other => panic!("expected Error::Storage, got {other:?}"),
    }

    // Both configured backends must be attempted so the error can list all failures.
    assert_eq!(cache.store_calls.load(Ordering::SeqCst), 1);
    assert_eq!(turso.store_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn complete_episode_errs_when_turso_store_fails() {
    let cache = Arc::new(OkStoreBackend::new());
    let turso = Arc::new(FailingStoreBackend::new("turso"));

    let memory = SelfLearningMemory::with_storage(
        test_config(),
        Arc::clone(&turso) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );

    let episode_id = memory
        .start_episode(
            "durable fail".into(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    cache.store_calls.store(0, Ordering::SeqCst);
    turso.store_calls.store(0, Ordering::SeqCst);

    let result = memory
        .complete_episode(episode_id, failure_outcome())
        .await;

    assert!(result.is_err(), "must not return Ok after turso store failure");
    let err = result.unwrap_err();
    match err {
        Error::Storage(msg) => {
            assert!(
                msg.contains("turso"),
                "error should name turso backend: {msg}"
            );
        }
        other => panic!("expected Error::Storage, got {other:?}"),
    }

    assert_eq!(cache.store_calls.load(Ordering::SeqCst), 1);
    assert_eq!(turso.store_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn complete_episode_errs_listing_all_failed_backends() {
    let cache = Arc::new(FailingStoreBackend::new("cache"));
    let turso = Arc::new(FailingStoreBackend::new("turso"));

    let memory = SelfLearningMemory::with_storage(
        test_config(),
        Arc::clone(&turso) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );

    let episode_id = memory
        .start_episode("both fail".into(), TaskContext::default(), TaskType::Testing)
        .await;

    let result = memory
        .complete_episode(episode_id, failure_outcome())
        .await;

    let err = result.expect_err("both backends failing must hard-fail complete");
    match err {
        Error::Storage(msg) => {
            assert!(msg.contains("cache"), "must list cache: {msg}");
            assert!(msg.contains("turso"), "must list turso: {msg}");
        }
        other => panic!("expected Error::Storage, got {other:?}"),
    }
}

#[tokio::test]
async fn complete_episode_ok_when_backends_store_successfully() {
    let cache = Arc::new(OkStoreBackend::new());
    let turso = Arc::new(OkStoreBackend::new());

    let memory = SelfLearningMemory::with_storage(
        test_config(),
        Arc::clone(&turso) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );

    let episode_id = memory
        .start_episode(
            "happy path".into(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    cache.store_calls.store(0, Ordering::SeqCst);
    turso.store_calls.store(0, Ordering::SeqCst);
    cache.last_complete.store(false, Ordering::SeqCst);
    turso.last_complete.store(false, Ordering::SeqCst);

    memory
        .complete_episode(episode_id, failure_outcome())
        .await
        .expect("configured backends succeeded");

    // complete_episode stores once; learning may re-store the episode afterward.
    assert!(cache.store_calls.load(Ordering::SeqCst) >= 1);
    assert!(turso.store_calls.load(Ordering::SeqCst) >= 1);
    assert!(cache.last_complete.load(Ordering::SeqCst));
    assert!(turso.last_complete.load(Ordering::SeqCst));

    let episode = memory.get_episode(episode_id).await.expect("in-memory");
    assert!(episode.is_complete());
}

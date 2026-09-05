//! Issue #967: bounded background durable writes off the completion path.
//!
//! Contract under test (D1 split): local cache writes stay synchronous and
//! hard-error; Turso writes move through the opt-in bounded queue; queue
//! backpressure is an explicit error; retries are idempotent; `flush` gives
//! operators the remote durability guarantee on demand.

#![allow(clippy::unwrap_used, clippy::panic)] // test-only mock backend

use async_trait::async_trait;
use chrono::Utc;
use do_memory_core::episode::PatternId;
use do_memory_core::storage::StorageBackend;
use do_memory_core::{
    Episode, Error, Heuristic, MemoryConfig, Pattern, Result, SelfLearningMemory, TaskContext,
    TaskOutcome, TaskType, WriteQueueConfig,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use uuid::Uuid;

/// Configurable mock backend: counting store with optional failure/sleep.
struct MockBackend {
    store_calls: AtomicUsize,
    batch_calls: AtomicUsize,
    /// Fail the next N batch commits, then succeed.
    fail_first_n_batches: AtomicUsize,
    always_fail: AtomicBool,
    store_sleep_ms: u64,
    stored: Mutex<HashMap<Uuid, Episode>>,
}

impl MockBackend {
    fn new() -> Self {
        Self {
            store_calls: AtomicUsize::new(0),
            batch_calls: AtomicUsize::new(0),
            fail_first_n_batches: AtomicUsize::new(0),
            always_fail: AtomicBool::new(false),
            store_sleep_ms: 0,
            stored: Mutex::new(HashMap::new()),
        }
    }

    fn failing() -> Self {
        let backend = Self::new();
        backend.always_fail.store(true, Ordering::SeqCst);
        backend
    }

    fn slow(store_sleep_ms: u64) -> Self {
        let mut backend = Self::new();
        backend.store_sleep_ms = store_sleep_ms;
        backend
    }

    async fn stored_count(&self) -> usize {
        self.stored.lock().await.len()
    }
}

#[async_trait]
impl StorageBackend for MockBackend {
    async fn store_episode(&self, episode: &Episode) -> Result<()> {
        self.store_calls.fetch_add(1, Ordering::SeqCst);
        if self.store_sleep_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.store_sleep_ms)).await;
        }
        if self.always_fail.load(Ordering::SeqCst) {
            return Err(Error::Storage("mock store_episode failed".into()));
        }
        self.stored
            .lock()
            .await
            .insert(episode.episode_id, episode.clone());
        Ok(())
    }

    async fn store_episodes_batch(&self, episodes: &[Episode]) -> Result<()> {
        self.batch_calls.fetch_add(1, Ordering::SeqCst);
        if self.always_fail.load(Ordering::SeqCst) {
            return Err(Error::Storage("mock batch failed".into()));
        }
        if self.fail_first_n_batches.fetch_sub(1, Ordering::SeqCst) > 0 {
            return Err(Error::Storage("mock flaky batch failed".into()));
        }
        for episode in episodes {
            self.store_episode(episode).await?;
        }
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

/// Test config: no quality gate, no summarization, queue enabled by default.
fn test_config() -> MemoryConfig {
    MemoryConfig {
        quality_threshold: 0.0,
        pattern_extraction_threshold: 1.0,
        enable_summarization: false,
        enable_embeddings: false,
        batch_config: None,
        durable_write_queue: Some(WriteQueueConfig {
            batch_size: 10,
            poll_interval_ms: 10,
            retry_base_delay_ms: 10,
            retry_max_delay_ms: 50,
            ..WriteQueueConfig::default()
        }),
        ..MemoryConfig::default()
    }
}

fn success_outcome() -> TaskOutcome {
    TaskOutcome::Success {
        verdict: "done".into(),
        artifacts: vec![],
    }
}

async fn start_episode(memory: &SelfLearningMemory, task: &str) -> Uuid {
    memory
        .start_episode(task.into(), TaskContext::default(), TaskType::Testing)
        .await
}

#[tokio::test]
async fn completion_returns_before_slow_remote_persists() {
    // Arrange: Turso takes 2s per write; queue enabled with workers running.
    let cache = Arc::new(MockBackend::new());
    let turso = Arc::new(MockBackend::slow(2000));
    let memory = SelfLearningMemory::with_storage(
        test_config(),
        Arc::clone(&turso) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );
    memory.start_durable_workers();
    let episode_id = start_episode(&memory, "slow remote").await;

    // Act: complete and time only the completion call.
    let start = Instant::now();
    memory
        .complete_episode(episode_id, success_outcome())
        .await
        .expect("queued completion must succeed");
    let elapsed = start.elapsed();

    // Assert: completion excluded the 2s remote write...
    assert!(
        elapsed < Duration::from_millis(1500),
        "completion took {elapsed:?}, remote work leaked onto the path"
    );
    // ...but local read-after-write holds immediately...
    assert!(memory.get_episode(episode_id).await.is_ok());
    // NOTE: no assertion on the remote store here — the background worker
    // may legitimately persist before we look (that is the point of the
    // queue); only completion latency is asserted above.
    // ...and the background worker persists it; flush gives the guarantee.
    memory
        .flush_durable_writes(Duration::from_secs(10))
        .await
        .expect("flush must succeed");
    assert_eq!(turso.stored_count().await, 1);
}

#[tokio::test]
async fn flush_surfaces_permanent_remote_failure_but_keeps_local_state() {
    // Arrange: Turso always fails; no retries so parking is immediate.
    let cache = Arc::new(MockBackend::new());
    let turso = Arc::new(MockBackend::failing());
    let mut config = test_config();
    config.durable_write_queue = Some(WriteQueueConfig {
        max_retries: 0,
        batch_size: 10,
        poll_interval_ms: 10,
        ..WriteQueueConfig::default()
    });
    let memory = SelfLearningMemory::with_storage(
        config,
        Arc::clone(&turso) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );
    // Complete before workers start: the seam write and the sync
    // pattern-path re-store coalesce to a single queued entry, so exactly
    // one permanent failure is recorded deterministically.
    let episode_id = start_episode(&memory, "doomed remote").await;

    // Act: completion succeeds (queued); flush reports the parked failure.
    memory
        .complete_episode(episode_id, success_outcome())
        .await
        .expect("queued completion must succeed");
    memory.start_durable_workers();
    let flush_result = memory.flush_durable_writes(Duration::from_secs(10)).await;

    // Assert: flush names the permanent failure...
    let err = flush_result.expect_err("flush must surface parked failures");
    match err {
        Error::Storage(msg) => assert!(
            msg.contains("permanent failure"),
            "unexpected message: {msg}"
        ),
        other => panic!("expected Error::Storage, got {other:?}"),
    }
    // ...the failure is observable in stats...
    let stats = memory
        .durable_write_stats()
        .await
        .expect("stats must exist");
    assert_eq!(stats.total_failed, 1);
    assert_eq!(stats.failed_episode_ids, vec![episode_id]);
    // ...and recorded in the operation journal for operators...
    let pending = memory.operation_journal_pending().await;
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].episode_id, episode_id);
    assert_eq!(
        pending[0].kind,
        do_memory_core::JournalOpKind::EpisodeComplete
    );
    // ...and local read-after-write still holds.
    assert!(memory.get_episode(episode_id).await.is_ok());
}

#[tokio::test]
async fn retry_recovers_without_duplicate_episodes() {
    // Arrange: first two batch commits fail, then the backend recovers.
    let cache = Arc::new(MockBackend::new());
    let turso = Arc::new(MockBackend::new());
    turso.fail_first_n_batches.store(2, Ordering::SeqCst);
    let memory = SelfLearningMemory::with_storage(
        test_config(),
        Arc::clone(&turso) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );
    memory.start_durable_workers();
    let episode_id = start_episode(&memory, "flaky remote").await;

    // Act.
    memory
        .complete_episode(episode_id, success_outcome())
        .await
        .expect("queued completion must succeed");
    memory
        .flush_durable_writes(Duration::from_secs(10))
        .await
        .expect("flush must succeed after recovery");

    // Assert: retried batches are idempotent (INSERT OR REPLACE semantics).
    assert!(turso.batch_calls.load(Ordering::SeqCst) >= 3);
    assert_eq!(turso.stored_count().await, 1);
    let stats = memory
        .durable_write_stats()
        .await
        .expect("stats must exist");
    assert!(stats.total_retried >= 2);
    assert_eq!(stats.total_failed, 0);
}

#[tokio::test]
async fn full_queue_rejects_with_explicit_backpressure() {
    // Arrange: tiny queue, workers NOT started so nothing drains.
    let cache = Arc::new(MockBackend::new());
    let turso = Arc::new(MockBackend::new());
    let mut config = test_config();
    config.durable_write_queue = Some(WriteQueueConfig {
        max_queue_size: 2,
        batch_size: 10,
        poll_interval_ms: 10,
        ..WriteQueueConfig::default()
    });
    let memory = SelfLearningMemory::with_storage(
        config,
        Arc::clone(&turso) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );
    let first = start_episode(&memory, "queued one").await;
    let second = start_episode(&memory, "queued two").await;
    let third = start_episode(&memory, "rejected three").await;

    // Act.
    memory
        .complete_episode(first, success_outcome())
        .await
        .expect("first completion must queue");
    memory
        .complete_episode(second, success_outcome())
        .await
        .expect("second completion must queue");
    let rejected = memory.complete_episode(third, success_outcome()).await;

    // Assert: explicit backpressure, never a silent drop. The rejection is
    // collected like any backend failure so the error names the cause.
    let err = rejected.expect_err("third completion must be rejected");
    match err {
        Error::Storage(msg) => assert!(msg.contains("capacity"), "unexpected message: {msg}"),
        other => panic!("expected Error::Storage, got {other:?}"),
    }
    let stats = memory
        .durable_write_stats()
        .await
        .expect("stats must exist");
    assert_eq!(stats.current_depth, 2);
}

#[tokio::test]
async fn stop_workers_drains_queued_writes() {
    // Arrange.
    let cache = Arc::new(MockBackend::new());
    let turso = Arc::new(MockBackend::new());
    let memory = SelfLearningMemory::with_storage(
        test_config(),
        Arc::clone(&turso) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );
    memory.start_durable_workers();
    for i in 0..3 {
        let id = start_episode(&memory, &format!("drain me {i}")).await;
        memory
            .complete_episode(id, success_outcome())
            .await
            .expect("completion must queue");
    }

    // Act: shutdown path drains with a timeout (issue acceptance).
    let drained = memory.stop_workers(Duration::from_secs(10)).await;

    // Assert.
    assert!(drained, "stop_workers must drain the write queue");
    assert_eq!(turso.stored_count().await, 3);
    // Worker acknowledgements are journaled for operators.
    let snapshot = memory.operation_journal_snapshot().await;
    let successes = snapshot
        .iter()
        .filter(|e| e.kind == do_memory_core::JournalOpKind::EpisodeComplete)
        .count();
    assert_eq!(successes, 3);
}

#[tokio::test]
async fn retained_failure_ids_are_capped() {
    // Arrange: far more permanent failures than the retention cap.
    let cache = Arc::new(MockBackend::new());
    let turso = Arc::new(MockBackend::failing());
    let mut config = test_config();
    config.durable_write_queue = Some(WriteQueueConfig {
        max_queue_size: 1000,
        batch_size: 100,
        poll_interval_ms: 5,
        max_retries: 0,
        ..WriteQueueConfig::default()
    });
    let memory = SelfLearningMemory::with_storage(
        config,
        Arc::clone(&turso) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );
    for i in 0..300 {
        let id = start_episode(&memory, &format!("capped {i}")).await;
        memory
            .complete_episode(id, success_outcome())
            .await
            .expect("completion must queue");
    }
    memory.start_durable_workers();

    // Act.
    let flush_result = memory
        .flush_durable_writes(Duration::from_secs(30))
        .await;

    // Assert: flush reports the failures and retention is bounded.
    assert!(flush_result.is_err());
    let stats = memory
        .durable_write_stats()
        .await
        .expect("stats must exist");
    assert_eq!(stats.total_failed, 300);
    assert_eq!(stats.failed_episode_ids.len(), 256);
}

#[tokio::test]
async fn queue_without_turso_backend_stays_synchronous() {
    // Arrange: in-memory only, queue requested but nothing to drain to.
    let memory = SelfLearningMemory::with_config(test_config());
    let memory = memory.enable_durable_writes(WriteQueueConfig::default());

    // Act.
    let episode_id = start_episode(&memory, "local only").await;
    memory
        .complete_episode(episode_id, success_outcome())
        .await
        .expect("local completion must succeed");

    // Assert: no queue was wired, flush is a no-op success.
    assert!(memory.durable_write_stats().await.is_none());
    memory
        .flush_durable_writes(Duration::from_secs(1))
        .await
        .expect("flush without queue must succeed");
    assert!(memory.get_episode(episode_id).await.is_ok());
}

//! Guard-lifetime regression tests for the ADR-082 ranking index (issue #1077).
//!
//! `SelfLearningMemory::recommend_patterns_for_task` used to acquire
//! `ranking_index.read()` and keep the guard alive across `get_all_patterns()
//! .await` and the re-ranking await. A read guard parked over an await starves
//! the `refresh_ranking_index` writer, breaking the AGENTS.md "no locks held
//! across `.await`" invariant.
//!
//! Both tests pin the reader inside a storage await that only the test can
//! release, then assert the ranking lock is free at that moment. Pre-fix the
//! writer is blocked for as long as the reader stays parked, so each test fails
//! on an explicit timeout rather than hanging CI.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::{Semaphore, mpsc};
use uuid::Uuid;

use crate::memory::pattern_search::PatternSearchResult;
use crate::patterns::{Pattern, PatternEffectiveness};
use crate::storage::{StorageBackend, StorageBackendCapabilities};
use crate::types::{ComplexityLevel, TaskContext};
use crate::{Episode, MemoryConfig, Result, SelfLearningMemory};

/// How long the `refresh_ranking_index` writer is allowed to wait for
/// `ranking_index`. Pre-fix the held read guard never releases on its own, so
/// this timeout is what fails the test rather than hanging the run.
const WRITER_SLACK: Duration = Duration::from_millis(500);

/// Bounded wait so no await in these tests can hang the run.
const SLACK: Duration = Duration::from_secs(10);

fn pattern_for(success_rate: f32) -> Pattern {
    Pattern::ToolSequence {
        id: Uuid::new_v4(),
        tools: vec!["tool1".to_string(), "tool2".to_string()],
        context: TaskContext {
            language: Some("rust".to_string()),
            domain: "web-api".to_string(),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            tags: vec!["rest".to_string()],
        },
        success_rate,
        avg_latency: chrono::Duration::milliseconds(100),
        occurrence_count: 5,
        effectiveness: PatternEffectiveness::new(),
    }
}

fn recommend_context() -> TaskContext {
    TaskContext {
        language: Some("rust".to_string()),
        domain: "web-api".to_string(),
        framework: None,
        complexity: ComplexityLevel::Moderate,
        tags: vec!["rest".to_string()],
    }
}

fn test_config() -> MemoryConfig {
    MemoryConfig {
        enable_embeddings: false,
        enable_summarization: false,
        ..Default::default()
    }
}

/// Backend that parks the *first* `get_all_patterns` caller on a zero-permit
/// semaphore after announcing entry, so the test can inspect the ranking lock at
/// a deterministic await point. Later calls return empty immediately.
struct ParkingPatternBackend {
    entered: mpsc::UnboundedSender<()>,
    gate: Arc<Semaphore>,
    should_park: AtomicBool,
}

impl StorageBackendCapabilities for ParkingPatternBackend {}

#[async_trait]
impl StorageBackend for ParkingPatternBackend {
    async fn get_all_patterns(&self) -> Result<Vec<Pattern>> {
        if self.should_park.swap(false, Ordering::AcqRel) {
            let _ = self.entered.send(());
            // Park the caller until the test hands out a permit.
            let _permit = self.gate.acquire().await;
        }
        Ok(Vec::new())
    }

    async fn store_episode(&self, _episode: &Episode) -> Result<()> {
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

    async fn get_pattern(&self, _id: crate::PatternId) -> Result<Option<Pattern>> {
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
        _since: chrono::DateTime<chrono::Utc>,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        Ok(Vec::new())
    }

    async fn query_episodes_by_metadata(
        &self,
        _key: &str,
        _value: &str,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>> {
        Ok(Vec::new())
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
        Ok(Vec::new())
    }
}

/// A recommendation parked inside its storage await, plus the handles the test
/// needs to probe the ranking lock and then let it finish.
struct ParkedReader {
    memory: Arc<SelfLearningMemory>,
    gate: Arc<Semaphore>,
    entered: mpsc::UnboundedReceiver<()>,
    handle: tokio::task::JoinHandle<Result<Vec<PatternSearchResult>>>,
}

/// Build a memory whose pattern reads park, and start a recommendation on it.
async fn spawn_parked_recommendation(patterns: &[Pattern]) -> ParkedReader {
    let gate = Arc::new(Semaphore::new(0));
    let (entered_tx, entered) = mpsc::unbounded_channel();
    let parking = Arc::new(ParkingPatternBackend {
        entered: entered_tx,
        gate: Arc::clone(&gate),
        should_park: AtomicBool::new(true),
    });
    let backend = parking as Arc<dyn StorageBackend>;
    let memory = Arc::new(SelfLearningMemory::with_storage(
        test_config(),
        Arc::clone(&backend),
        backend,
    ));
    {
        let mut fallback = memory.patterns_fallback().write().await;
        for pattern in patterns {
            fallback.insert(pattern.id(), pattern.clone());
        }
    }

    let reader_memory = Arc::clone(&memory);
    let handle = tokio::spawn(async move {
        reader_memory
            .recommend_patterns_for_task("Build an async REST API", recommend_context(), 3)
            .await
    });

    ParkedReader {
        memory,
        gate,
        entered,
        handle,
    }
}

/// Await the entry signal with a bound, so a regression reports the real cause
/// instead of hanging.
async fn wait_for_entry(entered: &mut mpsc::UnboundedReceiver<()>) {
    tokio::time::timeout(SLACK, entered.recv())
        .await
        .ok()
        .flatten()
        .expect("reader never reached the parked storage call");
}

/// With the reader parked in storage, the `refresh_ranking_index` writer must
/// still acquire `ranking_index` — possible only if the read guard was dropped
/// before the await.
#[tokio::test(flavor = "multi_thread")]
async fn recommend_patterns_does_not_hold_ranking_guard_across_await() {
    let mut reader = spawn_parked_recommendation(&[pattern_for(0.9)]).await;

    // Reaching storage proves the reader is past the ranking-index acquisition
    // site and is now sitting on an await point.
    wait_for_entry(&mut reader.entered).await;

    let writer = tokio::time::timeout(WRITER_SLACK, reader.memory.ranking_index.write())
        .await
        .map_err(|_| "ranking_index writer starved: a read guard is held across an await")
        .expect("writer must acquire the ranking lock");
    drop(writer);

    reader.gate.add_permits(1);
    let results = tokio::time::timeout(SLACK, reader.handle)
        .await
        .expect("parked reader never finished")
        .expect("recommendation task panicked")
        .expect("recommendation failed");
    assert_eq!(
        results.len(),
        1,
        "the seeded domain-matching pattern must still be recommended"
    );
}

/// Same parking, checked without waiting at all: at the storage await the
/// ranking lock must already be completely free, so a non-blocking writer gets in.
#[tokio::test(flavor = "multi_thread")]
async fn ranking_lock_is_free_while_reader_awaits_storage() {
    let mut reader = spawn_parked_recommendation(&[pattern_for(0.9), pattern_for(0.1)]).await;

    wait_for_entry(&mut reader.entered).await;

    let probe = reader
        .memory
        .ranking_index
        .try_write()
        .expect("ranking lock must be released before the storage await");
    drop(probe);

    reader.gate.add_permits(1);
    let results = tokio::time::timeout(SLACK, reader.handle)
        .await
        .expect("parked reader never finished")
        .expect("recommendation task panicked")
        .expect("recommendation failed");
    assert!(
        !results.is_empty(),
        "recommendations must survive the guard release"
    );
}

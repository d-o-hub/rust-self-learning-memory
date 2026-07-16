//! S1.3 / S1.4: lock-free backend awaits and durable capacity eviction.

#![allow(clippy::unwrap_used)] // test-only recording backend

use async_trait::async_trait;
use chrono::Utc;
use do_memory_core::episode::{EvictionPolicy, PatternId};
use do_memory_core::storage::StorageBackend;
use do_memory_core::{
    Episode, ExecutionStep, Heuristic, MemoryConfig, Pattern, Result, SelfLearningMemory,
    TaskContext, TaskOutcome, TaskType,
};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;

/// Records `delete_episode` calls for durable eviction assertions (S1.4).
#[derive(Default)]
struct RecordingBackend {
    deleted: Mutex<HashSet<Uuid>>,
    stored: Mutex<HashSet<Uuid>>,
}

#[async_trait]
impl StorageBackend for RecordingBackend {
    async fn store_episode(&self, episode: &Episode) -> Result<()> {
        self.stored.lock().unwrap().insert(episode.episode_id);
        Ok(())
    }
    async fn get_episode(&self, _id: Uuid) -> Result<Option<Episode>> {
        Ok(None)
    }
    async fn delete_episode(&self, id: Uuid) -> Result<()> {
        self.deleted.lock().unwrap().insert(id);
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

fn success_outcome() -> TaskOutcome {
    TaskOutcome::Success {
        verdict: "ok".into(),
        artifacts: vec![],
    }
}

#[tokio::test]
async fn s14_capacity_eviction_deletes_from_backends() {
    let cache = Arc::new(RecordingBackend::default());
    let turso = Arc::new(RecordingBackend::default());

    let config = MemoryConfig {
        max_episodes: Some(1),
        eviction_policy: Some(EvictionPolicy::LRU),
        quality_threshold: 0.0, // allow empty episodes for capacity path
        batch_config: None,
        ..MemoryConfig::default()
    };

    let memory = SelfLearningMemory::with_storage(
        config,
        Arc::clone(&turso) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );

    let ctx = TaskContext::default();
    let ep1 = memory
        .start_episode("first".into(), ctx.clone(), TaskType::Testing)
        .await;
    memory
        .log_step(ep1, ExecutionStep::new(1, "tool".into(), "act".into()))
        .await;
    memory
        .complete_episode(ep1, success_outcome())
        .await
        .expect("complete first");

    // At capacity: completing second episode should evict the first from backends.
    let ep2 = memory
        .start_episode("second".into(), ctx, TaskType::Testing)
        .await;
    memory
        .log_step(ep2, ExecutionStep::new(1, "tool".into(), "act".into()))
        .await;
    memory
        .complete_episode(ep2, success_outcome())
        .await
        .expect("complete second");

    assert!(
        cache.deleted.lock().unwrap().contains(&ep1),
        "cache backend should delete capacity-evicted episode"
    );
    assert!(
        turso.deleted.lock().unwrap().contains(&ep1),
        "durable backend should delete capacity-evicted episode"
    );
    // Evicted episode must not remain in in-memory map.
    assert!(memory.get_episode(ep1).await.is_err());
    assert!(memory.get_episode(ep2).await.is_ok());
}

#[tokio::test]
async fn s13_concurrent_unique_steps_persist_exactly_once() {
    // Immediate persistence path (no batching) — short locks must not drop steps.
    let config = MemoryConfig {
        batch_config: None,
        ..MemoryConfig::default()
    };
    let memory = Arc::new(SelfLearningMemory::with_config(config));

    let episode_id = memory
        .start_episode(
            "concurrent steps".into(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    let n = 50usize;
    let mut handles = Vec::with_capacity(n);
    for i in 1..=n {
        let mem = Arc::clone(&memory);
        handles.push(tokio::spawn(async move {
            let step = ExecutionStep::new(i, format!("tool_{i}"), format!("action_{i}"));
            mem.log_step(episode_id, step).await;
        }));
    }
    for h in handles {
        h.await.expect("step task");
    }

    let episode = memory.get_episode(episode_id).await.expect("episode");
    let mut numbers: Vec<usize> = episode.steps.iter().map(|s| s.step_number).collect();
    numbers.sort_unstable();
    numbers.dedup();
    assert_eq!(
        numbers.len(),
        n,
        "expected {n} unique step numbers, got {} (lost concurrent updates?)",
        numbers.len()
    );
}

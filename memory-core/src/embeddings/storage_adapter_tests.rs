//! Tests for the identity-scoped embedding storage adapter.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::episode::{Episode, PatternId};
use crate::storage::StorageBackendCapabilities;
use crate::patterns::Pattern;
use crate::types::{TaskContext, TaskType};
use crate::{Heuristic, Result, StorageBackend};

use super::{EmbeddingStorageAdapter, SelectedEmbeddingStorage};
use crate::embeddings::storage::{
    EPISODE_NAMESPACE, EmbeddingStorageBackend, EmbeddingStorageScope, PATTERN_NAMESPACE,
};

/// Records namespaced generic-embedding stores/reads while serving episodes and
/// patterns, and can be told to fail specific operations.
#[derive(Default)]
struct RecordingBackend {
    embeddings: Mutex<HashMap<String, Vec<f32>>>,
    episodes: Mutex<HashMap<Uuid, Episode>>,
    patterns: Mutex<HashMap<Uuid, Pattern>>,
    stores: Mutex<Vec<String>>,
    reads: Mutex<Vec<String>>,
    fail_store: AtomicBool,
    fail_get: AtomicBool,
    fail_list: AtomicBool,
}

impl RecordingBackend {
    fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    fn stored_keys(&self) -> Vec<String> {
        self.stores.lock().expect("stores lock").clone()
    }

    fn read_keys(&self) -> Vec<String> {
        self.reads.lock().expect("reads lock").clone()
    }

    fn put_episode(&self, episode: Episode) {
        self.episodes
            .lock()
            .expect("episodes lock")
            .insert(episode.episode_id, episode);
    }

    fn put_pattern(&self, pattern: Pattern) {
        let id = pattern.id();
        self.patterns
            .lock()
            .expect("patterns lock")
            .insert(id, pattern);
    }

    fn set_fail_store(&self, fail: bool) {
        self.fail_store.store(fail, Ordering::SeqCst);
    }

    fn set_fail_get(&self, fail: bool) {
        self.fail_get.store(fail, Ordering::SeqCst);
    }

    fn set_fail_list(&self, fail: bool) {
        self.fail_list.store(fail, Ordering::SeqCst);
    }
}

impl StorageBackendCapabilities for RecordingBackend {}

#[async_trait]
impl StorageBackend for RecordingBackend {
    async fn store_episode(&self, _episode: &Episode) -> Result<()> {
        Ok(())
    }

    async fn get_episode(&self, id: Uuid) -> Result<Option<Episode>> {
        Ok(self
            .episodes
            .lock()
            .expect("episodes lock")
            .get(&id)
            .cloned())
    }

    async fn delete_episode(&self, _id: Uuid) -> Result<()> {
        Ok(())
    }

    async fn store_pattern(&self, _pattern: &Pattern) -> Result<()> {
        Ok(())
    }

    async fn get_pattern(&self, id: PatternId) -> Result<Option<Pattern>> {
        Ok(self
            .patterns
            .lock()
            .expect("patterns lock")
            .get(&id)
            .cloned())
    }

    async fn store_heuristic(&self, _heuristic: &Heuristic) -> Result<()> {
        Ok(())
    }

    async fn get_heuristic(&self, _id: Uuid) -> Result<Option<Heuristic>> {
        Ok(None)
    }

    async fn query_episodes_since(
        &self,
        _since: DateTime<Utc>,
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

    async fn store_embedding(&self, id: &str, embedding: Vec<f32>) -> Result<()> {
        if self.fail_store.load(Ordering::SeqCst) {
            return Err(crate::Error::Storage("injected store failure".into()));
        }
        self.stores
            .lock()
            .expect("stores lock")
            .push(id.to_string());
        self.embeddings
            .lock()
            .expect("embeddings lock")
            .insert(id.to_string(), embedding);
        Ok(())
    }

    async fn get_embedding(&self, id: &str) -> Result<Option<Vec<f32>>> {
        if self.fail_get.load(Ordering::SeqCst) {
            return Err(crate::Error::Storage("injected get failure".into()));
        }
        self.reads.lock().expect("reads lock").push(id.to_string());
        Ok(self
            .embeddings
            .lock()
            .expect("embeddings lock")
            .get(id)
            .cloned())
    }

    async fn delete_embedding(&self, id: &str) -> Result<bool> {
        Ok(self
            .embeddings
            .lock()
            .expect("embeddings lock")
            .remove(id)
            .is_some())
    }

    async fn store_embeddings_batch(&self, embeddings: Vec<(String, Vec<f32>)>) -> Result<()> {
        for (id, embedding) in embeddings {
            self.store_embedding(&id, embedding).await?;
        }
        Ok(())
    }

    async fn get_embeddings_batch(&self, ids: &[String]) -> Result<Vec<Option<Vec<f32>>>> {
        if self.fail_get.load(Ordering::SeqCst) {
            return Err(crate::Error::Storage("injected batch get failure".into()));
        }
        let guard = self.embeddings.lock().expect("embeddings lock");
        Ok(ids.iter().map(|id| guard.get(id).cloned()).collect())
    }

    async fn list_embedding_ids(&self) -> Result<Vec<String>> {
        if self.fail_list.load(Ordering::SeqCst) {
            return Err(crate::Error::Storage("injected list failure".into()));
        }
        Ok(self
            .embeddings
            .lock()
            .expect("embeddings lock")
            .keys()
            .cloned()
            .collect())
    }
}

fn scope(identity: &str, revision: u64) -> EmbeddingStorageScope {
    EmbeddingStorageScope::new(identity, revision)
}

fn episode(id: Uuid) -> Episode {
    let mut episode = Episode::new(
        "task".to_string(),
        TaskContext::default(),
        TaskType::CodeGeneration,
    );
    episode.episode_id = id;
    episode
}

fn tool_sequence(id: Uuid) -> Pattern {
    Pattern::ToolSequence {
        id,
        tools: vec!["read".to_string()],
        context: TaskContext::default(),
        success_rate: 1.0,
        avg_latency: Duration::milliseconds(5),
        occurrence_count: 1,
        effectiveness: crate::patterns::PatternEffectiveness::default(),
    }
}

#[tokio::test]
async fn durable_adapter_records_namespaced_stores_and_reads() {
    let backend = RecordingBackend::new();
    let scope = scope("openai:text-embedding-3-small:1536", 7);
    let adapter = EmbeddingStorageAdapter::new(backend.clone(), None, scope.clone());

    let episode_id = Uuid::new_v4();
    adapter
        .store_episode_embedding(episode_id, vec![1.0, 0.0, 0.0])
        .await
        .expect("store episode embedding");
    let pattern_id = Uuid::new_v4();
    adapter
        .store_pattern_embedding(pattern_id, vec![0.0, 1.0, 0.0])
        .await
        .expect("store pattern embedding");

    let stored = backend.stored_keys();
    assert!(stored.contains(&scope.logical_key(EPISODE_NAMESPACE, &episode_id.to_string())));
    assert!(stored.contains(&scope.logical_key(PATTERN_NAMESPACE, &pattern_id.to_string())));
    assert!(
        stored
            .iter()
            .all(|key| key.starts_with(&scope.key_prefix())),
        "every stored key must be namespaced: {stored:?}"
    );

    let fetched = adapter
        .get_episode_embedding(episode_id)
        .await
        .expect("get episode embedding");
    assert_eq!(fetched, Some(vec![1.0, 0.0, 0.0]));
    assert!(
        backend
            .read_keys()
            .contains(&scope.logical_key(EPISODE_NAMESPACE, &episode_id.to_string()))
    );
}

#[tokio::test]
async fn cache_is_written_through_and_backfills_reads() {
    let primary = RecordingBackend::new();
    let cache = RecordingBackend::new();
    let scope = scope("local:mini:384", 1);
    let adapter = EmbeddingStorageAdapter::new(primary.clone(), Some(cache.clone()), scope.clone());

    let episode_id = Uuid::new_v4();
    adapter
        .store_episode_embedding(episode_id, vec![0.5, 0.5])
        .await
        .expect("store");

    let key = scope.logical_key(EPISODE_NAMESPACE, &episode_id.to_string());
    assert!(primary.stored_keys().contains(&key));
    assert!(cache.stored_keys().contains(&key), "write-through to cache");

    // Drop the cache copy, then read again: primary hit must backfill the cache.
    cache.delete_embedding(&key).await.expect("drop cache copy");
    assert_eq!(
        adapter.get_episode_embedding(episode_id).await.unwrap(),
        Some(vec![0.5, 0.5])
    );
    assert!(cache.stored_keys().contains(&key), "read-through backfill");
}

#[tokio::test]
async fn primary_store_failure_propagates() {
    let primary = RecordingBackend::new();
    let cache = RecordingBackend::new();
    primary.set_fail_store(true);
    let adapter =
        EmbeddingStorageAdapter::new(primary.clone(), Some(cache.clone()), scope("a:b:1", 1));

    let error = adapter
        .store_episode_embedding(Uuid::new_v4(), vec![1.0])
        .await
        .expect_err("primary failure must surface");
    assert!(error.to_string().contains("injected store failure"));
    assert!(
        cache.stored_keys().is_empty(),
        "cache must not be written when the primary write fails"
    );
}

#[tokio::test]
async fn cache_store_failure_is_non_fatal() {
    let primary = RecordingBackend::new();
    let cache = RecordingBackend::new();
    cache.set_fail_store(true);
    let adapter =
        EmbeddingStorageAdapter::new(primary.clone(), Some(cache.clone()), scope("a:b:1", 1));

    adapter
        .store_episode_embedding(Uuid::new_v4(), vec![1.0])
        .await
        .expect("cache failure must not fail the write");
    assert_eq!(primary.stored_keys().len(), 1);
}

#[tokio::test]
async fn cache_read_failure_falls_back_to_primary() {
    let primary = RecordingBackend::new();
    let cache = RecordingBackend::new();
    let scope = scope("a:b:1", 1);
    let adapter = EmbeddingStorageAdapter::new(primary.clone(), Some(cache.clone()), scope.clone());

    let episode_id = Uuid::new_v4();
    adapter
        .store_episode_embedding(episode_id, vec![0.25])
        .await
        .expect("store");

    cache.set_fail_store(true);
    cache.set_fail_get(true);
    assert_eq!(
        adapter.get_episode_embedding(episode_id).await.unwrap(),
        Some(vec![0.25]),
        "cache read failure must fall through to primary"
    );
}

#[tokio::test]
async fn primary_read_failure_propagates() {
    let primary = RecordingBackend::new();
    primary.set_fail_get(true);
    let adapter = EmbeddingStorageAdapter::new(primary.clone(), None, scope("a:b:1", 1));

    let error = adapter
        .get_pattern_embedding(Uuid::new_v4())
        .await
        .expect_err("primary read failure must surface");
    assert!(error.to_string().contains("injected get failure"));
}

#[tokio::test]
async fn reconfiguration_cannot_read_another_scope() {
    let backend = RecordingBackend::new();
    let original = scope("openai:text-embedding-3-small:1536", 1);
    let producer = EmbeddingStorageAdapter::new(backend.clone(), None, original);
    let episode_id = Uuid::new_v4();
    producer
        .store_episode_embedding(episode_id, vec![1.0, 0.0])
        .await
        .expect("store");

    // Same identity, different configuration revision.
    let bumped = EmbeddingStorageAdapter::new(
        backend.clone(),
        None,
        scope("openai:text-embedding-3-small:1536", 2),
    );
    assert_eq!(
        bumped.get_episode_embedding(episode_id).await.unwrap(),
        None,
        "a newer config revision must not read older vectors"
    );

    // Different provider identity.
    let switched = EmbeddingStorageAdapter::new(backend.clone(), None, scope("local:mini:384", 1));
    assert_eq!(
        switched.get_episode_embedding(episode_id).await.unwrap(),
        None
    );
}

#[tokio::test]
async fn find_similar_episodes_returns_only_current_scope() {
    let backend = RecordingBackend::new();
    let scope_a = scope("openai:small:3", 1);
    let scope_b = scope("local:mini:3", 1);

    let matching = Uuid::new_v4();
    let other = Uuid::new_v4();
    backend.put_episode(episode(matching));
    backend.put_episode(episode(other));

    let adapter_a = EmbeddingStorageAdapter::new(backend.clone(), None, scope_a.clone());
    let adapter_b = EmbeddingStorageAdapter::new(backend.clone(), None, scope_b);
    adapter_a
        .store_episode_embedding(matching, vec![1.0, 0.0, 0.0])
        .await
        .expect("store a");
    adapter_b
        .store_episode_embedding(other, vec![1.0, 0.0, 0.0])
        .await
        .expect("store b");

    let results = adapter_a
        .find_similar_episodes(vec![1.0, 0.0, 0.0], 10, 0.5)
        .await
        .expect("search");
    assert_eq!(results.len(), 1, "only same-scope vectors are visible");
    assert_eq!(results[0].item.episode_id, matching);
    assert!(results[0].similarity > 0.99);
    assert_eq!(results[0].metadata.embedding_model, "openai:small:3");
}

#[tokio::test]
async fn find_similar_episodes_sorts_and_limits() {
    let backend = RecordingBackend::new();
    let adapter = EmbeddingStorageAdapter::new(backend.clone(), None, scope("p:m:2", 1));

    let near = Uuid::new_v4();
    let far = Uuid::new_v4();
    backend.put_episode(episode(near));
    backend.put_episode(episode(far));
    adapter
        .store_episode_embedding(near, vec![1.0, 0.05])
        .await
        .expect("store near");
    adapter
        .store_episode_embedding(far, vec![0.2, 1.0])
        .await
        .expect("store far");

    let results = adapter
        .find_similar_episodes(vec![1.0, 0.0], 1, 0.0)
        .await
        .expect("search");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].item.episode_id, near);
}

#[tokio::test]
async fn find_similar_patterns_returns_only_current_scope() {
    let backend = RecordingBackend::new();
    let scope_a = scope("openai:small:3", 1);
    let scope_b = scope("openai:small:3", 2);

    let matching = Uuid::new_v4();
    let other = Uuid::new_v4();
    backend.put_pattern(tool_sequence(matching));
    backend.put_pattern(tool_sequence(other));

    let adapter_a = EmbeddingStorageAdapter::new(backend.clone(), None, scope_a.clone());
    let adapter_b = EmbeddingStorageAdapter::new(backend.clone(), None, scope_b);
    adapter_a
        .store_pattern_embedding(matching, vec![0.9, 0.1])
        .await
        .expect("store a");
    adapter_b
        .store_pattern_embedding(other, vec![0.9, 0.1])
        .await
        .expect("store b");

    let results = adapter_a
        .find_similar_patterns(vec![1.0, 0.0], 10, 0.5)
        .await
        .expect("search");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].item.id(), matching);
    assert_eq!(results[0].metadata.embedding_model, "openai:small:3");
}

#[tokio::test]
async fn find_similar_propagates_primary_listing_failure() {
    let backend = RecordingBackend::new();
    backend.set_fail_list(true);
    let adapter = EmbeddingStorageAdapter::new(backend.clone(), None, scope("a:b:1", 1));

    let error = adapter
        .find_similar_episodes(vec![1.0], 10, 0.0)
        .await
        .expect_err("listing failure must surface");
    assert!(error.to_string().contains("injected list failure"));
}

#[tokio::test]
async fn ephemeral_storage_reports_non_durable_scope() {
    let scope = scope("local:mini:384", 3);
    let selected = SelectedEmbeddingStorage::ephemeral(scope.clone());

    assert!(!selected.mode.is_durable());
    assert_eq!(selected.mode.label(), "ephemeral");
    assert_eq!(selected.mode.scope(), &scope);
    assert!(!selected.storage.is_durable());
    assert_eq!(selected.storage.storage_scope().as_ref(), Some(&scope));

    let episode_id = Uuid::new_v4();
    selected
        .storage
        .store_episode_embedding(episode_id, vec![1.0])
        .await
        .expect("ephemeral store");
    assert_eq!(
        selected
            .storage
            .get_episode_embedding(episode_id)
            .await
            .unwrap(),
        Some(vec![1.0])
    );
}

#[tokio::test]
async fn memory_without_backends_selects_ephemeral_store() {
    let memory = crate::SelfLearningMemory::new();
    let selected = memory.embedding_storage(scope("local:mini:384", 1));

    assert!(!selected.mode.is_durable());
    assert!(!selected.storage.is_durable());
    assert_eq!(selected.mode.label(), "ephemeral");
}

#[tokio::test]
async fn memory_with_backends_selects_durable_adapter() {
    let primary = RecordingBackend::new();
    let memory = crate::SelfLearningMemory::with_storage(
        crate::MemoryConfig::default(),
        primary.clone(),
        primary.clone(),
    );
    let scope = scope("openai:small:1536", 4);
    let selected = memory.embedding_storage(scope.clone());

    assert!(selected.mode.is_durable());
    assert_eq!(selected.mode.label(), "durable");
    assert_eq!(selected.mode.scope(), &scope);
    assert_eq!(selected.storage.storage_scope().as_ref(), Some(&scope));
    assert!(selected.storage.is_durable());

    let episode_id = Uuid::new_v4();
    selected
        .storage
        .store_episode_embedding(episode_id, vec![1.0, 2.0])
        .await
        .expect("durable store");
    assert_eq!(
        primary.stored_keys(),
        vec![scope.logical_key(EPISODE_NAMESPACE, &episode_id.to_string())],
        "primary backend must receive the namespaced key exactly once"
    );
}

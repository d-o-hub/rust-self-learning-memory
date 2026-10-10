//! Cache gating for procedural memory (#1087 slices 2-3).
//!
//! `store_procedural_memory` / `delete_procedural_memory` mirror into the cache
//! backend only when it advertises `supports_procedural_memory()`: the trait
//! default now returns `Error::CapabilityUnavailable`, and a best-effort cache
//! write must be skipped rather than presented as a durable one. The read path
//! refreshes the cache from durable storage under the same predicate.

// Integration tests are separate crate roots and don't inherit .clippy.toml settings
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use async_trait::async_trait;
use do_memory_core::episode::PatternId;
use do_memory_core::memory::SelfLearningMemory;
use do_memory_core::procedural::ProceduralMemory;
use do_memory_core::{
    Episode, Heuristic, MemoryConfig, Pattern, PatternEffectiveness, Result, StorageBackend,
    StorageBackendCapabilities, TaskContext,
};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Backend double recording the procedural calls it receives.
///
/// The capability answer is configurable, so one double covers both the
/// advertising and the non-advertising backend in the same test.
struct ProceduralCache {
    advertises: bool,
    procedurals: Mutex<HashMap<Uuid, ProceduralMemory>>,
    store_calls: Mutex<Vec<Uuid>>,
    delete_calls: Mutex<Vec<Uuid>>,
}

impl ProceduralCache {
    fn new(advertises: bool) -> Self {
        Self {
            advertises,
            procedurals: Mutex::new(HashMap::new()),
            store_calls: Mutex::new(Vec::new()),
            delete_calls: Mutex::new(Vec::new()),
        }
    }

    fn store_calls(&self) -> Vec<Uuid> {
        self.store_calls.lock().clone()
    }

    fn delete_calls(&self) -> Vec<Uuid> {
        self.delete_calls.lock().clone()
    }
}

impl StorageBackendCapabilities for ProceduralCache {
    fn supports_procedural_memory(&self) -> bool {
        self.advertises
    }
}

#[async_trait]
impl StorageBackend for ProceduralCache {
    async fn store_procedural(&self, procedural: &ProceduralMemory) -> Result<()> {
        self.store_calls.lock().push(procedural.id);
        self.procedurals
            .lock()
            .insert(procedural.id, procedural.clone());
        Ok(())
    }

    async fn get_procedural(&self, id: Uuid) -> Result<Option<ProceduralMemory>> {
        Ok(self.procedurals.lock().get(&id).cloned())
    }

    async fn delete_procedural(&self, id: Uuid) -> Result<()> {
        self.delete_calls.lock().push(id);
        self.procedurals.lock().remove(&id);
        Ok(())
    }

    // Required methods: inert, this double exists for procedural traffic only.

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
        _since: chrono::DateTime<chrono::Utc>,
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

fn procedural(name: &str) -> ProceduralMemory {
    ProceduralMemory {
        id: Uuid::new_v4(),
        name: name.to_string(),
        description: format!("How to {name}"),
        context: TaskContext::default(),
        steps: vec![],
        effectiveness: PatternEffectiveness::new(),
        source_episodes: vec![],
        source_patterns: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

/// An unadvertised cache receives neither the write nor the delete.
#[tokio::test]
async fn unsupported_cache_is_skipped_for_procedural_traffic() {
    let durable = Arc::new(ProceduralCache::new(true));
    let cache = Arc::new(ProceduralCache::new(false));
    let memory = SelfLearningMemory::with_storage(
        MemoryConfig::default(),
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );

    let skill = procedural("unsupported-cache");
    memory
        .store_procedural_memory(skill.clone())
        .await
        .expect("a durable write must succeed even when the cache cannot store procedural memory");

    assert_eq!(
        durable.store_calls(),
        vec![skill.id],
        "the durable backend must receive the write"
    );
    assert!(
        cache.store_calls().is_empty(),
        "an unadvertised cache must not be written"
    );

    memory
        .delete_procedural_memory(skill.id)
        .await
        .expect("the delete must succeed even when the cache cannot delete procedural memory");

    assert_eq!(durable.delete_calls(), vec![skill.id]);
    assert!(
        cache.delete_calls().is_empty(),
        "an unadvertised cache must not be deleted from"
    );
}

/// An advertised cache mirrors the durable write and the delete.
#[tokio::test]
async fn advertised_cache_mirrors_procedural_writes_and_deletes() {
    let durable = Arc::new(ProceduralCache::new(true));
    let cache = Arc::new(ProceduralCache::new(true));
    let memory = SelfLearningMemory::with_storage(
        MemoryConfig::default(),
        Arc::clone(&durable) as Arc<dyn StorageBackend>,
        Arc::clone(&cache) as Arc<dyn StorageBackend>,
    );

    let skill = procedural("advertised-cache");
    memory.store_procedural_memory(skill.clone()).await.unwrap();
    assert_eq!(cache.store_calls(), vec![skill.id]);

    memory.delete_procedural_memory(skill.id).await.unwrap();
    assert_eq!(cache.delete_calls(), vec![skill.id]);
}

/// A durable hit refreshes an advertising cache, and is skipped otherwise.
#[tokio::test]
async fn durable_hit_refreshes_only_an_advertising_cache() {
    let skill = procedural("refresh-on-read");

    for advertises in [true, false] {
        let durable = Arc::new(ProceduralCache::new(true));
        // Seed only the durable side, as if the cache were cold.
        durable.procedurals.lock().insert(skill.id, skill.clone());
        let cache = Arc::new(ProceduralCache::new(advertises));
        let memory = SelfLearningMemory::with_storage(
            MemoryConfig::default(),
            Arc::clone(&durable) as Arc<dyn StorageBackend>,
            Arc::clone(&cache) as Arc<dyn StorageBackend>,
        );

        let fetched = memory
            .get_procedural_memory(skill.id)
            .await
            .unwrap()
            .expect("the durable backend holds the record");

        assert_eq!(fetched.id, skill.id);
        if advertises {
            assert_eq!(
                cache.store_calls(),
                vec![skill.id],
                "an advertising cache must be refreshed from the durable hit"
            );
        } else {
            assert!(
                cache.store_calls().is_empty(),
                "a non-advertising cache must not be refreshed"
            );
        }
    }
}

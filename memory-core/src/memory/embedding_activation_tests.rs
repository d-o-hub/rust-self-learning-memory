use super::*;
use crate::embeddings::{EmbeddingConfig, InMemoryEmbeddingStorage, MockLocalModel};

fn make_service(name: &str) -> Arc<SemanticService> {
    let provider = Box::new(MockLocalModel::new(name.to_string(), 4));
    let storage = Box::new(InMemoryEmbeddingStorage::new());
    Arc::new(SemanticService::new(
        provider,
        storage,
        EmbeddingConfig::default(),
    ))
}

#[tokio::test]
async fn test_activate_first_time_sets_revision_one() {
    let memory = SelfLearningMemory::new();
    let svc = make_service("model-a");

    memory
        .activate_semantic_service(Arc::clone(&svc), "local:model-a:4".to_string())
        .await;

    let act = memory
        .embedding_activation()
        .await
        .expect("activation should be set");
    assert_eq!(act.revision, 1);
    assert_eq!(act.provider_identity, "local:model-a:4");
    assert!(
        !act.reindex_required,
        "first activation never requires reindex"
    );
}

#[tokio::test]
async fn test_activate_twice_increments_revision_and_sets_reindex() {
    let memory = SelfLearningMemory::new();

    // First activation
    memory
        .activate_semantic_service(make_service("model-a"), "local:model-a:4".to_string())
        .await;

    // Second activation with a different provider identity
    memory
        .activate_semantic_service(
            make_service("model-b"),
            "openai:text-embedding-3-small:1536".to_string(),
        )
        .await;

    let act = memory
        .embedding_activation()
        .await
        .expect("activation should be set");

    assert_eq!(act.revision, 2, "revision should increment on each call");
    assert!(
        act.reindex_required,
        "reindex_required must be true when provider identity changes"
    );
    assert_eq!(act.provider_identity, "openai:text-embedding-3-small:1536");
}

#[tokio::test]
async fn test_activate_same_identity_does_not_require_reindex() {
    let memory = SelfLearningMemory::new();

    memory
        .activate_semantic_service(make_service("model-a"), "local:model-a:4".to_string())
        .await;
    memory
        .activate_semantic_service(make_service("model-a"), "local:model-a:4".to_string())
        .await;

    let act = memory.embedding_activation().await.unwrap();
    assert_eq!(act.revision, 2);
    assert!(
        !act.reindex_required,
        "same identity should not require reindex"
    );
}

#[tokio::test]
async fn test_semantic_service_returns_some_after_activation() {
    let memory = SelfLearningMemory::new();

    // Before activation, semantic_service() returns None
    assert!(
        memory.semantic_service().is_none(),
        "should be None before activation"
    );

    // After activation, active_embedding holds the service
    memory
        .activate_semantic_service(make_service("model-a"), "local:model-a:4".to_string())
        .await;

    let act = memory.embedding_activation().await;
    assert!(
        act.is_some(),
        "active_embedding should be Some after activation"
    );
}

/// `live_semantic_service` must fall back to the static `semantic_service`
/// field when `active_embedding` is None.
#[tokio::test]
async fn test_live_semantic_service_falls_back_to_static_field() {
    use crate::embeddings::{EmbeddingConfig, InMemoryEmbeddingStorage};
    use std::sync::Arc;

    let mut memory = SelfLearningMemory::new();

    // active_embedding is None; static field also None — expect None.
    let live = memory.live_semantic_service().await;
    assert!(live.is_none(), "should be None when both slots are empty");

    // Directly set the static semantic_service field (pub(super) within this module).
    let provider = Box::new(MockLocalModel::new("static-model".to_string(), 4));
    let storage = Box::new(InMemoryEmbeddingStorage::new());
    let static_svc = Arc::new(SemanticService::new(
        provider,
        storage,
        EmbeddingConfig::default(),
    ));
    memory.semantic_service = Some(Arc::clone(&static_svc));

    // active_embedding is still None — must fall back to static field.
    let live = memory.live_semantic_service().await;
    assert!(
        live.is_some(),
        "live_semantic_service must return static field when active_embedding is None"
    );

    // After activation, the runtime slot takes priority over the static field.
    memory
        .activate_semantic_service(make_service("runtime-model"), "local:rt:4".to_string())
        .await;
    let live = memory.live_semantic_service().await;
    assert!(
        live.is_some(),
        "runtime slot must be returned after activation"
    );
}

/// REA-2026-07-26-A6: reader routine for the concurrency test below.
///
/// Repeatedly snapshots the live service and activation.  Extracted into its
/// own coroutine so the spawned task stays shallow; must never deadlock or
/// panic while a writer replaces the provider concurrently.
async fn read_activation_snapshots(memory: Arc<SelfLearningMemory>, reads: usize) {
    for _ in 0..reads {
        // Snapshot before any provider/storage await — must never deadlock or
        // panic while a writer holds the write lock.
        let _svc = memory.live_semantic_service().await;
        if let Some(act) = memory.embedding_activation().await {
            assert!(
                !act.provider_identity.is_empty(),
                "identity must never be observed empty"
            );
            assert!(act.revision >= 1, "revision must be positive once set");
        }
        tokio::task::yield_now().await;
    }
}

/// REA-2026-07-26-A6: reads during a replacement must never deadlock, panic,
/// or observe a half-built activation.  Many reader tasks snapshot the live
/// service and activation concurrently with a writer that replaces the
/// provider repeatedly.  Runs on a multi-thread runtime so the readers and
/// writer execute on distinct OS threads (ADR-077 §4).  The final revision
/// must equal the number of writes, proving every replacement landed and no
/// reader observed a torn slot.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_reads_during_activation_replacement() {
    const WRITES: u64 = 25;
    const READERS: usize = 8;
    const READS_PER_TASK: usize = 50;

    let memory = Arc::new(SelfLearningMemory::new());
    let mut reader_handles = Vec::with_capacity(READERS);

    for _ in 0..READERS {
        let m = Arc::clone(&memory);
        reader_handles.push(tokio::spawn(read_activation_snapshots(m, READS_PER_TASK)));
    }

    let writer_memory = Arc::clone(&memory);
    let writer = tokio::spawn(async move {
        for i in 0..WRITES {
            let model = format!("model-{}", i % 3);
            writer_memory
                .activate_semantic_service(make_service(&model), format!("local:{model}:4"))
                .await;
        }
    });

    writer.await.expect("writer must not panic");
    for handle in reader_handles {
        handle.await.expect("reader must not panic");
    }

    let final_act = memory
        .embedding_activation()
        .await
        .expect("activation must be set after writes");
    assert_eq!(
        final_act.revision, WRITES,
        "every replacement must advance the revision exactly once"
    );
}

/// Issue #1072: activations must serialise on the activation slot, so many
/// concurrent writers each advance the revision exactly once instead of
/// deriving the same next revision from a shared stale read.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_activate_semantic_service_concurrent_calls_are_unique_and_monotonic() {
    const WRITERS: usize = 16;

    let memory = Arc::new(SelfLearningMemory::new());
    let barrier = Arc::new(tokio::sync::Barrier::new(WRITERS));
    let mut handles = Vec::with_capacity(WRITERS);

    for i in 0..WRITERS {
        let writer_memory = Arc::clone(&memory);
        let writer_barrier = Arc::clone(&barrier);
        handles.push(tokio::spawn(async move {
            // Release all writers at once to maximise contention.
            writer_barrier.wait().await;
            let model = format!("model-{i}");
            writer_memory
                .activate_semantic_service(make_service(&model), format!("local:{model}:4"))
                .await;
        }));
    }

    for handle in handles {
        handle.await.expect("writer must not panic");
    }

    let final_act = memory
        .embedding_activation()
        .await
        .expect("activation must be set after writes");
    assert_eq!(
        final_act.revision, WRITERS as u64,
        "each concurrent activation must advance the revision exactly once"
    );

    // One coherent winning snapshot: the activation and the live service
    // must be the same installed provider instance.
    let live = memory
        .live_semantic_service()
        .await
        .expect("live service must exist after activation");
    assert!(
        Arc::ptr_eq(&final_act.service, &live),
        "activation snapshot and live service must agree"
    );
}

/// Issue #1072: a rejected activation must not mutate the prior snapshot,
/// ANN index binding or cache generation.
#[tokio::test]
async fn test_failed_activation_preserves_prior_snapshot_and_generation() {
    let memory = SelfLearningMemory::new();
    memory
        .activate_semantic_service(make_service("model-a"), "local:model-a:4".to_string())
        .await;

    let before = memory
        .embedding_activation()
        .await
        .expect("activation should be set");
    let generation_before = memory.query_cache.index_generation();

    let result = memory
        .try_activate_semantic_service(make_service("model-b"), "   ".to_string())
        .await;
    assert!(result.is_err(), "blank provider identity must be rejected");

    let after = memory
        .embedding_activation()
        .await
        .expect("activation must still be set");
    assert_eq!(after.revision, before.revision);
    assert_eq!(after.provider_identity, before.provider_identity);
    assert!(Arc::ptr_eq(&after.service, &before.service));
    assert_eq!(memory.query_cache.index_generation(), generation_before);
    assert_eq!(memory.effective_provider_identity(), "local:model-a:4");
}

/// Issue #1072: cache identity must follow the activated provider and a new
/// generation must be used so results cached for the previous provider can
/// never be served again.
#[tokio::test]
async fn test_activation_changes_cache_identity_and_invalidates_old_results() {
    use crate::retrieval::CacheKey;
    use crate::types::TaskContext;

    let memory = SelfLearningMemory::new();
    let context = TaskContext::default();

    let startup_key: CacheKey = memory.build_retrieval_cache_key("q", &context, 5);

    memory
        .activate_semantic_service(make_service("model-a"), "local:model-a:4".to_string())
        .await;
    let key_a = memory.build_retrieval_cache_key("q", &context, 5);
    assert_eq!(key_a.provider_identity, "local:model-a:4");
    assert_ne!(
        key_a.provider_identity, startup_key.provider_identity,
        "cache identity must switch to the activated provider"
    );
    assert_ne!(
        key_a.index_generation, startup_key.index_generation,
        "activation must advance the cache generation"
    );

    // Cache a result for the active provider.
    memory.query_cache.put(key_a.clone(), Vec::new());

    memory
        .activate_semantic_service(
            make_service("model-b"),
            "openai:text-embedding-3-small:1536".to_string(),
        )
        .await;

    let key_b = memory.build_retrieval_cache_key("q", &context, 5);
    assert_eq!(
        key_b.provider_identity,
        "openai:text-embedding-3-small:1536"
    );
    assert_ne!(
        key_a.compute_hash(),
        key_b.compute_hash(),
        "keys from the previous provider must not collide with the new one"
    );
    assert!(
        memory.query_cache.get(&key_b).is_none(),
        "an entry cached for the previous provider must not be served"
    );
}

/// Issue #1072: an ANN snapshot persisted for a different provider must be
/// ignored instead of queried, and activation must rebind the index.
#[tokio::test]
async fn test_activation_invalidates_incompatible_ann_index() {
    use crate::embeddings::{SimpleVectorIndex, VectorIndex};
    use crate::types::MemoryConfig;
    use uuid::Uuid;

    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("ann.json");

    let episode_id = Uuid::new_v4().to_string();
    let mut persisted =
        SimpleVectorIndex::with_provider_identity("openai:text-embedding-3-small:1536");
    persisted.upsert(&episode_id, &[1.0, 0.0]).unwrap();
    persisted.save(&path).unwrap();

    let config = MemoryConfig {
        ann_index_path: Some(path),
        ..MemoryConfig::default()
    };
    let memory = SelfLearningMemory::with_config(config);

    let retriever = memory
        .semantic_retriever()
        .expect("retriever must exist")
        .clone();

    // Startup provider is the default local provider, which does not match
    // the persisted snapshot: its vectors must not be loaded.
    assert_eq!(
        retriever.vector_index.read().len(),
        0,
        "incompatible ANN snapshot must not be loaded"
    );

    // Activation rebinds the index to the new provider.
    memory
        .activate_semantic_service(
            make_service("model-b"),
            "openai:text-embedding-3-small:1536".to_string(),
        )
        .await;
    assert_eq!(
        retriever.provider_identity().as_deref(),
        Some("openai:text-embedding-3-small:1536")
    );
    assert_eq!(retriever.vector_index.read().len(), 0);
}

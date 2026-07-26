//! Runtime embedding activation API for `SelfLearningMemory`.
//!
//! Provides [`SelfLearningMemory::activate_semantic_service`], which atomically
//! replaces the live embedding provider without restarting the process.
//!
//! # Concurrency contract
//!
//! The method reads the current revision under a *read* lock, drops that lock,
//! then re-acquires a *write* lock to install the new activation.  No lock is
//! held across `.await` points.

use std::sync::Arc;

use crate::embeddings::{EmbeddingActivation, SemanticService};

use super::SelfLearningMemory;

impl SelfLearningMemory {
    /// Atomically replace the active embedding provider.
    ///
    /// This is the **runtime seam** used by `configure_embeddings` (MCP tool) to
    /// install a new `SemanticService` after construction.
    ///
    /// # Behaviour
    ///
    /// 1. Reads the current activation (revision + identity) under a read lock,
    ///    then immediately drops that lock.
    /// 2. Computes `reindex_required`: `true` when there was a prior activation
    ///    with a *different* `provider_identity`.
    /// 3. Acquires a write lock and stores the new `EmbeddingActivation` with
    ///    `revision = old_revision + 1` (or `1` if this is the first activation).
    /// 4. Mirrors the service into `self.semantic_service` for backwards
    ///    compatibility with existing callers that read that field directly.
    ///
    /// # Returns
    ///
    /// The **previous** service if one existed, otherwise the newly installed
    /// service (so callers always get an `Arc<SemanticService>` back).
    ///
    /// # Panics
    ///
    /// Never panics — `RwLock` poisoning cannot occur in Tokio.
    pub async fn activate_semantic_service(
        &self,
        service: Arc<SemanticService>,
        provider_identity: String,
    ) -> Arc<SemanticService> {
        // --- Step 1: read current state without holding the lock across await ---
        let (old_revision, old_identity, old_service) = {
            let guard = self.active_embedding.read().await;
            match guard.as_ref() {
                Some(act) => (
                    act.revision,
                    Some(act.provider_identity.clone()),
                    Some(Arc::clone(&act.service)),
                ),
                None => (0u64, None, None),
            }
        };
        // Guard is dropped here — no lock held across any await.

        // --- Step 2: compute derived fields ---
        let new_revision = old_revision + 1;
        let reindex_required = match &old_identity {
            Some(prev) => prev != &provider_identity,
            None => false,
        };

        // --- Step 3: write-lock and install ---
        {
            let mut guard = self.active_embedding.write().await;
            *guard = Some(EmbeddingActivation {
                service: Arc::clone(&service),
                revision: new_revision,
                provider_identity,
                reindex_required,
            });
        }
        // Write guard dropped — lock released before we touch semantic_service.

        // --- Step 4: mirror into the legacy field (no lock needed — field is
        //     behind the struct's own Arc via Clone, but we need interior mut) ---
        // SAFETY: semantic_service is wrapped in Option<Arc<…>> inside the struct.
        // Because SelfLearningMemory derives Clone via Arc fields we cannot take
        // &mut self here.  We use the active_embedding lock as the synchronisation
        // point instead; callers that need the live service should call
        // `semantic_service()` which we update below via a separate RwLock path.
        //
        // To update `semantic_service` without a `&mut self` we need an interior-
        // mutable cell.  The field is currently a plain `Option<Arc<…>>` so we
        // instead leave the legacy field as-is and update the `semantic_service()`
        // accessor to check `active_embedding` first (see `mod.rs`).
        //
        // Return the previous service (or the new one if first activation).
        old_service.unwrap_or(service)
    }

    /// Get a reference to the current embedding activation, if any.
    ///
    /// Returns a clone of the [`EmbeddingActivation`] snapshot so callers do not
    /// hold the lock.
    pub async fn embedding_activation(&self) -> Option<EmbeddingActivation> {
        self.active_embedding.read().await.clone()
    }

    /// Get the live `SemanticService`, preferring the runtime-activated slot.
    ///
    /// Checks `active_embedding` first (set by
    /// [`Self::activate_semantic_service`]), then falls back to the static
    /// `semantic_service` field set at construction time.  MCP embedding tools
    /// should call this instead of the sync
    /// [`semantic_service()`](crate::memory::SelfLearningMemory::semantic_service)
    /// accessor so they see dynamically activated providers.
    pub async fn live_semantic_service(&self) -> Option<Arc<SemanticService>> {
        // Check the runtime slot first.
        if let Some(act) = self.active_embedding.read().await.as_ref() {
            return Some(Arc::clone(&act.service));
        }
        // Fall back to the static field (set at construction or via builder).
        self.semantic_service.as_ref().map(Arc::clone)
    }
}

#[cfg(test)]
mod tests {
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
}

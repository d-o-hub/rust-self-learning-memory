//! Runtime embedding activation API for `SelfLearningMemory`.
//!
//! Provides [`SelfLearningMemory::activate_semantic_service`], which atomically
//! replaces the live embedding provider without restarting the process.
//!
//! # Concurrency contract
//!
//! The activation slot is a single `tokio::sync::RwLock` holding the whole
//! snapshot (service, revision, provider identity, reindex flag). The next
//! revision is derived and installed while the *write* guard is held, so
//! concurrent activations serialise and can never derive the same revision from
//! a stale read. Readers only ever see a complete snapshot:
//! [`SelfLearningMemory::embedding_activation`] and
//! [`SelfLearningMemory::live_semantic_service`] clone the snapshot and drop the
//! guard before awaiting any provider call. The synchronous cache-identity
//! projection uses a non-blocking read (see
//! [`SelfLearningMemory::effective_provider_identity`]).

use std::sync::Arc;

use crate::embeddings::{EmbeddingActivation, SemanticService};

use super::SelfLearningMemory;

/// Failure modes of [`SelfLearningMemory::try_activate_semantic_service`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmbeddingActivationError {
    /// The supplied provider identity was empty or whitespace-only.
    ///
    /// An empty identity would poison cache keys and provenance envelopes, so
    /// activation is refused and the previous snapshot is left untouched.
    EmptyProviderIdentity,
}

impl std::fmt::Display for EmbeddingActivationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyProviderIdentity => {
                write!(f, "provider identity must not be empty")
            }
        }
    }
}

impl std::error::Error for EmbeddingActivationError {}

impl SelfLearningMemory {
    /// Atomically replace the active embedding provider.
    ///
    /// This is the **runtime seam** used by `configure_embeddings` (MCP tool) to
    /// install a new `SemanticService` after construction.
    ///
    /// # Behaviour
    ///
    /// The read-and-derive of the previous revision and identity happens *under
    /// the write lock*, together with the installation of the new snapshot, so
    /// concurrent calls cannot derive the same revision. The installed
    /// [`EmbeddingActivation`] carries the new service, the next revision, the
    /// provider identity and the `reindex_required` flag as one unit.
    ///
    /// `reindex_required` is `true` when a previous activation exists with a
    /// different `provider_identity`. The ANN index is realigned with the new
    /// provider in the same critical section, and the query-cache generation is
    /// bumped whenever the effective provider identity changes so cached results
    /// from the previous provider can never be served again.
    ///
    /// # Returns
    ///
    /// The **previous** service if one existed, otherwise the newly installed
    /// service (so callers always get an `Arc<SemanticService>` back).
    ///
    /// # Panics
    ///
    /// Never panics — tokio `RwLock`s are not poisoned.
    pub async fn activate_semantic_service(
        &self,
        service: Arc<SemanticService>,
        provider_identity: String,
    ) -> Arc<SemanticService> {
        self.install_activation(service, provider_identity).await
    }

    /// Fallible variant of [`Self::activate_semantic_service`].
    ///
    /// Validates the provider identity *before* touching any runtime state, so
    /// a rejected activation leaves the previous snapshot, ANN index and cache
    /// generation fully intact.
    ///
    /// # Errors
    ///
    /// Returns [`EmbeddingActivationError::EmptyProviderIdentity`] when
    /// `provider_identity` is empty or whitespace-only. Such an identity would
    /// poison cache keys and provenance envelopes, which is why
    /// [`Self::activate_semantic_service`] callers are expected to supply a
    /// non-empty identity.
    pub async fn try_activate_semantic_service(
        &self,
        service: Arc<SemanticService>,
        provider_identity: String,
    ) -> Result<Arc<SemanticService>, EmbeddingActivationError> {
        if provider_identity.trim().is_empty() {
            return Err(EmbeddingActivationError::EmptyProviderIdentity);
        }
        Ok(self.install_activation(service, provider_identity).await)
    }

    /// Install `service` as the active provider under a single write lock.
    ///
    /// No guard is held across a provider `.await`; all state (activation
    /// snapshot, ANN index identity, cache generation) is updated in one
    /// critical section so concurrent activations serialise and readers can
    /// never observe a torn snapshot.
    async fn install_activation(
        &self,
        service: Arc<SemanticService>,
        provider_identity: String,
    ) -> Arc<SemanticService> {
        let previous_service;
        let identity_changed;
        {
            let mut guard = self.active_embedding.write().await;

            let (new_revision, reindex_required, changed) = match guard.as_ref() {
                Some(previous) => {
                    let changed = previous.provider_identity != provider_identity;
                    (previous.revision + 1, changed, changed)
                }
                None => (
                    1,
                    false,
                    // First runtime activation: compare against the provider the
                    // instance was configured with so a switch still invalidates
                    // the query cache.
                    self.semantic_config.provider.cache_identity() != provider_identity,
                ),
            };

            previous_service = guard.as_ref().map(|act| Arc::clone(&act.service));
            identity_changed = changed;

            // Keep the ANN index consistent with the provider that will serve
            // queries; incompatible vectors are dropped, never queried.
            if let Some(retriever) = &self.semantic_retriever {
                retriever.reconcile_provider_identity(&provider_identity);
            }

            *guard = Some(EmbeddingActivation {
                service: Arc::clone(&service),
                revision: new_revision,
                provider_identity,
                reindex_required,
            });

            if identity_changed {
                // Entries from an older generation of the same identity can no
                // longer match, and entries from the previous provider must not
                // be served either.
                self.query_cache.bump_index_generation();
            }
        }
        // Guard dropped — no lock held across any await.

        previous_service.unwrap_or(service)
    }

    /// Get a reference to the current embedding activation, if any.
    ///
    /// Returns a clone of the [`EmbeddingActivation`] snapshot taken under the
    /// read lock, so callers never hold the lock and never observe a
    /// half-installed activation.
    pub async fn embedding_activation(&self) -> Option<EmbeddingActivation> {
        self.active_embedding.read().await.clone()
    }

    /// Identity used for cache keys and provenance.
    ///
    /// Prefers the runtime-activated provider so cache identity follows
    /// provider switches; falls back to the construction-time
    /// `semantic_config` when no runtime activation has happened.
    ///
    /// Reads the activation slot without awaiting. Should an activation hold the
    /// write lock at that instant, the startup identity is returned instead;
    /// that can never produce a stale hit because every activation that changes
    /// the effective identity also advances the query-cache generation, so the
    /// resulting key differs from every entry the previous provider wrote.
    #[must_use]
    pub fn effective_provider_identity(&self) -> String {
        self.active_embedding
            .try_read()
            .ok()
            .and_then(|guard| guard.as_ref().map(|act| act.provider_identity.clone()))
            .unwrap_or_else(|| self.semantic_config.provider.cache_identity())
    }

    /// Get the live `SemanticService`, preferring the runtime-activated slot.
    ///
    /// Clones the provider snapshot under the read lock, then falls back to the
    /// static `semantic_service` field set at construction time. Callers must
    /// use this (rather than the sync
    /// [`semantic_service()`](crate::memory::SelfLearningMemory::semantic_service)
    /// accessor) so they see dynamically activated providers.
    pub async fn live_semantic_service(&self) -> Option<Arc<SemanticService>> {
        let activated = self
            .active_embedding
            .read()
            .await
            .as_ref()
            .map(|act| Arc::clone(&act.service));
        if activated.is_some() {
            return activated;
        }
        // Fall back to the static field (set at construction or via builder).
        self.semantic_service.as_ref().map(Arc::clone)
    }
}

#[cfg(test)]
#[path = "embedding_activation_tests.rs"]
mod tests;

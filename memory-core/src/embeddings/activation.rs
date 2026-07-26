//! Runtime embedding activation — the live slot for a `SemanticService`.
//!
//! `EmbeddingActivation` records which provider is currently wired into
//! `SelfLearningMemory` at runtime, together with a monotonic revision counter
//! that lets callers detect provider switches and decide whether a re-index is
//! needed.

use std::sync::Arc;

use super::SemanticService;

/// A snapshot of the currently active embedding provider.
///
/// Stored inside `SelfLearningMemory` behind an
/// `Arc<tokio::sync::RwLock<Option<EmbeddingActivation>>>` so that the slot
/// can be swapped at runtime without restarting the process.
#[derive(Clone)]
pub struct EmbeddingActivation {
    /// The running semantic service.
    pub service: Arc<SemanticService>,
    /// Monotonically increasing revision — starts at 1, incremented on each
    /// call to [`activate_semantic_service`](crate::memory::SelfLearningMemory::activate_semantic_service).
    pub revision: u64,
    /// Stable identity string of the active provider (`kind:model:dims`).
    ///
    /// Derived from [`ProviderConfig::cache_identity()`](crate::embeddings::ProviderConfig::cache_identity).
    pub provider_identity: String,
    /// `true` when the provider identity changed since the previous activation,
    /// meaning that previously stored embeddings are no longer comparable and
    /// a full re-index should be considered.
    pub reindex_required: bool,
}

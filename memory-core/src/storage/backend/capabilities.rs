//! Capability queries for the optional operations of [`StorageBackend`](super::StorageBackend).
//!
//! [`StorageBackend`](super::StorageBackend) has optional methods whose default
//! implementations cannot honor the request. Relationship storage, procedural
//! memory, and episode cleanup report that honestly: their defaults return
//! [`Error::CapabilityUnavailable`](crate::Error::CapabilityUnavailable)
//! instead of a fabricated success (`Ok(())`, `None`, or an empty vector).
//! Recommendation attribution and ranking adaptation follow ADR-081/ADR-082:
//! their predicates gate the checked persistence and ranking paths, so an
//! unadvertised backend is never counted as durable.
//!
//! The predicates live here, next to each other and apart from the method
//! surface, so a backend's answers can be reviewed as one capability matrix.

/// Capability queries for the optional operations of
/// [`StorageBackend`](super::StorageBackend).
///
/// Every predicate defaults to `false`: a backend only advertises a capability
/// it actually implements. Backends that truly persist the corresponding data
/// override the predicate with `true` in a separate `impl` block from their
/// [`StorageBackend`](super::StorageBackend) implementation — the trait impl is
/// split so the capability matrix stays readable in one place.
///
/// A predicate is *not* changed by this contract when the operation is merely
/// process-local (for example the in-memory fallback inside
/// [`SelfLearningMemory`](crate::memory::SelfLearningMemory)): only durable
/// persistence counts.
pub trait StorageBackendCapabilities: Send + Sync {
    /// Whether recommendation attribution (ADR-081 §2) is durably persisted.
    ///
    /// The checked persistence path (`persist_session_checked` /
    /// `persist_feedback_checked`) writes and counts only backends that
    /// advertise this, so an unadvertised backend is reported as memory-only.
    /// The recommendation methods keep their ADR-081 no-op defaults; callers
    /// must gate on this predicate before treating a write as durable.
    fn supports_recommendation_attribution(&self) -> bool {
        false
    }

    /// Whether durable recommendation history can back feedback-to-ranking
    /// adaptation (ADR-082).
    ///
    /// When `false`, the ranking rebuild skips this backend, and its
    /// `list_recommendation_sessions` / `list_recommendation_feedback` results
    /// (empty by default) contribute nothing.
    fn supports_ranking_adaptation(&self) -> bool {
        false
    }

    /// Whether expired episodes can be durably cleaned up (WG-075).
    ///
    /// When `false`, `cleanup_episodes` / `count_cleanup_candidates` return
    /// [`Error::CapabilityUnavailable`](crate::Error::CapabilityUnavailable)
    /// instead of reporting a successful no-op.
    fn supports_episode_cleanup(&self) -> bool {
        false
    }

    /// Whether episode↔episode and episode↔pattern relationships are durably
    /// persisted and queryable.
    ///
    /// When `false`, the nine relationship methods (`store_relationship`,
    /// `remove_relationship`, `get_relationships`, `get_all_relationships`,
    /// `get_relationship_by_id`, `relationship_exists`,
    /// `store_episode_pattern_relationship`,
    /// `get_episode_pattern_relationships`, `get_weighted_neighbors`) return
    /// [`Error::CapabilityUnavailable`](crate::Error::CapabilityUnavailable)
    /// instead of an empty result.
    fn supports_relationship_persistence(&self) -> bool {
        false
    }

    /// Whether procedural memory is durably persisted and queryable.
    ///
    /// When `false`, `store_procedural`, `get_procedural`, `delete_procedural`,
    /// and `query_procedural` return
    /// [`Error::CapabilityUnavailable`](crate::Error::CapabilityUnavailable)
    /// instead of a fabricated success.
    fn supports_procedural_memory(&self) -> bool {
        false
    }
}

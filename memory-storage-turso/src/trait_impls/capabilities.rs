//! Capability matrix for the `memory-storage-turso` backends (ADR-081).
//!
//! The capability impls are split from the `StorageBackend` impls so every
//! backend's answers can be reviewed in one place. Predicates default to
//! `false`; only domains a backend actually routes and persists are overridden.
//!
//! Relationship persistence and procedural memory are advertised only by
//! `TursoStorage`. The wrappers do not implement those methods, so they keep
//! the `false` default: calling them returns
//! `Error::CapabilityUnavailable` instead of a silent `Ok(())` (the pre-#1087
//! fake-success behavior that could drop data without a trace).

use do_memory_core::StorageBackendCapabilities;

use crate::cache::CachedTursoStorage;
use crate::{ResilientStorage, TursoStorage};

/// Durable SQL-backed capabilities of [`TursoStorage`].
///
/// Relationships, episode↔pattern relationships, procedural memory, and
/// recommendation attribution are persisted by `src/relationships.rs`,
/// `src/storage/procedural.rs`, and `src/recommendations.rs`. Episode cleanup
/// has no Turso implementation, so it keeps the `false` default.
impl StorageBackendCapabilities for TursoStorage {
    fn supports_recommendation_attribution(&self) -> bool {
        true
    }

    fn supports_ranking_adaptation(&self) -> bool {
        true
    }

    fn supports_relationship_persistence(&self) -> bool {
        true
    }

    fn supports_procedural_memory(&self) -> bool {
        true
    }
}

/// Circuit-breaker wrapper: advertises exactly what it routes.
///
/// [`ResilientStorage`] forwards the core, embedding, and
/// recommendation-attribution operations to the wrapped [`TursoStorage`], so it
/// delegates those predicates. It does not implement relationship or
/// procedural-memory methods.
impl StorageBackendCapabilities for ResilientStorage {
    fn supports_recommendation_attribution(&self) -> bool {
        self.storage.supports_recommendation_attribution()
    }

    fn supports_ranking_adaptation(&self) -> bool {
        self.storage.supports_ranking_adaptation()
    }
}

/// Adaptive-cache wrapper: advertises exactly what it routes.
///
/// [`CachedTursoStorage`] forwards episodes, patterns, heuristics, embeddings,
/// and recommendation attribution to the wrapped [`TursoStorage`]. It does not
/// implement relationship or procedural-memory methods.
impl StorageBackendCapabilities for CachedTursoStorage {
    fn supports_recommendation_attribution(&self) -> bool {
        self.storage.supports_recommendation_attribution()
    }

    fn supports_ranking_adaptation(&self) -> bool {
        self.storage.supports_ranking_adaptation()
    }
}

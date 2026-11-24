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
/// Relationships, episode↔pattern relationships, procedural memory,
/// recommendation attribution, semantic episode summaries, and retention
/// cleanup are persisted by `src/relationships.rs`,
/// `src/storage/procedural.rs`, `src/recommendations.rs`,
/// `src/storage/summaries.rs`, and `src/storage/episodes/cleanup.rs`.
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

    fn supports_episode_summaries(&self) -> bool {
        true
    }

    fn supports_episode_cleanup(&self) -> bool {
        true
    }
}

/// Circuit-breaker wrapper: advertises exactly what it routes.
///
/// [`ResilientStorage`] forwards the core, embedding, recommendation-attribution,
/// episode-summary, and episode-cleanup operations to the wrapped [`TursoStorage`],
/// so it delegates those predicates. It does not implement relationship or
/// procedural-memory methods.
impl StorageBackendCapabilities for ResilientStorage {
    fn supports_recommendation_attribution(&self) -> bool {
        self.storage.supports_recommendation_attribution()
    }

    fn supports_ranking_adaptation(&self) -> bool {
        self.storage.supports_ranking_adaptation()
    }

    fn supports_episode_summaries(&self) -> bool {
        self.storage.supports_episode_summaries()
    }

    fn supports_episode_cleanup(&self) -> bool {
        self.storage.supports_episode_cleanup()
    }
}

/// Adaptive-cache wrapper: advertises exactly what it routes.
///
/// [`CachedTursoStorage`] forwards episodes, patterns, heuristics, embeddings,
/// recommendation attribution, episode summaries, and episode cleanup to the
/// wrapped [`TursoStorage`]. It does not implement relationship or
/// procedural-memory methods.
impl StorageBackendCapabilities for CachedTursoStorage {
    fn supports_recommendation_attribution(&self) -> bool {
        self.storage.supports_recommendation_attribution()
    }

    fn supports_ranking_adaptation(&self) -> bool {
        self.storage.supports_ranking_adaptation()
    }

    fn supports_episode_summaries(&self) -> bool {
        self.storage.supports_episode_summaries()
    }

    fn supports_episode_cleanup(&self) -> bool {
        self.storage.supports_episode_cleanup()
    }
}

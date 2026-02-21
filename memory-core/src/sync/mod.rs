//! # Storage Synchronization
//!
//! Coordinates data synchronization between Turso (durable) and redb (cache) storage layers.
//!
//! The synchronizer ensures:
//! - Two-phase commit for consistency
//! - Conflict resolution with Turso as source of truth
//! - Periodic synchronization of cache from durable storage
//! - Resilience to partial failures
//!
//! ## Example
//!
//! ```ignore
//! use memory_core::sync::StorageSynchronizer;
//!
//! let sync = StorageSynchronizer::new(turso_storage, redb_storage);
//! sync.start_periodic_sync(Duration::from_secs(300)).await;
//! ```

pub use self::conflict::{
    ConflictResolution, resolve_episode_conflict, resolve_heuristic_conflict,
    resolve_pattern_conflict,
};
pub use self::synchronizer::StorageSynchronizer;
pub use self::two_phase_commit::TwoPhaseCommit;
pub use self::types::{SyncConfig, SyncState, SyncStats};

mod conflict;
mod synchronizer;
mod two_phase_commit;
mod types;

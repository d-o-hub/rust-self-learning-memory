//! Capacity manager implementation
//!
//! Provides the core capacity management logic for episodic storage.

pub mod eviction;
pub mod scoring;

pub use super::calculator::CapacityManager;

//! Capacity management for episodic storage.
//!
//! Implements capacity-constrained episodic storage with relevance-weighted
//! eviction based on the GENESIS research (arXiv Oct 2025).
//!
//! The capacity manager enforces storage limits and intelligently evicts
//! low-relevance episodes using a combination of quality scores and recency.

pub mod manager;
pub mod policy;

pub use manager::CapacityManager;
pub use policy::EvictionPolicy;

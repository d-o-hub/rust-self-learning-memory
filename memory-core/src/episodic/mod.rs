//! Episodic memory management components.
//!
//! This module provides tools for managing episodic memory storage, including
//! capacity constraints and eviction policies based on the GENESIS research.
//!
//! # Components
//!
//! - [`CapacityManager`]: Capacity-constrained episodic storage with eviction
//!
//! # Examples
//!
//! ```no_run
//! use memory_core::episodic::{CapacityManager, EvictionPolicy};
//! use memory_core::{Episode, TaskContext, TaskType};
//!
//! let manager = CapacityManager::new(1000, EvictionPolicy::RelevanceWeighted);
//!
//! let episodes = vec![/* ... */];
//! if !manager.can_store(episodes.len()) {
//!     let to_evict = manager.evict_if_needed(&episodes);
//!     println!("Need to evict {} episodes", to_evict.len());
//! }
//! ```

pub mod capacity;

pub use capacity::{CapacityManager, EvictionPolicy};

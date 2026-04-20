//! Bounded context assembly for retrieval results.
//!
//! This module provides `BundleAccumulator`, a sliding window accumulator that
//! bounds retrieval context size by recency and salience instead of flat accumulation.
//!
//! ## Purpose
//!
//! When retrieving episodes and patterns for downstream prompts (e.g., LLM context),
//! flat accumulation can result in:
//! - Excessive token usage (too many items)
//! - Irrelevant items (low-quality matches included)
//! - Stale context (old items dominating recent ones)
//!
//! `BundleAccumulator` addresses these by:
//! - Capping the number of items via a sliding window
//! - Prioritizing by combined recency + salience scoring
//! - Evicting lowest-priority items when capacity exceeded
//!
//! ## Architecture
//!
//! ```text
//! Retrieval Results
//!        |
//!        v
//!   ContextItem (Episode/Pattern + Salience)
//!        |
//!        v
//!   BundleAccumulator (Sliding Window)
//!        |  - Check salience threshold
//!        |  - Compute priority = recency_weight * recency + salience_weight * salience
//!        |  - Evict lowest priority if full
//!        v
//!   Bounded Bundle (sorted by priority)
//!        |
//!        v
//!   Downstream Prompt
//! ```
//!
//! ## Quick Start
//!
//! ```
//! use do_memory_core::context::{BundleAccumulator, BundleConfig, ContextItem};
//! use do_memory_core::episode::Episode;
//! use do_memory_core::TaskContext;
//! use do_memory_core::types::TaskType;
//! use std::sync::Arc;
//!
//! // Create accumulator with default config (20 items max)
//! let mut accumulator = BundleAccumulator::default_config();
//!
//! // Add retrieved episodes with their salience scores
//! let episode = Episode::new(
//!     "Fix authentication bug".to_string(),
//!     TaskContext::default(),
//!     TaskType::Debugging,
//! );
//! let item = ContextItem::from_episode(Arc::new(episode), 0.85);
//! accumulator.add(item);
//!
//! // Finalize bundle for prompt
//! let bundle = accumulator.to_bundle();
//! println!("Bundle contains {} items for prompt", bundle.len());
//! ```
//!
//! ## Configuration Options
//!
//! ```
//! use do_memory_core::context::BundleConfig;
//!
//! // Token-efficient: smaller bundle, higher quality threshold
//! let token_config = BundleConfig::token_efficient();
//!
//! // Comprehensive: larger bundle, lower threshold
//! let full_config = BundleConfig::comprehensive();
//!
//! // Custom: tune for specific needs
//! let custom = BundleConfig {
//!     max_items: 15,            // Allow 15 items
//!     recency_weight: 0.6,      // Favor recent items
//!     salience_weight: 0.3,     // Still consider retrieval score
//!     min_salience_threshold: 0.3, // Reject very low salience
//!     recency_half_life_days: 14.0, // Recent = last 2 weeks
//! };
//! ```
//!
//! ## Integration with Retrieval
//!
//! The accumulator is designed to sit between retrieval results and prompt construction:
//!
//! ```no_run
//! use do_memory_core::memory::SelfLearningMemory;
//! use do_memory_core::context::{BundleAccumulator, BundleConfig};
//! use do_memory_core::TaskContext;
//!
//! # async fn example(memory: SelfLearningMemory) {
//! // Retrieve episodes (unbounded)
//! let episodes = memory.retrieve_relevant_context(
//!     "Implement OAuth2".to_string(),
//!     TaskContext::default(),
//!     50,  // May retrieve up to 50
//! ).await;
//!
//! // Create bounded bundle
//! let bundle = BundleAccumulator::from_episodes_with_config(
//!     episodes,
//!     BundleConfig::token_efficient(),
//!     |ep| ep.reward.as_ref().map_or(0.5, |r| r.total),
//! );
//!
//! // bundle.len() <= 10 (bounded for token efficiency)
//! # }
//! ```
//!
//! ## Modules
//!
//! - [`types`]: Core types (`ContextItem`, `BundleConfig`, `BundleStats`)
//! - [`accumulator`]: `BundleAccumulator` implementation
//! - [`scoring`]: Priority and recency scoring functions

pub mod accumulator;
pub mod scoring;
pub mod types;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_edge;

// Re-export public types
pub use accumulator::BundleAccumulator;
pub use scoring::{
    calculate_priority_score, calculate_recency_score, compare_by_priority, compare_by_recency,
    compare_by_salience,
};
pub use types::{AddResult, BundleConfig, BundleStats, ContextItem, ContextItemType};

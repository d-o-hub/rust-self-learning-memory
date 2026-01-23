//! # Pattern Effectiveness Tracking
//!
//! Tracks how patterns are used over time and their success rates.
//! Implements decay for patterns that are ineffective or rarely used.
//!
//! ## Example
//!
//! ```
//! use memory_core::patterns::effectiveness::EffectivenessTracker;
//! use uuid::Uuid;
//!
//! let mut tracker = EffectivenessTracker::new();
//!
//! let pattern_id = Uuid::new_v4();
//!
//! // Track pattern retrieval
//! tracker.record_retrieval(pattern_id);
//!
//! // Track pattern application
//! tracker.record_application(pattern_id, true);
//!
//! // Check effectiveness
//! if let Some(stats) = tracker.get_stats(pattern_id) {
//!     println!("Success rate: {:.2}", stats.success_rate);
//!     println!("Usage count: {}", stats.usage_count);
//! }
//! ```

pub mod types;

pub use types::{EffectivenessTracker, OverallStats, PatternUsage, UsageStats};

//! Pattern command module - refactored for better organization
//!
//! This module provides commands for managing patterns in the memory system.
//! It is split into submodules to keep each file under 500 LOC.

pub mod core;

pub use core::EffectivenessRanking;
pub use core::EffectivenessRankings;
pub use core::PatternAnalysis;
pub use core::PatternCommands;
pub use core::PatternDetail;
pub use core::PatternList;
pub use core::PatternSummary;
pub use core::PatternType;

pub use core::analyze_pattern;
pub use core::decay_patterns;
#[cfg(feature = "turso")]
pub use core::execute_pattern_batch_command;
pub use core::list_patterns;
pub use core::pattern_effectiveness;
pub use core::recommend_patterns;
pub use core::search_patterns;
pub use core::view_pattern;

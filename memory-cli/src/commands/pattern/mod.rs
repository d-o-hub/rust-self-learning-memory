//! Pattern command module - refactored for better organization
//!
//! This module provides commands for managing patterns in the memory system.
//! It is split into submodules to keep each file under 500 LOC.

pub mod pattern;

pub use pattern::EffectivenessRanking;
pub use pattern::EffectivenessRankings;
pub use pattern::PatternAnalysis;
pub use pattern::PatternCommands;
pub use pattern::PatternDetail;
pub use pattern::PatternList;
pub use pattern::PatternSummary;
pub use pattern::PatternType;

pub use pattern::analyze_pattern;
pub use pattern::decay_patterns;
pub use pattern::list_patterns;
pub use pattern::pattern_effectiveness;
pub use pattern::recommend_patterns;
pub use pattern::search_patterns;
pub use pattern::view_pattern;

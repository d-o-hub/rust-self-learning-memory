//! Batch operations tools for the MCP server
//!
//! This module contains submodules for efficient batch operations on episodes,
//! including querying, pattern analysis, and comparison.

pub mod batch_analysis;
pub mod batch_compare;
pub mod batch_patterns;
pub mod batch_query;

pub use batch_analysis::calculate_pattern_avg_reward;
pub use batch_analysis::calculate_pattern_success_rate;
pub use batch_analysis::generate_recommendations;
pub use batch_analysis::get_pattern_signature;
pub use batch_analysis::get_pattern_type;
pub use batch_analysis::group_patterns;
pub use batch_compare::compare_episode_approaches;
pub use batch_compare::generate_comparison_insights;
pub use batch_patterns::*;
pub use batch_query::compute_aggregate_stats;
pub use batch_query::parse_episode_filter;
pub use batch_query::parse_task_type;

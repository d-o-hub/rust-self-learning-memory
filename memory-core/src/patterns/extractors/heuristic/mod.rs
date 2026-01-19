//! Heuristic rule extractor
//!
//! Extracts condition→action heuristic rules from successful decision points in episodes.
//! Heuristics represent learned guidelines that can improve future task execution.

mod config;
mod extraction;
mod extractor;

#[cfg(test)]
mod tests;

pub use config::HeuristicExtractorConfig;
pub use extraction::{extract_action, extract_condition, is_decision_action};
pub use extractor::HeuristicExtractor;

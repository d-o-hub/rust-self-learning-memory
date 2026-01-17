//! # Advanced Pattern Analysis MCP Tool
//!
//! MCP tool integration for advanced pattern analysis combining
//! statistical analysis and predictive modeling.

pub mod executor;
pub mod summary;
pub mod time_series;
pub mod tool;
pub mod types;
pub mod validator;

// Re-export for testing
#[cfg(test)]
pub(crate) mod tests;

// Re-export public API
pub use tool::AdvancedPatternAnalysisTool;
pub use types::{
    AdvancedPatternAnalysisInput, AdvancedPatternAnalysisOutput, AnalysisConfig, AnalysisSummary,
    AnalysisType, PerformanceMetrics,
};

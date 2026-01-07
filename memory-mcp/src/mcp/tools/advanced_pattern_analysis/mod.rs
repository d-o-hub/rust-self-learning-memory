//! # Advanced Pattern Analysis MCP Tool
//!
//! MCP tool integration for advanced pattern analysis combining
//! statistical analysis and predictive modeling.

pub mod tool;
pub mod types;

// Re-export public API
pub use tool::AdvancedPatternAnalysisTool;
pub use types::{
    AdvancedPatternAnalysisInput, AdvancedPatternAnalysisOutput, AnalysisConfig, AnalysisSummary,
    AnalysisType, PerformanceMetrics,
};

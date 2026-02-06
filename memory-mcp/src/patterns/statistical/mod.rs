//! # Statistical Analysis Engine
//!
//! Core statistical engine providing Bayesian changepoint detection,
//! correlation analysis with significance testing, and time-series trend detection.
//!
//! This module is organized as follows:
//! - [`analysis`](analysis/index.html) - Core implementation (BOCPD, StatisticalEngine, types)
//! - [`tests`](tests/index.html) - Test suite

pub mod analysis;
#[cfg(test)]
pub mod bocpd_tests;
#[allow(unused)]
pub mod tests;

// Re-export public API from analysis module
pub use analysis::{
    AnalysisMetadata, BOCPDConfig, BOCPDResult, BOCPDState, ChangeType, ChangepointConfig,
    ChangepointDetector, ChangepointResult, CorrelationAnalyzer, CorrelationResult, SimpleBOCPD,
    StatisticalConfig, StatisticalEngine, StatisticalResults, TrendDirection, TrendResult,
};

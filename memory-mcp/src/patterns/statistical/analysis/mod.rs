//! # Statistical Analysis Implementation
//!
//! Core statistical engine providing Bayesian changepoint detection,
//! correlation analysis with significance testing, and time-series trend detection.

pub use bocpd::{log_sum_exp, SimpleBOCPD};
pub use engine::{ChangepointDetector, CorrelationAnalyzer, StatisticalEngine};
pub use types::{
    AnalysisMetadata, BOCPDConfig, BOCPDResult, BOCPDState, ChangeType, ChangepointConfig,
    ChangepointResult, CorrelationResult, StatisticalConfig, StatisticalResults, TrendDirection,
    TrendResult,
};

pub use SimpleBOCPD as BocpdDetector;

pub mod bocpd;
mod engine;
pub mod types;

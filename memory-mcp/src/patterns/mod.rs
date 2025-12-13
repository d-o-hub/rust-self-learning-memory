//! # Advanced Pattern Analysis Module
//!
//! This module provides sophisticated statistical analysis capabilities for the MCP server,
//! including changepoint detection, correlation analysis, forecasting, and causal inference.
//!
//! ## Features
//!
//! - **Statistical Engine**: Bayesian changepoint detection and correlation analysis
//! - **Predictive Models**: Time series forecasting and anomaly detection
//! - **Causal Inference**: Hyper-geometric computational causality
//! - **Performance**: Streaming algorithms and parallel processing
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
//! │  Statistical    │    │   Predictive     │    │   MCP Tool      │
//! │   Engine        │    │   Models         │    │   Integration   │
//! │                 │    │                  │    │                 │
//! │ - Changepoint   │    │ - Forecasting    │    │ - Tool Reg.     │
//! │ - Correlation   │    │ - Anomaly Det.  │    │ - Input Val.    │
//! │ - Significance  │    │ - Causal Inf.   │    │ - Memory Query  │
//! └─────────────────┘    └──────────────────┘    └─────────────────┘
//!         │                       │                       │
//!         └───────────────────────┼───────────────────────┘
//!                                 ▼
//!                    ┌─────────────────────┐
//!                    │   Performance &     │
//!                    │   Caching Layer     │
//!                    │                     │
//!                    │ - redb Cache        │
//!                    │ - Streaming Alg.    │
//!                    │ - Parallel Proc.    │
//!                    └─────────────────────┘
//! ```

pub mod predictive;
pub mod statistical;

pub use predictive::{AnomalyDetector, CausalAnalyzer, ForecastingEngine};
pub use statistical::{ChangepointDetector, CorrelationAnalyzer, StatisticalEngine};

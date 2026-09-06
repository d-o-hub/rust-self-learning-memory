//! # Advanced Pattern Analysis Executor
//!
//! Execution logic for advanced pattern analysis including
//! statistical and predictive analysis orchestration.

use crate::patterns::{predictive, statistical};
use anyhow::Result;
use std::collections::HashMap;

use super::tool::AdvancedPatternAnalysisTool;
use super::types::AnalysisConfig;

/// Configuration builder for analysis execution
pub struct AnalysisConfigBuilder {
    significance_level: Option<f64>,
    forecast_horizon: Option<usize>,
    anomaly_sensitivity: Option<f64>,
    enable_causal_inference: Option<bool>,
    max_data_points: Option<usize>,
    parallel_processing: Option<bool>,
}

impl AnalysisConfigBuilder {
    /// Create a new config builder from user config
    pub fn from_input(config: &Option<AnalysisConfig>) -> Self {
        let mut builder = Self::default();
        if let Some(cfg) = config {
            builder.significance_level = cfg.significance_level;
            builder.forecast_horizon = cfg.forecast_horizon;
            builder.anomaly_sensitivity = cfg.anomaly_sensitivity;
            builder.enable_causal_inference = cfg.enable_causal_inference;
            builder.max_data_points = cfg.max_data_points;
            builder.parallel_processing = cfg.parallel_processing;
        }
        builder
    }

    /// Create with defaults
    fn default() -> Self {
        Self {
            significance_level: None,
            forecast_horizon: None,
            anomaly_sensitivity: None,
            enable_causal_inference: None,
            max_data_points: None,
            parallel_processing: None,
        }
    }
}

impl AdvancedPatternAnalysisTool {
    /// Perform statistical analysis on time series data
    pub(super) async fn perform_statistical_analysis(
        &self,
        data: &HashMap<String, Vec<f64>>,
        config: &Option<AnalysisConfig>,
    ) -> Result<statistical::StatisticalResults> {
        let mut engine_config = statistical::StatisticalConfig::default();

        if let Some(cfg) = config {
            if let Some(sig) = cfg.significance_level {
                engine_config.significance_level = sig;
            }
            if let Some(max_points) = cfg.max_data_points {
                engine_config.max_data_points = max_points;
            }
            if let Some(parallel) = cfg.parallel_processing {
                engine_config.parallel_processing = parallel;
            }
        }

        let mut engine = statistical::StatisticalEngine::with_config(engine_config)?;
        engine.analyze_time_series(data)
    }

    /// Perform predictive analysis on time series data
    pub(super) async fn perform_predictive_analysis(
        &self,
        data: &HashMap<String, Vec<f64>>,
        config: &Option<AnalysisConfig>,
    ) -> Result<predictive::PredictiveResults> {
        let mut predictive_config = predictive::PredictiveConfig::default();

        if let Some(cfg) = config {
            if let Some(horizon) = cfg.forecast_horizon {
                predictive_config.forecast_horizon = horizon;
            }
            if let Some(sens) = cfg.anomaly_sensitivity {
                predictive_config.anomaly_sensitivity = sens;
            }
            if let Some(enable_causal) = cfg.enable_causal_inference {
                predictive_config.enable_causal_inference = enable_causal;
            }
        }

        predictive::run_predictive_analysis(data, predictive_config)
    }
}

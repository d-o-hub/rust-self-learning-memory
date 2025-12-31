//! # Predictive Analysis Module
//!
//! Provides forecasting models, anomaly detection, and causal inference capabilities.

#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
use anyhow::Result;
use std::collections::HashMap;
use tracing::{info, instrument};

pub mod anomaly;
pub mod causal;
pub mod dbscan;
pub mod forecasting;
pub mod types;

pub use anomaly::AnomalyDetector;
pub use causal::CausalAnalyzer;
pub use dbscan::{AdaptiveDBSCAN, ClusterLabel, DBSCANConfig, Point};
pub use forecasting::ForecastingEngine;
pub use types::{
    CausalResult, CausalType, ETSErrorType, ETSForecastResult, ETSModel, ETSModelSpec,
    ETSSeasonalType, ETSState, ETSTrendType, ForecastResult, PredictiveConfig,
    PredictiveMetadata, PredictiveResults, SeasonalityResult,
};

#[instrument(skip(data))]
pub fn run_predictive_analysis(data: &HashMap<String, Vec<f64>>, config: PredictiveConfig) -> Result<PredictiveResults> {
    let start_time = std::time::Instant::now();
    info!("Starting comprehensive predictive analysis");
    let mut forecaster = ForecastingEngine::with_config(config.clone())?;
    let forecasts = forecaster.forecast(data)?;
    let mut anomaly_detector = AnomalyDetector::with_config(config.clone())?;
    let anomalies = anomaly_detector.detect_anomalies(data)?;
    let causal_analyzer = CausalAnalyzer::with_config(config.clone())?;
    let causal_relationships = causal_analyzer.analyze_causality(data)?;
    let duration = start_time.elapsed();
    let metadata = PredictiveMetadata {
        variables_analyzed: data.len(),
        duration_ms: duration.as_millis() as u64,
        memory_usage: data.values().map(|v| v.len() * 8).sum(),
        methods_used: vec!["ETS Forecasting".to_string(), "DBSCAN Anomaly Detection".to_string(), "Granger Causality".to_string()],
    };
    let results = PredictiveResults { forecasts, anomalies, causal_relationships, metadata };
    info!("Predictive analysis completed in {}ms", results.metadata.duration_ms);
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn test_forecasting_engine_creation() {
        let engine = ForecastingEngine::new();
        assert!(engine.is_ok());
    }
    #[test]
    fn test_anomaly_detector_creation() {
        let detector = AnomalyDetector::new();
        assert!(detector.is_ok());
    }
    #[test]
    fn test_causal_analyzer_creation() {
        let analyzer = CausalAnalyzer::new();
        assert!(analyzer.is_ok());
    }
    #[test]
    fn test_comprehensive_analysis() -> Result<()> {
        let mut data = HashMap::new();
        data.insert("series1".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        data.insert("series2".to_string(), vec![2.0, 4.0, 6.0, 8.0, 10.0]);
        let config = PredictiveConfig::default();
        let results = run_predictive_analysis(&data, config)?;
        assert!(!results.forecasts.is_empty());
        assert!(!results.anomalies.is_empty());
        assert_eq!(results.metadata.variables_analyzed, 2);
        Ok(())
    }
}

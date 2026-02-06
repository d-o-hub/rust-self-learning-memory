//! # Predictive Analysis Module
//!
//! Provides forecasting models, anomaly detection, and causal inference capabilities
//! using advanced algorithms from augurs and deep_causality.

#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
use anyhow::Result;
use std::collections::HashMap;
use tracing::{info, instrument};

// Re-export submodules
pub mod anomaly;
pub mod causal;
pub mod dbscan;
pub mod extraction;
pub mod forecasting;
pub mod kdtree;

#[cfg(test)]
mod dbscan_tests;

// Re-export main types for convenience
pub use anomaly::AnomalyDetector;
pub use causal::CausalAnalyzer;
pub use dbscan::{AdaptiveDBSCAN, Cluster, ClusterLabel, DBSCANConfig, StreamingClusters};
pub use forecasting::{
    engine::ForecastingEngine,
    ets_types::{ETSErrorType, ETSSeasonalType, ETSTrendType, SeasonalityResult},
    types::{ForecastResult, PredictiveConfig, PredictiveMetadata, PredictiveResults},
};
pub use kdtree::{KDTree, Point};

// Re-export result types
pub use anomaly::AnomalyResult;
pub use causal::{CausalResult, CausalType};
pub use extraction::{
    ClusterCharacteristics, ExtractedPattern, ExtractionConfig, PatternExtractor, PatternType,
};

/// Comprehensive predictive analysis combining all methods
#[instrument(skip(data))]
pub fn run_predictive_analysis(
    data: &HashMap<String, Vec<f64>>,
    config: PredictiveConfig,
) -> Result<PredictiveResults> {
    let start_time = std::time::Instant::now();

    info!("Starting comprehensive predictive analysis");

    // Forecasting
    let mut forecaster = ForecastingEngine::with_config(config.clone())?;
    let forecasts = forecaster.forecast(data)?;

    // Anomaly detection
    let mut anomaly_detector = AnomalyDetector::with_config(config.clone())?;
    let anomalies = anomaly_detector.detect_anomalies(data)?;

    // Causal inference
    let causal_analyzer = CausalAnalyzer::with_config(config.clone())?;
    let causal_relationships = causal_analyzer.analyze_causality(data)?;

    // Calculate metadata
    let duration = start_time.elapsed();
    let metadata = PredictiveMetadata {
        variables_analyzed: data.len(),
        duration_ms: duration.as_millis() as u64,
        memory_usage: data.values().map(|v| v.len() * 8).sum(),
        methods_used: vec![
            "ETS Forecasting".to_string(),
            "DBSCAN Anomaly Detection".to_string(),
            "Granger Causality".to_string(),
        ],
    };

    let results = PredictiveResults {
        forecasts,
        anomalies,
        causal_relationships,
        metadata,
    };

    info!(
        "Predictive analysis completed in {}ms",
        results.metadata.duration_ms
    );

    Ok(results)
}

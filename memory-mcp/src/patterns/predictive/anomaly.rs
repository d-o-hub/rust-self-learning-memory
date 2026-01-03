use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, instrument, warn};

use super::dbscan::{AdaptiveDBSCAN, ClusterLabel, DBSCANConfig};
use super::forecasting::types::PredictiveConfig;

pub struct AnomalyDetector {
    #[allow(dead_code)]
    config: PredictiveConfig,
    dbscan: AdaptiveDBSCAN,
}

impl AnomalyDetector {
    /// Create a new anomaly detector
    pub fn new() -> Result<Self> {
        Self::with_config(PredictiveConfig::default())
    }

    /// Create a new anomaly detector with custom config
    pub fn with_config(config: PredictiveConfig) -> Result<Self> {
        let dbscan_config = DBSCANConfig {
            density: 0.1 * config.anomaly_sensitivity, // Scale density with sensitivity
            min_cluster_size: 3,
            max_distance: 1.0,
            window_size: config.reservoir_size.min(1000),
        };
        let dbscan = AdaptiveDBSCAN::new(dbscan_config)?;
        Ok(Self { config, dbscan })
    }

    /// Detect anomalies in time series data
    #[instrument(skip(self, data))]
    pub fn detect_anomalies(
        &mut self,
        data: &HashMap<String, Vec<f64>>,
    ) -> Result<Vec<AnomalyResult>> {
        let mut results = Vec::new();

        info!("Detecting anomalies in {} variables", data.len());

        for (var_name, series) in data {
            if series.len() < 3 {
                warn!(
                    "Skipping anomaly detection for {}: insufficient data points (need at least 3)",
                    var_name
                );
                continue;
            }

            let anomaly_result = self.detect_variable_anomalies(var_name, series)?;
            results.push(anomaly_result);
        }

        debug!("Detected anomalies in {} variables", results.len());
        Ok(results)
    }

    /// Detect anomalies in a single variable using DBSCAN
    fn detect_variable_anomalies(
        &mut self,
        variable: &str,
        series: &[f64],
    ) -> Result<AnomalyResult> {
        // Create timestamps for temporal context
        let timestamps: Vec<f64> = (0..series.len()).map(|i| i as f64).collect();

        // Use DBSCAN-based anomaly detection
        let cluster_labels = self.dbscan.detect_anomalies_dbscan(series, &timestamps);

        // Process results
        let mut anomaly_indices = Vec::new();
        let mut anomaly_scores = Vec::new();

        for (i, &label) in cluster_labels.iter().enumerate() {
            match label {
                ClusterLabel::Noise => {
                    anomaly_indices.push(i);
                    // Higher score for noise points (anomalies)
                    let deviation =
                        (series[i] - series.iter().sum::<f64>() / series.len() as f64).abs();
                    let variance: f64 = series
                        .iter()
                        .map(|&x| {
                            let mean = series.iter().sum::<f64>() / series.len() as f64;
                            (x - mean).powi(2)
                        })
                        .sum::<f64>()
                        / series.len() as f64;
                    let std_dev = variance.sqrt();
                    anomaly_scores.push(if std_dev > 0.0 {
                        deviation / std_dev
                    } else {
                        1.0
                    });
                }
                ClusterLabel::Cluster(_) => {
                    // Normal point - low anomaly score
                    anomaly_scores.push(0.0);
                }
            }
        }

        // Calculate confidence based on cluster quality
        let confidence = if !series.is_empty() {
            let cluster_count = cluster_labels
                .iter()
                .filter(|&label| !matches!(label, ClusterLabel::Noise))
                .count();
            let noise_ratio = (series.len() - cluster_count) as f64 / series.len() as f64;
            // Higher confidence when we have good clustering (low noise ratio)
            (1.0 - noise_ratio).clamp(0.0, 1.0)
        } else {
            0.0
        };

        Ok(AnomalyResult {
            variable: variable.to_string(),
            anomaly_indices,
            anomaly_scores,
            method: "DBSCAN".to_string(),
            confidence: confidence.clamp(0.0, 1.0),
        })
    }
}

/// Anomaly detection results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyResult {
    /// Variable name
    pub variable: String,
    /// Indices of detected anomalies
    pub anomaly_indices: Vec<usize>,
    /// Anomaly scores for each point
    pub anomaly_scores: Vec<f64>,
    /// Detection method used
    pub method: String,
    /// Detection confidence
    pub confidence: f64,
}

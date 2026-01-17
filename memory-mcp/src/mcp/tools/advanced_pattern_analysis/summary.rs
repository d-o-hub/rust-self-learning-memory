//! # Advanced Pattern Analysis Summary
//!
//! Summary generation and performance metrics for advanced pattern analysis.

use crate::patterns::{predictive, statistical};
use std::collections::HashMap;

use super::types::{AnalysisSummary, PerformanceMetrics};

/// Summary generator for analysis results
pub struct SummaryGenerator;

impl SummaryGenerator {
    /// Create a new summary generator
    pub fn new() -> Self {
        Self
    }

    /// Generate analysis summary and recommendations
    pub fn generate(
        &self,
        statistical: &Option<statistical::StatisticalResults>,
        predictive: &Option<predictive::PredictiveResults>,
        data: &HashMap<String, Vec<f64>>,
    ) -> AnalysisSummary {
        let mut key_findings = Vec::new();
        let mut recommendations = Vec::new();
        let mut confidence_level = 0.8; // Base confidence

        // Analyze statistical results
        if let Some(stats) = statistical {
            self.analyze_statistics(stats, &mut key_findings, &mut recommendations);
        }

        // Analyze predictive results
        if let Some(pred) = predictive {
            self.analyze_predictive(
                pred,
                &mut key_findings,
                &mut recommendations,
                &mut confidence_level,
            );
        }

        // Adjust confidence based on data quality
        if data.values().any(|series| series.len() < 10) {
            confidence_level *= 0.9;
            recommendations.push(
                "Consider collecting more data points for more reliable analysis".to_string(),
            );
        }

        AnalysisSummary {
            variables_analyzed: data.len(),
            key_findings,
            recommendations,
            confidence_level,
        }
    }

    /// Analyze statistical results for key findings
    fn analyze_statistics(
        &self,
        stats: &statistical::StatisticalResults,
        key_findings: &mut Vec<String>,
        recommendations: &mut Vec<String>,
    ) {
        // Significant correlations
        let sig_correlations: Vec<_> = stats
            .correlations
            .iter()
            .filter(|c| c.significant)
            .collect();

        if !sig_correlations.is_empty() {
            key_findings.push(format!(
                "Found {} significant correlations between variables",
                sig_correlations.len()
            ));
            recommendations
                .push("Consider these correlated variables when making decisions".to_string());
        }

        // Changepoints
        if !stats.changepoints.is_empty() {
            key_findings.push(format!(
                "Detected {} behavioral changepoints",
                stats.changepoints.len()
            ));
            recommendations.push("Investigate causes of detected changepoints".to_string());
        }

        // Trends
        let significant_trends: Vec<_> = stats.trends.iter().filter(|t| t.significant).collect();

        if !significant_trends.is_empty() {
            let increasing: Vec<_> = significant_trends
                .iter()
                .filter(|t| matches!(t.direction, statistical::TrendDirection::Increasing))
                .collect();
            let decreasing: Vec<_> = significant_trends
                .iter()
                .filter(|t| matches!(t.direction, statistical::TrendDirection::Decreasing))
                .collect();

            if !increasing.is_empty() {
                key_findings.push(format!(
                    "{} variables showing significant increasing trends",
                    increasing.len()
                ));
            }
            if !decreasing.is_empty() {
                key_findings.push(format!(
                    "{} variables showing significant decreasing trends",
                    decreasing.len()
                ));
            }
        }
    }

    /// Analyze predictive results for key findings
    fn analyze_predictive(
        &self,
        pred: &predictive::PredictiveResults,
        key_findings: &mut Vec<String>,
        recommendations: &mut Vec<String>,
        confidence_level: &mut f64,
    ) {
        // Forecasting quality
        let avg_forecast_quality: f64 =
            pred.forecasts.iter().map(|f| f.fit_quality).sum::<f64>() / pred.forecasts.len() as f64;

        if avg_forecast_quality > 0.7 {
            key_findings.push(format!(
                "High-quality forecasts generated (avg quality: {:.2})",
                avg_forecast_quality
            ));
            recommendations
                .push("Forecasts can be used for planning and decision making".to_string());
        } else if avg_forecast_quality < 0.5 {
            key_findings.push(format!(
                "Low-quality forecasts (avg quality: {:.2})",
                avg_forecast_quality
            ));
            recommendations.push(
                "Consider collecting more data or using different forecasting methods".to_string(),
            );
            *confidence_level *= 0.8;
        }

        // Anomalies
        let total_anomalies: usize = pred.anomalies.iter().map(|a| a.anomaly_indices.len()).sum();

        if total_anomalies > 0 {
            key_findings.push(format!(
                "Detected {} anomalous data points",
                total_anomalies
            ));
            recommendations.push("Investigate detected anomalies for potential issues".to_string());
        }

        // Causal relationships
        let significant_causal: Vec<_> = pred
            .causal_relationships
            .iter()
            .filter(|c| c.significant)
            .collect();

        if !significant_causal.is_empty() {
            key_findings.push(format!(
                "Found {} significant causal relationships",
                significant_causal.len()
            ));
            recommendations
                .push("Use causal relationships to understand system dynamics".to_string());
        }
    }
}

/// Performance metrics calculator
pub struct MetricsCalculator;

impl MetricsCalculator {
    /// Create a new metrics calculator
    pub fn new() -> Self {
        Self
    }

    /// Calculate performance metrics
    pub fn calculate(&self, start_time: std::time::Instant) -> PerformanceMetrics {
        let total_time_ms = start_time.elapsed().as_millis() as u64;

        // Estimate memory usage (rough approximation)
        let memory_usage_bytes = 1024 * 1024; // 1MB base estimate

        // Estimate CPU usage (simplified)
        let cpu_usage_percent = if total_time_ms > 1000 { 50.0 } else { 10.0 };

        PerformanceMetrics {
            total_time_ms,
            memory_usage_bytes,
            cpu_usage_percent,
        }
    }
}

impl Default for SummaryGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MetricsCalculator {
    fn default() -> Self {
        Self::new()
    }
}

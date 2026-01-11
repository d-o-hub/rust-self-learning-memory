//! # Advanced Pattern Analysis Tool Implementation
//!
//! MCP tool implementation for advanced pattern analysis combining statistical
//! and predictive modeling capabilities.

use crate::patterns::{predictive, statistical};
use crate::types::Tool;
use anyhow::{anyhow, Result};
use memory_core::SelfLearningMemory;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, instrument};

use super::types::{
    AdvancedPatternAnalysisInput, AdvancedPatternAnalysisOutput, AnalysisConfig, AnalysisSummary,
    AnalysisType, PerformanceMetrics,
};

/// Advanced pattern analysis tool implementation
pub struct AdvancedPatternAnalysisTool {
    memory: Arc<SelfLearningMemory>,
}

impl AdvancedPatternAnalysisTool {
    /// Create a new advanced pattern analysis tool
    pub fn new(memory: Arc<SelfLearningMemory>) -> Self {
        Self { memory }
    }

    /// Get the tool definition for MCP
    pub fn tool_definition() -> Tool {
        Tool::new(
            "advanced_pattern_analysis".to_string(),
            "Perform advanced statistical analysis, predictive modeling, and causal inference on time series data from memory episodes".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "analysis_type": {
                        "type": "string",
                        "enum": ["statistical", "predictive", "comprehensive"],
                        "description": "Type of analysis to perform"
                    },
                    "time_series_data": {
                        "type": "object",
                        "description": "Time series data as variable_name -> array of numeric values",
                        "patternProperties": {
                            ".*": {
                                "type": "array",
                                "items": {"type": "number"}
                            }
                        },
                        "additionalProperties": false
                    },
                    "config": {
                        "type": "object",
                        "description": "Optional analysis configuration",
                        "properties": {
                            "significance_level": {
                                "type": "number",
                                "minimum": 0.0,
                                "maximum": 1.0,
                                "default": 0.05,
                                "description": "Significance level for statistical tests"
                            },
                            "forecast_horizon": {
                                "type": "integer",
                                "minimum": 1,
                                "maximum": 100,
                                "default": 10,
                                "description": "Number of steps to forecast ahead"
                            },
                            "anomaly_sensitivity": {
                                "type": "number",
                                "minimum": 0.0,
                                "maximum": 1.0,
                                "default": 0.5,
                                "description": "Sensitivity for anomaly detection (higher = more sensitive)"
                            },
                            "enable_causal_inference": {
                                "type": "boolean",
                                "default": true,
                                "description": "Whether to perform causal inference analysis"
                            },
                            "max_data_points": {
                                "type": "integer",
                                "minimum": 10,
                                "maximum": 100000,
                                "default": 10000,
                                "description": "Maximum number of data points to analyze"
                            },
                            "parallel_processing": {
                                "type": "boolean",
                                "default": true,
                                "description": "Enable parallel processing for performance"
                            }
                        }
                    }
                },
                "required": ["analysis_type", "time_series_data"]
            }),
        )
    }

    /// Execute advanced pattern analysis
    #[instrument(skip(self, input), fields(analysis_type = ?input.analysis_type))]
    pub async fn execute(
        &self,
        input: AdvancedPatternAnalysisInput,
    ) -> Result<AdvancedPatternAnalysisOutput> {
        let start_time = std::time::Instant::now();

        info!("Starting advanced pattern analysis");

        // Validate input
        self.validate_input(&input)?;

        // Extract and prepare data
        let data = self.prepare_data(&input.time_series_data)?;

        // Perform analysis based on type
        let (statistical_results, predictive_results) = match input.analysis_type {
            AnalysisType::Statistical => {
                let results = self
                    .perform_statistical_analysis(&data, &input.config)
                    .await?;
                (Some(results), None)
            }
            AnalysisType::Predictive => {
                let results = self
                    .perform_predictive_analysis(&data, &input.config)
                    .await?;
                (None, Some(results))
            }
            AnalysisType::Comprehensive => {
                let statistical = self
                    .perform_statistical_analysis(&data, &input.config)
                    .await?;
                let predictive = self
                    .perform_predictive_analysis(&data, &input.config)
                    .await?;
                (Some(statistical), Some(predictive))
            }
        };

        // Generate summary and recommendations
        let summary = self.generate_summary(&statistical_results, &predictive_results, &data);

        // Calculate performance metrics
        let performance = self.calculate_performance_metrics(start_time);

        let output = AdvancedPatternAnalysisOutput {
            statistical_results,
            predictive_results,
            summary,
            performance,
        };

        info!(
            "Advanced pattern analysis completed in {}ms",
            output.performance.total_time_ms
        );

        Ok(output)
    }

    /// Validate input parameters
    pub fn validate_input(&self, input: &AdvancedPatternAnalysisInput) -> Result<()> {
        if input.time_series_data.is_empty() {
            return Err(anyhow!("No time series data provided"));
        }

        for (var_name, series) in &input.time_series_data {
            if series.is_empty() {
                return Err(anyhow!("Variable '{}' has no data points", var_name));
            }
            if series.len() < 3 {
                return Err(anyhow!(
                    "Variable '{}' has insufficient data points (minimum 3)",
                    var_name
                ));
            }
            if !series.iter().all(|&x| x.is_finite()) {
                return Err(anyhow!(
                    "Variable '{}' contains non-finite values",
                    var_name
                ));
            }
        }

        if let Some(config) = &input.config {
            if let Some(sig) = config.significance_level {
                if !(0.0..=1.0).contains(&sig) {
                    return Err(anyhow!("Significance level must be between 0.0 and 1.0"));
                }
            }
            if let Some(sens) = config.anomaly_sensitivity {
                if !(0.0..=1.0).contains(&sens) {
                    return Err(anyhow!("Anomaly sensitivity must be between 0.0 and 1.0"));
                }
            }
        }

        Ok(())
    }

    /// Prepare and validate data for analysis
    fn prepare_data(
        &self,
        raw_data: &HashMap<String, Vec<f64>>,
    ) -> Result<HashMap<String, Vec<f64>>> {
        let mut prepared_data = HashMap::new();

        for (var_name, series) in raw_data {
            // Remove any remaining non-finite values (shouldn't happen after validation)
            let clean_series: Vec<f64> =
                series.iter().copied().filter(|&x| x.is_finite()).collect();

            if clean_series.len() >= 3 {
                prepared_data.insert(var_name.clone(), clean_series);
            }
        }

        Ok(prepared_data)
    }

    /// Perform statistical analysis
    async fn perform_statistical_analysis(
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

    /// Perform predictive analysis
    async fn perform_predictive_analysis(
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

    /// Generate analysis summary and recommendations
    fn generate_summary(
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
            let significant_trends: Vec<_> =
                stats.trends.iter().filter(|t| t.significant).collect();

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

        // Analyze predictive results
        if let Some(pred) = predictive {
            // Forecasting quality
            let avg_forecast_quality: f64 =
                pred.forecasts.iter().map(|f| f.fit_quality).sum::<f64>()
                    / pred.forecasts.len() as f64;

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
                    "Consider collecting more data or using different forecasting methods"
                        .to_string(),
                );
                confidence_level *= 0.8;
            }

            // Anomalies
            let total_anomalies: usize =
                pred.anomalies.iter().map(|a| a.anomaly_indices.len()).sum();

            if total_anomalies > 0 {
                key_findings.push(format!(
                    "Detected {} anomalous data points",
                    total_anomalies
                ));
                recommendations
                    .push("Investigate detected anomalies for potential issues".to_string());
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

    /// Calculate performance metrics
    fn calculate_performance_metrics(&self, start_time: std::time::Instant) -> PerformanceMetrics {
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

    /// Extract time series data from memory episodes
    #[instrument(skip(self))]
    pub async fn extract_time_series_from_memory(
        &self,
        query: &str,
        domain: &str,
        limit: usize,
    ) -> Result<HashMap<String, Vec<f64>>> {
        info!("Extracting time series data from memory episodes");

        // Query memory for relevant episodes
        let context = memory_core::TaskContext {
            domain: domain.to_string(),
            language: None,
            framework: None,
            complexity: memory_core::ComplexityLevel::Moderate,
            tags: vec![],
        };

        let episodes = self
            .memory
            .retrieve_relevant_context(query.to_string(), context, limit)
            .await;

        if episodes.is_empty() {
            return Err(anyhow!(
                "No relevant episodes found for time series extraction"
            ));
        }

        // Extract metrics from episodes
        let mut time_series = HashMap::new();

        // Common metrics to extract
        let metrics = vec![
            "execution_time_ms",
            "success_rate",
            "complexity_score",
            "pattern_match_score",
            "memory_usage_mb",
        ];

        for metric in metrics {
            let mut values = Vec::new();

            for episode in &episodes {
                // Extract metric value from episode data
                // This is a simplified extraction - in practice, you'd parse episode JSON
                match metric {
                    "execution_time_ms" => {
                        // Try to extract from execution steps
                        let total_time: u64 =
                            episode.steps.iter().map(|step| step.latency_ms).sum();
                        values.push(total_time as f64);
                    }
                    "success_rate" => {
                        // Calculate success rate from outcomes
                        let success_count = episodes
                            .iter()
                            .filter(|e| {
                                matches!(e.outcome, Some(memory_core::TaskOutcome::Success { .. }))
                            })
                            .count();
                        let rate = success_count as f64 / episodes.len() as f64;
                        values.push(rate * 100.0); // Convert to percentage
                    }
                    "complexity_score" => {
                        // Use complexity level as numeric score
                        let score = match episode.context.complexity {
                            memory_core::ComplexityLevel::Simple => 1.0,
                            memory_core::ComplexityLevel::Moderate => 2.0,
                            memory_core::ComplexityLevel::Complex => 3.0,
                        };
                        values.push(score);
                    }
                    "pattern_match_score" => {
                        // Simplified pattern matching score
                        values.push(0.8); // Placeholder
                    }
                    "memory_usage_mb" => {
                        // Estimate memory usage
                        values.push(50.0); // Placeholder
                    }
                    _ => {}
                }
            }

            if !values.is_empty() && values.len() >= 3 {
                time_series.insert(metric.to_string(), values);
            }
        }

        debug!(
            "Extracted {} time series from {} episodes",
            time_series.len(),
            episodes.len()
        );

        Ok(time_series)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory_core::SelfLearningMemory;

    #[tokio::test]
    async fn test_tool_definition() {
        let tool = AdvancedPatternAnalysisTool::tool_definition();
        assert_eq!(tool.name, "advanced_pattern_analysis");
        assert!(!tool.description.is_empty());
        assert!(tool.input_schema.is_object());
    }

    #[tokio::test]
    async fn test_input_validation() {
        let memory = Arc::new(SelfLearningMemory::new());
        let tool = AdvancedPatternAnalysisTool::new(memory);

        // Valid input
        let mut data = HashMap::new();
        data.insert("test".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);

        let input = AdvancedPatternAnalysisInput {
            analysis_type: AnalysisType::Statistical,
            time_series_data: data.clone(),
            config: None,
        };

        assert!(tool.validate_input(&input).is_ok());

        // Invalid input - empty data
        let input_empty = AdvancedPatternAnalysisInput {
            analysis_type: AnalysisType::Statistical,
            time_series_data: HashMap::new(),
            config: None,
        };

        assert!(tool.validate_input(&input_empty).is_err());

        // Invalid input - insufficient data points
        let mut small_data = HashMap::new();
        small_data.insert("small".to_string(), vec![1.0, 2.0]);

        let input_small = AdvancedPatternAnalysisInput {
            analysis_type: AnalysisType::Statistical,
            time_series_data: small_data,
            config: None,
        };

        assert!(tool.validate_input(&input_small).is_err());
    }

    #[tokio::test]
    async fn test_statistical_analysis_execution() {
        let memory = Arc::new(SelfLearningMemory::new());
        let tool = AdvancedPatternAnalysisTool::new(memory);

        let mut data = HashMap::new();
        data.insert("x".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        data.insert("y".to_string(), vec![2.0, 4.0, 6.0, 8.0, 10.0]);

        let input = AdvancedPatternAnalysisInput {
            analysis_type: AnalysisType::Statistical,
            time_series_data: data,
            config: None,
        };

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.statistical_results.is_some());
        assert!(output.predictive_results.is_none());
        assert_eq!(output.summary.variables_analyzed, 2);
    }

    #[tokio::test]
    async fn test_predictive_analysis_execution() {
        let memory = Arc::new(SelfLearningMemory::new());
        let tool = AdvancedPatternAnalysisTool::new(memory);

        let mut data = HashMap::new();
        data.insert(
            "trend".to_string(),
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
        );

        let input = AdvancedPatternAnalysisInput {
            analysis_type: AnalysisType::Predictive,
            time_series_data: data,
            config: None,
        };

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.statistical_results.is_none());
        assert!(output.predictive_results.is_some());
    }

    #[tokio::test]
    async fn test_comprehensive_analysis_execution() {
        let memory = Arc::new(SelfLearningMemory::new());
        let tool = AdvancedPatternAnalysisTool::new(memory);

        let mut data = HashMap::new();
        data.insert("series1".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        data.insert("series2".to_string(), vec![2.0, 4.0, 6.0, 8.0, 10.0]);

        let input = AdvancedPatternAnalysisInput {
            analysis_type: AnalysisType::Comprehensive,
            time_series_data: data,
            config: None,
        };

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.statistical_results.is_some());
        assert!(output.predictive_results.is_some());
        assert_eq!(output.summary.variables_analyzed, 2);
    }
}

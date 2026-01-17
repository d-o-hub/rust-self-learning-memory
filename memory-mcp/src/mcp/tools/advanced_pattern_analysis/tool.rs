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

use super::summary::{MetricsCalculator, SummaryGenerator};
use super::time_series::TimeSeriesExtractor;
use super::validator::{DataPreparer, InputValidator};

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
        let validator = InputValidator::new();
        validator.validate(input)
    }

    /// Prepare and validate data for analysis
    fn prepare_data(
        &self,
        raw_data: &HashMap<String, Vec<f64>>,
    ) -> Result<HashMap<String, Vec<f64>>> {
        let preparer = DataPreparer::new();
        preparer.prepare(raw_data)
    }

    /// Generate analysis summary and recommendations
    fn generate_summary(
        &self,
        statistical: &Option<statistical::StatisticalResults>,
        predictive: &Option<predictive::PredictiveResults>,
        data: &HashMap<String, Vec<f64>>,
    ) -> AnalysisSummary {
        let generator = SummaryGenerator::new();
        generator.generate(statistical, predictive, data)
    }

    /// Calculate performance metrics
    fn calculate_performance_metrics(&self, start_time: std::time::Instant) -> PerformanceMetrics {
        let calculator = MetricsCalculator::new();
        calculator.calculate(start_time)
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
            return Err(anyhow::anyhow!(
                "No relevant episodes found for time series extraction"
            ));
        }

        // Extract metrics from episodes using TimeSeriesExtractor
        let extractor = TimeSeriesExtractor::new();
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
                if let Some(value) = extractor.extract_metric(metric, episode, &episodes) {
                    values.push(value);
                }
            }

            if extractor.meets_threshold(&values, 3) {
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

// Advanced tool handlers
//!
//! This module contains advanced tool handlers: advanced_pattern_analysis and quality_metrics.

use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use tracing::debug;

impl crate::server::MemoryMCPServer {
    /// Execute the advanced_pattern_analysis tool
    ///
    /// # Arguments
    ///
    /// * `input` - Analysis input parameters
    ///
    /// # Returns
    ///
    /// Returns comprehensive analysis results
    pub async fn execute_advanced_pattern_analysis(
        &self,
        input: crate::mcp::tools::advanced_pattern_analysis::AdvancedPatternAnalysisInput,
    ) -> Result<serde_json::Value> {
        self.track_tool_usage("advanced_pattern_analysis").await;

        debug!(
            "Executing advanced pattern analysis: {:?}",
            input.analysis_type
        );

        let tool = crate::mcp::tools::advanced_pattern_analysis::AdvancedPatternAnalysisTool::new(
            Arc::clone(&self.memory),
        );

        let result = tool.execute(input).await?;

        // Convert result to JSON
        Ok(json!(result))
    }

    /// Execute the analyze_concept_drift tool
    ///
    /// # Arguments
    ///
    /// * `input` - Drift analysis input parameters
    ///
    /// # Returns
    ///
    /// Returns drift analysis results
    pub async fn execute_analyze_concept_drift(
        &self,
        input: crate::mcp::tools::concept_drift::ConceptDriftInput,
    ) -> Result<serde_json::Value> {
        self.track_tool_usage("analyze_concept_drift").await;

        debug!("Executing concept drift analysis for: {}", input.parent_id);

        let tool = crate::mcp::tools::concept_drift::ConceptDriftTool::new(Arc::clone(&self.memory));

        let result = tool.execute(input).await?;

        // Convert result to JSON
        Ok(json!(result))
    }

    /// Execute the quality_metrics tool
    ///
    /// # Arguments
    ///
    /// * `input` - Quality metrics input parameters
    ///
    /// # Returns
    ///
    /// Returns quality metrics and noise reduction statistics
    pub async fn execute_quality_metrics(
        &self,
        input: crate::mcp::tools::quality_metrics::QualityMetricsInput,
    ) -> Result<serde_json::Value> {
        self.track_tool_usage("quality_metrics").await;

        debug!(
            "Executing quality metrics query: time_range={}, include_trends={}",
            input.time_range, input.include_trends
        );

        let tool =
            crate::mcp::tools::quality_metrics::QualityMetricsTool::new(Arc::clone(&self.memory));

        let result = tool.execute(input).await?;

        // Convert result to JSON
        Ok(json!(result))
    }
}
